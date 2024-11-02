use futures::{StreamExt as _, TryStreamExt as _};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::{
    app_state::{JoinHandleStoreState, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum PlainWatchStreamEvent {
    Created {
        uid: String,
        namespace: String,
        name: String,
    },
    Updated {
        uid: String,
        namespace: String,
        name: String,
    },
    Deleted {
        uid: String,
        namespace: String,
        name: String,
    },
}

#[tauri::command]
pub async fn watch_gvk_plain(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    client_id: Uuid,
    gvk: kube::api::GroupVersionKind,
    channel: tauri::ipc::Channel<PlainWatchStreamEvent>,
    namespace: Option<&str>,
) -> Result<(), BackendError> {
    let channel_id = channel.id();
    println!("Streaming plain names {:?} in namespace {:?} to channel {channel_id}", gvk, namespace);

    let client = client_registry_arc.try_clone(&client_id)?;

    let (api_resource, _) = kube::discovery::oneshot::pinned_kind(&client, &gvk).await?;

    let api: kube::Api<kube::api::DynamicObject> = match namespace {
        None => kube::Api::all_with(client, &api_resource),
        Some(namespace) => kube::Api::namespaced_with(client, namespace, &api_resource),
    };

    let mut stream = api
        .watch(&kube::api::WatchParams::default(), "0")
        .await?
        .boxed();

    let stream = async move {
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
                Some(kube::api::WatchEvent::Added(obj)) => Some(PlainWatchStreamEvent::Created {
                    uid: obj.metadata.uid.expect("no uid"),
                    namespace: obj.metadata.namespace.or(Some("".into())).unwrap(),
                    name: obj.metadata.name.or(Some("".into())).unwrap(),
                }),
                Some(kube::api::WatchEvent::Modified(obj)) => {
                    Some(PlainWatchStreamEvent::Updated {
                        uid: obj.metadata.uid.expect("no uid"),
                        namespace: obj.metadata.namespace.or(Some("".into())).unwrap(),
                        name: obj.metadata.name.or(Some("".into())).unwrap(),
                    })
                }
                Some(kube::api::WatchEvent::Deleted(obj)) => Some(PlainWatchStreamEvent::Deleted {
                    namespace: obj.metadata.namespace.or(Some("".into())).unwrap(),
                    name: obj.metadata.name.or(Some("".into())).unwrap(),
                    uid: obj.metadata.uid.expect("no uid"),
                }),
                Some(kube::api::WatchEvent::Bookmark(_obj)) => None,
                Some(kube::api::WatchEvent::Error(error)) => {
                    eprintln!("{error}");
                    return;
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
    };

    join_handle_store.submit(channel_id, stream);

    Ok(())
}
