use kube::{
    api::{DeleteParams, DynamicObject},
    Api,
};
use tauri::State;
use tracing::info;

use crate::{
    cluster_discovery::ClusterRegistryState, frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn delete_resource(
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    gvk: kube::api::GroupVersionKind,
    namespace: &str,
    name: &str,
    dry_run: Option<bool>,
) -> Result<(), BackendError> {
    info!("Deleting {:?} in namespace {}", gvk, namespace);

    let client = clusters.client_for(&context_source)?;

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
