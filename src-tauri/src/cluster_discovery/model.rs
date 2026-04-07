use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use futures::Stream;
use kube::api::GroupVersionKind;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::{
    cluster_discovery::FrontendDiscoveryEvent,
    frontend_commands::KubeContextSource,
    scripting::{
        resource_context_menu_facade::ResourceContextMenuFacade,
        resource_presentation_facade::ResourcePresentationFacade,
    },
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredResource {
    pub group: String,
    pub version: String,
    pub kind: String,
    pub plural: String,
    pub source: ApiGroupSource,
    pub scope: Scope,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ApiGroupSource {
    Builtin,
    CustomResource,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    Cluster,
    Namespaced,
}

impl From<kube::discovery::Scope> for Scope {
    fn from(value: kube::discovery::Scope) -> Self {
        match value {
            kube::discovery::Scope::Cluster => Self::Cluster,
            kube::discovery::Scope::Namespaced => Self::Namespaced,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompletedDiscovery {
    pub resources: HashMap<GroupVersionKind, DiscoveredResource>,
}

#[derive(Clone)]
pub enum ClusterDiscovery {
    Inflight(Arc<InflightDiscovery>),
    Completed(CompletedDiscovery),
}

pub struct ClusterState {
    pub context_source: KubeContextSource,
    pub client: kube::Client,
    discovery: RwLock<Arc<ClusterDiscovery>>,
    kube_discovery: RwLock<Option<Arc<kube::Discovery>>>,
    pub context_menu_facade: Arc<ResourceContextMenuFacade>,
    pub resource_presentation_facade: Arc<ResourcePresentationFacade>,
}

impl ClusterState {
    pub fn new(
        context_source: KubeContextSource,
        client: kube::Client,
        discovery: Arc<ClusterDiscovery>,
        context_menu_facade: Arc<ResourceContextMenuFacade>,
        resource_presentation_facade: Arc<ResourcePresentationFacade>,
    ) -> Self {
        Self {
            context_source,
            client,
            discovery: RwLock::new(discovery),
            kube_discovery: RwLock::new(None),
            context_menu_facade,
            resource_presentation_facade,
        }
    }

    pub fn discovery(&self) -> Arc<ClusterDiscovery> {
        self.discovery.read().unwrap().clone()
    }

    pub fn kube_discovery(&self) -> Option<Arc<kube::Discovery>> {
        self.kube_discovery.read().unwrap().clone()
    }

    pub fn finalize_discovery(
        &self,
        result: CompletedDiscovery,
        kube_discovery: Option<Arc<kube::Discovery>>,
    ) {
        if let Some(discovery) = &kube_discovery {
            self.context_menu_facade.set_discovery(discovery.clone());
        }

        *self.kube_discovery.write().unwrap() = kube_discovery;
        *self.discovery.write().unwrap() = Arc::new(ClusterDiscovery::Completed(result));
    }
}

#[derive(Clone)]
pub struct InflightDiscovery {
    tx: broadcast::Sender<FrontendDiscoveryEvent>,
    messages: Arc<RwLock<Vec<FrontendDiscoveryEvent>>>,
}

impl InflightDiscovery {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);

        Self {
            tx,
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn subscribe(&self) -> impl Stream<Item = FrontendDiscoveryEvent> + use<'_> {
        async_stream::stream! {
            {
                let messages = &*self.messages.read().unwrap().clone();

                for message in messages {
                    yield message.clone();
                }
            }

            while let Ok(message) = self.tx.subscribe().recv().await {
                yield message;
            }
        }
    }

    pub fn send(&self, value: FrontendDiscoveryEvent) {
        let messages = &mut *self.messages.write().unwrap();
        messages.push(value.clone());

        // When there are no subscribers, the replay logic in `subscribe` will kick in.
        let _ = self.tx.send(value);
    }
}
