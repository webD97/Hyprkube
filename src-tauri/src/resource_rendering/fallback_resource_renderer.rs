use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;

use crate::frontend_types::{BackendError, FrontendValue};

use super::ResourceRenderer;

pub struct FallbackRenderer {}

impl ResourceRenderer for FallbackRenderer {
    fn display_name(&self) -> &str {
        "Minimal default"
    }

    fn titles(
        &self,
        _gvk: &GroupVersionKind,
        _crd: Option<&CustomResourceDefinition>,
    ) -> Result<Vec<String>, BackendError> {
        Ok(vec!["Namespace".into(), "Name".into(), "Age".into()])
    }

    fn render(
        &self,
        _gvk: &GroupVersionKind,
        _crd: Option<&CustomResourceDefinition>,
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