// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod frontend_commands;
mod frontend_types;
mod resource_views;

use std::sync::Mutex;

use resource_views::ResourceView;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let script = r#"
        #{
            name: "Pods (default)",
            matchApiVersion: "v1",
            matchKind: "Pod",
            columns: [
                #{
                    title: "Namespace",
                    accessor: |obj| {
                        obj.metadata.namespace
                    }
                },
                #{
                    title: "Name",
                    accessor: |obj| {
                        obj.metadata.name
                    }
                }
            ]
        }
    "#;

    let view = match ResourceView::new(script) {
        Ok(view) => view,
        Err(error) => panic!("{error}"),
    };

    let mut pod = k8s_openapi::api::core::v1::Pod {
        ..Default::default()
    };

    pod.metadata.namespace = Some("kube-system".into());
    pod.metadata.name = Some("some-pod".into());

    let pod = pod;

    let cols = view.render_columns(&pod);

    println!("{:?}", pod);
    println!("{:?}", view.render_titles());
    println!("{:?}", cols);

    // view.columns.iter().for_each(|column| {
    //     match context.eval_fnptr::<String>(&column.accessor, ("hello",)) {
    //         Ok(res) => println!("Success! We got: {res}"),
    //         Err(e) => eprintln!("{e}"),
    //     }
    // });

    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(app_state::AppState::new()));
            app.manage(Mutex::new(app_state::KubernetesClientRegistry::new()));
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
