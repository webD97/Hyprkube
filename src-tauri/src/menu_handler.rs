use tauri::{menu::MenuEvent, AppHandle, Manager};
use tracing::{debug, warn};

use crate::{
    frontend_commands::context_menu::kubernetes_resource::KubernetesResourceMenuState,
    resource_menu::ResourceMenuContext,
};

pub fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    let current_context_menu = app.state::<KubernetesResourceMenuState<ResourceMenuContext>>();
    let mut current_context_menu = current_context_menu.lock().unwrap();

    if current_context_menu.is_none() {
        return;
    }

    let (resource, handlers) = current_context_menu.take().unwrap();

    let gvk = resource.gvk;
    let namespace = resource.namespace;
    let name = resource.name;
    let client = resource.client;
    let event_id = event.id.0;
    debug!("{event_id}: {gvk:?}: {namespace}/{name}");

    let context = ResourceMenuContext {
        gvk,
        client,
        namespace,
        name,
    };

    let app = app.clone();

    tauri::async_runtime::spawn(async move {
        if let Some(handler) = handlers.get(&event_id) {
            handler.run(&app, context).await;
        } else {
            warn!("No handler found for event id {event_id}");
        }
    });
}
