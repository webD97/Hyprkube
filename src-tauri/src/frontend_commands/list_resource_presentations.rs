use kube::api::GroupVersionKind;

use crate::{
    app_state::{ClusterStateRegistry, ManagerExt as _},
    frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};

pub const SERVER_SIDE_PRESENTATION: &str = "Server-side table";

#[tauri::command]
pub async fn list_resource_presentations(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    group: &str,
    version: &str,
    kind: &str,
) -> Result<Vec<String>, BackendError> {
    let clusters = app.state::<ClusterStateRegistry>();
    let views = clusters.presentation_scripting_for(&context_source)?;
    let gvk = GroupVersionKind::gvk(group, version, kind);

    let mut renderers = views.get_renderers(&gvk)?;
    renderers.push(SERVER_SIDE_PRESENTATION.to_owned());

    Ok(renderers)
}
