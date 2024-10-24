use super::ResourceRenderer;
use crate::frontend_types::{BackendError, FrontendValue};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;
use serde_json::json;
use serde_json_path::JsonPath;

#[derive(Default)]
pub struct CrdRenderer {}

impl ResourceRenderer for CrdRenderer {
    fn display_name(&self) -> &str {
        "Custom resource default view"
    }

    fn titles(
        &self,
        _gvk: &GroupVersionKind,
        crd: Option<&CustomResourceDefinition>,
    ) -> Result<Vec<String>, BackendError> {
        let crd = crd.expect("must pass a CustomResourceDefinition");

        let crd_version = crd.spec.versions.first().ok_or("CRD version not found")?;

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

    fn render(
        &self,
        _gvk: &GroupVersionKind,
        crd: Option<&CustomResourceDefinition>,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<Vec<FrontendValue>, String>>, BackendError> {
        let crd = crd.expect("must pass a CustomResourceDefinition");

        let crd_version = crd
            .spec
            .versions
            .first()
            .ok_or(BackendError::from("CRD version not found"))?;

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
