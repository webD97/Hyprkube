use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum WatchEvent<T: Serialize> {
    #[serde(rename_all = "camelCase")]
    Created {
        repr: T,
    },
    Updated {
        repr: T,
    },
    Deleted {
        repr: T,
    },
}
