use std::collections::BTreeMap;

use async_trait::async_trait;
use k8s_openapi::api::core::v1::{ConfigMap, Secret};
use kube::api::{DynamicObject, GroupVersionKind};

use crate::{
    menus::{HyprkubeActionMenuItem, HyprkubeActionSubMenuItem, HyprkubeMenuItem, MenuAction},
    resource_menu::DynamicResourceMenuProvider,
};

/// A menu that can copy the data of Secrets and ConfigMaps to clipboard
pub struct DataKeysResourceMenu;

impl DynamicResourceMenuProvider for DataKeysResourceMenu {
    fn matches(&self, gvk: &GroupVersionKind) -> bool {
        gvk.api_version() == "v1" && (gvk.kind == "Secret" || gvk.kind == "ConfigMap")
    }

    fn build(&self, gvk: &GroupVersionKind, resource: &DynamicObject) -> Vec<HyprkubeMenuItem> {
        let data = {
            match gvk.kind.as_str() {
                "Secret" => {
                    // Kinda hacky and ugly but idc for now
                    let secret: Secret =
                        serde_json::from_value(serde_json::to_value(resource).unwrap()).unwrap();

                    secret
                        .data
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(key, value)| {
                            (
                                key,
                                String::from_utf8(value.0)
                                    .expect("Non UTF-8 secret content is not yet supported"),
                            )
                        })
                        .collect::<BTreeMap<String, String>>()
                }
                "ConfigMap" => {
                    // Kinda hacky and ugly but idc for now
                    let configmap: ConfigMap =
                        serde_json::from_value(serde_json::to_value(resource).unwrap()).unwrap();

                    configmap.data.unwrap_or_default()
                }
                _ => unreachable!(),
            }
        };

        let mut menu = Vec::new();
        let mut data_submenu = Vec::new();

        for (key, data) in data {
            data_submenu.push(HyprkubeMenuItem::Action(HyprkubeActionMenuItem {
                id: format!("bultin:copy_data-{key}"),
                text: key.clone(),
                action: Box::new(CopySecretData { data }),
            }));
        }

        menu.push(HyprkubeMenuItem::Submenu(HyprkubeActionSubMenuItem {
            text: "Copy data".into(),
            items: data_submenu,
        }));

        menu
    }
}

struct CopySecretData {
    pub data: String,
}

#[async_trait]
impl MenuAction for CopySecretData {
    async fn run(&self, app: &tauri::AppHandle, _client: kube::Client) {
        use tauri_plugin_clipboard_manager::ClipboardExt;

        app.clipboard().write_text(self.data.clone()).unwrap();
    }
}
