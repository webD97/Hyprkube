mod crd_renderer;
mod fallback_resource_renderer;
mod frontend;
mod renderer_registry;
mod resource_view_definition;
mod scripted_resource_renderer;

use crate::frontend_types::{BackendError, FrontendValue};
pub use crd_renderer::*;
pub use frontend::*;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;
pub use renderer_registry::*;
pub use scripted_resource_renderer::*;

pub trait ResourceRenderer: Send + Sync {
    fn display_name(&self) -> &str;
    fn titles(
        &self,
        gvk: &GroupVersionKind,
        crd: Option<&CustomResourceDefinition>,
    ) -> Result<Vec<String>, BackendError>;

    fn render(
        &self,
        gvk: &GroupVersionKind,
        crd: Option<&CustomResourceDefinition>,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<Vec<FrontendValue>, String>>, BackendError>;
}
