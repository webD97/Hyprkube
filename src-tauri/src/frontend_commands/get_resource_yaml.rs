use kube::api::DynamicObject;
use tauri::State;
use uuid::Uuid;

use crate::{app_state::KubernetesClientRegistryState, frontend_types::BackendError};

#[tauri::command]
pub async fn get_resource_yaml(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    client_id: Uuid,
    gvk: kube::api::GroupVersionKind,
    namespace: &str,
    name: &str,
) -> Result<String, BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;

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
