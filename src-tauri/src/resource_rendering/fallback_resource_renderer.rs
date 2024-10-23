use async_trait::async_trait;
use kube::api::GroupVersionKind;
use uuid::Uuid;

use crate::frontend_types::{BackendError, FrontendValue};

use super::ResourceRenderer;

pub struct FallbackRenderer {}

#[async_trait]
impl ResourceRenderer for FallbackRenderer {
    fn display_name(&self) -> &str {
        "Minimal default"
    }

    async fn titles(
        &self,
        _app_handle: tauri::AppHandle,
        _client_id: &Uuid,
        _gvk: &GroupVersionKind,
    ) -> Result<Vec<String>, BackendError> {
        Ok(vec!["Namespace".into(), "Name".into(), "Age".into()])
    }

    async fn render(
        &self,
        _app_handle: tauri::AppHandle,
        _client_id: &Uuid,
        _gvk: &GroupVersionKind,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<Vec<FrontendValue>, String>>, BackendError> {
        Ok(vec![
            Ok(vec![FrontendValue::PlainString(
                obj.metadata.clone().namespace.or(Some("".into())).unwrap(),
            )]),
            Ok(vec![FrontendValue::PlainString(
                obj.metadata.clone().name.or(Some("".into())).unwrap(),
            )]),
            Ok(vec![FrontendValue::PlainString(
                obj.metadata
                    .clone()
                    .creation_timestamp
                    .map_or("".into(), |v| v.0.to_rfc3339()),
            )]),
        ])
    }
}
