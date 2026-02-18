use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock, RwLock},
};

use kube::api::DynamicObject;
use rhai::{exported_module, EvalAltResult};
use serde::Serialize;
use tauri::Manager;

use crate::{
    cluster_discovery::ClusterRegistryState,
    frontend_commands::KubeContextSource,
    frontend_types::BackendError,
    scripting::{
        modules,
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

        let mut sections: Vec<Vec<(types::MenuItem, Option<Arc<rhai::AST>>)>> = Vec::new();

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
                        .call::<bool>(engine, ast, ("", "", kind.clone()))
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

            sections.push(items);
        }

        let stack_id = generate_id(5);
        // println!("{stack_id}: {sections:?}");

        let mut items = Vec::new();

        {
            let mut menu_stack = MenuStack::default();

            for section in sections {
                for section_item in section {
                    items.push(match section_item {
                        (types::MenuItem::ActionButton(action_button), ast) => {
                            let action_id = generate_id(5);
                            menu_stack.actions.entry(action_id.clone()).insert_entry(
                                FnPtrWithAst {
                                    fnptr: action_button.action,
                                    ast: ast.expect("must be set"),
                                },
                            );

                            FrontendMenuItem::ActionButton(FrontendActionButton {
                                title: action_button.title,
                                dangerous: action_button.dangerous,
                                action_ref: action_id,
                            })
                        }
                    });
                }

                items.push(FrontendMenuItem::Separator);
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

    pub fn call_menustack_action(&self, menu_id: &str, action_ref: &str) {
        let menus = self.menu_stacks.read().unwrap();
        let menu = menus.get(menu_id).unwrap();
        let action = menu.actions.get(action_ref).unwrap().clone();

        let engine = self.resource_contextmenu_engine.get().unwrap();

        action.fnptr.call::<()>(engine, &action.ast, ()).unwrap();
    }

    /// Registers a script with its source code in some file system location for later use.
    pub fn register_user_script(&self, source: PathBuf) {
        self.resource_action_scripts
            .write()
            .expect("failed to lock Mutex")
            .entry(source)
            .insert_entry(UserScript { ast: None });
    }

    // Run a previously registered script
    pub fn evaluate_all(&self) -> Result<(), Box<EvalAltResult>> {
        let engine = self
            .resource_contextmenu_engine
            .get()
            .expect("Engine not initialized");

        let paths: Vec<PathBuf> = {
            let scripts = self.resource_action_scripts.read().unwrap();
            scripts.keys().cloned().collect()
        };

        for path in paths {
            let ast_arc = {
                let mut scripts = self.resource_action_scripts.write().unwrap();
                let script = scripts
                    .get_mut(&path)
                    .ok_or_else(|| format!("Unknown script: {}", path.display()))?;

                let ast_result = script
                    .ast
                    .get_or_insert_with(|| engine.compile_file(path.clone()).map(Arc::new));

                ast_result
                    .as_ref()
                    .map_err(|e| format!("Compilation failed for {}: {e}", path.display()))?
                    .clone()
            };

            engine.eval_ast::<()>(&ast_arc)?;
        }

        Ok(())
    }
}

fn generate_id(len: usize) -> String {
    use rand::RngExt as _;

    const CHARSET: &[u8] = b"abcdefghklmnpqrstuvwxyz123456789";
    let mut rng = rand::rng();

    (0..len)
        .map(|_| {
            let i = rng.random_range(0..32);
            CHARSET[i] as char
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendActionButton {
    title: String,
    action_ref: String,
    dangerous: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum FrontendMenuItem {
    ActionButton(FrontendActionButton),
    Separator,
}

#[derive(Debug, Clone, Serialize)]
pub struct MenuBlueprint {
    id: String,
    items: Vec<FrontendMenuItem>,
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn call_menustack_action(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    menustack_id: &str,
    action_ref: &str,
) -> Result<(), BackendError> {
    let clusters = app.state::<ClusterRegistryState>();
    let facade = clusters.scripting_for(&context_source)?;
    facade.call_menustack_action(menustack_id, action_ref);

    Ok(())
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn create_resource_menustack(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    gvk: kube::api::GroupVersionKind,
    namespace: &str,
    name: &str,
) -> Result<MenuBlueprint, BackendError> {
    crate::internal::tracing::set_span_request_id();

    let clusters = app.state::<ClusterRegistryState>();
    let facade = clusters.scripting_for(&context_source)?;
    let discovery = clusters.discovery_cache_for(&context_source)?;
    let client = clusters.client_for(&context_source)?;

    let (api_resource, capabilities) = discovery
        .resolve_gvk(&gvk)
        .ok_or("GroupVersionKind not found")?;

    let api = match capabilities.scope {
        kube::discovery::Scope::Cluster => {
            kube::Api::<DynamicObject>::all_with(client, &api_resource)
        }
        kube::discovery::Scope::Namespaced => match namespace {
            "" => kube::Api::all_with(client, &api_resource),
            namespace => kube::Api::namespaced_with(client, namespace, &api_resource),
        },
    };

    let obj = api.get(name).await?;
    let blueprint = facade.create_resource_menustack(obj);

    println!("{blueprint:?}");

    Ok(blueprint)
}

#[cfg(test)]
mod tests {
    use kube::api::DynamicObject;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test() {
        let client = kube::Client::try_default()
            .await
            .expect("Failed to create Kubernetes client");

        let discovery = Arc::new(
            kube::Discovery::new(client.clone())
                .run_aggregated()
                .await
                .unwrap(),
        );

        let engine = ResourceContextMenuFacade::new();
        engine.initialize_engines(client.clone(), Arc::clone(&discovery));

        engine.register_user_script("/home/christian/Downloads/test.rhai".into());

        if let Err(e) = engine.evaluate_all() {
            eprintln!("Runtime error: {e}");
        }

        let (ar, _) = discovery.get("").unwrap().recommended_kind("Pod").unwrap();

        let api: kube::Api<DynamicObject> =
            kube::Api::namespaced_with(client, "monitoring-system", &ar);
        let pod = api
            .get("alertmanager-kube-prometheus-stack-alertmanager-0")
            .await
            .unwrap();

        let blueprint = engine.create_resource_menustack(pod);
        println!("{blueprint:?}");
        let first_action = {
            blueprint
                .items
                .iter()
                .filter_map(|i: &FrontendMenuItem| match i {
                    FrontendMenuItem::ActionButton(b) => Some(b.clone()),
                    _ => None,
                })
                .collect::<Vec<FrontendActionButton>>()
                .first()
                .unwrap()
                .action_ref
                .clone()
        };
        engine.call_menustack_action(&blueprint.id, &first_action);
        engine.drop_resource_menustack(&blueprint.id);
    }
}
