use std::{collections::HashMap, sync::Mutex};

use kube::api::GroupVersionKind;
use tauri::{menu::MenuEvent, AppHandle, Manager};
use tracing::{debug, warn};

use crate::resource_menu::api::MenuAction;

pub struct ResourceMenuContext {
    pub client: kube::Client,
    pub gvk: GroupVersionKind,
    pub namespace: String,
    pub name: String,
}

pub type KubernetesResourceMenuState =
    Mutex<Option<(ResourceMenuContext, HashMap<String, Box<dyn MenuAction>>)>>;

impl ResourceMenuContext {
    pub fn new_state() -> KubernetesResourceMenuState {
        Mutex::new(None)
    }
}

pub fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    let current_context_menu = app.state::<KubernetesResourceMenuState>();
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

    let app = app.clone();

    tauri::async_runtime::spawn(async move {
        if let Some(handler) = handlers.get(&event_id) {
            handler.run(&app, client.clone()).await;
        } else {
            warn!("No handler found for event id {event_id}");
        }
    });
}
