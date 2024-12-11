use kube::api::GroupVersionKind;
use tauri::State;

use super::{gvk_service, ClusterProfileId, GvkService};

#[tauri::command]
pub fn cluster_profile_list_pinned_gvks(
    gvk_service: State<'_, GvkService>,
    profile: ClusterProfileId,
) -> Result<Vec<GroupVersionKind>, gvk_service::Error> {
    Ok(gvk_service.list_pinned_gvks(&profile)?)
}

#[tauri::command]
pub fn cluster_profile_add_pinned_gvk(
    gvk_service: State<'_, GvkService>,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), gvk_service::Error> {
    Ok(gvk_service.add_pinned_gvk(&profile, gvk.clone())?)
}

#[tauri::command]
pub fn cluster_profile_remove_pinned_gvk(
    gvk_service: State<'_, GvkService>,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), gvk_service::Error> {
    Ok(gvk_service.remove_pinned_gvk(&profile, &gvk)?)
}
