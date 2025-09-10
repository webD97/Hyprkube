use k8s_openapi::api::core::v1::Pod;
use serde::Serialize;
use tauri::State;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::info;

use crate::{
    app_state::{ClientId, JoinHandleStoreState, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum LogStreamEvent {
    #[serde(rename_all = "camelCase")]
    NewLine {
        lines: Vec<String>,
    },
    Error {
        msg: String,
    },
    EndOfStream {},
}

#[tauri::command]
pub async fn kube_stream_podlogs(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    client_id: ClientId,
    namespace: &str,
    name: &str,
    container: &str,
    channel: tauri::ipc::Channel<LogStreamEvent>,
) -> Result<(), BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;
    let namespace = namespace.to_string();
    let name = name.to_string();
    let container = container.to_string();
    let channel_id = channel.id();

    info!("kube_stream_podlogs: channel {namespace}/{name} to {channel_id}");

    let stream_task = async move {
        let pods: kube::Api<Pod> = kube::Api::namespaced(client, &namespace);

        let log_params = kube::api::LogParams {
            follow: true,
            tail_lines: Some(1000),
            timestamps: false,
            container: Some(container),
            ..Default::default()
        };

        match pods.log_stream(&name, &log_params).await {
            Ok(logs) => {
                let log_stream = logs.compat();
                let mut reader = BufReader::new(log_stream).lines();

                loop {
                    match reader.next_line().await {
                        Ok(Some(mut line)) => {
                            line.push('\n');
                            let _ = channel.send(LogStreamEvent::NewLine { lines: vec![line] });
                        }
                        Ok(None) => {
                            let _ = channel.send(LogStreamEvent::EndOfStream {});
                            break;
                        }
                        Err(error) => {
                            let _ = channel.send(LogStreamEvent::Error {
                                msg: error.to_string(),
                            });
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                let _ = channel.send(LogStreamEvent::Error {
                    msg: err.to_string(),
                });
            }
        };
    };

    join_handle_store.submit(channel_id, stream_task)?;

    Ok(())
}
