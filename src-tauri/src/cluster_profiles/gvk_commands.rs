use kube::api::GroupVersionKind;

use crate::{
    app_state::ManagerExt,
    persistence::cluster_profile_service::{self, ClusterProfileService},
};

use super::cluster_profile_registry::ClusterProfileId;

#[tauri::command]
pub fn cluster_profile_list_pinned_gvks(
    app: tauri::AppHandle,
    profile: ClusterProfileId,
) -> Result<Vec<GroupVersionKind>, cluster_profile_service::Error> {
    let service = app.state::<ClusterProfileService>();
    service.list_pinned_gvks(&profile)
}

#[tauri::command]
pub fn cluster_profile_add_pinned_gvk(
    app: tauri::AppHandle,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), cluster_profile_service::Error> {
    let service = app.state::<ClusterProfileService>();
    service.add_pinned_gvk(&profile, gvk.clone())
}

#[tauri::command]
pub fn cluster_profile_remove_pinned_gvk(
    app: tauri::AppHandle,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), cluster_profile_service::Error> {
    let service = app.state::<ClusterProfileService>();
    service.remove_pinned_gvk(&profile, &gvk)
}

#[tauri::command]
pub fn cluster_profile_list_hidden_gvks(
    app: tauri::AppHandle,
    profile: ClusterProfileId,
) -> Result<Vec<GroupVersionKind>, cluster_profile_service::Error> {
    let service = app.state::<ClusterProfileService>();
    service.list_hidden_gvks(&profile)
}

#[tauri::command]
pub fn cluster_profile_add_hidden_gvk(
    app: tauri::AppHandle,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), cluster_profile_service::Error> {
    let service = app.state::<ClusterProfileService>();
    service.add_hidden_gvk(&profile, gvk.clone())
}

#[tauri::command]
pub fn cluster_profile_remove_hidden_gvk(
    app: tauri::AppHandle,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<(), cluster_profile_service::Error> {
    let service = app.state::<ClusterProfileService>();
    service.remove_hidden_gvk(&profile, &gvk)
}

#[tauri::command]
pub fn get_default_namespace(
    app: tauri::AppHandle,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
) -> Result<String, cluster_profile_service::Error> {
    let service = app.state::<ClusterProfileService>();
    service.get_default_namespace(&profile, &gvk)
}

#[tauri::command]
pub fn set_default_namespace(
    app: tauri::AppHandle,
    profile: ClusterProfileId,
    gvk: GroupVersionKind,
    namespace: &str,
) -> Result<(), cluster_profile_service::Error> {
    let service = app.state::<ClusterProfileService>();
    service.set_default_namespace(&profile, &gvk, namespace)
}
