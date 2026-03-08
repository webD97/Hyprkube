use crate::{
    app_state::StateFacade, cluster_profiles::ClusterProfileRegistry, frontend_types::BackendError,
};

use super::cluster_profile_registry::ClusterProfileId;

#[tauri::command]
pub fn list_cluster_profiles(
    app: tauri::AppHandle,
) -> Result<Vec<(ClusterProfileId, String)>, BackendError> {
    let registry = app.state::<ClusterProfileRegistry>();
    Ok(registry.get_profiles())
}
