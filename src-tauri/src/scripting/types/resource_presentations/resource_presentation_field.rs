use serde::Serialize;

use crate::scripting::types::resource_presentations::{
    ColoredBox, ColoredBoxes, Hyperlink, PresentationComponent, RelativeTime, Text,
};

#[derive(Clone, Serialize)]
pub enum ResourcePresentationField {
    Text(Text),
    RelativeTime(RelativeTime),
    Hyperlink(Hyperlink),
    ColoredBox(ColoredBox),
    ColoredBoxes(ColoredBoxes),
}

impl From<ResourcePresentationField> for PresentationComponent {
    fn from(value: ResourcePresentationField) -> Self {
        match value {
            ResourcePresentationField::Text(value) => value.into(),
            ResourcePresentationField::RelativeTime(value) => value.into(),
            ResourcePresentationField::Hyperlink(value) => value.into(),
            ResourcePresentationField::ColoredBox(value) => value.into(),
            ResourcePresentationField::ColoredBoxes(value) => value.into(),
        }
    }
}
