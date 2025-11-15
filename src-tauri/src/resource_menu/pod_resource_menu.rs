use async_trait::async_trait;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{DynamicObject, GroupVersionKind};
use serde::Serialize;

use crate::{
    menus::{HyprkubeActionMenuItem, HyprkubeActionSubMenuItem, HyprkubeMenuItem, MenuAction},
    resource_menu::DynamicResourceMenuProvider,
};

pub struct PodResourceMenu;

impl DynamicResourceMenuProvider for PodResourceMenu {
    fn matches(&self, gvk: &GroupVersionKind) -> bool {
        gvk.api_version() == "v1" && gvk.kind == "Pod"
    }

    fn build(&self, _gvk: &GroupVersionKind, resource: &DynamicObject) -> Vec<HyprkubeMenuItem> {
        // Kinda hacky and ugly but idc for now
        let pod: Pod = serde_json::from_value(serde_json::to_value(resource).unwrap()).unwrap();

        let init_container_names: Vec<String> = pod
            .spec
            .as_ref()
            .map(|spec| spec.init_containers.clone().unwrap_or_default())
            .map(|containers| containers.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default();

        let container_names: Vec<String> = pod
            .spec
            .as_ref()
            .map(|spec| spec.containers.clone())
            .map(|containers| containers.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default();

        let mut menu = Vec::new();
        let mut logs_submenu = Vec::new();
        let mut exec_submenu = Vec::new();

        for name in container_names {
            logs_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("builtin:container_logs-{name}"),
                text: name.clone(),
                action: Box::new(LogsAction {
                    container_name: name.clone(),
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource.metadata.name.clone().unwrap_or_default(),
                }),
            }));

            exec_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("builtin:container_exec-{name}"),
                text: name.clone(),
                action: Box::new(ExecAction {
                    container_name: name,
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource.metadata.name.clone().unwrap_or_default(),
                }),
            }));
        }

        if !init_container_names.is_empty() {
            logs_submenu.push(HyprkubeMenuItem::Separator);
            exec_submenu.push(HyprkubeMenuItem::Separator);
        }

        for name in init_container_names {
            logs_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("builtin:container_logs-{name}"),
                text: name.clone(),
                action: Box::new(LogsAction {
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource.metadata.name.clone().unwrap_or_default(),
                    container_name: name.clone(),
                }),
            }));

            exec_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("builtin:container_exec-{name}"),
                text: name.clone(),
                action: Box::new(ExecAction {
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource.metadata.name.clone().unwrap_or_default(),
                    container_name: name,
                }),
            }));
        }

        menu.push(HyprkubeMenuItem::Submenu(HyprkubeActionSubMenuItem {
            text: "Show logs".into(),
            items: logs_submenu,
        }));

        menu.push(HyprkubeMenuItem::Submenu(HyprkubeActionSubMenuItem {
            text: "Attach shell".into(),
            items: exec_submenu,
        }));

        menu.push(HyprkubeMenuItem::Separator);

        menu
    }
}

#[derive(Serialize, Clone)]
struct FrontendTriggerLogView {
    pub namespace: String,
    pub name: String,
    pub container: String,
}

struct LogsAction {
    pub namespace: String,
    pub name: String,
    pub container_name: String,
}

#[async_trait]
impl MenuAction for LogsAction {
    async fn run(&self, app: &tauri::AppHandle, _client: kube::Client) {
        use tauri::Emitter as _;

        app.emit(
            "hyprkube:menu:resource:trigger_logs",
            FrontendTriggerLogView {
                namespace: self.namespace.clone(),
                name: self.name.clone(),
                container: self.container_name.clone(),
            },
        )
        .unwrap();
    }
}

#[derive(Serialize, Clone)]
struct FrontendTriggerExec {
    pub namespace: String,
    pub name: String,
    pub container: String,
}

struct ExecAction {
    pub namespace: String,
    pub name: String,
    pub container_name: String,
}

#[async_trait]
impl MenuAction for ExecAction {
    async fn run(&self, app: &tauri::AppHandle, _client: kube::Client) {
        use tauri::Emitter as _;

        app.emit(
            "hyprkube:menu:resource:trigger_exec",
            FrontendTriggerExec {
                namespace: self.namespace.clone(),
                name: self.name.clone(),
                container: self.container_name.clone(),
            },
        )
        .unwrap();
    }
}
