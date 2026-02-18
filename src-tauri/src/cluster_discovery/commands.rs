use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use futures::{Stream, StreamExt};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{GroupVersionKind, ListParams},
    config::{KubeConfigOptions, Kubeconfig},
};
use serde::Serialize;
use tauri::State;

use crate::{
    app_state::JoinHandleStoreState,
    cluster_discovery::{
        ApiGroupSource, ClusterDiscovery, ClusterRegistryState, ClusterState, CompletedDiscovery,
        DiscoveredResource, InflightDiscovery,
    },
    frontend_commands::KubeContextSource,
    persistence::{discovery_cache_service::DiscoveryCacheService, repository::Repository},
    scripting::resource_context_menu_facade::ResourceContextMenuFacade,
};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FrontendDiscoveryEvent {
    /// A resource kind that was either discovered from the cluster or read from cache
    DiscoveredResource(DiscoveredResource),
    /// A resource kind that was previously cached but vanished from the cluster (i.e. CRD uninstall)
    RemovedResource(DiscoveredResource),
    CustomResourceDefinition((GroupVersionKind, Box<CustomResourceDefinition>)),
    DiscoveryComplete(()),
}

impl From<&InternalDiscoveryEvent> for FrontendDiscoveryEvent {
    fn from(value: &InternalDiscoveryEvent) -> Self {
        match value {
            InternalDiscoveryEvent::DiscoveredResource(resource) => {
                FrontendDiscoveryEvent::DiscoveredResource(resource.clone())
            }
            InternalDiscoveryEvent::RemovedResource(resource) => {
                FrontendDiscoveryEvent::RemovedResource(resource.clone())
            }
            InternalDiscoveryEvent::CustomResourceDefinition(inner) => {
                FrontendDiscoveryEvent::CustomResourceDefinition(inner.clone())
            }
            InternalDiscoveryEvent::DiscoveryComplete(_) => {
                FrontendDiscoveryEvent::DiscoveryComplete(())
            }
        }
    }
}

pub enum InternalDiscoveryEvent {
    /// A resource kind that was either discovered from the cluster or read from cache
    DiscoveredResource(DiscoveredResource),
    /// A resource kind that was previously cached but vanished from the cluster (i.e. CRD uninstall)
    RemovedResource(DiscoveredResource),
    CustomResourceDefinition((GroupVersionKind, Box<CustomResourceDefinition>)),
    DiscoveryComplete(kube::Discovery),
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn connect_cluster(
    background_tasks: State<'_, JoinHandleStoreState>,
    repository: State<'_, Arc<Repository>>,
    clusters: State<'_, ClusterRegistryState>,
    channel: tauri::ipc::Channel<FrontendDiscoveryEvent>,
    context_source: KubeContextSource,
) -> Result<(), String> {
    crate::internal::tracing::set_span_request_id();

    let repository = Arc::clone(&repository);
    let clusters = Arc::clone(&clusters);

    let _ = background_tasks.submit(channel.id(), async move {
        // Fast path: If there is already a discovery, use its results
        if let Ok(discovery) = clusters.discovery_for(&context_source) {
            match &*discovery {
                ClusterDiscovery::Inflight(discovery) => {
                    tracing::info!("Attaching to inflight discovery");

                    let mut stream = std::pin::pin!(discovery.subscribe());
                    while let Some(event) = stream.next().await {
                        channel.send(event.clone())?;
                    }
                }
                ClusterDiscovery::Completed(discovery) => {
                    tracing::info!("Skipping discovery, serving in-memory results");

                    for resource in discovery.resources.values() {
                        channel
                            .send(FrontendDiscoveryEvent::DiscoveredResource(resource.clone()))?;
                    }
                }
            }

            channel.send(FrontendDiscoveryEvent::DiscoveryComplete(()))?;

            return Ok(());
        }

        tracing::info!("Starting discovery for cluster {}", context_source);

        let client = make_client(&context_source).await?;

        // Attachable stuff
        let inflight = Arc::new(InflightDiscovery::new());

        clusters.manage(ClusterState {
            context_source: context_source.clone(),
            client: client.clone(),
            discovery: Arc::new(ClusterDiscovery::Inflight(Arc::clone(&inflight))),
            kube_discovery: None,
            script_facade: None,
        });

        // Cached part
        tracing::info!("Serving cached resources");
        let discovery_cache =
            DiscoveryCacheService::new(&context_source.context, Arc::clone(&repository));

        let previously_cached = discovery_cache.read_cache().unwrap_or_else(|e| {
            tracing::error!("Failed to read on-disk discovery cache: {e}");
            HashSet::new()
        });

        for cached in &previously_cached {
            inflight.send(FrontendDiscoveryEvent::DiscoveredResource(cached.clone()));
            channel.send(FrontendDiscoveryEvent::DiscoveredResource(cached.clone()))?;
        }

        // Online part
        tracing::info!("Performing live-discovery against cluster");
        let clusters = Arc::clone(&clusters);

        let mut confirmed_resources = HashSet::new();
        let mut resources: HashMap<GroupVersionKind, DiscoveredResource> = HashMap::new();
        let mut crds: HashMap<GroupVersionKind, CustomResourceDefinition> = HashMap::new();

        let mut discovery_stream = std::pin::pin!(online_discovery(client.clone()));
        let mut kube_discovery: Option<Arc<kube::Discovery>> = None;

        while let Some(msg) = discovery_stream.next().await {
            let msg = msg?;

            // Save
            match msg {
                InternalDiscoveryEvent::DiscoveredResource(resource) => {
                    confirmed_resources.insert(resource.clone());

                    let gvk =
                        GroupVersionKind::gvk(&resource.group, &resource.version, &resource.kind);

                    resources.entry(gvk).insert_entry(resource.clone());

                    // Forward to inflight cache
                    inflight.send(FrontendDiscoveryEvent::DiscoveredResource(resource.clone()));

                    // Forward to frontend
                    channel.send(FrontendDiscoveryEvent::DiscoveredResource(resource))?;
                }
                InternalDiscoveryEvent::CustomResourceDefinition((gvk, crd)) => {
                    crds.entry(gvk.clone()).insert_entry(*crd.clone());
                    // Forward to inflight cache
                    inflight.send(FrontendDiscoveryEvent::CustomResourceDefinition((
                        gvk.clone(),
                        crd.clone(),
                    )));

                    // Forward to frontend
                    channel.send(FrontendDiscoveryEvent::CustomResourceDefinition((
                        gvk.clone(),
                        crd.clone(),
                    )))?;
                }
                InternalDiscoveryEvent::DiscoveryComplete(discovery) => {
                    kube_discovery = Some(Arc::new(discovery))
                }
                _ => {}
            }
        }

        // Find resource kinds that vanished from the cluster and remove them from UI and cache
        for removed_resource in previously_cached.difference(&confirmed_resources) {
            tracing::info!(
                "Removing stale resource {}.{} from cache as it no longer exists in the cluster",
                removed_resource.kind,
                removed_resource.group
            );

            let msg = FrontendDiscoveryEvent::RemovedResource(removed_resource.to_owned());

            inflight.send(msg.clone());
            channel.send(msg)?;
        }

        if let Err(e) = discovery_cache.set_cache(confirmed_resources) {
            tracing::warn!("Error updating resource cache: {}", e);
        }

        let result = CompletedDiscovery { resources, crds };

        let facade = ResourceContextMenuFacade::new();
        facade.register_user_script("/home/christian/Downloads/test.rhai".into());
        facade.initialize_engines(client.clone(), Arc::clone(kube_discovery.as_ref().unwrap()));
        facade.evaluate_all().unwrap();

        clusters.manage(ClusterState {
            context_source,
            client,
            discovery: Arc::new(ClusterDiscovery::Completed(result)),
            kube_discovery,
            script_facade: Some(facade),
        });

        channel.send(FrontendDiscoveryEvent::DiscoveryComplete(()))?;

        Ok(())
    });

    Ok(())
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn get_apiserver_gitversion(context_source: KubeContextSource) -> Result<String, String> {
    crate::internal::tracing::set_span_request_id();

    let client = make_client(&context_source)
        .await
        .map_err(|e| e.to_string())?;

    let version = client
        .apiserver_version()
        .await
        .map_err(|e| e.to_string())?;

    Ok(version.git_version)
}

async fn make_client(context_source: &KubeContextSource) -> anyhow::Result<kube::Client> {
    if context_source.provider != "file" {
        anyhow::bail!(
            "Unsupported kubeconfig provider: {}",
            context_source.provider
        );
    }

    let context_name = &context_source.context;
    let kubeconfig = Kubeconfig::read_from(&context_source.source)?;

    let options = &KubeConfigOptions {
        context: Some(context_name.clone()),
        ..Default::default()
    };

    let client_config = kube::Config::from_custom_kubeconfig(kubeconfig, options).await?;

    Ok(kube::Client::try_from(client_config)?)
}

/// Performs a discovery of available resources against the given cluster.
///
/// The discovery will first try to discover builtin (i.e. non-crd) resources to optimize
/// the user experience. CRD-based resources will be yielded after that.
fn online_discovery(
    client: kube::Client,
) -> impl Stream<Item = anyhow::Result<InternalDiscoveryEvent>> {
    async_stream::stream! {
        tracing::info!("Discovering builtins");
        let apigroups = &client.list_api_groups().await?;
        let builtins: Vec<&str> = apigroups
            .groups
            .iter()
            .filter(|group| group.name.ends_with(".k8s.io") || !group.name.contains("."))
            .map(|group| group.name.as_str())
            .chain(Some(""))
            .collect();
        tracing::debug!("Finished discovering builtins");

        let full_discovery = kube::Discovery::new(client.clone())
            .run_aggregated()
            .await?;

        // Discover builtin resources
        tracing::info!("Starting discovery of builtin resources");
        {
            for group in full_discovery.groups().filter(|i| builtins.contains(&i.name())) {
                for (ar, capabilities) in group.resources_by_stability() {
                    if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                        continue;
                    }

                    let resource = DiscoveredResource {
                        group: ar.group.clone(),
                        kind: ar.kind.clone(),
                        plural: ar.plural.clone(),
                        version: ar.version.clone(),
                        source: ApiGroupSource::Builtin,
                        scope: capabilities.scope.into(),
                    };

                    yield Ok(InternalDiscoveryEvent::DiscoveredResource(resource.clone()));
                }
            }
        }
        tracing::debug!("Finished discovery of builtin resources");

        // Discover custom resources
        tracing::info!("Starting discovery of custom resources");
        {
            for group in full_discovery.groups().filter(|i| !builtins.contains(&i.name())) {
                for (ar, capabilities) in group.resources_by_stability() {
                    if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                        continue;
                    }

                    let resource = DiscoveredResource {
                        group: ar.group.clone(),
                        kind: ar.kind.clone(),
                        plural: ar.plural.clone(),
                        version: ar.version.clone(),
                        source: ApiGroupSource::CustomResource,
                        scope: capabilities.scope.into(),
                    };

                    yield Ok(InternalDiscoveryEvent::DiscoveredResource(resource.clone()));
                }
            }
        }
        tracing::debug!("Finished discovery of custom resources");

        // Cache custom resource definitions
        tracing::info!("Starting caching of custom resource definitions");
        {
            let api: kube::Api<CustomResourceDefinition> = kube::Api::all(client.clone());
            let crd_list = api.list(&ListParams::default()).await?;

            // Handle groups for custom resources
            for crd in crd_list.items {
                let latest = crd.spec.versions.first();

                if let Some(latest) = latest {
                    let gvk = GroupVersionKind::gvk(&crd.spec.group, &latest.name, &crd.spec.names.kind);
                    yield Ok(InternalDiscoveryEvent::CustomResourceDefinition((gvk, Box::new(crd))));
                } else {
                    tracing::error!("CustomResourceDefinition {}.{} has no versions.", crd.spec.names.kind, crd.spec.group);
                    continue;
                }
            }
        }
        tracing::debug!("Finished caching of custom resource definitions");

        yield Ok(InternalDiscoveryEvent::DiscoveryComplete(full_discovery));
    }
}
