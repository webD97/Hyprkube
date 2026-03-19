// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod cluster_discovery;
mod cluster_profiles;
mod frontend_commands;
mod frontend_types;
mod internal;
// mod kubers;
mod logging;
mod panic_handler;
mod persistence;
mod resource_menu;
mod resource_rendering;
mod scripting;

use app_state::{ChannelTasks, ExecSessions};
use persistence::cluster_profile_service::ClusterProfileService;
use tauri::{async_runtime::spawn, Listener, Manager as _};
use tracing::info;

use crate::{
    app_state::{ClusterStateRegistry, ManagedState},
    cluster_profiles::ClusterProfileRegistry,
    persistence::repository::Repository,
    scripting::scripts_provider::ScriptsProvider,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::setup().expect("failed to configure logging");

    info!("Hyprkube is starting.");

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            panic_handler::setup(app_handle.clone());

            app.manage(ChannelTasks::build(app_handle.clone()));
            app.manage(ExecSessions::build(app_handle.clone()));
            app.manage(ClusterStateRegistry::build(app_handle.clone()));
            app.manage(ScriptsProvider::build(app_handle.clone()));
            app.manage(Repository::build(app_handle.clone()));
            app.manage(ClusterProfileService::build(app.handle().clone()));
            app.manage({
                let cluster_profile_registry = ClusterProfileRegistry::build(app_handle.clone());
                cluster_profile_registry.ensure_default_profile().unwrap();
                cluster_profile_registry.scan_profiles();
                cluster_profile_registry
            });

            app.listen("frontend-onbeforeunload", move |_event| {
                let app_handle = app_handle.clone();
                spawn(async move {
                    reset_state(app_handle).await;
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            frontend_commands::kube_stream_podlogs,
            frontend_commands::watch_gvk_with_presentation,
            frontend_commands::watch_namespaces,
            frontend_commands::cleanup_channel,
            frontend_commands::discover_contexts,
            frontend_commands::delete_resource,
            frontend_commands::list_resource_presentations,
            frontend_commands::pod_exec_start_session,
            frontend_commands::pod_exec_write_stdin,
            frontend_commands::pod_exec_abort_session,
            frontend_commands::pod_exec_resize_terminal,
            frontend_commands::get_resource_yaml,
            frontend_commands::apply_resource_yaml,
            cluster_profiles::list_cluster_profiles,
            cluster_profiles::cluster_profile_add_pinned_gvk,
            cluster_profiles::cluster_profile_remove_pinned_gvk,
            cluster_profiles::cluster_profile_list_pinned_gvks,
            cluster_profiles::cluster_profile_add_hidden_gvk,
            cluster_profiles::cluster_profile_remove_hidden_gvk,
            cluster_profiles::cluster_profile_list_hidden_gvks,
            cluster_profiles::get_default_namespace,
            cluster_profiles::set_default_namespace,
            frontend_commands::log_stdout,
            crate::cluster_discovery::connect_cluster,
            crate::cluster_discovery::get_apiserver_gitversion,
            crate::resource_menu::create_resource_menustack,
            crate::resource_menu::drop_resource_menustack,
            crate::resource_menu::call_menustack_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn reset_state(app_handle: tauri::AppHandle) {
    use crate::app_state::ManagerExt;

    let channel_tasks = ManagerExt::state::<ChannelTasks>(&app_handle);
    channel_tasks.abort_all();
}
