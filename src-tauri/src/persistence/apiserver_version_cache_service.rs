use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::persistence::repository::{self, Context, Repository};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CachedApiserverVersion {
    pub git_version: String,
    /// RFC3339 timestamp of when the version was last fetched from the cluster.
    pub fetched_at: String,
}

pub struct ApiserverVersionCacheService {
    repository: Arc<Repository>,
    persistence_context: Context,
}

impl ApiserverVersionCacheService {
    pub fn new(cluster: &str, repository: Arc<Repository>) -> Self {
        Self {
            persistence_context: repository::Context::PerCluster(cluster.to_string()),
            repository,
        }
    }

    pub fn read(&self) -> Result<Option<CachedApiserverVersion>, Error> {
        Ok(self
            .repository
            .read_key(&self.persistence_context, "apiserver_version")?
            .map(serde_json::from_value)
            .transpose()?)
    }

    pub fn set(&self, value: &CachedApiserverVersion) -> Result<(), Error> {
        Ok(self.repository.set_key(
            &self.persistence_context,
            "apiserver_version",
            serde_json::to_value(value)?,
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
