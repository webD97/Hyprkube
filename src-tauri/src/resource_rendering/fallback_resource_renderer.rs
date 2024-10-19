use crate::frontend_types::FrontendValue;

use super::ResourceRenderer;

pub struct FallbackRenderer {}

impl ResourceRenderer for FallbackRenderer {
    fn display_name(&self) -> &str {
        "Minimal default"
    }

    fn titles(&self) -> Vec<String> {
        vec!["Namespace".into(), "Name".into(), "Age".into()]
    }

    fn render(&self, obj: &kube::api::DynamicObject) -> Vec<Result<Vec<FrontendValue>, String>> {
        vec![
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
        ]
    }
}
