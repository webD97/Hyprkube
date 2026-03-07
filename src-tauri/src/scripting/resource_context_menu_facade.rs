use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock, RwLock},
};

use rhai::exported_module;

use crate::{
    internal::{gvk_extraction::GvkExtraction, mini_id::random_id},
    scripting::{
        commons::{CallbackContext, ContentScript, FnPtrWithAst},
        modules,
        resource_context_menu::{
            ContextMenuSection, FrontendMenuItem, FrontendMenuItemKind, FrontendMenuSection,
            MenuBlueprint,
        },
        scripts_provider::{self, ScriptType, ScriptsProvider},
        types::{self},
    },
};

pub struct ResourceContextMenuFacade {
    app: tauri::AppHandle,
    engine: OnceLock<rhai::Engine>,
    registered_sections: RwLock<Vec<ContextMenuSection>>,
    scripts: RwLock<HashMap<PathBuf, ContentScript>>,
    menu_stacks: RwLock<HashMap<String, MenuStack>>,
}

#[derive(Debug, Clone)]
struct MenuStack {
    context: Arc<CallbackContext>,
    actions: HashMap<String, FnPtrWithAst>,
}

impl MenuStack {
    fn new(context: CallbackContext) -> Self {
        Self {
            context: Arc::new(context),
            actions: HashMap::new(),
        }
    }
}

impl ResourceContextMenuFacade {
    pub fn new(app: tauri::AppHandle) -> Arc<Self> {
        Arc::new(Self {
            app,
            engine: OnceLock::new(),
            registered_sections: RwLock::new(Vec::new()),
            scripts: RwLock::new(HashMap::new()),
            menu_stacks: RwLock::new(HashMap::new()),
        })
    }

    pub fn initialize_engines(
        self: &Arc<Self>,
        client: kube::Client,
        discovery: Arc<kube::Discovery>,
    ) {
        let engine = Self::make_resource_contextmenu_engine(Arc::clone(self), client, discovery);
        self.engine.get_or_init(|| engine);
    }

    fn make_resource_contextmenu_engine(
        facade: Arc<Self>,
        client: kube::Client,
        discovery: Arc<kube::Discovery>,
    ) -> rhai::Engine {
        let mut engine = rhai::Engine::new();

        engine.build_type::<types::ResourceRef>();
        engine.build_type::<types::ActionButton>();
        engine.build_type::<types::SubMenu>();
        engine.build_type::<types::MenuSection>();

        engine.register_static_module(
            "kube",
            modules::kube::build_module(client, discovery).into(),
        );
        engine.register_static_module(
            "clipboard",
            modules::clipboard::build_module(facade.app.clone()).into(),
        );
        engine.register_static_module("base64", exported_module!(modules::base64_rhai).into());
        engine.register_static_module("frontend", exported_module!(modules::frontend_rhai).into());

        {
            let facade = Arc::clone(&facade);
            engine.register_fn(
                "register_resource_contextmenu_section",
                move |ctx: rhai::NativeCallContext,
                      definition: types::MenuSection|
                      -> Result<(), Box<rhai::EvalAltResult>> {
                    let script = ctx
                        .call_source()
                        .ok_or("only file-based scripts supported")?;

                    facade
                        .register_resource_contextmenu_section(definition, script)
                        .map_err(|e| e.to_string().into())
                },
            );
        }

        engine.set_max_expr_depths(64, 32);

        engine
    }

    fn register_resource_contextmenu_section(
        &self,
        section: types::MenuSection,
        script: &str,
    ) -> Result<(), ResourceContextMenuError> {
        let script: PathBuf = script.into();

        let ast = self.scripts.read().unwrap();
        let ast = ast
            .get(&script)
            .unwrap()
            .ast
            .as_ref()
            .ok_or(ResourceContextMenuError::PendingCompilation)?
            .as_ref()
            .map_err(|_| ResourceContextMenuError::CompilationError)?;

        let mut sections = self.registered_sections.write().unwrap();

        sections.push(ContextMenuSection {
            title: section.title,
            matcher: section.matcher,
            items: section.items,
            ast: Arc::clone(ast),
        });

        Ok(())
    }

    pub fn create_resource_menustack(
        &self,
        obj: kube::api::DynamicObject,
        tab_id: &str,
    ) -> Result<MenuBlueprint, ResourceContextMenuError> {
        let gvk = obj.types.as_ref().unwrap().gvk();
        let obj = rhai::serde::to_dynamic(obj)?;

        let mut menu_stack =
            MenuStack::new(CallbackContext::new(self.app.clone(), tab_id.to_owned()));

        let engine = self
            .engine
            .get()
            .ok_or(ResourceContextMenuError::EngineUninitialized)?;

        let frontend_sections: Vec<Result<FrontendMenuSection, ResourceContextMenuError>> = {
            let section_templates = self.registered_sections.read().unwrap();
            section_templates
                .iter()
                .map(|section_template| {
                    let matches = section_template
                        .matches_gvk(engine, &section_template.ast, gvk.clone())
                        .map_err(ResourceContextMenuError::Matcher)?;

                    if !matches {
                        return Ok(None);
                    }

                    let items: Vec<types::MenuItem> = section_template
                        .render_items_for(engine, &section_template.ast, obj.clone())
                        .map_err(ResourceContextMenuError::Items)?;

                    let frontend_items = items
                        .into_iter()
                        .map(|item| {
                            let (item, actions) = types::MenuItem::transform_for_frontend(item);

                            for (action_id, action) in actions {
                                menu_stack.actions.entry(action_id).insert_entry(
                                    FnPtrWithAst::new(action, Arc::clone(&section_template.ast)),
                                );
                            }

                            item
                        })
                        .collect();

                    let mut section_items =
                        FrontendMenuSection::new(section_template.title.clone(), frontend_items);

                    section_items
                        .push_item(FrontendMenuItem::new(FrontendMenuItemKind::Separator, None));

                    Ok(Some(section_items))
                })
                .filter(|i| match i {
                    Ok(Some(_)) => true,
                    Ok(None) => false,
                    Err(_) => true,
                })
                .filter_map(Result::transpose)
                .collect()
        };

        let menu_stack_id = random_id(5);
        {
            let mut menu_stacks = self.menu_stacks.write().unwrap();
            menu_stacks
                .entry(menu_stack_id.clone())
                .insert_entry(menu_stack);
        }

        let (oks, errs): (Vec<_>, Vec<_>) = frontend_sections.into_iter().partition(Result::is_ok);

        for err in errs.into_iter().map(Result::unwrap_err) {
            tracing::warn!("Error building MenuBlueprint: {err}");
        }

        Ok(MenuBlueprint::new(
            menu_stack_id,
            oks.into_iter().map(Result::unwrap).collect(),
        ))
    }

    pub fn drop_resource_menustack(&self, id: &str) -> Result<(), ResourceContextMenuError> {
        let mut menu_stacks = self.menu_stacks.write().unwrap();
        menu_stacks.remove(id);

        Ok(())
    }

    pub fn call_menustack_action(
        &self,
        menu_id: &str,
        action_ref: &str,
    ) -> Result<(), ResourceContextMenuError> {
        let menus = self.menu_stacks.read().unwrap();

        let menu = menus
            .get(menu_id)
            .ok_or_else(|| ResourceContextMenuError::NoSuchMenuStack(menu_id.to_owned()))?;

        let action = menu
            .actions
            .get(action_ref)
            .ok_or_else(|| ResourceContextMenuError::NoSuchMenuAction(action_ref.to_owned()))?;

        let engine = self
            .engine
            .get()
            .ok_or(ResourceContextMenuError::EngineUninitialized)?;

        let ctx = Arc::clone(&menu.context);

        action.fnptr.call::<()>(engine, &action.ast, (ctx,))?;

        Ok(())
    }

    pub fn evaluate(
        &self,
        scripts_provider: &ScriptsProvider,
    ) -> Result<(), ResourceContextMenuError> {
        let engine = self
            .engine
            .get()
            .ok_or(ResourceContextMenuError::EngineUninitialized)?;

        let menu_scripts = scripts_provider
            .get_scripts_for_type(ScriptType::Menu)
            .unwrap();

        for entrypoint in &menu_scripts {
            tracing::info!("Evaluating {}", entrypoint.to_string_lossy());

            let ast_arc = {
                let mut scripts = self.scripts.write().unwrap();

                let script = scripts
                    .entry(entrypoint.to_owned())
                    .or_insert(ContentScript::new());

                let ast_result = script
                    .ast
                    .get_or_insert_with(|| engine.compile_file(entrypoint.clone()).map(Arc::new));

                ast_result
                    .as_ref()
                    .map_err(|_| ResourceContextMenuError::CompilationError)?
                    .clone()
            };

            engine.eval_ast::<()>(&ast_arc)?;
        }

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ResourceContextMenuError {
    #[error("No MenuStack with id {0}")]
    NoSuchMenuStack(String),

    #[error("No menu action with id {0}")]
    NoSuchMenuAction(String),

    #[error("Scripting engine has not been initialized")]
    EngineUninitialized,

    #[error("Error evaluating script: {0}")]
    EvaluationResult(#[from] Box<rhai::EvalAltResult>),

    #[error("The script has not yet been compiled")]
    PendingCompilation,

    #[error("The script has a compilation error")]
    CompilationError,

    #[error(transparent)]
    ScriptDirectoryResolution(#[from] scripts_provider::Error),

    #[error("Call to matcher failed: {0}")]
    Matcher(Box<rhai::EvalAltResult>),

    #[error("Call to items failed: {0}")]
    Items(Box<rhai::EvalAltResult>),
}
