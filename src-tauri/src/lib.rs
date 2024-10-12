// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod frontend_types;
mod frontend_commands;

use std::sync::Mutex;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(app_state::AppState::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_commands::kube_watch_gvk,
            frontend_commands::kube_discover,
            frontend_commands::cleanup_channel,
            frontend_commands::initialize_kube_client
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
