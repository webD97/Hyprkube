use std::sync::Mutex;

use tauri::Manager as _;

use crate::app_state::AppState;

#[tauri::command]
pub async fn initialize_kube_client(app: tauri::AppHandle) -> Result<(), String> {
    let client = match kube::Client::try_default().await {
        Ok(client) => client,
        Err(error) => return Err(error.to_string()),
    };

    let app_state = app.state::<Mutex<AppState>>();
    let mut app_state = app_state.lock().unwrap();

    app_state.kubernetes_client = Some(Box::new(client));

    Ok(())
}
