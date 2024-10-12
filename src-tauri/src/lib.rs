// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod resource_event;

use std::{collections::HashMap, sync::Mutex};

use futures::{StreamExt, TryStreamExt};
use kube::{
    api::{DynamicObject, GroupVersionKind},
    discovery::verbs,
    Api, Discovery,
};
use resource_event::WatchEvent;
use tauri::{ipc::Channel, AppHandle, Manager};

struct AppState {
    channel_handlers: HashMap<u32, tokio::task::JoinHandle<()>>,
}

#[tauri::command]
async fn kube_discover() -> Result<HashMap<String, Vec<(String, String)>>, ()> {
    let client = kube::Client::try_default()
        .await
        .expect("expected default kubernetes client");

    let discovery = Discovery::new(client.clone()).run().await.unwrap();

    let mut kinds = HashMap::<String, Vec<(String, String)>>::new();

    for group in discovery.groups() {
        for (ar, capabilities) in group.recommended_resources() {
            if !capabilities.supports_operation(verbs::WATCH) {
                continue;
            }

            let g = ar.group;
            let v = ar.version;
            let k = ar.kind;

            if !kinds.contains_key(&g) {
                kinds.insert(g.clone(), vec![]);
            }

            kinds.get_mut(&g).unwrap().push((k, v));
        }
    }

    Ok(kinds)
}

#[tauri::command]
async fn kube_watch_gvk(
    app: AppHandle,
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

    let channel_id = channel.id();
    println!("Streaming {kind} to channel {channel_id}");

    let handle = tokio::spawn(async move {
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
    });

    let app_state = app.state::<Mutex<AppState>>();
    let mut app_state = app_state.lock().unwrap();

    app_state.channel_handlers.insert(channel_id, handle);

    Ok(())
}

#[tauri::command]
fn cleanup_channel(app: AppHandle, id: u32) {
    println!("Clean up channel {id}");

    let app_state = app.state::<Mutex<AppState>>();
    let mut app_state = app_state.lock().unwrap();

    if !app_state.channel_handlers.contains_key(&id) {
        return;
    }

    let handler = app_state.channel_handlers.get(&id).unwrap();
    handler.abort();

    app_state.channel_handlers.remove(&id);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState {
                channel_handlers: HashMap::new(),
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            kube_watch_gvk,
            kube_discover,
            cleanup_channel
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
