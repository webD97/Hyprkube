use serde::Serialize;

use crate::resource_views::ColoredString;

#[derive(Clone, Serialize)]
pub enum FrontendValue {
    PlainString(String),
    ColoredString(ColoredString),
}
