// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use k8s_openapi::api::core::v1::{Namespace, Node, Pod};
use kube::{api::ListParams, Api};

#[tauri::command]
async fn kube_get_nodes() -> Result<Vec<Node>, ()> {
  let client = kube::Client::try_default().await.expect("expected default kubernetes client");
  let nodes_api: Api<Node> = Api::all(client.clone());
  let node_list = nodes_api.list(&ListParams::default()).await.expect("expected node list");

  Ok(node_list.items)
}

#[tauri::command]
async fn kube_get_namespaces() -> Result<Vec<Namespace>, ()> {
  let client = kube::Client::try_default().await.expect("expected default kubernetes client");
  let namespaces_api: Api<Namespace> = Api::all(client.clone());
  let namespace_list = namespaces_api.list(&ListParams::default()).await.expect("expected namespace list");

  Ok(namespace_list.items)
}

#[tauri::command]
async fn kube_get_pods() -> Result<Vec<Pod>, ()> {
  let client = kube::Client::try_default().await.expect("expected default kubernetes client");
  let namespaces_api: Api<Pod> = Api::all(client.clone());
  let namespace_list = namespaces_api.list(&ListParams::default()).await.expect("expected pod list");

  Ok(namespace_list.items)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      kube_get_nodes,
      kube_get_namespaces,
      kube_get_pods
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
