use std::{collections::HashMap, sync::Mutex};

use tauri::Manager;

use crate::frontend_types::BackendError;

pub struct AppState {
    pub kubernetes_client: Option<Box<kube::Client>>,
    pub channel_handlers: HashMap<u32, tokio::task::JoinHandle<()>>,
    pub podlog_stream_handles: HashMap<u32, tokio::task::JoinHandle<()>>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            kubernetes_client: None,
            channel_handlers: HashMap::new(),
            podlog_stream_handles: HashMap::new(),
        }
    }
}

pub fn clone_client(app: &tauri::AppHandle) -> Result<kube::Client, BackendError> {
    let client;
    {
        let app_state = app.state::<Mutex<AppState>>();
        let app_state = app_state
            .lock()
            .map_err(|x| BackendError::Generic(x.to_string()))?;

        client = app_state
            .kubernetes_client
            .clone()
            .ok_or(BackendError::Generic(
                "Kubernetes Client not yet initialized".into(),
            ))?;
    }

    Ok(*client)
}
