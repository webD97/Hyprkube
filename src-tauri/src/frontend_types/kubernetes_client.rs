use serde::Serialize;
use uuid::Uuid;

use crate::frontend_commands::DiscoveryResult;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredCluster {
    pub client_id: Uuid,
    pub discovery: DiscoveryResult,
}
