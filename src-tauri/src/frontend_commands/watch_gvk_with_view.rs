use std::{pin::pin, sync::Arc};

use futures::StreamExt as _;
use kube::runtime::{
    watcher::{self, Event},
    WatchStreamExt,
};
use serde::Serialize;
use tauri::State;

use crate::{
    app_state::{ClientId, JoinHandleStoreState, KubernetesClientRegistryState, RendererRegistry},
    frontend_types::{BackendError, FrontendValue},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum WatchStreamEvent {
    #[serde(rename_all = "camelCase")]
    AnnounceColumns { titles: Vec<String> },
    Applied {
        uid: String,
        namespace: String,
        name: String,
        columns: Vec<Result<Vec<FrontendValue>, String>>,
    },
    Deleted {
        uid: String,
        namespace: String,
        name: String,
    },
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn watch_gvk_with_view(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    views: State<'_, Arc<RendererRegistry>>,
    client_id: ClientId,
    gvk: kube::api::GroupVersionKind,
    view_name: String,
    channel: tauri::ipc::Channel<WatchStreamEvent>,
    namespace: &str,
) -> Result<(), BackendError> {
    let channel_id = channel.id();
    println!(
        "Streaming {:?} in namespace {:?} to channel {channel_id}",
        gvk, namespace
    );

    let client = client_registry_arc.try_clone(&client_id)?;

    let (api_resource, resource_capabilities) =
        kube::discovery::oneshot::pinned_kind(&client, &gvk).await?;

    let api = match resource_capabilities.scope {
        kube::discovery::Scope::Cluster => kube::Api::all_with(client, &api_resource),
        kube::discovery::Scope::Namespaced => match namespace {
            "" => kube::Api::all_with(client, &api_resource),
            namespace => kube::Api::namespaced_with(client, namespace, &api_resource),
        },
    };

    let views = Arc::clone(&views);
    let client_registry_arc = Arc::clone(&client_registry_arc);

    let stream = async move {
        let view = views.get_renderer(&gvk, view_name.as_str()).await;

        let (_, _, discovery) = client_registry_arc.get_cluster(&client_id).unwrap();

        let crd = discovery.crds.get(&gvk);

        let column_titles = view.titles(&gvk, crd);

        channel
            .send(WatchStreamEvent::AnnounceColumns {
                titles: column_titles.unwrap(),
            })
            .unwrap();

        let watch_stream = kube::runtime::watcher(
            api,
            watcher::Config {
                initial_list_strategy: watcher::InitialListStrategy::StreamingList,
                ..Default::default()
            },
        );

        let mut stream = pin!(watch_stream.default_backoff());

        while let Some(event) = stream.next().await {
            let downstream_event = match event {
                Ok(Event::Init) => {
                    println!("Watch init");
                    None
                }
                Ok(Event::InitDone) => {
                    println!("Watch init done");
                    None
                }
                Ok(Event::InitApply(obj)) | Ok(Event::Apply(obj)) => {
                    let columns = view.render(&gvk, crd, &obj).unwrap();
                    Some(WatchStreamEvent::Applied {
                        uid: obj.metadata.uid.expect("no uid"),
                        namespace: obj.metadata.namespace.unwrap_or("".into()),
                        name: obj.metadata.name.unwrap_or("".into()),
                        columns,
                    })
                }
                Ok(Event::Delete(obj)) => Some(WatchStreamEvent::Deleted {
                    namespace: obj.metadata.namespace.unwrap_or("".into()),
                    name: obj.metadata.name.unwrap_or("".into()),
                    uid: obj.metadata.uid.expect("no uid"),
                }),
                Err(e) => {
                    eprintln!("Watch error: {e}");
                    None
                }
            };

            if let Some(message) = downstream_event {
                match channel.send(message) {
                    Ok(()) => (),
                    Err(error) => eprintln!("error sending to channel: {error}"),
                }
            }
        }
    };

    join_handle_store.submit(channel_id, stream);

    Ok(())
}
