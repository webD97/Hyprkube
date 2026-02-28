use std::collections::HashMap;
use std::iter;

use kube::api::{DynamicObject, GroupVersionKind};
use kube::discovery::pinned_kind;
use kube::Api;
use tauri::menu::{
    ContextMenu, Menu, MenuItemBuilder, MenuItemKind, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{LogicalPosition, Manager as _, Position, State, Window, Wry};
use tracing::{debug, error};

use crate::cluster_discovery::ClusterRegistryState;
use crate::frontend_commands::KubeContextSource;
use crate::frontend_types::BackendError;
use crate::menus::{HyprkubeMenuItem, MenuAction};
use crate::resource_menu::{
    BasicResourceMenu, DataKeysResourceMenu, DynamicResourceMenuProvider,
    KubernetesResourceMenuState, PodResourceMenu, ResourceMenuContext, RolloutRestartResourceMenu,
};
use crate::scripting::resource_context_menu_facade::MenuBlueprint;

#[allow(clippy::too_many_arguments)]
#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn popup_kubernetes_resource_menu(
    window: Window,
    resource_menu_state: State<'_, KubernetesResourceMenuState>,
    clusters: State<'_, ClusterRegistryState>,
    context_source: KubeContextSource,
    gvk: GroupVersionKind,
    namespace: String,
    name: String,
    position: LogicalPosition<f64>,
    tab_id: String,
) -> Result<(), BackendError> {
    crate::internal::tracing::set_span_request_id();

    let client = clusters.client_for(&context_source)?;

    let (api_resource, capabilities) = pinned_kind(&client, &gvk).await.unwrap();

    let api: Api<DynamicObject> = match capabilities.scope {
        kube::discovery::Scope::Cluster => kube::Api::all_with(client.clone(), &api_resource),
        kube::discovery::Scope::Namespaced => {
            kube::Api::namespaced_with(client.clone(), &namespace, &api_resource)
        }
    };

    let resource = api.get(&name).await.unwrap();

    let menu_providers: Vec<Box<dyn DynamicResourceMenuProvider>> = vec![
        Box::new(BasicResourceMenu),
        Box::new(RolloutRestartResourceMenu),
        Box::new(PodResourceMenu),
        Box::new(DataKeysResourceMenu),
    ];

    let mut menu_items: Vec<HyprkubeMenuItem> = menu_providers
        .into_iter()
        .filter(|provider| provider.matches(&gvk))
        .flat_map(
            |provider| match provider.build(&gvk, &resource, tab_id.clone()) {
                Err(e) => {
                    error!("MenuProvider failed: {e}");
                    None
                }
                Ok(menu) => Some(
                    menu.into_iter()
                        .chain(iter::once(HyprkubeMenuItem::Separator)),
                ),
            },
        )
        .flatten()
        .collect();

    menu_items.pop_if(|item| matches!(item, HyprkubeMenuItem::Separator));

    let context_menu = Menu::new(&window).unwrap();

    for tauri_menu_item in make_tauri_menu(&window, &menu_items) {
        context_menu.append(&tauri_menu_item).unwrap();
    }

    let action_map = create_action_map(menu_items);

    let mut resource_menu_state = resource_menu_state.lock().unwrap();
    *resource_menu_state = Some((
        ResourceMenuContext {
            gvk,
            namespace,
            name,
            client,
        },
        action_map,
    ));

    context_menu.popup_at(window, Position::Logical(position))?;

    Ok(())
}

fn make_tauri_menu<T: tauri::Manager<Wry>>(
    manager: &T,
    items: &[HyprkubeMenuItem],
) -> Vec<MenuItemKind<Wry>> {
    items
        .iter()
        .map(|item| match item {
            HyprkubeMenuItem::Action(item) => MenuItemKind::MenuItem(
                MenuItemBuilder::new(item.text.clone())
                    .id(item.id.clone())
                    .enabled(item.enabled)
                    .build(manager)
                    .unwrap(),
            ),
            HyprkubeMenuItem::Separator => {
                MenuItemKind::Predefined(PredefinedMenuItem::separator(manager).unwrap())
            }
            HyprkubeMenuItem::Submenu(item) => {
                let sub = SubmenuBuilder::new(manager, item.text.clone())
                    .build()
                    .unwrap();
                let subitems = make_tauri_menu(manager, &item.items);

                for subitem in subitems {
                    sub.append(&subitem).unwrap();
                }

                MenuItemKind::Submenu(sub)
            }
        })
        .collect()
}

fn create_action_map(items: Vec<HyprkubeMenuItem>) -> HashMap<String, Box<dyn MenuAction>> {
    let mut actions = HashMap::new();

    for item in items {
        match item {
            HyprkubeMenuItem::Separator => {}
            HyprkubeMenuItem::Action(item) => {
                actions.entry(item.id).insert_entry(item.action);
            }
            HyprkubeMenuItem::Submenu(submenu) => {
                actions.extend(create_action_map(submenu.items));
            }
        }
    }

    actions
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn call_menustack_action(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    menustack_id: &str,
    action_ref: &str,
) -> Result<(), BackendError> {
    let clusters = app.state::<ClusterRegistryState>();
    let facade = clusters.scripting_for(&context_source)?;
    facade.call_menustack_action(menustack_id, action_ref);

    Ok(())
}

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn create_resource_menustack(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    gvk: kube::api::GroupVersionKind,
    namespace: &str,
    name: &str,
) -> Result<MenuBlueprint, BackendError> {
    crate::internal::tracing::set_span_request_id();

    let clusters = app.state::<ClusterRegistryState>();
    let facade = clusters.scripting_for(&context_source)?;
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
    let blueprint = facade.create_resource_menustack(obj);
    debug!("Created menu stack {}", blueprint.id);

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

    let clusters = app.state::<ClusterRegistryState>();
    let facade = clusters.scripting_for(&context_source)?;

    facade.drop_resource_menustack(menu_id);
    debug!("Dropped menu stack {menu_id}");

    Ok(())
}
