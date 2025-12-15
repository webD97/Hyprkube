use k8s_openapi::api::apps::v1::{Deployment, StatefulSet};
use tauri::State;

use crate::{
    cluster_discovery::ClusterRegistryState, frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn restart_deployment(
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    namespace: &str,
    name: &str,
) -> Result<(), BackendError> {
    let client = clusters.get(&context_source).ok_or("not found")?.client;

    let api: kube::Api<Deployment> = kube::Api::namespaced(client, namespace);

    api.restart(name).await?;

    Ok(())
}

#[tauri::command]
pub async fn restart_statefulset(
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    namespace: &str,
    name: &str,
) -> Result<(), BackendError> {
    let client = clusters.get(&context_source).ok_or("not found")?.client;

    let api: kube::Api<StatefulSet> = kube::Api::namespaced(client, namespace);

    api.restart(name).await?;

    Ok(())
}
