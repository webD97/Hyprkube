// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod cluster_profiles;
mod frontend_commands;
mod frontend_types;
mod internal;
mod persistence;
mod resource_rendering;

use std::sync::Arc;

use app_state::{
    ChannelTasks, ExecSessions, JoinHandleStoreState, KubernetesClientRegistry, RendererRegistry,
};
use persistence::{cluster_profile_service::ClusterProfileService, Repository};
use tauri::{async_runtime::spawn, Listener, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            app.manage(RendererRegistry::new_state(app_handle.clone()));
            app.manage(KubernetesClientRegistry::new_state());
            app.manage(ChannelTasks::new_state(app_handle.clone()));
            app.manage(ExecSessions::new_state());

            let mut cluster_profile_registry =
                cluster_profiles::ClusterProfileRegistry::new(app_handle.clone());
            cluster_profile_registry.ensure_default_profile().unwrap();
            cluster_profile_registry.scan_profiles();
            app.manage(Arc::new(cluster_profile_registry));

            let repo = Arc::new(Repository::new(app.handle().clone()));
            app.manage(repo.clone());

            let pinned_cluster_profile_service =
                ClusterProfileService::new(app.handle().clone(), repo.clone());
            app.manage(pinned_cluster_profile_service);

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
            frontend_commands::pod_exec_start_session,
            frontend_commands::pod_exec_write_stdin,
            frontend_commands::pod_exec_abort_session,
            frontend_commands::pod_exec_resize_terminal,
            frontend_commands::list_pod_container_names,
            frontend_commands::get_resource_yaml,
            frontend_commands::apply_resource_yaml,
            cluster_profiles::list_cluster_profiles,
            frontend_commands::restart_deployment,
            frontend_commands::restart_statefulset,
            cluster_profiles::cluster_profile_add_pinned_gvk,
            cluster_profiles::cluster_profile_remove_pinned_gvk,
            cluster_profiles::cluster_profile_list_pinned_gvks,
            cluster_profiles::cluster_profile_add_hidden_gvk,
            cluster_profiles::cluster_profile_remove_hidden_gvk,
            cluster_profiles::cluster_profile_list_hidden_gvks,
            cluster_profiles::get_default_namespace,
            cluster_profiles::set_default_namespace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn reset_state(app_handle: tauri::AppHandle) {
    let join_handle_store = app_handle.state::<JoinHandleStoreState>();
    join_handle_store.abort_all();
}
