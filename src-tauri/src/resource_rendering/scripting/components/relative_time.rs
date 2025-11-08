use std::collections::HashMap;

use rhai::{CustomType, Dynamic, EvalAltResult, Position, TypeBuilder};
use serde::Serialize;

use crate::resource_rendering::scripting::types::{Properties, ViewComponent};

/// Displays a relative time from a timestamp, e.g. "1h15m".
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

impl From<RelativeTime> for ViewComponent {
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
