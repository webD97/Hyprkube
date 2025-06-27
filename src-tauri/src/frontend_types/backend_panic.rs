#[derive(serde::Serialize, Clone)]
pub struct BackendPanic {
    pub thread: Option<String>,
    pub location: Option<String>,
    pub message: Option<String>,
}
