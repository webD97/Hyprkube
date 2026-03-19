use std::collections::HashMap;

use futures::StreamExt as _;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{DynamicObject, GroupVersionKind};
use serde::Serialize;
use serde_json::json;
use tracing::{error, info};

use crate::{
    app_state::{ChannelTasks, ClusterStateRegistry, ManagerExt},
    cluster_discovery::ClusterDiscovery,
    frontend_commands::{KubeContextSource, SERVER_SIDE_PRESENTATION},
    frontend_types::BackendError,
    internal::resources::ResourceWatchStreamEvent,
    resource_rendering::ResourceColumnDefinition,
    scripting::types::resource_presentations::PresentationComponent,
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
        columns: Vec<Result<PresentationComponent, String>>,
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
pub async fn watch_gvk_with_presentation(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    gvk: kube::api::GroupVersionKind,
    presentation_name: String,
    channel: tauri::ipc::Channel<ResourceEvent>,
    namespace: &str,
) -> Result<(), BackendError> {
    crate::internal::tracing::set_span_request_id();

    let clusters = app.state::<ClusterStateRegistry>();
    let channel_tasks = app.state::<ChannelTasks>();

    let channel_id = channel.id();
    info!("Streaming {gvk:?} in namespace {namespace} to channel {channel_id}");

    let discovery = clusters.discovery_for(&context_source)?;
    let client = clusters.client_for(&context_source)?;

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

    let views = clusters.presentation_scripting_for(&context_source)?;

    let stream = async move {
        let view = views.get_renderer(&gvk, presentation_name.as_str()).await;

        let crds: HashMap<GroupVersionKind, CustomResourceDefinition> = match &*discovery {
            ClusterDiscovery::Inflight(inflight) => inflight.block_until_done().await.unwrap().crds,
            ClusterDiscovery::Completed(resources) => resources.crds.clone(),
        };

        let table_mode = presentation_name.as_str() == SERVER_SIDE_PRESENTATION;

        if table_mode {
            let mut age_like_column_index: Option<usize> = None;
            let mut non_default_columns: Vec<usize> = Vec::new();

            crate::internal::resources::watch_table(api)
                .await
                .flat_map(|event| {
                    futures::stream::iter(match event {
                        ResourceWatchStreamEvent::Applied { resource: table } => {
                            if let Some(columns) = table.column_definitions {
                                channel
                                    .send(ResourceEvent::AnnounceColumns {
                                        columns: columns
                                            .into_iter()
                                            .enumerate()
                                            .filter(|(idx, column)| {
                                                let is_non_default = column.priority > 0;
                                                if is_non_default {
                                                    non_default_columns.push(*idx);
                                                }

                                                !is_non_default
                                            })
                                            .map(|(idx, c)| {
                                                if c.name == "Age" {
                                                    age_like_column_index = Some(idx);
                                                }
                                                c
                                            })
                                            .map(|c| ResourceColumnDefinition {
                                                title: c.name,
                                                filterable: true,
                                            })
                                            .collect(),
                                    })
                                    .unwrap();
                            }

                            table
                                .rows
                                .into_iter()
                                .map(|row| ResourceEvent::Applied {
                                    uid: row.object.metadata.uid.expect("no uid"),
                                    namespace: row
                                        .object
                                        .metadata
                                        .namespace
                                        .clone()
                                        .unwrap_or_default(),
                                    name: row.object.metadata.name.clone().unwrap_or_default(),
                                    columns: row
                                        .cells
                                        .into_iter()
                                        .enumerate()
                                        .flat_map(|(idx, cell)| {
                                            if non_default_columns.contains(&idx) {
                                                return None;
                                            }

                                            let cell = if cell.is_string() {
                                                cell.as_str().unwrap().to_owned()
                                            } else {
                                                cell.to_string()
                                            };

                                            if age_like_column_index.is_some_and(|i| i == idx) {
                                                let created_at = row
                                                    .object
                                                    .metadata
                                                    .creation_timestamp
                                                    .as_ref()
                                                    .unwrap();
                                                return Some(Ok(PresentationComponent {
                                                    kind: "RelativeTime",
                                                    args: json!({"timestamp": created_at}),
                                                    properties: None,
                                                    sortable_value: created_at
                                                        .0
                                                        .as_second()
                                                        .to_string(),
                                                }));
                                            }

                                            Some(Ok(PresentationComponent {
                                                kind: "Text",
                                                args: json!({"content": cell}),
                                                properties: None,
                                                sortable_value: cell,
                                            }))
                                        })
                                        .collect(),
                                })
                                .collect::<Vec<_>>()
                        }
                        ResourceWatchStreamEvent::Deleted { resource } => resource
                            .rows
                            .into_iter()
                            .map(|row| ResourceEvent::Deleted {
                                uid: row.object.metadata.uid.expect("no uid"),
                                namespace: row
                                    .object
                                    .metadata
                                    .namespace
                                    .clone()
                                    .unwrap_or_default(),
                                name: row.object.metadata.name.clone().unwrap_or_default(),
                            })
                            .collect(),
                    })
                })
                .for_each(|frontend_event| async {
                    if let Err(error) = channel.send(frontend_event) {
                        error!("error sending to channel: {error}")
                    }
                })
                .await;
        } else {
            let crd = crds.get(&gvk);
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
        }
    };

    channel_tasks.submit(channel_id, stream)?;

    Ok(())
}
