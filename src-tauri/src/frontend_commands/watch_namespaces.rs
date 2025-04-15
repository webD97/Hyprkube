use std::pin::pin;

use futures::StreamExt as _;
use k8s_openapi::api::core::v1::Namespace;
use kube::runtime::{
    watcher::{self, Event},
    WatchStreamExt as _,
};
use serde::Serialize;
use tauri::State;

use crate::{
    app_state::{ClientId, JoinHandleStoreState, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum WatchNamespacesEvent {
    Applied(String),
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

    let watch_stream = kube::runtime::watcher(
        api,
        watcher::Config {
            initial_list_strategy: watcher::InitialListStrategy::StreamingList,
            ..Default::default()
        },
    );

    let stream = async move {
        let mut stream = pin!(watch_stream.default_backoff());

        while let Some(event) = stream.next().await {
            let downstream_event = match event {
                Ok(Event::Init) => {
                    println!("Watch init");
                    None
                }
                Ok(Event::InitDone) => {
                    println!("Watch init done");
                    None
                }
                Ok(Event::InitApply(obj)) | Ok(Event::Apply(obj)) => Some(
                    WatchNamespacesEvent::Applied(obj.metadata.name.unwrap_or("".into())),
                ),
                Ok(Event::Delete(obj)) => Some(WatchNamespacesEvent::Deleted(
                    obj.metadata.name.unwrap_or("".into()),
                )),
                Err(e) => {
                    eprintln!("Watch error: {e}");
                    None
                }
            };

            if let Some(message) = downstream_event {
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
