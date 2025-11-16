use async_trait::async_trait;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{DynamicObject, GroupVersionKind},
    core::GroupVersion,
};
use tracing::warn;

use crate::{
    menus::{HyprkubeActionMenuItem, HyprkubeMenuItem, MenuAction},
    resource_menu::DynamicResourceMenuProvider,
};

pub struct RolloutRestartResourceMenu;

impl DynamicResourceMenuProvider for RolloutRestartResourceMenu {
    fn matches(&self, gvk: &GroupVersionKind) -> bool {
        match gvk.api_version().as_str() {
            "v1" => gvk.kind == "Pod",
            "apps/v1" => matches!(
                gvk.kind.as_str(),
                "Deployment" | "StatefulSet" | "DaemonSet"
            ),
            _ => false,
        }
    }

    fn build(&self, gvk: &GroupVersionKind, resource: &DynamicObject) -> Vec<HyprkubeMenuItem> {
        match gvk.kind.as_str() {
            "Pod" => {
                // Kinda hacky and ugly but idc for now
                let pod: Pod =
                    serde_json::from_value(serde_json::to_value(resource).unwrap()).unwrap();

                let owner = pod
                    .metadata
                    .owner_references
                    .and_then(|owners| owners.first().cloned());

                if owner.is_none() {
                    return Vec::new();
                }

                let owner = owner.unwrap();
                let gv: GroupVersion = owner.api_version.parse().unwrap();
                let gvk = GroupVersionKind::gvk(&gvk.group, &gv.version, &owner.kind);

                vec![HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                    id: "builtin:rollout_restart".into(),
                    text: "Rollout restart".into(),
                    enabled: true,
                    action: Box::new(RolloutRestart {
                        gvk: gvk.clone(),
                        namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                        name: owner.name,
                    }),
                })]
            }
            "Deployment" | "StatefulSet" | "DaemonSet" => {
                vec![HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                    id: "builtin:rollout_restart".into(),
                    text: "Rollout restart".into(),
                    enabled: true,
                    action: Box::new(RolloutRestart {
                        gvk: gvk.clone(),
                        namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                        name: resource.metadata.name.clone().unwrap_or_default(),
                    }),
                })]
            }
            _ => {
                warn!("Unhandled kind: {}", gvk.kind);
                Vec::new()
            }
        }
    }
}

struct RolloutRestart {
    pub gvk: GroupVersionKind,
    pub namespace: String,
    pub name: String,
}

#[async_trait]
impl MenuAction for RolloutRestart {
    async fn run(&self, _app: &tauri::AppHandle, client: kube::Client) {
        use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
        use kube::Api;

        match self.gvk.kind.as_str() {
            "ReplicaSet" => {
                let api: Api<ReplicaSet> = Api::namespaced(client.clone(), &self.namespace);
                let replicaset = api.get(&self.name).await.unwrap();

                let owner = replicaset
                    .metadata
                    .owner_references
                    .and_then(|owners| owners.first().cloned())
                    .expect("RepliaSet should have an owner reference");

                let api: Api<Deployment> = Api::namespaced(client, &self.namespace);
                api.restart(&owner.name).await.unwrap();
            }
            "Deployment" => {
                let api: Api<Deployment> = Api::namespaced(client, &self.namespace);
                api.restart(&self.name).await.unwrap();
            }
            "StatefulSet" => {
                let api: Api<StatefulSet> = Api::namespaced(client, &self.namespace);
                api.restart(&self.name).await.unwrap();
            }
            "DaemonSet" => {
                let api: Api<DaemonSet> = Api::namespaced(client, &self.namespace);
                api.restart(&self.name).await.unwrap();
            }
            _ => warn!("Unhandled kind: {}", self.gvk.kind),
        }
    }
}
