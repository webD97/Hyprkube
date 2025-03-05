use kube::api::GroupVersionKind;
use tauri::State;

use crate::persistence::cluster_profile_service::{self, ClusterProfileService};

use super::cluster_profile_registry::ClusterProfileId;

#[tauri::command]
pub fn cluster_profile_list_pinned_gvks(
    cluster_profile_service: State<'_, ClusterProfileService>,
    profile: ClusterProfileId,
) -> Result<Vec<GroupVersionKind>, cluster_profile_service::Error> {
    cluster_profile_service.list_pinned_gvks(&profile)
}

#[tauri::command]
pub fn cluster_profile_add_pinned_gvk(
    cluster_profile_service: State<'_, ClusterProfileService>,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), cluster_profile_service::Error> {
    cluster_profile_service.add_pinned_gvk(&profile, gvk.clone())
}

#[tauri::command]
pub fn cluster_profile_remove_pinned_gvk(
    cluster_profile_service: State<'_, ClusterProfileService>,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), cluster_profile_service::Error> {
    cluster_profile_service.remove_pinned_gvk(&profile, &gvk)
}

#[tauri::command]
pub fn cluster_profile_list_hidden_gvks(
    cluster_profile_service: State<'_, ClusterProfileService>,
    profile: ClusterProfileId,
) -> Result<Vec<GroupVersionKind>, cluster_profile_service::Error> {
    cluster_profile_service.list_hidden_gvks(&profile)
}

#[tauri::command]
pub fn cluster_profile_add_hidden_gvk(
    cluster_profile_service: State<'_, ClusterProfileService>,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), cluster_profile_service::Error> {
    cluster_profile_service.add_hidden_gvk(&profile, gvk.clone())
}

#[tauri::command]
pub fn cluster_profile_remove_hidden_gvk(
    cluster_profile_service: State<'_, ClusterProfileService>,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), cluster_profile_service::Error> {
    cluster_profile_service.remove_hidden_gvk(&profile, &gvk)
}

#[tauri::command]
pub fn get_default_namespace(
    cluster_profile_service: State<'_, ClusterProfileService>,
    profile: ClusterProfileId,
) -> Result<String, cluster_profile_service::Error> {
    cluster_profile_service.get_default_namespace(&profile)
}

#[tauri::command]
pub fn set_default_namespace(
    cluster_profile_service: State<'_, ClusterProfileService>,
    profile: ClusterProfileId,
    namespace: &str,
) -> Result<(), cluster_profile_service::Error> {
    cluster_profile_service.set_default_namespace(&profile, namespace)
}
