use std::sync::Arc;

use kube::{
    api::GroupVersionKind,
    config::{KubeConfigOptions, Kubeconfig},
};
use serde::Serialize;
use tauri::State;

use crate::{
    app_state::{
        AsyncDiscoveryResult, DiscoveredResource, JoinHandleStoreState,
        KubernetesClientRegistryState, RendererRegistry,
    },
    frontend_types::{BackendError, DiscoveredCluster},
};

use super::KubeContextSource;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DiscoveryResult {
    DiscoveredResource((DiscoveredResource, Vec<String>)),
}

#[tauri::command]
pub async fn discover_kubernetes_cluster(
    client_registry: tauri::State<'_, KubernetesClientRegistryState>,
    view_registry: tauri::State<'_, Arc<RendererRegistry>>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    channel: tauri::ipc::Channel<DiscoveryResult>,
    context_source: KubeContextSource,
) -> Result<DiscoveredCluster, BackendError> {
    let (kubeconfig_path, context_name) = context_source;
    let kubeconfig = Kubeconfig::read_from(kubeconfig_path).unwrap();

    let kubeconfig_options = &KubeConfigOptions {
        context: Some(context_name),
        ..Default::default()
    };

    let client_config = kube::Config::from_custom_kubeconfig(kubeconfig, &kubeconfig_options)
        .await
        .unwrap();

    let (client_id, internal_discovery, discovery_handle) =
        client_registry.manage(client_config)?;

    join_handle_store.submit(channel.id(), discovery_handle);

    while let Ok(discovery) = internal_discovery.recv() {
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
