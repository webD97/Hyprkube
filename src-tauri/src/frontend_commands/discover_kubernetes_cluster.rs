use std::sync::Arc;

use kube::config::{KubeConfigOptions, Kubeconfig};
use serde::Serialize;
use tauri::{Emitter, State};

use crate::{
    app_state::{
        AsyncDiscoveryResult, DiscoveredResource, JoinHandleStoreState,
        KubernetesClientRegistryState,
    },
    frontend_types::{BackendError, DiscoveredCluster},
    persistence::{DiscoveryCacheService, Repository},
};

use super::KubeContextSource;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DiscoveryResult {
    DiscoveredResource(DiscoveredResource),
    ClientId(String),
}

#[tauri::command]
pub async fn discover_kubernetes_cluster(
    app_handle: tauri::AppHandle,
    client_registry: tauri::State<'_, KubernetesClientRegistryState>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    channel: tauri::ipc::Channel<DiscoveryResult>,
    context_source: KubeContextSource,
    repository: State<'_, Arc<Repository>>,
) -> Result<DiscoveredCluster, BackendError> {
    let (kubeconfig_path, context_name) = context_source;
    let kubeconfig = Kubeconfig::read_from(kubeconfig_path)?;

    let kubeconfig_options = &KubeConfigOptions {
        context: Some(context_name.clone()),
        ..Default::default()
    };

    let client_config =
        kube::Config::from_custom_kubeconfig(kubeconfig, kubeconfig_options).await?;

    let discovery_cache =
        DiscoveryCacheService::new(&context_name.clone(), Arc::clone(&repository));
    for cached in discovery_cache.read_cache() {
        channel
            .send(DiscoveryResult::DiscoveredResource(cached))
            .unwrap();
    }

    let (client_id, internal_discovery, discovery_handle) =
        client_registry.manage(client_config)?;

    join_handle_store.submit(channel.id(), async move {
        if let Err(e) = discovery_handle.await {
            eprintln!("Error during cluster discovery: {e}");
            app_handle.emit("ERR_CLUSTER_DISCOVERY", &e).unwrap();
        }
    });

    while let Ok(discovery) = internal_discovery.recv() {
        let send_result = match discovery {
            AsyncDiscoveryResult::DiscoveredResource(resource) => {
                discovery_cache.cache_resource(resource.clone());
                channel.send(DiscoveryResult::DiscoveredResource(resource))
            }
            AsyncDiscoveryResult::ObtainedClientId(client_id) => {
                channel.send(DiscoveryResult::ClientId(client_id))
            }
        };

        send_result.unwrap();
    }

    Ok(DiscoveredCluster { client_id })
}
