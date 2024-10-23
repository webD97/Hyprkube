use super::ResourceRenderer;
use crate::{
    app_state::KubernetesClientRegistry,
    frontend_types::{BackendError, FrontendValue},
};
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
    ) -> Result<Vec<String>, BackendError> {
        let kubernetes_client_registry =
            app_handle.state::<tokio::sync::Mutex<KubernetesClientRegistry>>();
        let kubernetes_client_registry = kubernetes_client_registry.lock().await;

        let (_, discovery) = &kubernetes_client_registry
            .registered
            .get(&client_id)
            .ok_or("Client not found")
            .map_err(|e| BackendError::Generic(e.to_owned()))?;

        let crd = discovery
            .crds
            .get(&gvk)
            .ok_or("CRD not found")
            .map_err(|e| BackendError::Generic(e.to_owned()))?;

        let crd_version = crd
            .spec
            .versions
            .first()
            .ok_or("CRD version not found")
            .map_err(|e| BackendError::Generic(e.to_owned()))?;

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

        Ok(columns)
    }

    async fn render(
        &self,
        app_handle: tauri::AppHandle,
        client_id: &Uuid,
        gvk: &GroupVersionKind,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<Vec<FrontendValue>, String>>, BackendError> {
        let kubernetes_client_registry =
            app_handle.state::<tokio::sync::Mutex<KubernetesClientRegistry>>();
        let kubernetes_client_registry = kubernetes_client_registry.lock().await;

        let (_, discovery) = &kubernetes_client_registry
            .registered
            .get(&client_id)
            .ok_or("Client not found")
            .map_err(|e| BackendError::Generic(e.to_owned()))?;

        let crd = discovery
            .crds
            .get(&gvk)
            .ok_or("CRD not found")
            .map_err(|e| BackendError::Generic(e.to_owned()))?;

        let crd_version = crd
            .spec
            .versions
            .first()
            .ok_or("CRD version not found")
            .map_err(|e| BackendError::Generic(e.to_owned()))?;

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

        Ok(values)
    }
}

impl CrdRenderer {}
