use serde::Serialize;

use crate::resource_rendering::{ColoredBox, ColoredString, Hyperlink, RelativeTime};

#[derive(Clone, Serialize)]
pub enum FrontendValue {
    PlainString(String),
    Hyperlink(Hyperlink),
    ColoredString(ColoredString),
    ColoredBox(ColoredBox),
    RelativeTime(RelativeTime),
}
