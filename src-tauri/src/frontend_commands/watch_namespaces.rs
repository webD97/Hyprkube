use futures::StreamExt as _;
use k8s_openapi::api::core::v1::Namespace;
use serde::Serialize;
use tauri::State;

use crate::{
    app_state::{ClientId, JoinHandleStoreState, KubernetesClientRegistryState},
    frontend_types::BackendError,
    internal::resources::ResourceWatchStreamEvent,
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

    let stream = async move {
        crate::internal::resources::watch(api)
            .await
            .map(|event| match event {
                ResourceWatchStreamEvent::Applied { resource } => {
                    WatchNamespacesEvent::Applied(resource.metadata.name.unwrap_or_default())
                }
                ResourceWatchStreamEvent::Deleted { resource } => {
                    WatchNamespacesEvent::Deleted(resource.metadata.name.unwrap_or_default())
                }
            })
            .for_each(|frontend_event| async {
                if let Err(error) = channel.send(frontend_event) {
                    eprintln!("error sending to channel: {error}")
                }
            })
            .await;
    };

    join_handle_store.submit(channel_id, stream);

    Ok(())
}
