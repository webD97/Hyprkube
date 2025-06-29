use std::{collections::HashSet, sync::Arc};

use crate::app_state::DiscoveredResource;

use super::{Context, Repository};

pub struct DiscoveryCacheService {
    repository: Arc<Repository>,
    persistence_context: Context,
}

impl DiscoveryCacheService {
    pub fn new(cluster: &str, repository: Arc<Repository>) -> Self {
        Self {
            persistence_context: super::Context::PerCluster(cluster.to_string()),
            repository,
        }
    }

    pub fn read_cache(&self) -> HashSet<DiscoveredResource> {
        self.repository
            .read_key(&self.persistence_context, "discovery_cache")
            .unwrap()
            .map_or(HashSet::new(), |current| {
                serde_json::from_value(current).unwrap()
            })
    }

    pub fn cache_resource(&self, resource: DiscoveredResource) {
        let mut current = self
            .repository
            .read_key(&self.persistence_context, "discovery_cache")
            .unwrap()
            .map_or(HashSet::new(), |current| {
                serde_json::from_value(current).unwrap()
            });

        if current.contains(&resource) {
            return;
        }

        current.insert(resource);

        self.repository
            .set_key(
                &self.persistence_context,
                "discovery_cache",
                serde_json::to_value(current).unwrap(),
            )
            .unwrap();
    }

    pub fn forget_resource(&self, resource: &DiscoveredResource) {
        let mut current = self
            .repository
            .read_key(&self.persistence_context, "discovery_cache")
            .unwrap()
            .map_or(Vec::new(), |current| {
                serde_json::from_value(current).unwrap()
            });

        if !current.contains(resource) {
            return;
        }

        current.retain(|cached| *cached != *resource);

        self.repository
            .set_key(
                &self.persistence_context,
                "discovery_cache",
                serde_json::to_value(current).unwrap(),
            )
            .unwrap();
    }
}
