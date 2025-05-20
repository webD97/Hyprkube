use futures::{Stream, StreamExt as _};
use kube::{
    runtime::{
        watcher::{self, Event, InitialListStrategy},
        WatchStreamExt as _,
    },
    Api, Client, Resource,
};
use semver::VersionReq;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ResourceWatchStreamEvent<K: Resource + Clone + DeserializeOwned + Debug + Send + 'static> {
    Applied { resource: K },
    Deleted { resource: K },
}

/// Create a resource watch stream for a Kubernetes resource.
pub async fn watch<K>(api: Api<K>) -> impl Stream<Item = ResourceWatchStreamEvent<K>>
where
    K: Resource + Clone + DeserializeOwned + std::fmt::Debug + Send + 'static,
{
    kube::runtime::watcher(
        api.clone(),
        watcher::Config {
            initial_list_strategy: determine_initial_list_strategy(api.into_client()).await,
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

pub async fn determine_initial_list_strategy(client: Client) -> InitialListStrategy {
    let streaming_list_requirement =
        VersionReq::parse(">=1.32,<1.33").expect("must be a valid semver");

    let apiserver_version = client.apiserver_version().await.unwrap();
    let apiserver_version = apiserver_version.git_version.trim_start_matches('v');

    let apiserver_version = semver::Version::parse(apiserver_version)
        .expect("expected a valid version from apiserver request");

    match streaming_list_requirement.matches(&apiserver_version) {
        true => {
            println!("Using StreamingList");
            InitialListStrategy::StreamingList
        }
        false => {
            println!("Using ListWatch");
            InitialListStrategy::ListWatch
        }
    }
}
