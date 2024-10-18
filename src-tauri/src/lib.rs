// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod dirs;
mod frontend_commands;
mod frontend_types;
mod resource_rendering;

use std::sync::Mutex;

use resource_rendering::RendererRegistry;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(RendererRegistry::new());
            app.manage(Mutex::new(app_state::KubernetesClientRegistry::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_commands::kube_discover,
            frontend_commands::initialize_kube_client,
            frontend_commands::kube_stream_podlogs,
            frontend_commands::watch_gvk_with_view,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
