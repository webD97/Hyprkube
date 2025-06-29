use core::str;
use std::io::Read;

use crate::{
    app_state::{
        ClientId, ExecSessionError, ExecSessionId, ExecSessionsState, JoinHandleStoreState,
        KubernetesClientRegistryState,
    },
    frontend_types::BackendError,
};
use futures::TryStreamExt as _;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{AttachParams, TerminalSize};
use serde::Serialize;
use tauri::{ipc::Channel, State};
use tokio::{io::AsyncWriteExt, sync::mpsc};
use tokio_util::io::ReaderStream;
use tracing::info;

#[derive(Serialize, Clone)]
pub enum ExecSessionEvent {
    Bytes(Vec<u8>),
    Ready,
    End,
}

pub enum ExecSessionRequest {
    Input(Vec<u8>),
    Resize(u16, u16),
    Abort,
}

#[tauri::command]
pub async fn pod_exec_write_stdin(
    consoles_state: State<'_, ExecSessionsState>,
    exec_session_id: ExecSessionId,
    buf: Vec<u8>,
) -> Result<(), ExecSessionError> {
    consoles_state
        .send(&exec_session_id, ExecSessionRequest::Input(buf))
        .await
}

#[tauri::command]
pub async fn pod_exec_abort_session(
    consoles_state: State<'_, ExecSessionsState>,
    exec_session_id: ExecSessionId,
) -> Result<(), ExecSessionError> {
    consoles_state
        .send(&exec_session_id, ExecSessionRequest::Abort)
        .await
}

#[tauri::command]
pub async fn pod_exec_resize_terminal(
    consoles_state: State<'_, ExecSessionsState>,
    exec_session_id: ExecSessionId,
    columns: u16,
    rows: u16,
) -> Result<(), ExecSessionError> {
    consoles_state
        .send(&exec_session_id, ExecSessionRequest::Resize(columns, rows))
        .await
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn pod_exec_start_session(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    join_handle_store: State<'_, JoinHandleStoreState>,
    consoles_state: State<'_, ExecSessionsState>,
    client_id: ClientId,
    pod_namespace: &str,
    pod_name: &str,
    container: &str,
    session_event_channel: Channel<ExecSessionEvent>,
) -> Result<ExecSessionId, BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;
    let pods: kube::Api<Pod> = kube::Api::namespaced(client, pod_namespace);

    let channel_id = session_event_channel.id();

    let mut attached_process = pods
        .exec(
            pod_name,
            vec!["sh", "-c", "exec bash -i || exec sh -i"],
            &AttachParams {
                container: Some(container.into()),
                ..AttachParams::interactive_tty()
            },
        )
        .await?;

    let (request_tx, mut request_rx) = mpsc::channel::<ExecSessionRequest>(1);

    let exec_session_id = consoles_state.register(request_tx).await;

    let exec_task = async move {
        let mut terminal_stdout_stream =
            ReaderStream::new(attached_process.stdout().expect("stdout must be connected"));
        let mut terminal_stdin_writer = attached_process.stdin().expect("stdin must be connected");
        let mut terminal_size_writer = attached_process.terminal_size().expect("must be a tty");

        session_event_channel.send(ExecSessionEvent::Ready).unwrap();

        loop {
            tokio::select! {
                upstream_output = terminal_stdout_stream.try_next() => {
                    if let Ok(Some(stdout)) = upstream_output {
                        let data: Vec<u8> = stdout.bytes().map(|o| o.unwrap()).collect();
                        session_event_channel.send(ExecSessionEvent::Bytes(data)).unwrap();
                    } else {
                        break;
                    }
                },
                downstream_request = request_rx.recv() => {
                    if let Some(request) = downstream_request {
                        match request {
                            ExecSessionRequest::Input(buf) => {
                                terminal_stdin_writer.write_all(&buf).await.unwrap();
                                terminal_stdin_writer.flush().await.unwrap();
                            },
                            ExecSessionRequest::Resize(columns, rows) => {
                                info!("Resizing to {}x{}", columns, rows);
                                terminal_size_writer.try_send(TerminalSize {
                                    height: rows,
                                    width: columns,
                                }).unwrap();
                            },
                            ExecSessionRequest::Abort => {
                                info!("Aborting exec session");
                                attached_process.abort();
                            },
                        }
                    }
                },
            };
        }

        info!("End of loop");
        session_event_channel.send(ExecSessionEvent::End).unwrap();
    };

    join_handle_store.submit(channel_id, exec_task)?;

    Ok(exec_session_id)
}
