use std::sync::Mutex;

use futures::{StreamExt as _, TryStreamExt as _};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::{
    app_state::KubernetesClientRegistry,
    frontend_types::{BackendError, FrontendValue},
    state::ViewRegistry,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum WatchStreamEvent {
    #[serde(rename_all = "camelCase")]
    AnnounceColumns {
        titles: Vec<String>,
    },
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
    client_registry_arc: State<'_, Mutex<KubernetesClientRegistry>>,
    view_registry: State<'_, ViewRegistry>,
    client_id: Uuid,
    gvk: kube::api::GroupVersionKind,
    channel: tauri::ipc::Channel<WatchStreamEvent>,
) -> Result<(), BackendError> {
    let client = client_registry_arc.lock().unwrap().try_clone(&client_id)?;
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

    let column_titles = view_registry.render_default_column_titles_for_gvk(&gvk);

    channel
        .send(WatchStreamEvent::AnnounceColumns {
            titles: column_titles,
        })
        .unwrap();

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
                let columns = view_registry.render_default_view_for_gvk(&gvk, &obj);
                Some(WatchStreamEvent::Created {
                    uid: obj.metadata.uid.expect("no uid"),
                    columns,
                })
            }
            Some(kube::api::WatchEvent::Modified(obj)) => {
                let columns = view_registry.render_default_view_for_gvk(&gvk, &obj);
                Some(WatchStreamEvent::Updated {
                    uid: obj.metadata.uid.expect("no uid"),
                    columns,
                })
            }
            Some(kube::api::WatchEvent::Deleted(obj)) => Some(WatchStreamEvent::Deleted {
                uid: obj.metadata.uid.expect("no uid"),
            }),
            Some(kube::api::WatchEvent::Bookmark(_obj)) => None,
            Some(kube::api::WatchEvent::Error(error)) => {
                eprintln!("{error}");
                return Err(BackendError::Generic(format!(
                    "Something unexpected happened: {:?}",
                    error
                )));
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
}
