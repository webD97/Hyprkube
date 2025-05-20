use futures::{Stream, StreamExt as _};
use kube::{
    runtime::{
        watcher::{self, Event},
        WatchStreamExt as _,
    },
    Api, Resource,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ResourceWatchStreamEvent<K: Resource + Clone + DeserializeOwned + Debug + Send + 'static> {
    Applied { resource: K },
    Deleted { resource: K },
}

/// Create a resource watch stream for a Kubernetes resource.
pub fn watch<K>(api: Api<K>) -> impl Stream<Item = ResourceWatchStreamEvent<K>>
where
    K: Resource + Clone + DeserializeOwned + std::fmt::Debug + Send + 'static,
{
    kube::runtime::watcher(
        api,
        watcher::Config {
            initial_list_strategy: watcher::InitialListStrategy::StreamingList,
            ..Default::default()
        },
    )
    .default_backoff()
    .filter_map(|obj| async move {
        match obj {
            Ok(Event::Init) => {
                println!("Watch init");
                None
            }
            Ok(Event::InitDone) => {
                println!("Watch init done");
                None
            }
            Ok(Event::InitApply(obj)) | Ok(Event::Apply(obj)) => {
                Some(ResourceWatchStreamEvent::Applied { resource: obj })
            }
            Ok(Event::Delete(obj)) => Some(ResourceWatchStreamEvent::Deleted { resource: obj }),
            Err(e) => {
                eprintln!("Watch error: {e}");
                None
            }
        }
    })
}
