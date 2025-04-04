use std::sync::Arc;

use futures::{StreamExt as _, TryStreamExt as _};
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
    Created {
        uid: String,
        namespace: String,
        name: String,
        columns: Vec<Result<Vec<FrontendValue>, String>>,
    },
    Updated {
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

    let mut stream = api
        .watch(&kube::api::WatchParams::default(), "0")
        .await?
        .boxed();

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

        loop {
            let status = stream.try_next().await;
            let event = match status {
                Ok(event) => event,
                Err(error) => {
                    eprintln!("{error}");
                    None
                }
            };

            let to_send = match event {
                Some(kube::api::WatchEvent::Added(obj)) => {
                    let columns = view.render(&gvk, crd, &obj).unwrap();
                    Some(WatchStreamEvent::Created {
                        uid: obj.metadata.uid.expect("no uid"),
                        namespace: obj.metadata.namespace.unwrap_or("".into()),
                        name: obj.metadata.name.unwrap_or("".into()),
                        columns,
                    })
                }
                Some(kube::api::WatchEvent::Modified(obj)) => {
                    let columns = view.render(&gvk, crd, &obj).unwrap();
                    Some(WatchStreamEvent::Updated {
                        uid: obj.metadata.uid.expect("no uid"),
                        namespace: obj.metadata.namespace.unwrap_or("".into()),
                        name: obj.metadata.name.unwrap_or("".into()),
                        columns,
                    })
                }
                Some(kube::api::WatchEvent::Deleted(obj)) => Some(WatchStreamEvent::Deleted {
                    namespace: obj.metadata.namespace.unwrap_or("".into()),
                    name: obj.metadata.name.unwrap_or("".into()),
                    uid: obj.metadata.uid.expect("no uid"),
                }),
                Some(kube::api::WatchEvent::Bookmark(_obj)) => None,
                Some(kube::api::WatchEvent::Error(error)) => {
                    eprintln!("{error}");
                    return;
                }
                None => None,
            };

            if let Some(message) = to_send {
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
