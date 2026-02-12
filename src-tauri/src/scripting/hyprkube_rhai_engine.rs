use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use kube::{
    api::{ApiResource, DynamicObject},
    Api, Discovery,
};
use rhai::{exported_module, CallFnOptions, Dynamic, EvalAltResult, FuncRegistration, Module};

use crate::scripting::{modules, types::ResourceRef};

struct ContextMenuSection<'a> {
    pub matcher: rhai::FnPtr,
    pub matcher_context: rhai::NativeCallContext<'a>,

    pub items: rhai::FnPtr,
    pub items_context: rhai::NativeCallContext<'a>,
}

pub struct ScriptingFacade<'a> {
    engine: rhai::Engine,

    /// Handles scripts that render a resource context menu
    resource_contextmenu_engine: rhai::Engine,
    registered_contextmenu_sections: RwLock<HashMap<String, ContextMenuSection<'a>>>,

    resource_action_scripts: Mutex<HashMap<PathBuf, UserScript>>,
}

pub struct UserScript {
    ast: Option<Result<rhai::AST, Box<EvalAltResult>>>,
}

#[allow(dead_code)]
impl<'a> ScriptingFacade<'a> {
    pub async fn new(client: kube::Client, discovery: Arc<kube::Discovery>) -> Self {
        Self {
            engine: rhai::Engine::new(),
            resource_contextmenu_engine: Self::make_resource_contextmenu_engine(client, discovery),
            registered_contextmenu_sections: RwLock::new(HashMap::new()),
            resource_action_scripts: Mutex::new(HashMap::new()),
        }
    }

    fn api_resource_for(
        api_version: &str,
        kind: &str,
        discovery: &Discovery,
    ) -> Result<ApiResource, String> {
        let (ar, _) = discovery
            .get(api_version)
            .ok_or(format!("ApiVersion not found: {}", api_version))?
            .recommended_kind(kind)
            .ok_or(format!("Kind not found: {}", kind))?;

        Ok(ar)
    }

    fn make_resource_contextmenu_engine(
        client: kube::Client,
        discovery: Arc<kube::Discovery>,
    ) -> rhai::Engine {
        let mut engine = rhai::Engine::new();

        engine.build_type::<ResourceRef>();
        engine.register_static_module("kube", Self::make_kube_module(client, discovery).into());
        engine.register_static_module("base64", exported_module!(modules::base64_rhai).into());

        engine
    }

    /// Runs all scripts that contribute to the given resource's context menu and returns the specification.
    #[allow(unused)]
    pub fn evaluate_resource_contextmenu(&self, obj: kube::api::DynamicObject) {
        let api_version = obj.types.as_ref().unwrap().api_version.to_owned();
        let kind = obj.types.as_ref().unwrap().kind.to_owned();
        let obj = rhai::serde::to_dynamic(obj).unwrap();

        let sections_defs = self.registered_contextmenu_sections.read().unwrap();

        let mut sections: Vec<Vec<Dynamic>> = Vec::new();

        for section_def in sections_defs.values() {
            let matches = section_def
                .matcher
                .call_within_context::<bool>(&section_def.matcher_context, ("", "", kind.clone()))
                .unwrap();

            if !matches {
                continue;
            }

            let section_items = section_def
                .items
                .call_within_context::<rhai::Array>(&section_def.items_context, (obj.clone(),))
                .unwrap();

            sections.push(section_items);
        }

        todo!();
    }

    fn make_kube_module(client: kube::Client, discovery: Arc<kube::Discovery>) -> Module {
        let mut kube_module = Module::new();

        {
            let client = client.clone();
            let discovery = Arc::clone(&discovery);

            FuncRegistration::new("get").set_into_module(
                &mut kube_module,
                move |api_version: &str,
                      kind: &str,
                      namespace: &str,
                      name: &str|
                      -> Result<rhai::Map, Box<rhai::EvalAltResult>> {
                    Self::block_on(async {
                        let ar = Self::api_resource_for(api_version, kind, &discovery)?;

                        let api: Api<DynamicObject> =
                            Api::namespaced_with(client.clone(), namespace, &ar);

                        let resource = api.get(name).await.map_err(|e| e.to_string())?;

                        Ok(rhai::serde::to_dynamic(resource)?.cast::<rhai::Map>())
                    })
                },
            );
        }

        {
            let client = client.clone();
            let discovery = Arc::clone(&discovery);

            FuncRegistration::new("get").set_into_module(
                &mut kube_module,
                move |api_version: &str,
                      kind: &str,
                      name: &str|
                      -> Result<rhai::Map, Box<rhai::EvalAltResult>> {
                    Self::block_on(async {
                        let ar = Self::api_resource_for(api_version, kind, &discovery)?;

                        let api: kube::Api<kube::api::DynamicObject> =
                            kube::Api::all_with(client.clone(), &ar);

                        let resource = api.get(name).await.map_err(|e| e.to_string())?;

                        Ok(rhai::serde::to_dynamic(resource)?.cast::<rhai::Map>())
                    })
                },
            );
        }

        kube_module
    }

    /// Registers a script with its source code in some file system location for later use.
    pub fn register_user_script(&self, source: PathBuf) {
        self.resource_action_scripts
            .lock()
            .expect("failed to lock Mutex")
            .entry(source)
            .insert_entry(UserScript { ast: None });
    }

    // Run a previously registered script
    pub fn run_user_script(&self, path: PathBuf) -> Result<(), Box<EvalAltResult>> {
        let mut scripts = self
            .resource_action_scripts
            .lock()
            .expect("failed to lock Mutex");

        let script = scripts
            .get_mut(&path)
            .ok_or_else(|| format!("Unknown script: {}", path.to_string_lossy()))?;

        let ast = script.ast.get_or_insert_with(|| {
            tracing::debug!("Compiling script {}", path.to_string_lossy());
            self.engine.compile_file(path)
        });

        let ast = ast
            .as_ref()
            .map_err(|e| format!("Cannot execute script with compilation error: {}", e))?;

        self.engine.call_fn_with_options(
            CallFnOptions::new().eval_ast(false).rewind_scope(true),
            &mut rhai::Scope::new(),
            ast,
            "main",
            (ResourceRef {
                api_version: "v1".into(),
                kind: "Pod".into(),
                namespace: Some("smart-home".into()),
                name: "home-assistant-0".into(),
            },),
        )
    }

    fn block_on<F, T>(future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test() {
        let client = kube::Client::try_default()
            .await
            .expect("Failed to create Kubernetes client");

        let discovery = kube::Discovery::new(client.clone())
            .run_aggregated()
            .await
            .unwrap();

        let engine = ScriptingFacade::new(client, Arc::new(discovery)).await;

        let script: PathBuf =
            "/home/christian/Repositories/github.com/webd97/_sandbox/kube-rhai/src/script.rhai"
                .into();
        engine.register_user_script(script.clone());
        if let Err(e) = engine.run_user_script(script) {
            eprintln!("Runtime error: ${e}");
        }
    }
}
