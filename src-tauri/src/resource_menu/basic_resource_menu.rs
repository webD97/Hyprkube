use async_trait::async_trait;
use kube::api::{DeleteParams, DynamicObject, GroupVersionKind};
use serde::Serialize;

use crate::resource_menu::{
    api::{HyprkubeActionMenuItem, HyprkubeMenuItem},
    DynamicResourceMenuProvider,
};

pub struct BasicResourceMenu;

pub struct ResourceMenuContext {
    pub client: kube::Client,
    pub gvk: GroupVersionKind,
    pub namespace: String,
    pub name: String,
}

impl DynamicResourceMenuProvider<ResourceMenuContext> for BasicResourceMenu {
    fn matches(&self, _gvk: &GroupVersionKind) -> bool {
        true
    }

    fn build(
        &self,
        _gvk: &GroupVersionKind,
        _resource: &DynamicObject,
    ) -> Vec<HyprkubeMenuItem<ResourceMenuContext>> {
        vec![
            HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: "builtin:edit".into(),
                text: "Edit YAML".into(),
                action: Box::new(EditAction),
            }),
            HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: "builtin:delete".into(),
                text: "Delete".into(),
                action: Box::new(DeleteAction),
            }),
            HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: "builtin:pick-namespace".into(),
                text: "Select namespace".into(),
                action: Box::new(PickNamespaceAction),
            }),
            HyprkubeMenuItem::Separator,
        ]
    }
}

#[derive(Serialize, Clone)]
struct FrontendTriggerResourceEdit {
    pub gvk: GroupVersionKind,
    pub namespace: String,
    pub name: String,
}

struct EditAction;

#[async_trait]
impl super::api::MenuAction<ResourceMenuContext> for EditAction {
    async fn run(&self, app: &tauri::AppHandle, ctx: ResourceMenuContext) {
        use tauri::Emitter as _;

        println!(
            "Edit was pressed for {:?}, {}/{}",
            ctx.gvk, ctx.namespace, ctx.name
        );

        app.emit(
            "hyprkube:menu:resource:trigger_edit",
            FrontendTriggerResourceEdit {
                gvk: ctx.gvk,
                namespace: ctx.namespace,
                name: ctx.name,
            },
        )
        .unwrap();
    }
}

struct DeleteAction;

#[async_trait]
impl super::api::MenuAction<ResourceMenuContext> for DeleteAction {
    async fn run(&self, _app: &tauri::AppHandle, ctx: ResourceMenuContext) {
        use kube::discovery::oneshot::pinned_kind;
        use kube::Api;

        let gvk = &ctx.gvk;
        let namespace = &ctx.namespace;
        let name = &ctx.name;
        let client = ctx.client;

        println!("Delete was pressed for {gvk:?}, {namespace}/{name}");

        let (api_resource, capabilities) = pinned_kind(&client, gvk).await.unwrap();

        let api: Api<DynamicObject> = match capabilities.scope {
            kube::discovery::Scope::Cluster => kube::Api::all_with(client, &api_resource),
            kube::discovery::Scope::Namespaced => {
                kube::Api::namespaced_with(client, namespace, &api_resource)
            }
        };

        api.delete(name, &DeleteParams::default()).await.unwrap();
    }
}

#[derive(Serialize, Clone)]
struct FrontendTriggerPickNamespace {
    pub namespace: String,
}

struct PickNamespaceAction;

#[async_trait]
impl super::api::MenuAction<ResourceMenuContext> for PickNamespaceAction {
    async fn run(&self, app: &tauri::AppHandle, ctx: ResourceMenuContext) {
        use tauri::Emitter as _;

        app.emit(
            "hyprkube:menu:resource:pick_namespace",
            FrontendTriggerPickNamespace {
                namespace: ctx.namespace,
            },
        )
        .unwrap();
    }
}
