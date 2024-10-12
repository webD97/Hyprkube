use std::sync::Mutex;

use k8s_openapi::api::core::v1::Pod;
use serde::Serialize;
use tauri::Manager as _;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::app_state::{self, AppState};

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
    app: tauri::AppHandle,
    namespace: &str,
    name: &str,
    channel: tauri::ipc::Channel<LogStreamEvent>,
) -> Result<(), String> {
    let client = app_state::clone_client(&app)?;
    let pods: kube::Api<Pod> = kube::Api::namespaced(client, namespace);

    let log_params = kube::api::LogParams {
        follow: true,
        tail_lines: Some(1000),
        timestamps: false,
        ..Default::default()
    };

    let logs = match pods.log_stream(name, &log_params).await {
        Ok(stream) => stream,
        Err(error) => return Err(error.to_string()),
    };

    let log_stream = logs.compat();
    let mut reader = BufReader::new(log_stream).lines();

    let channel_id = channel.id();
    println!("kube_stream_podlogs: channel {namespace}/{name} to {channel_id}");

    // Process the log lines
    let handle = tokio::spawn(async move {
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
                    channel.send(LogStreamEvent::Error { msg: error.to_string() }).unwrap();
                    break;
                }
            }
        }
    });

    let app_state = app.state::<Mutex<AppState>>();
    let mut app_state = app_state.lock().unwrap();

    app_state.podlog_stream_handles.insert(channel_id, handle);

    Ok(())
}

#[tauri::command]
pub fn kube_stream_podlogs_cleanup(app: tauri::AppHandle, channel_id: u32) {
    let app_state = app.state::<Mutex<AppState>>();
    let mut app_state = app_state.lock().unwrap();

    if !app_state.podlog_stream_handles.contains_key(&channel_id) {
        println!("kube_stream_podlogs_cleanup: channel {channel_id}, nothing to do.");
        return;
    }

    let handler = app_state.podlog_stream_handles.get(&channel_id).unwrap();
    handler.abort();
    println!("kube_stream_podlogs_cleanup: channel {channel_id}, handler aborted.");

    app_state.podlog_stream_handles.remove(&channel_id);
}
