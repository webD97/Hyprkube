use std::collections::HashMap;

use rhai::{CustomType, Dynamic, EvalAltResult, Position, TypeBuilder};
use serde::Serialize;

use crate::resource_rendering::scripting::types::{Properties, ViewComponent};

/// Displays a clickable hyperlink with a display text.
#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct Hyperlink {
    pub url: String,
    pub content: String,
    pub properties: Option<Properties>,
}

impl From<Hyperlink> for ViewComponent {
    fn from(value: Hyperlink) -> Self {
        Self {
            kind: "Hyperlink",
            args: serde_json::to_value(HashMap::from([
                ("url", value.url.clone()),
                ("content", value.content.clone()),
            ]))
            .unwrap(),
            properties: value.properties,
            sortable_value: value.content,
        }
    }
}

impl Hyperlink {
    pub fn new_with_props(url: String, content: String, props: rhai::Map) -> Self {
        Hyperlink {
            url,
            content,
            properties: Some(props.into()),
        }
    }

    pub fn new(url: String, content: String) -> Self {
        Hyperlink {
            url,
            content,
            properties: None,
        }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("Hyperlink", Self::new_with_props);
        builder.with_fn("Hyperlink", Self::new);
    }
}
