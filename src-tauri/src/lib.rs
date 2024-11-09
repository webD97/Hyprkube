// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod dirs;
mod frontend_commands;
mod frontend_types;
mod resource_rendering;

use app_state::{ChannelTasks, JoinHandleStoreState, KubernetesClientRegistry, RendererRegistry};
use tauri::{async_runtime::spawn, Listener, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            app.manage(RendererRegistry::new_state(app_handle.clone()));
            app.manage(KubernetesClientRegistry::new_state());
            app.manage(ChannelTasks::new_state(app_handle.clone()));

            app.listen("frontend-onbeforeunload", move |_event| {
                println!("ONBEFOREUNLOAD");
                let app_handle = app_handle.clone();
                spawn(async move {
                    reset_state(app_handle).await;
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_commands::discover_kubernetes_cluster,
            frontend_commands::kube_stream_podlogs,
            frontend_commands::watch_gvk_with_view,
            frontend_commands::watch_namespaces,
            frontend_commands::cleanup_channel,
            frontend_commands::discover_contexts,
            frontend_commands::delete_resource,
            frontend_commands::list_resource_views,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn reset_state(app_handle: tauri::AppHandle) {
    let join_handle_store = app_handle.state::<JoinHandleStoreState>();
    join_handle_store.abort_all();
}
