use k8s_openapi::{api::core::v1, ByteString};
use tauri::State;

use crate::{
    cluster_discovery::ClusterRegistryState, frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn decode_secret_key(
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    namespace: &str,
    name: &str,
    key: &str,
) -> Result<Option<ByteString>, BackendError> {
    let client = clusters.get(&context_source).ok_or("not found")?.client;
    let api: kube::Api<v1::Secret> = kube::Api::namespaced(client, namespace);
    let secret = api.get(name).await.unwrap();

    let value = secret.data.and_then(|data| data.get(key).cloned());

    Ok(value)
}
