use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver},
        Arc, RwLock,
    },
};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{GroupVersionKind, ListParams};
use serde::Serialize;
use tauri::async_runtime::{spawn, JoinHandle};
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
    ) -> Result<(Uuid, Receiver<AsyncDiscoveryResult>, JoinHandle<()>), BackendError> {
        let new_client = kube::Client::try_from(new_config.clone()).unwrap();
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

        let discovery_handle = spawn(async move {
            println!("Starting discovery");
            let discovery = kube::Discovery::new(new_client.clone())
                .run()
                .await
                .unwrap();
            println!("Discovery done");

            let api: kube::Api<CustomResourceDefinition> = kube::Api::all(new_client.clone());
            println!("Listing CRDs");

            let mut continuation_token: Option<String> = None;

            let mut builtin_group_names: Vec<String> = Vec::new();
            let mut crd_group_names: Vec<String> = Vec::new();

            let mut crds: Vec<CustomResourceDefinition> = Vec::new();

            loop {
                let crd_list = api
                    .list(&ListParams {
                        limit: Some(15),
                        timeout: Some(60),
                        continue_token: continuation_token,
                        ..ListParams::default()
                    })
                    .await
                    .unwrap();

                // Handle groups for custom resources
                for crd in &crd_list.items {
                    crds.push(crd.clone());

                    let latest = crd.spec.versions.first().unwrap();

                    if !crd_group_names.contains(&crd.spec.group) {
                        crd_group_names.push(crd.spec.group.clone());
                    }

                    let resource = DiscoveredResource {
                        group: crd.spec.group.to_owned(),
                        kind: crd.spec.names.kind.to_owned(),
                        version: latest.name.to_owned(),
                        source: ApiGroupSource::CustomResource,
                    };

                    let mut registered = registered_arc.write().unwrap();
                    let (_, _, discovery) = registered.get_mut(&id).unwrap();

                    downstream_tx
                        .send(AsyncDiscoveryResult::DiscoveredResource(resource.clone()))
                        .unwrap();

                    discovery
                        .gvks
                        .entry(resource.group.clone())
                        .or_insert(DiscoveredGroup {
                            name: resource.group.clone(),
                            kinds: Vec::new(),
                            is_crd: matches!(resource.source, ApiGroupSource::CustomResource),
                        })
                        .kinds
                        .push(resource);
                }

                // Handle groups for builtin resources
                for group in discovery
                    .groups()
                    .filter(|g| !crd_group_names.contains(&g.name().to_owned()))
                {
                    for (ar, capabilities) in group.recommended_resources() {
                        if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                            continue;
                        }

                        if !builtin_group_names.contains(&ar.group) {
                            builtin_group_names.push(ar.group.clone());
                            continue;
                        }
                    }
                }

                continuation_token = crd_list.metadata.continue_;

                if continuation_token.is_none() {
                    println!("Finished listing CRDs");
                    break;
                }

                println!(
                    "Still listing ({:?}) remaining",
                    crd_list.metadata.remaining_item_count
                );
            }

            // Handle resources themselves
            for group in discovery.groups() {
                for (ar, capabilities) in group.recommended_resources() {
                    if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                        continue;
                    }

                    let crd = &crds
                        .iter()
                        .find(|crd| crd.spec.group == ar.group && crd.spec.names.kind == ar.kind);

                    if crd.is_some() {
                        continue;
                    }

                    let resource = DiscoveredResource {
                        group: ar.group.clone(),
                        kind: ar.kind.clone(),
                        version: ar.version.clone(),
                        source: ApiGroupSource::Builtin,
                    };

                    let mut registered = registered_arc.write().unwrap();
                    let (_, _, discovery) = registered.get_mut(&id).unwrap();

                    downstream_tx
                        .send(AsyncDiscoveryResult::DiscoveredResource(resource.clone()))
                        .unwrap();

                    discovery
                        .gvks
                        .entry(resource.group.clone())
                        .or_insert(DiscoveredGroup {
                            name: resource.group.clone(),
                            kinds: Vec::new(),
                            is_crd: matches!(resource.source, ApiGroupSource::CustomResource),
                        })
                        .kinds
                        .push(resource);
                }
            }

            println!("End of future")
        });

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
