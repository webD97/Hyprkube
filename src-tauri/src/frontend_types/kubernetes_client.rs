use serde::Serialize;

use crate::app_state::ClientId;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredCluster {
    pub client_id: ClientId,
}
