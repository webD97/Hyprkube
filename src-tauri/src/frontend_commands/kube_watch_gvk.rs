use std::sync::Mutex;

use futures::{StreamExt as _, TryStreamExt as _};
use tauri::Manager as _;

use crate::{app_state::{self, AppState}, frontend_types};

#[tauri::command]
pub async fn kube_watch_gvk(
    app: tauri::AppHandle,
    group: &str,
    version: &str,
    kind: &str,
    channel: tauri::ipc::Channel<frontend_types::WatchEvent<kube::api::DynamicObject> > ,
) -> Result<(), String> {
    let client = app_state::clone_client(&app)?;

    let (api_resource, _) = kube::Discovery::new(client.clone())
        .run()
        .await
        .unwrap()
        .resolve_gvk(&kube::api::GroupVersionKind::gvk(group, version, kind))
        .unwrap();

    let api: kube::Api<kube::api::DynamicObject>  = kube::Api::all_with(client.clone(), &api_resource);

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
                    .send(frontend_types::WatchEvent::Created { repr: obj.clone() })
                    .unwrap(),
                kube::api::WatchEvent::Modified(obj) => channel
                    .send(frontend_types::WatchEvent::Updated { repr: obj.clone() })
                    .unwrap(),
                kube::api::WatchEvent::Deleted(obj) => channel
                    .send(frontend_types::WatchEvent::Deleted { repr: obj.clone() })
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