use serde::Serialize;

use crate::scripting::types::Properties;

/// A serializable generic representation of any component that the frontend can display in a resource view.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewComponent {
    pub kind: &'static str,
    pub args: serde_json::Value,
    pub properties: Option<Properties>,
    pub sortable_value: String,
}
