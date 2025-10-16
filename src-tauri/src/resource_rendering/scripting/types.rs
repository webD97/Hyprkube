use std::collections::HashMap;

use rhai::{CustomType, Dynamic, EvalAltResult, Position, TypeBuilder};
use serde::Serialize;
use tracing::warn;

#[derive(Clone, Serialize)]
pub enum ResourceViewField {
    Text(Text),
    RelativeTime(RelativeTime),
    Hyperlink(Hyperlink),
    ColoredBox(ColoredBox),
    ColoredBoxes(ColoredBoxes),
}

impl From<ResourceViewField> for DisplayValue {
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

#[derive(Default, Clone, Serialize)]
pub struct Properties {
    /// A CSS-compatiable color string, e.g. "red", "#ffffff", "rgb(0, 0, 0)"
    pub color: Option<String>,
    /// Additional info to show in a tooltip
    pub title: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct DisplayValue {
    pub kind: &'static str,
    pub args: serde_json::Value,
    pub properties: Option<Properties>,
    pub sortable_value: String,
}

impl From<Text> for DisplayValue {
    fn from(value: Text) -> Self {
        Self {
            kind: "Text",
            properties: value.properties,
            sortable_value: value.content.clone(),
            args: serde_json::to_value(HashMap::from([("content", value.content)])).unwrap(),
        }
    }
}

impl From<RelativeTime> for DisplayValue {
    fn from(value: RelativeTime) -> Self {
        Self {
            kind: "RelativeTime",
            args: serde_json::to_value(HashMap::from([("timestamp", value.timestamp.clone())]))
                .unwrap(),
            properties: value.properties,
            sortable_value: value
                .timestamp
                .parse::<chrono::DateTime<chrono::Utc>>()
                .expect("invalid iso timestamp")
                .timestamp()
                .to_string(),
        }
    }
}

impl From<Hyperlink> for DisplayValue {
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

impl From<ColoredBox> for DisplayValue {
    fn from(value: ColoredBox) -> Self {
        Self {
            kind: "ColoredBox",
            args: serde_json::to_value(HashMap::from([("color", value.color.clone())])).unwrap(),
            properties: value.properties,
            sortable_value: value.color,
        }
    }
}

impl From<ColoredBoxes> for DisplayValue {
    fn from(value: ColoredBoxes) -> Self {
        Self {
            kind: "ColoredBoxes",
            args: serde_json::to_value(HashMap::from([("boxes", value.boxes.clone())])).unwrap(),
            properties: value.properties,
            sortable_value: value.boxes.len().to_string(),
        }
    }
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

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct RelativeTime {
    pub timestamp: String,
    pub properties: Option<Properties>,
}

impl RelativeTime {
    pub fn new_with_props(timestamp: String, props: rhai::Map) -> Self {
        RelativeTime {
            timestamp,
            properties: Some(props.into()),
        }
    }

    pub fn new(timestamp: String) -> Self {
        RelativeTime {
            timestamp,
            properties: None,
        }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("RelativeTime", Self::new_with_props);
        builder.with_fn("RelativeTime", Self::new);
    }
}

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct Hyperlink {
    pub url: String,
    pub content: String,
    pub properties: Option<Properties>,
}

impl Hyperlink {
    pub fn new_with_props(content: String, url: String, props: rhai::Map) -> Self {
        Hyperlink {
            url,
            content,
            properties: Some(props.into()),
        }
    }

    pub fn new(content: String, url: String) -> Self {
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

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColoredBox {
    pub color: String,
    pub properties: Option<Properties>,
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

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColoredBoxes {
    pub boxes: Vec<Vec<ColoredBox>>,
    pub properties: Option<Properties>,
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
