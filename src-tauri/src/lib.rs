// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod frontend_commands;
mod frontend_types;
mod resource_views;
mod view_registry;

use std::sync::Mutex;

use tauri::Manager;
use view_registry::ViewRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut view_registry = ViewRegistry::default();
    view_registry.scan_directories();

    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(app_state::AppState::new()));
            app.manage(Mutex::new(app_state::KubernetesClientRegistry::new()));
            app.manage(Mutex::new(view_registry));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_commands::kube_watch_gvk,
            frontend_commands::kube_discover,
            frontend_commands::cleanup_channel,
            frontend_commands::initialize_kube_client,
            frontend_commands::kube_stream_podlogs,
            frontend_commands::kube_stream_podlogs_cleanup,
            frontend_commands::watch_gvk_with_view,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
