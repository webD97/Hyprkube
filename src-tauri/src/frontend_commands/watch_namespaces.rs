use futures::{StreamExt as _, TryStreamExt as _};
use k8s_openapi::api::core::v1::Namespace;
use serde::Serialize;
use tauri::State;

use crate::{
    app_state::{ClientId, JoinHandleStoreState, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum WatchNamespacesEvent {
    Created(String),
    Deleted(String),
}

#[tauri::command]
pub async fn watch_namespaces(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    client_id: ClientId,
    channel: tauri::ipc::Channel<WatchNamespacesEvent>,
) -> Result<(), BackendError> {
    let channel_id = channel.id();
    println!("Streaming namespaces to channel {channel_id}");

    let client = client_registry_arc.try_clone(&client_id)?;

    let api: kube::Api<Namespace> = kube::Api::all(client);

    let mut stream = api
        .watch(&kube::api::WatchParams::default(), "0")
        .await?
        .boxed();

    let stream = async move {
        loop {
            let event = match stream.try_next().await {
                Ok(event) => event,
                Err(error) => {
                    eprintln!("{error}");
                    None
                }
            };

            let to_send = match event {
                Some(kube::api::WatchEvent::Added(obj)) => Some(WatchNamespacesEvent::Created(
                    obj.metadata.name.unwrap_or("".into()),
                )),
                Some(kube::api::WatchEvent::Deleted(obj)) => Some(WatchNamespacesEvent::Deleted(
                    obj.metadata.name.unwrap_or("".into()),
                )),
                Some(kube::api::WatchEvent::Modified(_obj)) => None,
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
