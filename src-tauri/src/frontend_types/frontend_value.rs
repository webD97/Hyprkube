use serde::Serialize;

use crate::resource_rendering::{ColoredBox, ColoredString};

#[derive(Clone, Serialize)]
pub enum FrontendValue {
    PlainString(String),
    ColoredString(ColoredString),
    ColoredBox(ColoredBox),
}
