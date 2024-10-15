use serde::Serialize;

#[derive(Clone, Serialize)]
pub enum FrontendValue {
    PlainString(String),
    ColoredString(String, String),
}
