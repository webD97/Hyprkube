use kube::api::DynamicObject;
use tauri::State;

use crate::{
    cluster_discovery::ClusterRegistryState, frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn get_resource_yaml(
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    gvk: kube::api::GroupVersionKind,
    namespace: &str,
    name: &str,
) -> Result<String, BackendError> {
    let client = clusters.client_for(&context_source)?;

    let (api_resource, resource_capabilities) =
        kube::discovery::oneshot::pinned_kind(&client, &gvk).await?;

    let api = match resource_capabilities.scope {
        kube::discovery::Scope::Cluster => kube::Api::all_with(client, &api_resource),
        kube::discovery::Scope::Namespaced => match namespace {
            "" => kube::Api::all_with(client, &api_resource),
            namespace => kube::Api::namespaced_with(client, namespace, &api_resource),
        },
    };

    let pod: DynamicObject = api.get(name).await.unwrap();

    let yaml = serde_yaml::to_string(&pod).unwrap();

    Ok(yaml)
}
