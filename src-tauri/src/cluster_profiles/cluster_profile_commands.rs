use tauri::State;

use crate::frontend_types::BackendError;

use super::cluster_profile_registry::{ClusterProfileId, ClusterProfileRegistryState};

#[tauri::command]
pub fn list_cluster_profiles(
    cluster_profile_registry: State<'_, ClusterProfileRegistryState>,
) -> Result<Vec<(ClusterProfileId, String)>, BackendError> {
    Ok(cluster_profile_registry.get_profiles())
}
