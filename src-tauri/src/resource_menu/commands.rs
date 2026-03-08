use std::sync::Arc;

use kube::api::DynamicObject;
use tauri::Manager as _;
use tracing::debug;

use crate::app_state::ClusterStateRegistry;
use crate::frontend_commands::KubeContextSource;
use crate::frontend_types::BackendError;
use crate::scripting::resource_context_menu::MenuBlueprint;

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn call_menustack_action(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    menustack_id: &str,
    action_ref: &str,
) -> Result<(), BackendError> {
    let clusters = app.state::<Arc<ClusterStateRegistry>>();
    let facade = clusters.contextmenu_scripting_for(&context_source)?;
    facade.call_menustack_action(menustack_id, action_ref)?;

    Ok(())
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn create_resource_menustack(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    parent_menu: Option<&str>,
    gvk: kube::api::GroupVersionKind,
    namespace: &str,
    name: &str,
    tab_id: &str,
) -> Result<MenuBlueprint, BackendError> {
    crate::internal::tracing::set_span_request_id();

    let clusters = app.state::<Arc<ClusterStateRegistry>>();
    let facade = clusters.contextmenu_scripting_for(&context_source)?;
    let discovery = clusters.discovery_cache_for(&context_source)?;
    let client = clusters.client_for(&context_source)?;

    let (api_resource, capabilities) = discovery
        .resolve_gvk(&gvk)
        .ok_or("GroupVersionKind not found")?;

    let api = match capabilities.scope {
        kube::discovery::Scope::Cluster => {
            kube::Api::<DynamicObject>::all_with(client, &api_resource)
        }
        kube::discovery::Scope::Namespaced => match namespace {
            "" => kube::Api::all_with(client, &api_resource),
            namespace => kube::Api::namespaced_with(client, namespace, &api_resource),
        },
    };

    let obj = api.get(name).await?;
    let blueprint = facade.create_resource_menustack(parent_menu, obj, tab_id)?;

    Ok(blueprint)
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn drop_resource_menustack(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    menu_id: &str,
) -> Result<(), BackendError> {
    crate::internal::tracing::set_span_request_id();

    let clusters = app.state::<Arc<ClusterStateRegistry>>();
    let facade = clusters.contextmenu_scripting_for(&context_source)?;

    facade.drop_resource_menustack(menu_id)?;
    debug!("Dropped menu stack {menu_id}");

    Ok(())
}
