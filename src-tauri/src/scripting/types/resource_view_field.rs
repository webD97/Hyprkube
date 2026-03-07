use serde::Serialize;

use crate::scripting::types::{
    ColoredBox, ColoredBoxes, Hyperlink, RelativeTime, Text, ViewComponent,
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
