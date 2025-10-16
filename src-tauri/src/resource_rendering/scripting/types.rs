use serde::Serialize;
use tracing::warn;

use crate::resource_rendering::scripting::components::{
    ColoredBox, ColoredBoxes, Hyperlink, RelativeTime, Text,
};

#[derive(Clone, Serialize)]
pub enum ResourceViewField {
    Text(Text),
    RelativeTime(RelativeTime),
    Hyperlink(Hyperlink),
    ColoredBox(ColoredBox),
    ColoredBoxes(ColoredBoxes),
}

impl From<ResourceViewField> for ViewComponent {
    fn from(value: ResourceViewField) -> Self {
        match value {
            ResourceViewField::Text(value) => value.into(),
            ResourceViewField::RelativeTime(value) => value.into(),
            ResourceViewField::Hyperlink(value) => value.into(),
            ResourceViewField::ColoredBox(value) => value.into(),
            ResourceViewField::ColoredBoxes(value) => value.into(),
        }
    }
}

/// Properties that all components can use to modify their appearance or behaviour.
#[derive(Default, Clone, Serialize)]
pub struct Properties {
    /// A CSS-compatiable color string, e.g. "red", "#ffffff", "rgb(0, 0, 0)"
    pub color: Option<String>,
    /// Additional info to show in a tooltip
    pub title: Option<String>,
}

/// A serializable generic representation of any component that the frontend can display in a resource view.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewComponent {
    pub kind: &'static str,
    pub args: serde_json::Value,
    pub properties: Option<Properties>,
    pub sortable_value: String,
}

impl From<rhai::Map> for Properties {
    fn from(value: rhai::Map) -> Self {
        let color = value.get("color").and_then(|v| {
            v.clone()
                .into_string()
                .map_err(|_| warn!("Property 'color' must be a string"))
                .ok()
        });

        let title = value.get("title").and_then(|v| {
            v.clone()
                .into_string()
                .map_err(|_| warn!("Property 'title' must be a string"))
                .ok()
        });

        Self { color, title }
    }
}
