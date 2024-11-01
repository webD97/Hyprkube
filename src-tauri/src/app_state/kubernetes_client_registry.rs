use std::{
    collections::HashMap,
    future::Future,
    sync::{
        mpsc::{channel, Receiver},
        Arc, RwLock,
    },
};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{GroupVersionKind, ListParams};
use serde::Serialize;
use uuid::Uuid;

use crate::frontend_types::BackendError;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredGroup {
    pub name: String,
    pub is_crd: bool,
    pub kinds: Vec<DiscoveredResource>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredResource {
    pub group: String,
    pub version: String,
    pub kind: String,
    pub source: ApiGroupSource,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryResult {
    pub gvks: HashMap<String, DiscoveredGroup>,
    pub crds: HashMap<GroupVersionKind, CustomResourceDefinition>,
}

#[derive(Serialize, Clone, Debug)]
pub enum ApiGroupSource {
    Builtin,
    CustomResource,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum AsyncDiscoveryResult {
    DiscoveredResource(DiscoveredResource),
}

pub type KubernetesClientRegistryState = Arc<KubernetesClientRegistry>;

pub type ClusterState = (kube::Client, kube::Config, DiscoveryResult);

pub struct KubernetesClientRegistry {
    registered: Arc<RwLock<HashMap<Uuid, ClusterState>>>,
}

impl KubernetesClientRegistry {
    pub fn new() -> KubernetesClientRegistry {
        KubernetesClientRegistry {
            registered: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new_state() -> KubernetesClientRegistryState {
        Arc::new(KubernetesClientRegistry::new())
    }

    pub fn manage(
        &self,
        new_config: kube::Config,
    ) -> Result<
        (
            Uuid,
            Receiver<AsyncDiscoveryResult>,
            impl Future<Output = Result<(), BackendError>>,
        ),
        BackendError,
    > {
        let new_client = kube::Client::try_from(new_config.clone())?;
        let id = Uuid::new_v4();

        let (downstream_tx, downstream_rx) = channel::<AsyncDiscoveryResult>();

        self.registered.write().unwrap().insert(
            id,
            (
                new_client.clone(),
                new_config,
                DiscoveryResult {
                    crds: HashMap::new(),
                    gvks: HashMap::new(),
                },
            ),
        );

        let registered_arc = Arc::clone(&self.registered);

        let discovery_handle = async move {
            println!("Discovering builtins");
            let apigroups = &new_client.list_api_groups().await?;
            let builtins: Vec<&str> = apigroups
                .groups
                .iter()
                .filter(|group| group.name.ends_with(".k8s.io") || !group.name.contains("."))
                .map(|group| group.name.as_str())
                .chain(Some(""))
                .collect();
            println!("Finished discovering builtins");

            // Discover builtin resources
            println!("Starting discovery of builtin resources");
            {
                let discovery_builtins = kube::Discovery::new(new_client.clone())
                    .filter(&builtins)
                    .run()
                    .await?;

                for group in discovery_builtins.groups() {
                    for (ar, capabilities) in group.recommended_resources() {
                        if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                            continue;
                        }

                        let resource = DiscoveredResource {
                            group: ar.group.clone(),
                            kind: ar.kind.clone(),
                            version: ar.version.clone(),
                            source: ApiGroupSource::Builtin,
                        };

                        downstream_tx
                            .send(AsyncDiscoveryResult::DiscoveredResource(resource.clone()))
                            .unwrap();

                        let mut registered = registered_arc.write().unwrap();
                        let (_, _, discovery) = registered.get_mut(&id).unwrap();

                        discovery
                            .gvks
                            .entry(resource.group.clone())
                            .or_insert(DiscoveredGroup {
                                name: resource.group.clone(),
                                kinds: Vec::new(),
                                is_crd: false,
                            })
                            .kinds
                            .push(resource);
                    }
                }
            }
            println!("Finished discovery of builtin resources");

            // Discover custom resources
            println!("Starting discovery of custom resources");
            {
                let discovery_builtins = kube::Discovery::new(new_client.clone())
                    .exclude(&builtins)
                    .run()
                    .await?;

                for group in discovery_builtins.groups() {
                    for (ar, capabilities) in group.recommended_resources() {
                        if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                            continue;
                        }

                        let resource = DiscoveredResource {
                            group: ar.group.clone(),
                            kind: ar.kind.clone(),
                            version: ar.version.clone(),
                            source: ApiGroupSource::CustomResource,
                        };

                        downstream_tx
                            .send(AsyncDiscoveryResult::DiscoveredResource(resource.clone()))
                            .unwrap();

                        let mut registered = registered_arc.write().unwrap();
                        let (_, _, discovery) = registered.get_mut(&id).unwrap();

                        discovery
                            .gvks
                            .entry(resource.group.clone())
                            .or_insert(DiscoveredGroup {
                                name: resource.group.clone(),
                                kinds: Vec::new(),
                                is_crd: true,
                            })
                            .kinds
                            .push(resource);
                    }
                }
            }
            println!("Finished discovery of custom resources");

            // Cache custom resource definitions
            println!("Starting caching of custom resource definitions");
            {
                let api: kube::Api<CustomResourceDefinition> = kube::Api::all(new_client.clone());
                let crd_list = api.list(&ListParams::default()).await?;

                // Handle groups for custom resources
                for crd in crd_list.items {
                    let mut registered = registered_arc.write().unwrap();
                    let (_, _, discovery) = registered.get_mut(&id).unwrap();

                    let latest = crd
                        .spec
                        .versions
                        .first()
                        .expect("there should always be a version");
                    let gvk =
                        GroupVersionKind::gvk(&crd.spec.group, &latest.name, &crd.spec.names.kind);
                    discovery.crds.insert(gvk, crd);
                }
            }
            println!("Finished caching of custom resource definitions");
            Ok(())
        };

        println!("Managing new client {}", id);
        Ok((id, downstream_rx, discovery_handle))
    }

    pub fn try_clone(&self, id: &Uuid) -> Result<kube::Client, BackendError> {
        self.registered
            .read()
            .unwrap()
            .get(id)
            .map(|(client, _, _)| client.clone())
            .ok_or(format!("Kubernetes client with id {id} not found.").into())
    }

    pub fn get_cluster(&self, id: &Uuid) -> Result<ClusterState, BackendError> {
        self.registered
            .read()
            .unwrap()
            .get(id)
            .ok_or(format!("No cluster state found for ID {}", id).into())
            .cloned()
    }
}
