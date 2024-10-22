mod crd_renderer;
mod fallback_resource_renderer;
mod frontend;
mod renderer_registry;
mod resource_view_definition;
mod scripted_resource_renderer;

use crate::frontend_types::FrontendValue;
use async_trait::async_trait;
pub use crd_renderer::*;
pub use frontend::*;
use kube::api::GroupVersionKind;
pub use renderer_registry::*;
pub use scripted_resource_renderer::*;
use uuid::Uuid;

#[async_trait]
pub trait ResourceRenderer: Send + Sync {
    fn display_name(&self) -> &str;
    async fn titles(
        &self,
        app_handle: tauri::AppHandle,
        client_id: &Uuid,
        gvk: &GroupVersionKind,
    ) -> Vec<String>;

    async fn render(
        &self,
        app_handle: tauri::AppHandle,
        client_id: &Uuid,
        gvk: &GroupVersionKind,
        obj: &kube::api::DynamicObject,
    ) -> Vec<Result<Vec<FrontendValue>, String>>;
}
