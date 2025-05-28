use std::sync::Arc;

use kube::api::GroupVersionKind;
use tracing::info;

use crate::{
    app_state::{ClientId, RendererRegistry},
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn list_resource_views(
    view_registry: tauri::State<'_, Arc<RendererRegistry>>,
    client_id: ClientId,
    group: &str,
    version: &str,
    kind: &str,
) -> Result<Vec<String>, BackendError> {
    let gvk = GroupVersionKind::gvk(group, version, kind);
    info!("list_resource_views: {:?}", &gvk);

    Ok(view_registry.get_renderers(&client_id, &gvk).await)
}
