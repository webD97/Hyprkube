#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error(transparent)]
    KubeClientError(#[from] kube::Error),

    #[error("an error occurred")]
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
