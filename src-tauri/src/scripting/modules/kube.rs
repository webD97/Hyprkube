use std::sync::{Arc, OnceLock};

use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, StatefulSet};
use kube::{
    api::{ApiResource, DeleteParams, DynamicObject, Patch, PatchParams},
    core::{gvk::ParseGroupVersionError, GroupVersion},
    Api, Discovery,
};
use rhai::{FuncRegistration, Module};

use crate::scripting::types::ResourceRef;

pub fn build_module<F>(client: kube::Client, discovery: F) -> Module
where
    F: Fn() -> Arc<OnceLock<Arc<kube::Discovery>>> + Send + Sync + 'static,
{
    let mut kube_module = Module::new();
    let discovery = Arc::new(discovery);

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
                let discovery = discovery();
                let discovery = get_cache(discovery.get())?;

                block_on(async {
                    let ar = api_resource_for(api_version, kind, discovery)?;
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
                let discovery = discovery();
                let discovery = get_cache(discovery.get())?;

                block_on(async {
                    let ar = api_resource_for(api_version, kind, discovery)?;
                    let api: Api<DynamicObject> = Api::all_with(client.clone(), &ar);
                    let resource = api.get(name).await.map_err(|e| e.to_string())?;

                    Ok(rhai::serde::to_dynamic(resource)?.cast::<rhai::Map>())
                })
            },
        );
    }

    {
        let client = client.clone();
        let discovery = Arc::clone(&discovery);

        FuncRegistration::new("delete").set_into_module(
            &mut kube_module,
            move |api_version: &str,
                  kind: &str,
                  namespace: &str,
                  name: &str|
                  -> Result<(), Box<rhai::EvalAltResult>> {
                let discovery = discovery();
                let discovery = get_cache(discovery.get())?;

                block_on(async {
                    let ar = api_resource_for(api_version, kind, discovery)?;
                    let api: Api<DynamicObject> =
                        Api::namespaced_with(client.clone(), namespace, &ar);

                    api.delete(name, &DeleteParams::default())
                        .await
                        .map_err(|e| e.to_string())?;

                    Ok(())
                })
            },
        );
    }

    {
        let client = client.clone();
        let discovery = Arc::clone(&discovery);

        FuncRegistration::new("patch_merge").set_into_module(
            &mut kube_module,
            move |api_version: &str,
                  kind: &str,
                  namespace: &str,
                  name: &str,
                  patch: rhai::Map|
                  -> Result<(), Box<rhai::EvalAltResult>> {
                let discovery = discovery();
                let discovery = get_cache(discovery.get())?;

                block_on(async {
                    let ar = api_resource_for(api_version, kind, discovery)?;
                    let api: Api<DynamicObject> =
                        Api::namespaced_with(client.clone(), namespace, &ar);

                    api.patch(name, &PatchParams::default(), &Patch::Merge(patch))
                        .await
                        .map_err(|e| e.to_string())?;

                    Ok(())
                })
            },
        );
    }

    {
        let client = client.clone();

        FuncRegistration::new("rollout_restart").set_into_module(
            &mut kube_module,
            move |resource_ref: ResourceRef| -> Result<(), Box<rhai::EvalAltResult>> {
                block_on(async {
                    match resource_ref.kind.as_str() {
                        "Deployment" => {
                            let api: Api<Deployment> = Api::namespaced(
                                client.clone(),
                                &resource_ref.namespace.unwrap_or_default(),
                            );

                            api.restart(&resource_ref.name)
                                .await
                                .map_err(|e| e.to_string())?;
                        }
                        "StatefulSet" => {
                            let api: Api<StatefulSet> = Api::namespaced(
                                client.clone(),
                                &resource_ref.namespace.unwrap_or_default(),
                            );

                            api.restart(&resource_ref.name)
                                .await
                                .map_err(|e| e.to_string())?;
                        }
                        "DaemonSet" => {
                            let api: Api<DaemonSet> = Api::namespaced(
                                client.clone(),
                                &resource_ref.namespace.unwrap_or_default(),
                            );

                            api.restart(&resource_ref.name)
                                .await
                                .map_err(|e| e.to_string())?;
                        }
                        _ => panic!(),
                    };

                    Ok(())
                })
            },
        );
    }

    kube_module
}

fn get_cache(discovery: Option<&Arc<Discovery>>) -> Result<&Discovery, String> {
    if let Some(discovery) = discovery {
        return Ok(discovery);
    }

    Err("kube module does not have access to a discovery cache, cannot operate".to_owned())
}

fn api_resource_for(
    api_version: &str,
    kind: &str,
    discovery: &Discovery,
) -> Result<ApiResource, String> {
    let gv: GroupVersion = api_version
        .parse()
        .map_err(|e: ParseGroupVersionError| e.to_string())?;

    let (ar, _) = discovery
        .get(&gv.group)
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
