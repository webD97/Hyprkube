use std::sync::Arc;

use kube::{
    api::{ApiResource, DynamicObject},
    Api, Discovery,
};
use rhai::{FuncRegistration, Module};

pub fn build_module(client: kube::Client, discovery: Arc<kube::Discovery>) -> Module {
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
                block_on(async {
                    let ar = api_resource_for(api_version, kind, &discovery)?;
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
                block_on(async {
                    let ar = api_resource_for(api_version, kind, &discovery)?;
                    let api: Api<DynamicObject> = Api::all_with(client.clone(), &ar);
                    let resource = api.get(name).await.map_err(|e| e.to_string())?;

                    Ok(rhai::serde::to_dynamic(resource)?.cast::<rhai::Map>())
                })
            },
        );
    }

    kube_module
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

fn block_on<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(future))
}
