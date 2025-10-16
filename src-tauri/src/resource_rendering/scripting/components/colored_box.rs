use std::collections::HashMap;

use rhai::{CustomType, Dynamic, EvalAltResult, Position, TypeBuilder};
use serde::Serialize;

use crate::resource_rendering::scripting::types::{Properties, ViewComponent};

/// Displays a single colored box.
#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColoredBox {
    pub color: String,
    pub properties: Option<Properties>,
}

impl From<ColoredBox> for ViewComponent {
    fn from(value: ColoredBox) -> Self {
        Self {
            kind: "ColoredBox",
            args: serde_json::to_value(HashMap::from([("color", value.color.clone())])).unwrap(),
            properties: value.properties,
            sortable_value: value.color,
        }
    }
}

impl ColoredBox {
    pub fn new_with_props(color: String, props: rhai::Map) -> Self {
        Self {
            color,
            properties: Some(props.into()),
        }
    }

    pub fn new(color: String) -> Self {
        Self {
            color,
            properties: None,
        }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("Box", Self::new_with_props);
        builder.with_fn("Box", Self::new);
    }
}

impl TryFrom<rhai::Dynamic> for ColoredBox {
    type Error = &'static str;

    fn try_from(value: rhai::Dynamic) -> Result<Self, Self::Error> {
        value.try_cast::<Self>().ok_or("unsupported type")
    }
}
