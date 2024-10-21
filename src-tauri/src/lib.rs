// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod dirs;
mod frontend_commands;
mod frontend_types;
mod resource_rendering;

use std::sync::{Arc, Mutex};

use app_state::JoinHandleStore;
use resource_rendering::RendererRegistry;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Arc::new(RendererRegistry::new()));
            app.manage(Mutex::new(app_state::KubernetesClientRegistry::new()));
            app.manage(Arc::new(Mutex::new(JoinHandleStore::new(
                app.handle().clone(),
            ))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_commands::kube_discover,
            frontend_commands::initialize_kube_client,
            frontend_commands::kube_stream_podlogs,
            frontend_commands::watch_gvk_with_view,
            frontend_commands::cleanup_channel
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
