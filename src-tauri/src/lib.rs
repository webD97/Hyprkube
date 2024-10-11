// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod resource_event;

use futures::{StreamExt, TryStreamExt};
use kube::{
    api::{DynamicObject, GroupVersionKind},
    Api, Discovery,
};
use resource_event::WatchEvent;
use tauri::ipc::Channel;

#[tauri::command]
async fn kube_watch_gvk(
    group: &str,
    version: &str,
    kind: &str,
    channel: Channel<WatchEvent<DynamicObject>>,
) -> Result<(), ()> {
    let client = kube::Client::try_default()
        .await
        .expect("expected default kubernetes client");

    let (api_resource, _) = Discovery::new(client.clone())
        .run()
        .await
        .unwrap()
        .resolve_gvk(&GroupVersionKind::gvk(group, version, kind))
        .unwrap();

    let api: Api<DynamicObject> = Api::all_with(client.clone(), &api_resource);

    let mut stream = api
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![kube_watch_gvk])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
