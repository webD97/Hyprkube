use std::collections::HashMap;

use async_trait::async_trait;
use k8s_openapi::api::core::v1::{ContainerStatus, Pod};
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

    fn build(
        &self,
        _gvk: &GroupVersionKind,
        resource: &DynamicObject,
    ) -> anyhow::Result<Vec<HyprkubeMenuItem>> {
        // Kinda hacky and ugly but idc for now
        let pod: Pod = serde_json::from_value(serde_json::to_value(resource)?)?;

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

        let container_status: HashMap<String, ContainerStatus> = pod
            .status
            .as_ref()
            .and_then(|status| status.container_statuses.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect();

        let init_container_status: HashMap<String, ContainerStatus> = pod
            .status
            .as_ref()
            .and_then(|status| status.init_container_statuses.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|c| (c.name.clone(), c))
            .collect();

        let mut menu = Vec::new();
        let mut logs_submenu = Vec::new();
        let mut exec_submenu = Vec::new();

        for name in container_names {
            let is_running = container_status
                .get(&name)
                .map(|status| {
                    status
                        .state
                        .clone()
                        .is_some_and(|state| state.waiting.is_none())
                })
                .unwrap_or(false);

            logs_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("builtin:container_logs-{name}"),
                text: name.clone(),
                enabled: is_running,
                action: Box::new(LogsAction {
                    container_name: name.clone(),
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource.metadata.name.clone().unwrap_or_default(),
                }),
            }));

            exec_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("builtin:container_exec-{name}"),
                text: name.clone(),
                enabled: is_running,
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
            let is_running = init_container_status
                .get(&name)
                .map(|status| {
                    status
                        .state
                        .clone()
                        .is_some_and(|state| state.waiting.is_none())
                })
                .unwrap_or(false);

            logs_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("builtin:container_logs-{name}"),
                text: name.clone(),
                enabled: is_running,
                action: Box::new(LogsAction {
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource.metadata.name.clone().unwrap_or_default(),
                    container_name: name.clone(),
                }),
            }));

            exec_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("builtin:container_exec-{name}"),
                text: name.clone(),
                enabled: is_running,
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

        Ok(menu)
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
    async fn run(&self, app: &tauri::AppHandle, _client: kube::Client) -> anyhow::Result<()> {
        use tauri::Emitter as _;

        app.emit(
            "hyprkube:menu:resource:trigger_logs",
            FrontendTriggerLogView {
                namespace: self.namespace.clone(),
                name: self.name.clone(),
                container: self.container_name.clone(),
            },
        )?;

        Ok(())
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
    async fn run(&self, app: &tauri::AppHandle, _client: kube::Client) -> anyhow::Result<()> {
        use anyhow::Context;
        use tauri::Emitter as _;

        app.emit(
            "hyprkube:menu:resource:trigger_exec",
            FrontendTriggerExec {
                namespace: self.namespace.clone(),
                name: self.name.clone(),
                container: self.container_name.clone(),
            },
        )
        .context("Failed to notify frontend")?;

        Ok(())
    }
}
