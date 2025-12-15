use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{cluster_discovery::ClusterState, frontend_commands::KubeContextSource};

pub struct ClusterRegistry {
    clusters: RwLock<HashMap<KubeContextSource, ClusterState>>,
}

pub type ClusterRegistryState = Arc<ClusterRegistry>;

impl ClusterRegistry {
    pub fn new() -> Self {
        Self {
            clusters: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, context_source: &KubeContextSource) -> Option<ClusterState> {
        let contexts = self.clusters.read().unwrap();
        contexts.get(context_source).cloned()
    }

    pub fn manage(&self, state: ClusterState) {
        let mut contexts = self.clusters.write().unwrap();
        contexts
            .entry(state.context_source.clone())
            .insert_entry(state);
    }
}
