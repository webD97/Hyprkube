use std::sync::Arc;

use kube::api::GroupVersionKind;
use uuid::Uuid;

use crate::{app_state::RendererRegistry, frontend_types::BackendError};

#[tauri::command]
pub async fn list_resource_views(
    view_registry: tauri::State<'_, Arc<RendererRegistry>>,
    client_id: Uuid,
    group: &str,
    version: &str,
    kind: &str,
) -> Result<Vec<String>, BackendError> {
    let gvk = GroupVersionKind::gvk(group, version, kind);
    println!("list_resource_views: {:?}", &gvk);

    Ok(view_registry.get_renderers(&client_id, &gvk).await)
}
