use std::collections::HashMap;

use kube::api::{DynamicObject, GroupVersionKind};
use kube::discovery::pinned_kind;
use kube::Api;
use tauri::menu::{
    ContextMenu, Menu, MenuItemBuilder, MenuItemKind, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{LogicalPosition, Position, State, Window, Wry};

use crate::app_state::{ClientId, KubernetesClientRegistryState};
use crate::menus::{HyprkubeMenuItem, MenuAction};
use crate::resource_menu::{
    BasicResourceMenu, DataKeysResourceMenu, DynamicResourceMenuProvider,
    KubernetesResourceMenuState, PodResourceMenu, ResourceMenuContext, RolloutRestartResourceMenu,
};

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn popup_kubernetes_resource_menu(
    window: Window,
    resource_menu_state: State<'_, KubernetesResourceMenuState>,
    client_registry: State<'_, KubernetesClientRegistryState>,
    client_id: ClientId,
    gvk: GroupVersionKind,
    namespace: String,
    name: String,
    position: LogicalPosition<f64>,
) -> Result<(), tauri::Error> {
    let client = client_registry.try_clone(&client_id).unwrap();

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
        Box::new(PodResourceMenu),
        Box::new(RolloutRestartResourceMenu),
        Box::new(DataKeysResourceMenu),
    ];

    let menu_items: Vec<HyprkubeMenuItem> = menu_providers
        .into_iter()
        .filter(|provider| provider.matches(&gvk))
        .flat_map(|provider| provider.build(&gvk, &resource))
        .collect();

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
