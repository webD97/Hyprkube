use kube::{
    api::{DeleteParams, DynamicObject},
    Api,
};
use tauri::State;
use tracing::info;

use crate::{
    app_state::{ClientId, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn delete_resource(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    client_id: ClientId,
    gvk: kube::api::GroupVersionKind,
    namespace: &str,
    name: &str,
    dry_run: Option<bool>,
) -> Result<(), BackendError> {
    info!("Deleting {:?} in namespace {}", gvk, namespace);

    let client = client_registry_arc.try_clone(&client_id)?;

    // todo: Avoid discovery and use what we already have in cache
    let (api_resource, resource_capabilities) =
        kube::discovery::oneshot::pinned_kind(&client, &gvk).await?;

    let api: Api<DynamicObject> = match resource_capabilities.scope {
        kube::discovery::Scope::Cluster => kube::Api::all_with(client, &api_resource),
        kube::discovery::Scope::Namespaced => {
            kube::Api::namespaced_with(client, namespace, &api_resource)
        }
    };

    let params = DeleteParams {
        dry_run: dry_run.unwrap_or(false),
        ..DeleteParams::default()
    };

    api.delete(name, &params).await?;

    Ok(())
}
