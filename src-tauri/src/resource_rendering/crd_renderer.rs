use super::ResourceRenderer;
use crate::{
    frontend_types::BackendError,
    resource_rendering::scripting::{
        components::{RelativeTime, Text},
        types::{Properties, ResourceViewField},
    },
};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;
use serde_json::{json, Value};
use serde_json_path::JsonPath;

#[derive(Default)]
pub struct CrdRenderer {}

impl ResourceRenderer for CrdRenderer {
    fn display_name(&self) -> &str {
        "Custom resource default view"
    }

    fn column_definitions(
        &self,
        _gvk: &GroupVersionKind,
        crd: Option<&CustomResourceDefinition>,
    ) -> Result<Vec<super::ResourceColumnDefinition>, BackendError> {
        let crd = crd.expect("must pass a CustomResourceDefinition");

        let crd_version = crd.spec.versions.first().ok_or("CRD version not found")?;

        let mut columns = vec![super::ResourceColumnDefinition {
            title: "Name".into(),
            filterable: true,
        }];

        if crd.spec.scope == "Namespaced" {
            columns.push(super::ResourceColumnDefinition {
                title: "Namespace".into(),
                filterable: true,
            });
        }

        if let Some(apts) = crd_version.additional_printer_columns.as_ref() {
            apts.iter().map(|c| c.clone().name).for_each(|name| {
                columns.push(super::ResourceColumnDefinition {
                    title: name,
                    filterable: true,
                })
            });
        }

        if !columns.iter().any(|c| c.title == "Age") {
            columns.push(super::ResourceColumnDefinition {
                title: "Age".into(),
                filterable: true,
            });
        }

        Ok(columns)
    }

    fn render(
        &self,
        _gvk: &GroupVersionKind,
        crd: Option<&CustomResourceDefinition>,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<ResourceViewField, String>>, BackendError> {
        let crd = crd.expect("must pass a CustomResourceDefinition");

        let crd_version = crd
            .spec
            .versions
            .first()
            .ok_or(BackendError::from("CRD version not found"))?;

        let mut values: Vec<Result<ResourceViewField, String>> = vec![];

        values.push(Ok(ResourceViewField::Text(Text {
            content: obj
                .metadata
                .name
                .as_ref()
                .unwrap_or(&"".to_owned())
                .to_owned(),
            properties: None,
        })));

        if crd.spec.scope == "Namespaced" {
            values.push(Ok(ResourceViewField::Text(Text {
                content: obj
                    .metadata
                    .namespace
                    .as_ref()
                    .unwrap_or(&"".to_owned())
                    .to_owned(),
                properties: None,
            })));
        }

        let mut has_own_age_column = false;

        if let Some(apts) = crd_version.additional_printer_columns.as_ref() {
            let obj = json!(obj);
            let empty_str = json!("");

            apts.iter()
                .map(|c| (c.name.clone(), c.json_path.clone(), c.type_.clone()))
                .map(|(title, json_path, type_)| {
                    has_own_age_column = has_own_age_column || (title == *"Age");

                    let value = JsonPath::parse(format!("${json_path}").as_str())
                        .map_err(|e| format!("\"{json_path}\": {e}"))
                        .map(|jsonpath| {
                            jsonpath
                                .query(&obj)
                                .at_most_one()
                                .ok()
                                .flatten()
                                .unwrap_or(&empty_str)
                        })
                        .map(|e| match e {
                            Value::String(value) => value.to_owned(),
                            other => other.to_string(),
                        });

                    match value {
                        Err(e) => ResourceViewField::Text(Text {
                            content: e,
                            properties: Some(Properties {
                                color: Some("red".into()),
                                ..Default::default()
                            }),
                        }),
                        Ok(value) => {
                            if type_ == *"date" {
                                return ResourceViewField::RelativeTime(RelativeTime {
                                    timestamp: value.to_owned(),
                                    properties: None,
                                });
                            }

                            ResourceViewField::Text(Text {
                                content: value.to_owned(),
                                properties: None,
                            })
                        }
                    }
                })
                .for_each(|value| values.push(Ok(value)));
        }

        if !has_own_age_column {
            values.push(Ok(ResourceViewField::RelativeTime(RelativeTime {
                timestamp: obj
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .map_or("".into(), |v| v.0.to_rfc3339())
                    .to_owned(),
                properties: None,
            })));
        }

        Ok(values)
    }
}

impl CrdRenderer {}
