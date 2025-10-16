use std::collections::HashMap;

use rhai::{CustomType, Dynamic, EvalAltResult, Position, TypeBuilder};
use serde::Serialize;

use crate::resource_rendering::scripting::types::{Properties, ViewComponent};

/// Displays plain text from a string.
#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct Text {
    pub content: String,
    pub properties: Option<Properties>,
}

impl Text {
    pub fn new_with_props(content: String, props: rhai::Map) -> Self {
        Text {
            content,
            properties: Some(props.into()),
        }
    }

    pub fn new(content: String) -> Self {
        Text {
            content,
            properties: None,
        }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("Text", Self::new_with_props);
        builder.with_fn("Text", Self::new);
    }
}

impl From<Text> for ViewComponent {
    fn from(value: Text) -> Self {
        Self {
            kind: "Text",
            properties: value.properties,
            sortable_value: value.content.clone(),
            args: serde_json::to_value(HashMap::from([("content", value.content)])).unwrap(),
        }
    }
}
