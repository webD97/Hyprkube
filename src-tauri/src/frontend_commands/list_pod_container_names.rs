use k8s_openapi::api::core::v1::Pod;
use tauri::State;

use crate::{
    cluster_discovery::ClusterRegistryState, frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn list_pod_container_names(
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    namespace: &str,
    name: &str,
) -> Result<Vec<String>, BackendError> {
    let client = clusters.get(&context_source).ok_or("not found")?.client;

    let pods: kube::Api<Pod> = kube::Api::namespaced(client, namespace);

    let pod = pods.get(name).await?;

    Ok(pod
        .spec
        .expect("spec exists")
        .containers
        .iter()
        .map(|container| container.name.clone())
        .collect())
}
