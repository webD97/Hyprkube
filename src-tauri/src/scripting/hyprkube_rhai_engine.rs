use std::sync::Arc;

use kube::{api::DynamicObject, Api};

pub struct HyprkubeRhaiEngine {
    pub engine: rhai::Engine,
}

#[allow(dead_code)]
impl HyprkubeRhaiEngine {
    pub async fn new(client: kube::Client, discovery: Arc<kube::Discovery>) -> Self {
        let mut engine = rhai::Engine::new();

        {
            let client = client.clone();
            let discovery = Arc::clone(&discovery);
            engine.register_fn(
                "get",
                move |api_version: &str,
                      kind: &str,
                      namespace: &str,
                      name: &str|
                      -> Result<rhai::Map, Box<rhai::EvalAltResult>> {
                    Self::block_on(async {
                        let (ar, _) = discovery
                            .get(api_version)
                            .ok_or(format!("ApiVersion not found: {}", api_version))?
                            .recommended_kind(kind)
                            .ok_or(format!("Kind not found: {}", kind))?;

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
            engine.register_fn(
                "get",
                move |api_version: &str,
                      kind: &str,
                      name: &str|
                      -> Result<rhai::Map, Box<rhai::EvalAltResult>> {
                    Self::block_on(async {
                        let (ar, _) = discovery
                            .get(api_version)
                            .ok_or(format!("ApiVersion not found: {}", api_version))?
                            .recommended_kind(kind)
                            .ok_or(format!("Kind not found: {}", kind))?;

                        let api: kube::Api<kube::api::DynamicObject> =
                            kube::Api::all_with(client.clone(), &ar);

                        let resource = api.get(name).await.map_err(|e| e.to_string())?;

                        Ok(rhai::serde::to_dynamic(resource)?.cast::<rhai::Map>())
                    })
                },
            );
        }

        Self { engine }
    }

    pub fn run_script(&self, script: &str) -> Result<(), Box<rhai::EvalAltResult>> {
        self.engine.run(script)
    }

    fn block_on<F, T>(future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))
    }
}
