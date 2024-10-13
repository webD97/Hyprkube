use std::sync::Mutex;

use tauri::Manager as _;

use crate::{
    app_state::KubernetesClientRegistry,
    frontend_types::{BackendError, KubernetesClient},
};

#[tauri::command]
pub async fn initialize_kube_client(
    app: tauri::AppHandle,
) -> Result<KubernetesClient, BackendError> {
    let client = kube::Client::try_default().await?;

    let app_state = app.state::<Mutex<KubernetesClientRegistry>>();
    let mut app_state = app_state
        .lock()
        .map_err(|x| BackendError::Generic(x.to_string()))?;

    let id = app_state.insert(client);

    Ok(KubernetesClient { id })
}
