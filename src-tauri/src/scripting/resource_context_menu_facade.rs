use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock, RwLock},
};

use kube::api::GroupVersionKind;
use rhai::{exported_module, EvalAltResult};
use serde::Serialize;
use serde_json::json;

use crate::{
    internal::{gvk_extraction::GvkExtraction, mini_id::random_id},
    scripting::{
        modules,
        scripts_provider::ScriptsProvider,
        types::{self},
    },
};

struct ContextMenuSection {
    pub title: Option<String>,
    pub matcher: Option<rhai::FnPtr>,
    pub items: rhai::FnPtr,
    pub ast: Arc<rhai::AST>,
}

impl ContextMenuSection {
    /// Calls the `matcher` function defined in the script. If no matcher is defined, we assume that this menu section
    /// applies to every single Kubernetes resource. This is useful for general purpose actions like 'Delete'.
    pub fn matches_gvk(
        &self,
        engine: &rhai::Engine,
        ast: &rhai::AST,
        gvk: GroupVersionKind,
    ) -> Result<bool, Box<rhai::EvalAltResult>> {
        self.matcher
            .as_ref()
            .map(|matcher| matcher.call::<bool>(engine, ast, (gvk.group, gvk.version, gvk.kind)))
            .unwrap_or(Ok(true))
    }

    /// Calls the `items` function defined in the script with the given Kubernetes object. The script may inspect the
    /// object to customize the items per resource. This is useful to render repeating elements like an action per
    /// container within a Pod.
    pub fn render_items_for(
        &self,
        engine: &rhai::Engine,
        ast: &rhai::AST,
        obj: rhai::Dynamic,
    ) -> Result<Vec<types::MenuItem>, Box<rhai::EvalAltResult>> {
        Ok(self
            .items
            .call::<rhai::Array>(engine, ast, (obj.clone(),))?
            .into_iter()
            .filter(|i| !i.is_unit())
            .flat_map(|dynamic| {
                let type_name = dynamic.type_name();
                let something: Option<types::MenuItem> = dynamic
                    .try_into()
                    .map_err(|_| {
                        tracing::warn!("Unsupported menu item: {type_name}");
                    })
                    .ok();

                something
            })
            .collect())
    }
}

#[derive(Clone, Debug)]
pub struct CallbackContext {
    pub app_handle: tauri::AppHandle,
    pub frontend_tab: String,
}

pub struct ResourceContextMenuFacade {
    app: tauri::AppHandle,
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

#[derive(Debug, Clone)]
pub struct MenuStack {
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

#[allow(dead_code)]
impl ResourceContextMenuFacade {
    pub fn new(app: tauri::AppHandle) -> Arc<Self> {
        Arc::new(Self {
            app,
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
                move |ctx: rhai::NativeCallContext, definition: types::MenuSection| {
                    let script = ctx
                        .call_source()
                        .expect("only file-based scripts supported");
                    facade.register_resource_contextmenu_section(definition, script);
                },
            );
        }

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

    pub fn create_resource_menustack(
        &self,
        obj: kube::api::DynamicObject,
        tab_id: &str,
    ) -> MenuBlueprint {
        let gvk = obj.types.as_ref().unwrap().gvk();
        let obj = rhai::serde::to_dynamic(obj).unwrap();

        let mut menu_stack = MenuStack::new(CallbackContext {
            app_handle: self.app.clone(),
            frontend_tab: tab_id.to_owned(),
        });

        let engine = self
            .resource_contextmenu_engine
            .get()
            .expect("engine must be initialized");

        fn transform_item(item: types::MenuItem) -> (FrontendMenuItem, Vec<(String, rhai::FnPtr)>) {
            match item {
                types::MenuItem::ActionButton(action_button) => {
                    let action_id = random_id(5);

                    let frontend_item = FrontendMenuItem {
                        kind: FrontendMenuItemKind::ActionButton,
                        data: Some(HashMap::from_iter([
                            ("title", json!(action_button.title)),
                            ("dangerous", json!(action_button.dangerous)),
                            ("confirm", json!(action_button.confirm)),
                            ("actionRef", json!(action_id.clone())),
                        ])),
                    };

                    (frontend_item, vec![(action_id, action_button.action)])
                }

                types::MenuItem::SubMenu(submenu) => {
                    let (sub_items, actions): (Vec<_>, Vec<_>) =
                        submenu.items.into_iter().map(transform_item).unzip();

                    let actions = actions.into_iter().flatten().collect();

                    let frontend_item = FrontendMenuItem {
                        kind: FrontendMenuItemKind::SubMenu,
                        data: Some(HashMap::from_iter([
                            ("title", json!(submenu.title)),
                            ("items", json!(sub_items)),
                        ])),
                    };

                    (frontend_item, actions)
                }
            }
        }

        let frontend_sections: Vec<FrontendMenuSection> = {
            let section_templates = self.registered_contextmenu_sections.read().unwrap();
            section_templates
                .iter()
                .flat_map(|section_template| {
                    let matches = section_template
                        .matches_gvk(engine, &section_template.ast, gvk.clone())
                        .expect("Call to `matcher` failed"); // todo: error handling

                    if !matches {
                        return None;
                    }

                    let items: Vec<types::MenuItem> = section_template
                        .render_items_for(engine, &section_template.ast, obj.clone())
                        .expect("Call to `items` failed"); // todo: error handling

                    let mut section_items = FrontendMenuSection {
                        title: section_template.title.clone(),
                        items: Vec::new(),
                    };

                    for item in items {
                        let (item, actions) = transform_item(item);
                        section_items.items.push(item);

                        for (action_id, action) in actions {
                            menu_stack
                                .actions
                                .entry(action_id)
                                .insert_entry(FnPtrWithAst {
                                    fnptr: action,
                                    ast: Arc::clone(&section_template.ast),
                                });
                        }
                    }

                    section_items.items.push(FrontendMenuItem {
                        kind: FrontendMenuItemKind::Separator,
                        data: None,
                    });

                    Some(section_items)
                })
                .collect()
        };

        let menu_stack_id = random_id(5);
        {
            let mut menu_stacks = self.menu_stacks.write().unwrap();
            menu_stacks
                .entry(menu_stack_id.clone())
                .insert_entry(menu_stack);
        }

        MenuBlueprint {
            id: menu_stack_id,
            items: frontend_sections,
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
        let action = menu.actions.get(action_ref).unwrap();

        let engine = self.resource_contextmenu_engine.get().unwrap();
        let ctx = Arc::clone(&menu.context);

        action
            .fnptr
            .call::<()>(engine, &action.ast, (ctx,))
            .unwrap();
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
    SubMenu,
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
