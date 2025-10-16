use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;

use crate::{
    frontend_types::BackendError,
    resource_rendering::scripting::{
        components::{RelativeTime, Text},
        types::ResourceViewField,
    },
};

use super::ResourceRenderer;

pub struct FallbackRenderer {}

impl ResourceRenderer for FallbackRenderer {
    fn display_name(&self) -> &str {
        "Simple list"
    }

    fn column_definitions(
        &self,
        _gvk: &GroupVersionKind,
        _crd: Option<&CustomResourceDefinition>,
    ) -> Result<Vec<super::ResourceColumnDefinition>, BackendError> {
        Ok(vec![
            super::ResourceColumnDefinition {
                title: "Namespace".into(),
                filterable: true,
            },
            super::ResourceColumnDefinition {
                title: "Name".into(),
                filterable: true,
            },
            super::ResourceColumnDefinition {
                title: "Age".into(),
                filterable: true,
            },
        ])
    }

    fn render(
        &self,
        _gvk: &GroupVersionKind,
        _crd: Option<&CustomResourceDefinition>,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<ResourceViewField, String>>, BackendError> {
        Ok(vec![
            Ok(ResourceViewField::Text(Text {
                content: obj.metadata.clone().namespace.unwrap_or("".into()),
                properties: None,
            })),
            Ok(ResourceViewField::Text(Text {
                content: obj.metadata.clone().name.unwrap_or("".into()),
                properties: None,
            })),
            Ok(ResourceViewField::RelativeTime(RelativeTime {
                timestamp: obj
                    .metadata
                    .clone()
                    .creation_timestamp
                    .map_or("".into(), |v| v.0.to_rfc3339()),
                properties: None,
            })),
        ])
    }
}
