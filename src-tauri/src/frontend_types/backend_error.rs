use crate::{
    app_state::Rejected, frontend_commands::KubeContextSource,
    persistence::discovery_cache_service,
    scripting::resource_context_menu_facade::ResourceContextMenuError,
};

#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error(transparent)]
    KubeClientError(#[from] kube::Error),

    #[error(transparent)]
    KubeconfigError(#[from] kube::config::KubeconfigError),

    #[error(transparent)]
    TauriError(#[from] tauri::Error),

    #[error("BackgroundTaskRejected")]
    BackgroundTaskRejected,

    #[error(transparent)]
    DiscoveryCacheServiceError(#[from] discovery_cache_service::Error),

    #[error("Cluster with KubeContextSource {0} is not managed")]
    Unmanaged(KubeContextSource),

    #[error("Cluster with KubeContextSource {0} has not been fully initialized yet")]
    IncompleteClusterDiscovery(KubeContextSource),

    #[error(transparent)]
    ResourceContextMenu(#[from] ResourceContextMenuError),

    #[error("{0}")]
    Generic(String),
}

impl serde::Serialize for BackendError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<String> for BackendError {
    fn from(value: String) -> Self {
        Self::Generic(value)
    }
}

impl From<&str> for BackendError {
    fn from(value: &str) -> Self {
        Self::Generic(value.to_owned())
    }
}

impl From<Rejected> for BackendError {
    fn from(_: Rejected) -> Self {
        Self::BackgroundTaskRejected
    }
}
