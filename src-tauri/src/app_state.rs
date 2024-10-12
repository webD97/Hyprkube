use std::{collections::HashMap, sync::Mutex};

use tauri::Manager;

pub struct AppState {
    pub kubernetes_client: Option<Box<kube::Client>>,
    pub channel_handlers: HashMap<u32, tokio::task::JoinHandle<()>>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            kubernetes_client: None,
            channel_handlers: HashMap::new(),
        }
    }
}

pub fn clone_client(app: &tauri::AppHandle) -> Result<kube::Client, String> {
    let client;
    {
        let app_state = app.state::<Mutex<AppState>>();
        let app_state = app_state.lock().unwrap();
        
        client = match &app_state.kubernetes_client {
            Some(boxed_client) => (**boxed_client).clone(),
            None => return Err("Kubernetes Client not yet initialized".into()),
        };
    }

    Ok(client)
}
