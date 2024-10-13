use std::sync::Mutex;

use futures::{StreamExt as _, TryStreamExt as _};
use tauri::Manager as _;

use crate::{
    app_state::{self, AppState},
    frontend_types::{self, BackendError},
};

#[tauri::command]
pub async fn kube_watch_gvk(
    app: tauri::AppHandle,
    group: &str,
    version: &str,
    kind: &str,
    channel: tauri::ipc::Channel<frontend_types::WatchEvent<kube::api::DynamicObject>>,
) -> Result<(), BackendError> {
    let client = app_state::clone_client(&app)?;
    let disovery = kube::Discovery::new(client.clone()).run().await?;
    let gvk = &kube::api::GroupVersionKind::gvk(group, version, kind);

    let (api_resource, _) = disovery
        .resolve_gvk(&gvk)
        .ok_or(BackendError::Generic(format!(
            "API resource {kind} not found in {group}/{version}."
        )))?;

    let api: kube::Api<kube::api::DynamicObject> =
        kube::Api::all_with(client.clone(), &api_resource);

    let mut stream = api
        .watch(&kube::api::WatchParams::default(), "0")
        .await?
        .boxed();

    let channel_id = channel.id();
    println!("Streaming {kind} to channel {channel_id}");

    let handle = tokio::spawn(async move {
        loop {
            let status = stream.try_next().await;
            let event = match status {
                Ok(event) => event,
                Err(error) => {
                    eprintln!("{error}");
                    None
                }
            };

            let to_send = match event {
                Some(kube::api::WatchEvent::Added(obj)) => {
                    Some(frontend_types::WatchEvent::Created { repr: obj })
                }
                Some(kube::api::WatchEvent::Modified(obj)) => {
                    Some(frontend_types::WatchEvent::Updated { repr: obj })
                }
                Some(kube::api::WatchEvent::Deleted(obj)) => {
                    Some(frontend_types::WatchEvent::Deleted { repr: obj })
                }
                Some(kube::api::WatchEvent::Bookmark(_obj)) => None,
                Some(kube::api::WatchEvent::Error(error)) => {
                    eprintln!("{error}");
                    None
                }
                None => None,
            };

            if let Some(message) = to_send {
                match channel.send(message) {
                    Ok(()) => (),
                    Err(error) => eprintln!("error sending to channel: {error}"),
                }
            }
        }
    });

    let app_state = app.state::<Mutex<AppState>>();
    let mut app_state = app_state
        .lock()
        .map_err(|x| BackendError::Generic(x.to_string()))?;

    app_state.channel_handlers.insert(channel_id, handle);

    Ok(())
}
