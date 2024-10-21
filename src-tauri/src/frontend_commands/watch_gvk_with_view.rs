use std::sync::{Arc, Mutex};

use futures::{StreamExt as _, TryStreamExt as _};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::{
    app_state::{JoinHandleStore, KubernetesClientRegistry},
    frontend_types::{BackendError, FrontendValue},
    resource_rendering::RendererRegistry,
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

#[tauri::command]
pub async fn watch_gvk_with_view(
    client_registry_arc: State<'_, Mutex<KubernetesClientRegistry>>,
    join_handle_store: State<'_, Arc<Mutex<JoinHandleStore>>>,
    views: State<'_, Arc<RendererRegistry>>,
    client_id: Uuid,
    gvk: kube::api::GroupVersionKind,
    view_name: String,
    channel: tauri::ipc::Channel<WatchStreamEvent>,
) -> Result<(), BackendError> {
    let channel_id = channel.id();
    println!("Streaming {:?} to channel {channel_id}", gvk);

    let client = client_registry_arc
        .lock()
        .unwrap()
        .try_clone(&client_id)
        .unwrap();

    let disovery = kube::Discovery::new(client.clone()).run().await.unwrap();

    let (api_resource, _) = disovery
        .resolve_gvk(&gvk)
        .ok_or(BackendError::Generic(format!(
            "API resource {:?} not found",
            gvk
        )))
        .unwrap();

    let api: kube::Api<kube::api::DynamicObject> =
        kube::Api::all_with(client.clone(), &api_resource);

    let views = Arc::clone(&views);

    let handle = tauri::async_runtime::spawn(async move {
        let mut stream = api
            .watch(&kube::api::WatchParams::default(), "0")
            .await
            .unwrap()
            .boxed();

        let view = views.get_renderer(&gvk, view_name.as_str());

        let column_titles = view.titles();

        channel
            .send(WatchStreamEvent::AnnounceColumns {
                titles: column_titles,
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
                    let columns = view.render(&obj);
                    Some(WatchStreamEvent::Created {
                        uid: obj.metadata.uid.expect("no uid"),
                        namespace: obj.metadata.namespace.or(Some("".into())).unwrap(),
                        name: obj.metadata.name.or(Some("".into())).unwrap(),
                        columns,
                    })
                }
                Some(kube::api::WatchEvent::Modified(obj)) => {
                    let columns = view.render(&obj);
                    Some(WatchStreamEvent::Updated {
                        uid: obj.metadata.uid.expect("no uid"),
                        namespace: obj.metadata.namespace.or(Some("".into())).unwrap(),
                        name: obj.metadata.name.or(Some("".into())).unwrap(),
                        columns,
                    })
                }
                Some(kube::api::WatchEvent::Deleted(obj)) => Some(WatchStreamEvent::Deleted {
                    namespace: obj.metadata.namespace.or(Some("".into())).unwrap(),
                    name: obj.metadata.name.or(Some("".into())).unwrap(),
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
    });

    let mut join_handle_store = join_handle_store.lock().unwrap();
    join_handle_store.insert(channel_id, handle);

    println!(
        "Now there are {} channels with a handle. Total handles: {}",
        join_handle_store.handles.len(),
        join_handle_store
            .handles
            .values()
            .map(|v| v.len())
            .reduce(|p, n| p + n)
            .unwrap()
    );

    Ok(())
}
