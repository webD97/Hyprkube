use k8s_openapi::api::core::v1;
use tauri::State;

use crate::{
    cluster_discovery::ClusterRegistryState, frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn list_secret_keys(
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    namespace: &str,
    name: &str,
) -> Result<Vec<String>, BackendError> {
    let client = clusters.client_for(&context_source)?;
    let api: kube::Api<v1::Secret> = kube::Api::namespaced(client, namespace);
    let secret = api.get(name).await.unwrap();

    let keys = secret
        .data
        .map(|data| data.keys().cloned().collect())
        .unwrap_or(Vec::new());

    Ok(keys)
}
