use k8s_openapi::api::core::v1;
use tauri::State;

use crate::{
    app_state::{ClientId, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn list_secret_keys(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    client_id: ClientId,
    namespace: &str,
    name: &str,
) -> Result<Vec<String>, BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;
    let api: kube::Api<v1::Secret> = kube::Api::namespaced(client, namespace);
    let secret = api.get(name).await.unwrap();

    let keys = secret
        .data
        .map(|data| data.keys().cloned().collect())
        .unwrap_or(Vec::new());

    Ok(keys)
}
