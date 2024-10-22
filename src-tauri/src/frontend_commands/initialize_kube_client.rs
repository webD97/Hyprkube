use crate::{
    app_state::KubernetesClientRegistry,
    frontend_types::{BackendError, KubernetesClient},
};

#[tauri::command]
pub async fn initialize_kube_client(
    client_registry: tauri::State<'_, tokio::sync::Mutex<KubernetesClientRegistry>>,
) -> Result<KubernetesClient, BackendError> {
    let client = kube::Client::try_default().await?;

    let id = client_registry.lock().await.manage(client).await?;

    Ok(KubernetesClient { id })
}
