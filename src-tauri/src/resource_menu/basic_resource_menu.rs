use anyhow::Context as _;
use async_trait::async_trait;
use kube::api::{DeleteParams, DynamicObject, GroupVersionKind};
use serde::Serialize;

use crate::{
    menus::{HyprkubeActionMenuItem, HyprkubeMenuItem, MenuAction},
    resource_menu::DynamicResourceMenuProvider,
};

pub struct BasicResourceMenu;

impl DynamicResourceMenuProvider for BasicResourceMenu {
    fn matches(&self, _gvk: &GroupVersionKind) -> bool {
        true
    }

    fn build(
        &self,
        gvk: &GroupVersionKind,
        resource: &DynamicObject,
    ) -> anyhow::Result<Vec<HyprkubeMenuItem>> {
        Ok(vec![
            HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: "builtin:edit".into(),
                text: "Edit YAML".into(),
                enabled: true,
                action: Box::new(EditAction {
                    gvk: gvk.clone(),
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource
                        .metadata
                        .name
                        .clone()
                        .context(".metadata.name not set")?,
                }),
            }),
            HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: "builtin:delete".into(),
                enabled: true,
                text: "Delete".into(),
                action: Box::new(DeleteAction {
                    gvk: gvk.clone(),
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource
                        .metadata
                        .name
                        .clone()
                        .context(".metadata.name not set")?,
                }),
            }),
            HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: "builtin:pick-namespace".into(),
                enabled: true,
                text: "Select namespace".into(),
                action: Box::new(PickNamespaceAction {
                    namespace: resource
                        .metadata
                        .namespace
                        .clone()
                        .context(".metadata.namespace not set")?,
                }),
            }),
        ])
    }
}

#[derive(Serialize, Clone)]
struct FrontendTriggerResourceEdit {
    pub gvk: GroupVersionKind,
    pub namespace: String,
    pub name: String,
}

struct EditAction {
    pub gvk: GroupVersionKind,
    pub namespace: String,
    pub name: String,
}

#[async_trait]
impl MenuAction for EditAction {
    async fn run(&self, app: &tauri::AppHandle, _client: kube::Client) -> anyhow::Result<()> {
        use tauri::Emitter as _;

        app.emit(
            "hyprkube:menu:resource:trigger_edit",
            FrontendTriggerResourceEdit {
                gvk: self.gvk.clone(),
                namespace: self.namespace.clone(),
                name: self.name.clone(),
            },
        )?;

        Ok(())
    }
}

struct DeleteAction {
    pub gvk: GroupVersionKind,
    pub namespace: String,
    pub name: String,
}

#[async_trait]
impl MenuAction for DeleteAction {
    async fn run(&self, _app: &tauri::AppHandle, client: kube::Client) -> anyhow::Result<()> {
        use kube::discovery::oneshot::pinned_kind;
        use kube::Api;

        let gvk = &self.gvk;
        let namespace = &self.namespace;
        let name = &self.name;

        println!("Delete was pressed for {gvk:?}, {namespace}/{name}");

        let (api_resource, capabilities) = pinned_kind(&client, gvk).await?;

        let api: Api<DynamicObject> = match capabilities.scope {
            kube::discovery::Scope::Cluster => kube::Api::all_with(client, &api_resource),
            kube::discovery::Scope::Namespaced => {
                kube::Api::namespaced_with(client, namespace, &api_resource)
            }
        };

        api.delete(name, &DeleteParams::default()).await?;

        Ok(())
    }
}

#[derive(Serialize, Clone)]
struct FrontendTriggerPickNamespace {
    pub namespace: String,
}

struct PickNamespaceAction {
    namespace: String,
}

#[async_trait]
impl MenuAction for PickNamespaceAction {
    async fn run(&self, app: &tauri::AppHandle, _client: kube::Client) -> anyhow::Result<()> {
        use tauri::Emitter as _;

        app.emit(
            "hyprkube:menu:resource:pick_namespace",
            FrontendTriggerPickNamespace {
                namespace: self.namespace.clone(),
            },
        )
        .context("Failed to notify frontend")?;

        Ok(())
    }
}
