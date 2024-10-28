use k8s_openapi::api::core::v1::Pod;
use serde::Serialize;
use tauri::State;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use uuid::Uuid;

use crate::{
    app_state::{JoinHandleStoreState, KubernetesClientRegistryState},
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
    client_id: Uuid,
    namespace: &str,
    name: &str,
    channel: tauri::ipc::Channel<LogStreamEvent>,
) -> Result<(), BackendError> {
    println!("kube_stream_podlogs: Getting client");
    let client = client_registry_arc.try_clone(&client_id).await?;
    println!("kube_stream_podlogs: Got client");

    let pods: kube::Api<Pod> = kube::Api::namespaced(client, namespace);

    let log_params = kube::api::LogParams {
        follow: true,
        tail_lines: Some(1000),
        timestamps: false,
        ..Default::default()
    };

    println!("kube_stream_podlogs: Setting up stream");
    let logs = pods.log_stream(name, &log_params).await?;
    println!("kube_stream_podlogs: Stream set up");

    let log_stream = logs.compat();
    let mut reader = BufReader::new(log_stream).lines();

    let channel_id = channel.id();
    println!("kube_stream_podlogs: channel {namespace}/{name} to {channel_id}");

    let handle = tauri::async_runtime::spawn(async move {
        loop {
            match reader.next_line().await {
                Ok(Some(mut line)) => {
                    line.push('\n');
                    channel
                        .send(LogStreamEvent::NewLine { lines: vec![line] })
                        .unwrap();
                }
                Ok(None) => {
                    channel.send(LogStreamEvent::EndOfStream {}).unwrap();
                    break;
                }
                Err(error) => {
                    channel
                        .send(LogStreamEvent::Error {
                            msg: error.to_string(),
                        })
                        .unwrap();
                    break;
                }
            }
        }
    });

    join_handle_store.lock().await.insert(channel_id, handle);

    Ok(())
}
