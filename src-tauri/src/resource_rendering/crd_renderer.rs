use super::ResourceRenderer;

#[derive(Default)]
pub struct CrdRenderer {}

impl ResourceRenderer for CrdRenderer {
    fn display_name(&self) -> &str {
        "Custom resource default view"
    }

    fn titles(&self) -> Vec<String> {
        todo!()
    }

    fn render(
        &self,
        _obj: &kube::api::DynamicObject,
    ) -> Vec<Result<Vec<crate::frontend_types::FrontendValue>, String>> {
        todo!()
    }
}

impl CrdRenderer {}
