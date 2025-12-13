// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod cluster_discovery;
mod cluster_profiles;
mod frontend_commands;
mod frontend_types;
mod internal;
mod persistence;
mod resource_rendering;

use std::sync::Arc;

use app_state::{ChannelTasks, ExecSessions, JoinHandleStoreState, RendererRegistry};
use persistence::cluster_profile_service::ClusterProfileService;
use tauri::{async_runtime::spawn, Emitter as _, Listener, Manager};
use tracing::{info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

use crate::{
    cluster_discovery::ClusterRegistry, frontend_types::BackendPanic,
    persistence::repository::Repository,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_tracing();

    info!("Hyprkube is starting.");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            setup_panic_handler(app_handle.clone());

            app.manage(RendererRegistry::new_state(app_handle.clone()));
            app.manage(ChannelTasks::new_state(app_handle.clone()));
            app.manage(ExecSessions::new_state());
            app.manage(Arc::new(ClusterRegistry::new()));

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
                warn!("ONBEFOREUNLOAD");
                let app_handle = app_handle.clone();
                spawn(async move {
                    reset_state(app_handle).await;
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // frontend_commands::discover_kubernetes_cluster,
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
            frontend_commands::log_stdout,
            crate::cluster_discovery::connect_cluster
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn reset_state(app_handle: tauri::AppHandle) {
    let join_handle_store = app_handle.state::<JoinHandleStoreState>();
    join_handle_store.abort_all();
}

fn setup_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .try_init()
        .expect("tracing-subscriber setup failed");
}

fn setup_panic_handler(app_handle: tauri::AppHandle) {
    use std::panic;

    let default_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        default_hook(info);

        let panic_msg = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned());

        let frontend_panic_info = BackendPanic {
            thread: std::thread::current().name().map(|s| s.to_owned()),
            location: info.location().map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column(),
                )
            }),
            message: panic_msg,
        };

        if let Err(e) = app_handle.emit("background_task_panic", frontend_panic_info) {
            eprintln!("Failed to emit panic event to frontend: {e}");
        }
    }));
}
