use std::sync::Mutex;

use k8s_openapi::api::core::v1::Pod;
use serde::Serialize;
use tauri::State;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use uuid::Uuid;

use crate::{app_state::KubernetesClientRegistry, frontend_types::BackendError};

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
    client_registry_arc: State<'_, Mutex<KubernetesClientRegistry>>,
    client_id: Uuid,
    namespace: &str,
    name: &str,
    channel: tauri::ipc::Channel<LogStreamEvent>,
) -> Result<(), BackendError> {
    let client = client_registry_arc
        .lock()
        .map_err(|x| BackendError::Generic(x.to_string()))?
        .try_clone(&client_id)?;

    let pods: kube::Api<Pod> = kube::Api::namespaced(client, namespace);

    let log_params = kube::api::LogParams {
        follow: true,
        tail_lines: Some(1000),
        timestamps: false,
        ..Default::default()
    };

    let logs = pods.log_stream(name, &log_params).await?;

    let log_stream = logs.compat();
    let mut reader = BufReader::new(log_stream).lines();

    let channel_id = channel.id();
    println!("kube_stream_podlogs: channel {namespace}/{name} to {channel_id}");

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

    Ok(())
}
