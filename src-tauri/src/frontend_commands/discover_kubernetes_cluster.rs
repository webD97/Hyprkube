use std::sync::Arc;

use kube::api::GroupVersionKind;
use serde::Serialize;
use tauri::{async_runtime::Mutex, State};

use crate::{
    app_state::{
        AsyncDiscoveryResult, DiscoveredResource, JoinHandleStore, KubernetesClientRegistryState,
    },
    frontend_types::{BackendError, DiscoveredCluster},
    resource_rendering::RendererRegistry,
};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DiscoveryResult {
    DiscoveredResource((DiscoveredResource, Vec<String>)),
}

#[tauri::command]
pub async fn discover_kubernetes_cluster(
    client_registry: tauri::State<'_, KubernetesClientRegistryState>,
    view_registry: tauri::State<'_, Arc<RendererRegistry>>,
    join_handle_store: State<'_, Arc<Mutex<JoinHandleStore>>>,
    channel: tauri::ipc::Channel<DiscoveryResult>,
) -> Result<DiscoveredCluster, BackendError> {
    let config = kube::Config::infer().await.unwrap();
    let client = kube::Client::try_default().await?;
    let (client_id, mut internal_discovery, disovery_handle) =
        client_registry.lock().await.manage(client, config).await?;

    join_handle_store
        .lock()
        .await
        .insert(channel.id(), disovery_handle);

    while let Some(discovery) = internal_discovery.recv().await {
        let send_result = match discovery {
            AsyncDiscoveryResult::DiscoveredResource(resource) => {
                let gvk = GroupVersionKind::gvk(&resource.group, &resource.version, &resource.kind);
                let views = view_registry.get_renderers(&client_id, &gvk).await;
                channel.send(DiscoveryResult::DiscoveredResource((resource, views)))
            }
        };

        send_result.unwrap();
    }

    Ok(DiscoveredCluster { client_id })
}