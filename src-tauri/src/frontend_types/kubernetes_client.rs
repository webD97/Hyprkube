use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct KubernetesClient {
    pub id: Uuid
}
