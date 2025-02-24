use kube::api::DynamicObject;
use tauri::State;
use uuid::Uuid;

use crate::{app_state::KubernetesClientRegistryState, frontend_types::BackendError};

#[tauri::command]
pub async fn apply_resource_yaml(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    client_id: Uuid,
    gvk: kube::api::GroupVersionKind,
    namespace: &str,
    name: &str,
    new_yaml: &str,
) -> Result<String, BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;

    let (api_resource, resource_capabilities) =
        kube::discovery::oneshot::pinned_kind(&client, &gvk).await?;

    let api: kube::Api<DynamicObject> = match resource_capabilities.scope {
        kube::discovery::Scope::Cluster => kube::Api::all_with(client, &api_resource),
        kube::discovery::Scope::Namespaced => match namespace {
            "" => kube::Api::all_with(client, &api_resource),
            namespace => kube::Api::namespaced_with(client, namespace, &api_resource),
        },
    };

    let obj: DynamicObject = serde_yaml::from_str(new_yaml).unwrap();

    let updated_object = api
        .replace(name, &kube::api::PostParams::default(), &obj)
        .await
        .map_err(|e| match e {
            kube::Error::Api(api_error) => {
                BackendError::Generic(serde_json::to_string(&api_error).unwrap())
            }
            e => BackendError::Generic(e.to_string()),
        })?;

    Ok(serde_yaml::to_string(&updated_object).unwrap())
}
