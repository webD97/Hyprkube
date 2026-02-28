use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock, RwLock},
};

use rhai::{exported_module, EvalAltResult};
use serde::Serialize;
use serde_json::json;

use crate::{
    internal::mini_id::random_id,
    scripting::{
        modules,
        scripts_provider::ScriptsProvider,
        types::{self},
    },
};

#[allow(unused)]
struct ContextMenuSection {
    pub title: Option<String>,
    pub matcher: Option<rhai::FnPtr>,
    pub items: rhai::FnPtr,
    pub ast: Arc<rhai::AST>,
}

pub struct ResourceContextMenuFacade {
    /// Handles scripts that render a resource context menu
    resource_contextmenu_engine: OnceLock<rhai::Engine>,
    registered_contextmenu_sections: RwLock<Vec<ContextMenuSection>>,
    resource_action_scripts: RwLock<HashMap<PathBuf, UserScript>>,
    menu_stacks: RwLock<HashMap<String, MenuStack>>,
}

pub struct UserScript {
    ast: Option<Result<Arc<rhai::AST>, Box<EvalAltResult>>>,
}

#[derive(Debug, Clone)]
pub struct FnPtrWithAst {
    fnptr: rhai::FnPtr,
    ast: Arc<rhai::AST>,
}

#[derive(Debug, Clone, Default)]
pub struct MenuStack {
    actions: HashMap<String, FnPtrWithAst>,
}

#[allow(dead_code)]
impl ResourceContextMenuFacade {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            resource_contextmenu_engine: OnceLock::new(),
            registered_contextmenu_sections: RwLock::new(Vec::new()),
            resource_action_scripts: RwLock::new(HashMap::new()),
            menu_stacks: RwLock::new(HashMap::new()),
        })
    }

    pub fn initialize_engines(
        self: &Arc<Self>,
        client: kube::Client,
        discovery: Arc<kube::Discovery>,
    ) {
        let engine = Self::make_resource_contextmenu_engine(Arc::clone(self), client, discovery);
        self.resource_contextmenu_engine.get_or_init(|| engine);
    }

    fn make_resource_contextmenu_engine(
        facade: Arc<Self>,
        client: kube::Client,
        discovery: Arc<kube::Discovery>,
    ) -> rhai::Engine {
        let mut engine = rhai::Engine::new();

        engine.build_type::<types::ResourceRef>();
        engine.build_type::<types::ActionButton>();
        engine.build_type::<types::MenuSection>();

        engine.register_static_module(
            "kube",
            modules::kube::build_module(client, discovery).into(),
        );
        engine.register_static_module("base64", exported_module!(modules::base64_rhai).into());

        engine.register_fn(
            "register_resource_contextmenu_section",
            move |ctx: rhai::NativeCallContext, definition: types::MenuSection| {
                let script = ctx
                    .call_source()
                    .expect("only file-based scripts supported");
                facade.register_resource_contextmenu_section(definition, script);
            },
        );

        engine.set_max_expr_depths(64, 32);

        engine
    }

    fn register_resource_contextmenu_section(&self, section: types::MenuSection, script: &str) {
        let script: PathBuf = script.into();

        let ast = self.resource_action_scripts.read().unwrap();
        let ast = ast
            .get(&script)
            .unwrap()
            .ast
            .as_ref()
            .expect("already compiled")
            .as_ref()
            .expect("compiled without errors");

        let mut sections = self.registered_contextmenu_sections.write().unwrap();
        sections.push(ContextMenuSection {
            title: section.title,
            matcher: section.matcher,
            items: section.items,
            ast: Arc::clone(ast),
        });
    }

    #[allow(unused)]
    pub fn create_resource_menustack(&self, obj: kube::api::DynamicObject) -> MenuBlueprint {
        let api_version = obj.types.as_ref().unwrap().api_version.to_owned();
        let kind = obj.types.as_ref().unwrap().kind.to_owned();
        let obj = rhai::serde::to_dynamic(obj).unwrap();

        let sections_defs = self.registered_contextmenu_sections.read().unwrap();

        type SectionItem = (types::MenuItem, Option<Arc<rhai::AST>>);
        type SectionTitle = Option<String>;
        type Section = (SectionTitle, Vec<SectionItem>);

        let mut sections: Vec<Section> = Vec::new();

        let engine = self
            .resource_contextmenu_engine
            .get()
            .expect("engine must be initialized");

        for section_def in sections_defs.iter() {
            let ast = &section_def.ast;

            let matches = section_def
                .matcher
                .as_ref()
                .map(|matcher| {
                    matcher
                        .call::<bool>(engine, ast, ("", "", kind.clone())) // todo: fill in full gvk
                        .unwrap()
                })
                .unwrap_or(true);

            if !matches {
                println!("Does not match");
                continue;
            }

            let items = section_def
                .items
                .call::<rhai::Array>(engine, ast, (obj.clone(),))
                .unwrap() // TODO: The callback might fail
                .into_iter()
                .filter(|i| !i.is_unit())
                .flat_map(|dynamic| {
                    let type_name = dynamic.type_name();
                    if let Some(mut item) = dynamic.try_cast::<types::ActionButton>() {
                        // Curry the callback with the object that we want to operate on when called later
                        item.action = item.action.add_curry(obj.clone()).to_owned();
                        Some((types::MenuItem::ActionButton(item), Some(Arc::clone(ast))))
                    } else {
                        tracing::warn!("Unsupported menu item: {type_name}");
                        None
                    }
                })
                .collect();

            sections.push((section_def.title.clone(), items));
        }

        let stack_id = random_id(5);
        let mut items = Vec::new();

        {
            let mut menu_stack = MenuStack::default();

            for (title, section) in sections {
                let mut section_items = FrontendMenuSection {
                    title,
                    items: Vec::new(),
                };

                for section_item in section {
                    section_items.items.push(match section_item {
                        (types::MenuItem::ActionButton(action_button), ast) => {
                            let action_id = random_id(5);
                            menu_stack.actions.entry(action_id.clone()).insert_entry(
                                FnPtrWithAst {
                                    fnptr: action_button.action,
                                    ast: ast.expect("must be set"),
                                },
                            );

                            FrontendMenuItem {
                                kind: FrontendMenuItemKind::ActionButton,
                                data: Some(HashMap::from_iter([
                                    ("title", json!(action_button.title)),
                                    ("dangerous", json!(action_button.dangerous)),
                                    ("actionRef", json!(action_id)),
                                ])),
                            }
                        }
                    });
                }

                section_items.items.push(FrontendMenuItem {
                    kind: FrontendMenuItemKind::Separator,
                    data: None,
                });

                items.push(section_items);
            }

            let mut menu_stacks = self.menu_stacks.write().unwrap();
            menu_stacks
                .entry(stack_id.to_owned())
                .insert_entry(menu_stack);
        }

        MenuBlueprint {
            id: stack_id,
            items,
        }
    }

    pub fn drop_resource_menustack(&self, id: &str) {
        let mut menu_stacks = self.menu_stacks.write().unwrap();
        menu_stacks.remove(id);
    }

    // todo: error handling
    pub fn call_menustack_action(&self, menu_id: &str, action_ref: &str) {
        let menus = self.menu_stacks.read().unwrap();
        let menu = menus.get(menu_id).unwrap();
        let action = menu.actions.get(action_ref).unwrap().clone();

        let engine = self.resource_contextmenu_engine.get().unwrap();

        action.fnptr.call::<()>(engine, &action.ast, ()).unwrap();
    }

    pub fn evaluate(&self, scripts_provider: &ScriptsProvider) {
        let engine = self
            .resource_contextmenu_engine
            .get()
            .expect("Engine not initialized");

        let builtins = scripts_provider.get_builtins_entrypoints().unwrap();
        let extensions = scripts_provider.get_extensions_entrypoints().unwrap();

        for entrypoint in builtins.iter().chain(&extensions) {
            tracing::info!("Evaluating {}", entrypoint.to_string_lossy());

            if !std::fs::exists(entrypoint).unwrap() {
                tracing::warn!("Entrypoint does not exist");
                continue;
            }

            let ast_arc = {
                let mut scripts = self.resource_action_scripts.write().unwrap();

                let script = scripts
                    .entry(entrypoint.to_owned())
                    .or_insert(UserScript { ast: None });

                let ast_result = script
                    .ast
                    .get_or_insert_with(|| engine.compile_file(entrypoint.clone()).map(Arc::new));

                ast_result
                    .as_ref()
                    .map_err(|e| format!("Compilation failed for {}: {e}", entrypoint.display()))
                    .unwrap()
                    .clone()
            };

            engine.eval_ast::<()>(&ast_arc).unwrap();
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum FrontendMenuItemKind {
    ActionButton,
    Separator,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendMenuItem {
    kind: FrontendMenuItemKind,
    data: Option<HashMap<&'static str, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendMenuSection {
    title: Option<String>,
    items: Vec<FrontendMenuItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MenuBlueprint {
    pub id: String,
    items: Vec<FrontendMenuSection>,
}
