use serde::Serialize;

use crate::resource_views::{ColoredBox, ColoredString};

#[derive(Clone, Serialize)]
pub enum FrontendValue {
    PlainString(String),
    ColoredString(ColoredString),
    ColoredBox(ColoredBox),
}
