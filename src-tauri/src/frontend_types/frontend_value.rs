use serde::Serialize;

use crate::resource_rendering::{ColoredBox, ColoredString, Hyperlink};

#[derive(Clone, Serialize)]
pub enum FrontendValue {
    PlainString(String),
    Hyperlink(Hyperlink),
    ColoredString(ColoredString),
    ColoredBox(ColoredBox),
}
