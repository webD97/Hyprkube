use crate::{
    app_state::Rejected, persistence::discovery_cache_service,
    resource_rendering::ResourceViewError,
};

#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error(transparent)]
    KubeClientError(#[from] kube::Error),

    #[error(transparent)]
    KubeconfigError(#[from] kube::config::KubeconfigError),

    #[error(transparent)]
    ResourceViewError(#[from] ResourceViewError),

    #[error(transparent)]
    TauriError(#[from] tauri::Error),

    #[error("BackgroundTaskRejected")]
    BackgroundTaskRejected,

    #[error("UnsupportedKubeconfigProvider")]
    UnsupportedKubeconfigProvider,

    #[error(transparent)]
    DiscoveryCacheServiceError(#[from] discovery_cache_service::Error),

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
