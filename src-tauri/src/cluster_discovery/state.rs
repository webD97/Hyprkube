use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    cluster_discovery::{ClusterDiscovery, ClusterState},
    frontend_commands::KubeContextSource,
};

pub struct ClusterRegistry {
    clusters: RwLock<HashMap<KubeContextSource, ClusterState>>,
}

pub type ClusterRegistryState = Arc<ClusterRegistry>;

#[derive(thiserror::Error, Debug)]
pub enum ClusterRegistryError {
    #[error("No cluster found for KubeContextSource: {0}")]
    NotFound(KubeContextSource),
}

impl ClusterRegistry {
    pub fn new() -> Self {
        Self {
            clusters: RwLock::new(HashMap::new()),
        }
    }

    /// Returns a cloned client for the given KubeContextSource if such a managed cluster exists.
    pub fn client_for(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<kube::Client, ClusterRegistryError> {
        Ok(self
            .clusters
            .read()
            .unwrap()
            .get(context_source)
            .ok_or_else(|| ClusterRegistryError::NotFound(context_source.to_owned()))?
            .client
            .to_owned())
    }

    pub fn discovery_for(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<Arc<ClusterDiscovery>, ClusterRegistryError> {
        Ok(self
            .clusters
            .read()
            .unwrap()
            .get(context_source)
            .ok_or_else(|| ClusterRegistryError::NotFound(context_source.to_owned()))?
            .discovery
            .clone())
    }

    pub fn discovery_cache_for(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<Arc<kube::Discovery>, ClusterRegistryError> {
        Ok(self
            .clusters
            .read()
            .unwrap()
            .get(context_source)
            .ok_or_else(|| ClusterRegistryError::NotFound(context_source.to_owned()))?
            .kube_discovery
            .as_ref()
            .map(Arc::clone)
            .expect("todo: discovery might not be set yet"))
    }

    pub fn manage(&self, state: ClusterState) {
        let mut contexts = self.clusters.write().unwrap();
        contexts
            .entry(state.context_source.clone())
            .insert_entry(state);
    }
}
