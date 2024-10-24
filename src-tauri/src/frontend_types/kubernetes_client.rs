use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredCluster {
    pub client_id: Uuid,
}
