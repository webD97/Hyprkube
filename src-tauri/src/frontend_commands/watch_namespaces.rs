use futures::StreamExt as _;
use k8s_openapi::api::core::v1::Namespace;
use serde::Serialize;
use tauri::State;
use tracing::{error, info};

use crate::{
    app_state::JoinHandleStoreState, cluster_discovery::ClusterRegistryState,
    frontend_commands::KubeContextSource, frontend_types::BackendError,
    internal::resources::ResourceWatchStreamEvent,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum WatchNamespacesEvent {
    Applied(String),
    Deleted(String),
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn watch_namespaces(
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    join_handle_store: State<'_, JoinHandleStoreState>,
    channel: tauri::ipc::Channel<WatchNamespacesEvent>,
) -> Result<(), BackendError> {
    crate::internal::tracing::set_span_request_id();

    let channel_id = channel.id();
    info!("Streaming namespaces to channel {channel_id}");

    let client = clusters.client_for(&context_source)?;

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
                    error!("error sending to channel: {error}")
                }
            })
            .await;
    };

    join_handle_store.submit(channel_id, stream)?;

    Ok(())
}
