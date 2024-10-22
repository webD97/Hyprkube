use super::ResourceRenderer;
use crate::{app_state::KubernetesClientRegistry, frontend_types::FrontendValue};
use async_trait::async_trait;
use kube::api::GroupVersionKind;
use serde_json::json;
use serde_json_path::JsonPath;
use tauri::Manager as _;
use uuid::Uuid;

#[derive(Default)]
pub struct CrdRenderer {}

#[async_trait]
impl ResourceRenderer for CrdRenderer {
    fn display_name(&self) -> &str {
        "Custom resource default view"
    }

    async fn titles(
        &self,
        app_handle: tauri::AppHandle,
        client_id: &Uuid,
        gvk: &GroupVersionKind,
    ) -> Vec<String> {
        let kubernetes_client_registry =
            app_handle.state::<tokio::sync::Mutex<KubernetesClientRegistry>>();
        let kubernetes_client_registry = kubernetes_client_registry.lock().await;

        let (_, discovery) = &kubernetes_client_registry
            .registered
            .get(&client_id)
            .unwrap();

        let crd = discovery.crds.get(&gvk).unwrap();
        let crd_version = crd.spec.versions.first().unwrap();

        let mut columns = vec!["Name".to_owned()];

        if crd.spec.scope == "Namespaced" {
            columns.push("Namespace".to_owned());
        }

        if let Some(apts) = crd_version.additional_printer_columns.as_ref() {
            apts.iter()
                .map(|c| c.clone().name)
                .for_each(|name| columns.push(name));
        }

        columns.push("Age".to_owned());

        columns
    }

    async fn render(
        &self,
        app_handle: tauri::AppHandle,
        client_id: &Uuid,
        gvk: &GroupVersionKind,
        obj: &kube::api::DynamicObject,
    ) -> Vec<Result<Vec<crate::frontend_types::FrontendValue>, String>> {
        let kubernetes_client_registry =
            app_handle.state::<tokio::sync::Mutex<KubernetesClientRegistry>>();
        let kubernetes_client_registry = kubernetes_client_registry.lock().await;

        let (_, discovery) = &kubernetes_client_registry
            .registered
            .get(&client_id)
            .unwrap();

        let crd = discovery.crds.get(&gvk).unwrap();
        let crd_version = crd.spec.versions.first().unwrap();

        let mut values: Vec<Result<Vec<crate::frontend_types::FrontendValue>, String>> = vec![];

        values.push(Ok(vec![FrontendValue::PlainString(
            obj.metadata
                .name
                .as_ref()
                .unwrap_or(&"".to_owned())
                .to_owned(),
        )]));

        if crd.spec.scope == "Namespaced" {
            values.push(Ok(vec![FrontendValue::PlainString(
                obj.metadata
                    .namespace
                    .as_ref()
                    .unwrap_or(&"".to_owned())
                    .to_owned(),
            )]));
        }

        if let Some(apts) = crd_version.additional_printer_columns.as_ref() {
            let obj = json!(obj);
            let empty_str = json!("");

            apts.iter()
                .map(|c| c.clone().json_path)
                .map(|json_path| {
                    JsonPath::parse(format!("${}", json_path).as_str())
                        .unwrap()
                        .query(&obj)
                        .at_most_one()
                        .ok()
                        .flatten()
                        .unwrap_or(&empty_str)
                        .as_str()
                        .unwrap_or("")
                })
                .map(|s| Ok(vec![FrontendValue::PlainString(s.to_owned())]))
                .for_each(|value| values.push(value));
        }

        values.push(Ok(vec![FrontendValue::RelativeTime(super::RelativeTime {
            iso: obj
                .metadata
                .creation_timestamp
                .as_ref()
                .map_or("".into(), |v| v.0.to_rfc3339())
                .to_owned(),
        })]));

        values
    }
}

impl CrdRenderer {}
