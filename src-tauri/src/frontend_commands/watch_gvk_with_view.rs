use std::sync::Mutex;

use futures::{StreamExt as _, TryStreamExt as _};
use serde::Serialize;
use tauri::Manager as _;
use uuid::Uuid;

use crate::{
    app_state::{AppState, KubernetesClientRegistry},
    frontend_types::{BackendError, FrontendValue},
    state::ViewRegistry,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum WatchEvent {
    #[serde(rename_all = "camelCase")]
    Created {
        uid: String,
        columns: Vec<Result<Vec<FrontendValue>, String>>,
    },
    Updated {
        uid: String,
        columns: Vec<Result<Vec<FrontendValue>, String>>,
    },
    Deleted {
        uid: String,
    },
}

#[tauri::command]
pub async fn watch_gvk_with_view(
    app: tauri::AppHandle,
    client_id: Uuid,
    gvk: kube::api::GroupVersionKind,
    channel: tauri::ipc::Channel<WatchEvent>,
) -> Result<Vec<String>, BackendError> {
    let client;
    {
        let client_registry = app.state::<Mutex<KubernetesClientRegistry>>();
        let client_registry = client_registry
            .lock()
            .map_err(|x| BackendError::Generic(x.to_string()))?;

        client = client_registry.try_clone(&client_id)?
    };

    let disovery = kube::Discovery::new(client.clone()).run().await?;

    let (api_resource, _) = disovery
        .resolve_gvk(&gvk)
        .ok_or(BackendError::Generic(format!(
            "API resource {:?} not found",
            gvk
        )))?;

    let api: kube::Api<kube::api::DynamicObject> =
        kube::Api::all_with(client.clone(), &api_resource);

    let mut stream = api
        .watch(&kube::api::WatchParams::default(), "0")
        .await?
        .boxed();

    let channel_id = channel.id();
    println!("Streaming {:?} to channel {channel_id}", gvk);

    let view_registry = app.state::<Mutex<ViewRegistry>>();
    let view_registry = view_registry.lock().unwrap();

    let view = match view_registry.get_default_for_gvk(&gvk) {
        Some(view) => view.clone(),
        None => {
            return Err(BackendError::Generic(format!(
                "No view found for {:?}",
                gvk
            )))
        }
    };

    let column_titles = view.render_titles();

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
                    let columns = view.render_columns(&obj);
                    Some(WatchEvent::Created {
                        uid: obj.metadata.uid.expect("no uid"),
                        columns,
                    })
                }
                Some(kube::api::WatchEvent::Modified(obj)) => {
                    let columns = view.render_columns(&obj);
                    Some(WatchEvent::Updated {
                        uid: obj.metadata.uid.expect("no uid"),
                        columns,
                    })
                }
                Some(kube::api::WatchEvent::Deleted(obj)) => Some(WatchEvent::Deleted {
                    uid: obj.metadata.uid.expect("no uid"),
                }),
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

    Ok(column_titles)
}
