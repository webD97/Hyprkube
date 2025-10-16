use std::collections::HashMap;

use rhai::{CustomType, Dynamic, EvalAltResult, Position, TypeBuilder};
use serde::Serialize;

use crate::resource_rendering::scripting::{
    components::ColoredBox,
    types::{ViewComponent, Properties},
};

/// Displays one or more groups of colored boxes.
#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColoredBoxes {
    pub boxes: Vec<Vec<ColoredBox>>,
    pub properties: Option<Properties>,
}

impl From<ColoredBoxes> for ViewComponent {
    fn from(value: ColoredBoxes) -> Self {
        Self {
            kind: "ColoredBoxes",
            args: serde_json::to_value(HashMap::from([("boxes", value.boxes.clone())])).unwrap(),
            properties: value.properties,
            sortable_value: value.boxes.len().to_string(),
        }
    }
}

impl ColoredBoxes {
    pub fn new_with_props(boxes: rhai::Array, props: rhai::Map) -> Self {
        let boxes = boxes
            .into_iter()
            .map(|group| {
                group
                    .cast::<rhai::Array>()
                    .into_iter()
                    .map(|item| item.cast::<ColoredBox>())
                    .collect()
            })
            .collect();

        Self {
            boxes,
            properties: Some(props.into()),
        }
    }

    pub fn new(boxes: rhai::Array) -> Self {
        let boxes = boxes
            .into_iter()
            .map(|group| {
                group
                    .cast::<rhai::Array>()
                    .into_iter()
                    .map(|item| item.cast::<ColoredBox>())
                    .collect()
            })
            .collect();

        Self {
            boxes,
            properties: None,
        }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("Boxes", Self::new_with_props);
        builder.with_fn("Boxes", Self::new);
    }
}
