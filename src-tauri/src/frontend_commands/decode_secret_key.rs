use k8s_openapi::{api::core::v1, ByteString};
use tauri::State;

use crate::{
    app_state::{ClientId, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn decode_secret_key(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    client_id: ClientId,
    namespace: &str,
    name: &str,
    key: &str,
) -> Result<Option<ByteString>, BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;
    let api: kube::Api<v1::Secret> = kube::Api::namespaced(client, namespace);
    let secret = api.get(name).await.unwrap();

    let value = secret.data.and_then(|data| data.get(key).cloned());

    Ok(value)
}
