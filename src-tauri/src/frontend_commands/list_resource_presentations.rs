use kube::api::GroupVersionKind;

use crate::{
    app_state::{ClusterStateRegistry, StateFacade as _},
    frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn list_resource_presentations(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    group: &str,
    version: &str,
    kind: &str,
) -> Result<Vec<String>, BackendError> {
    let clusters = app.state::<ClusterStateRegistry>();
    let gvk = GroupVersionKind::gvk(group, version, kind);

    let renderers = {
        let views = clusters.presentation_scripting_for(&context_source)?;
        views
            .get_renderers(&context_source, &gvk)
            .await
            .expect("handle me")
    };

    Ok(renderers)
}
