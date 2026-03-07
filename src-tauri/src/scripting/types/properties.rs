use serde::Serialize;
use tracing::warn;

/// Properties that all components can use to modify their appearance or behaviour.
#[derive(Default, Clone, Serialize)]
pub struct Properties {
    /// A CSS-compatiable color string, e.g. "red", "#ffffff", "rgb(0, 0, 0)"
    pub color: Option<String>,
    /// Additional info to show in a tooltip
    pub title: Option<String>,
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
