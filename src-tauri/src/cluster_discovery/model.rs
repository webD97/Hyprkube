use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use futures::Stream;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::{cluster_discovery::DiscoveryEvent, frontend_commands::KubeContextSource};

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
    pub crds: HashMap<GroupVersionKind, CustomResourceDefinition>,
}

#[derive(Clone)]
pub enum ClusterDiscovery {
    Inflight(Arc<InflightDiscovery>),
    Completed(CompletedDiscovery),
}

#[derive(Clone)]
pub struct ClusterState {
    pub context_source: KubeContextSource,
    pub client: kube::Client,
    pub discovery: ClusterDiscovery,
}

#[derive(Clone)]
pub struct InflightDiscovery {
    tx: broadcast::Sender<DiscoveryEvent>,
    messages: Arc<RwLock<Vec<DiscoveryEvent>>>,
}

impl InflightDiscovery {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);

        Self {
            tx,
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn subscribe(&self) -> impl Stream<Item = DiscoveryEvent> + use<'_> {
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

    pub fn send(&self, value: DiscoveryEvent) {
        let messages = &mut *self.messages.write().unwrap();
        messages.push(value.clone());

        // When there are no subscribers, the replay logic in `subscribe` will kick in.
        let _ = self.tx.send(value);
    }

    pub async fn block_until_done(&self) -> anyhow::Result<CompletedDiscovery> {
        Ok(CompletedDiscovery {
            resources: HashMap::new(),
            crds: HashMap::new(),
        })
    }
}
