mod crd_renderer;
mod fallback_resource_renderer;
mod frontend;
mod resource_view_definition;
mod scripted_resource_renderer;

use crate::frontend_types::{BackendError, FrontendValue};
pub use crd_renderer::*;
pub use fallback_resource_renderer::*;
pub use frontend::*;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;
pub use scripted_resource_renderer::*;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ResourceColumnDefinition {
    pub title: String,
    pub filterable: bool,
}

pub trait ResourceRenderer: Send + Sync {
    fn display_name(&self) -> &str;

    fn column_definitions(
        &self,
        gvk: &GroupVersionKind,
        crd: Option<&CustomResourceDefinition>,
    ) -> Result<Vec<ResourceColumnDefinition>, BackendError>;

    fn render(
        &self,
        gvk: &GroupVersionKind,
        crd: Option<&CustomResourceDefinition>,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<Vec<FrontendValue>, String>>, BackendError>;
}
