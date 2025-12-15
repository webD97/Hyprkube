use std::{collections::HashSet, sync::Arc};

use crate::{
    cluster_discovery::DiscoveredResource,
    persistence::repository::{self, Context, Repository},
};

pub struct DiscoveryCacheService {
    repository: Arc<Repository>,
    persistence_context: Context,
}

impl DiscoveryCacheService {
    pub fn new(cluster: &str, repository: Arc<Repository>) -> Self {
        Self {
            persistence_context: repository::Context::PerCluster(cluster.to_string()),
            repository,
        }
    }

    pub fn read_cache(&self) -> Result<HashSet<DiscoveredResource>, Error> {
        Ok(self
            .repository
            .read_key(&self.persistence_context, "discovery_cache")?
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default())
    }

    pub fn set_cache(&self, state: HashSet<DiscoveredResource>) -> Result<(), Error> {
        Ok(self.repository.set_key(
            &self.persistence_context,
            "discovery_cache",
            serde_json::to_value(state)?,
        )?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    RepositoryError(#[from] repository::Error),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
}
