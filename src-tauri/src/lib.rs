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
use tauri::{Listener, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            app.manage(Arc::new(RendererRegistry::new(app_handle.clone())));
            app.manage(Arc::new(tokio::sync::Mutex::new(
                app_state::KubernetesClientRegistry::new(),
            )));
            app.manage(Arc::new(tauri::async_runtime::Mutex::new(
                JoinHandleStore::new(app_handle.clone()),
            )));

            app.listen("frontend-onbeforeunload", move |_event| {
                println!("ONBEFOREUNLOAD");
                reset_state(app_handle.clone());
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_commands::discover_kubernetes_cluster,
            frontend_commands::kube_stream_podlogs,
            frontend_commands::watch_gvk_with_view,
            frontend_commands::cleanup_channel
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn reset_state(app_handle: tauri::AppHandle) {
    let join_handle_store = app_handle.state::<Arc<Mutex<JoinHandleStore>>>();

    let mut join_handle_store = join_handle_store.lock().unwrap();

    let _ = std::mem::replace(
        &mut *join_handle_store,
        JoinHandleStore::new(app_handle.clone()),
    );
}
