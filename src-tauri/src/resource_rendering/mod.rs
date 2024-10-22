mod crd_renderer;
mod fallback_resource_renderer;
mod frontend;
mod renderer_registry;
mod resource_view_definition;
mod scripted_resource_renderer;

use crate::frontend_types::FrontendValue;
pub use crd_renderer::*;
pub use frontend::*;
pub use renderer_registry::*;
pub use scripted_resource_renderer::*;

pub trait ResourceRenderer: Send + Sync {
    fn display_name(&self) -> &str;
    fn titles(&self) -> Vec<String>;

    fn render(&self, obj: &kube::api::DynamicObject) -> Vec<Result<Vec<FrontendValue>, String>>;
}
