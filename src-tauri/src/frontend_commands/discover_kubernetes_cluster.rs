use std::{collections::HashSet, sync::Arc};

use kube::config::{KubeConfigOptions, Kubeconfig};
use serde::Serialize;
use tauri::{Emitter, State};
use tracing::{error, info, warn};

use crate::{
    app_state::{
        AsyncDiscoveryResult, DiscoveredResource, JoinHandleStoreState,
        KubernetesClientRegistryState,
    },
    frontend_types::{BackendError, DiscoveredCluster},
    persistence::{discovery_cache_service::DiscoveryCacheService, repository::Repository},
};

use super::KubeContextSource;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DiscoveryResult {
    /// The client id for later use
    ClientId(String),
    /// A resource kind that was either discovered from the cluster or read from cache
    DiscoveredResource(DiscoveredResource),
    /// A resource kind that was previously cached but vanished from the cluster (i.e. CRD uninstall)
    RemovedResource(DiscoveredResource),
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn discover_kubernetes_cluster(
    app_handle: tauri::AppHandle,
    client_registry: tauri::State<'_, KubernetesClientRegistryState>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    channel: tauri::ipc::Channel<DiscoveryResult>,
    context_source: KubeContextSource,
    repository: State<'_, Arc<Repository>>,
) -> Result<DiscoveredCluster, BackendError> {
    crate::internal::tracing::set_span_request_id();

    if context_source.provider != "file" {
        return Err(BackendError::UnsupportedKubeconfigProvider);
    }

    info!("Starting discovery for cluster {}", context_source);

    let context_name = &context_source.context;
    let kubeconfig = Kubeconfig::read_from(&context_source.source)?;

    let kubeconfig_options = &KubeConfigOptions {
        context: Some(context_name.clone()),
        ..Default::default()
    };

    let client_config =
        kube::Config::from_custom_kubeconfig(kubeconfig, kubeconfig_options).await?;

    let discovery_cache =
        DiscoveryCacheService::new(&context_name.clone(), Arc::clone(&repository));
    for cached in discovery_cache.read_cache()? {
        channel
            .send(DiscoveryResult::DiscoveredResource(cached))
            .unwrap();
    }

    let (client_id, internal_discovery, discovery_handle) =
        client_registry.manage(client_config, &context_source)?;

    channel
        .send(DiscoveryResult::ClientId(client_id.clone()))
        .unwrap();

    join_handle_store.submit(channel.id(), async move {
        if let Err(e) = discovery_handle.await {
            error!("Error during cluster discovery: {e}");
            app_handle.emit("ERR_CLUSTER_DISCOVERY", &e).unwrap();
        }
    })?;

    let previous_cache_contents = discovery_cache.read_cache()?;
    let mut discovered_resources = HashSet::new();

    while let Ok(discovery) = internal_discovery.recv() {
        let send_result = match discovery {
            AsyncDiscoveryResult::DiscoveredResource(resource) => {
                match discovery_cache.cache_resource(resource.clone()) {
                    Ok(_) => {}
                    Err(e) => warn!("Error caching resource: {}", e),
                };

                discovered_resources.insert(resource.clone());
                channel.send(DiscoveryResult::DiscoveredResource(resource))
            }
        };

        send_result.unwrap();
    }

    // Find resource kinds that vanished from the cluster and remove them from UI and cache
    for removed_resource in previous_cache_contents.difference(&discovered_resources) {
        info!(
            "Removing stale resource {}.{} from cache as it no longer exists in the cluster",
            removed_resource.kind, removed_resource.group
        );

        match discovery_cache.forget_resource(removed_resource) {
            Ok(_) => {}
            Err(e) => warn!("Error removing resource from cache: {}", e),
        };

        channel
            .send(DiscoveryResult::RemovedResource(
                removed_resource.to_owned(),
            ))
            .unwrap();
    }

    Ok(DiscoveredCluster { client_id })
}
