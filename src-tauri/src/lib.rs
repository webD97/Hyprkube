// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod resource_event;

use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::{Namespace, Node, Pod};
use kube::{api::ListParams, Api};
use resource_event::WatchEvent;
use tauri::ipc::Channel;

#[tauri::command]
async fn kube_watch_pods(channel: Channel<WatchEvent<Pod>>) -> Result<(), ()> {
    let client = kube::Client::try_default()
        .await
        .expect("expected default kubernetes client");

    let pods_api: Api<Pod> = Api::all(client.clone());

    let mut stream = pods_api
        .watch(&kube::api::WatchParams::default(), "0")
        .await
        .expect("stream")
        .boxed();

    while let Some(status) = stream.try_next().await.expect("next") {
        match status {
            kube::api::WatchEvent::Added(obj) => channel
                .send(WatchEvent::Created { repr: obj.clone() })
                .unwrap(),
            kube::api::WatchEvent::Modified(obj) => channel
                .send(WatchEvent::Updated { repr: obj.clone() })
                .unwrap(),
            kube::api::WatchEvent::Deleted(obj) => channel
                .send(WatchEvent::Deleted { repr: obj.clone() })
                .unwrap(),
            kube::api::WatchEvent::Bookmark(_obj) => {}
            kube::api::WatchEvent::Error(obj) => println!("{}", obj.message),
        }
    }

    Ok(())
}

#[tauri::command]
async fn kube_get_nodes() -> Result<Vec<Node>, ()> {
    let client = kube::Client::try_default()
        .await
        .expect("expected default kubernetes client");
    let nodes_api: Api<Node> = Api::all(client.clone());
    let node_list = nodes_api
        .list(&ListParams::default())
        .await
        .expect("expected node list");

    Ok(node_list.items)
}

#[tauri::command]
async fn kube_get_namespaces() -> Result<Vec<Namespace>, ()> {
    let client = kube::Client::try_default()
        .await
        .expect("expected default kubernetes client");
    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    let namespace_list = namespaces_api
        .list(&ListParams::default())
        .await
        .expect("expected namespace list");

    Ok(namespace_list.items)
}

#[tauri::command]
async fn kube_get_pods() -> Result<Vec<Pod>, ()> {
    let client = kube::Client::try_default()
        .await
        .expect("expected default kubernetes client");
    let namespaces_api: Api<Pod> = Api::all(client.clone());
    let namespace_list = namespaces_api
        .list(&ListParams::default())
        .await
        .expect("expected pod list");

    Ok(namespace_list.items)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            kube_get_nodes,
            kube_get_namespaces,
            kube_get_pods,
            kube_watch_pods
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
