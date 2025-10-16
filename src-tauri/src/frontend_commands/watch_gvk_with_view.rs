use std::sync::Arc;

use futures::StreamExt as _;
use kube::api::DynamicObject;
use serde::Serialize;
use tauri::State;
use tracing::{error, info};

use crate::{
    app_state::{ClientId, JoinHandleStoreState, KubernetesClientRegistryState, RendererRegistry},
    frontend_types::BackendError,
    internal::resources::ResourceWatchStreamEvent,
    resource_rendering::{scripting::types::ViewComponent, ResourceColumnDefinition},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ResourceEvent {
    #[serde(rename_all = "camelCase")]
    AnnounceColumns {
        columns: Vec<ResourceColumnDefinition>,
    },
    Applied {
        uid: String,
        namespace: String,
        name: String,
        columns: Vec<Result<ViewComponent, String>>,
    },
    Deleted {
        uid: String,
        namespace: String,
        name: String,
    },
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn watch_gvk_with_view(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    views: State<'_, Arc<RendererRegistry>>,
    client_id: ClientId,
    gvk: kube::api::GroupVersionKind,
    view_name: String,
    channel: tauri::ipc::Channel<ResourceEvent>,
    namespace: &str,
) -> Result<(), BackendError> {
    crate::internal::tracing::set_span_request_id();

    let channel_id = channel.id();
    info!("Streaming {gvk:?} in namespace {namespace} to channel {channel_id}");

    let client = client_registry_arc.try_clone(&client_id)?;

    let (api_resource, resource_capabilities) =
        kube::discovery::oneshot::pinned_kind(&client, &gvk).await?;

    let api = match resource_capabilities.scope {
        kube::discovery::Scope::Cluster => {
            kube::Api::<DynamicObject>::all_with(client, &api_resource)
        }
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
        let column_definitions = view.column_definitions(&gvk, crd).unwrap();

        channel
            .send(ResourceEvent::AnnounceColumns {
                columns: column_definitions,
            })
            .unwrap();

        crate::internal::resources::watch(api)
            .await
            .map(|event| match event {
                ResourceWatchStreamEvent::Applied { resource } => ResourceEvent::Applied {
                    uid: resource.metadata.uid.clone().expect("no uid"),
                    namespace: resource.metadata.namespace.clone().unwrap_or_default(),
                    name: resource.metadata.name.clone().unwrap_or_default(),
                    columns: view
                        .render(&gvk, crd, &resource)
                        .unwrap()
                        .into_iter()
                        .map(|value| value.map(|inner| inner.into()))
                        .collect(),
                },
                ResourceWatchStreamEvent::Deleted { resource } => ResourceEvent::Deleted {
                    uid: resource.metadata.uid.expect("no uid"),
                    namespace: resource.metadata.namespace.unwrap_or_default(),
                    name: resource.metadata.name.unwrap_or_default(),
                },
            })
            .for_each(|frontend_event| async {
                if let Err(error) = channel.send(frontend_event) {
                    error!("error sending to channel: {error}")
                }
            })
            .await;
    };

    join_handle_store.submit(channel_id, stream)?;

    Ok(())
}
