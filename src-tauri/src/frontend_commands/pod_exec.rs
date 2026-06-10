use crate::{
    app_state::{
        ChannelTasks, ClusterStateRegistry, ExecSessionError, ExecSessionId, ExecSessions,
        ManagerExt,
    },
    frontend_commands::KubeContextSource,
    frontend_types::BackendError,
};
use futures::TryStreamExt as _;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{AttachParams, TerminalSize};
use serde::Serialize;
use tauri::ipc::Channel;
use tokio::{io::AsyncWriteExt, sync::mpsc};
use tokio_util::io::ReaderStream;
use tracing::info;

#[derive(Serialize, Clone)]
pub enum ExecSessionEvent {
    Bytes(Vec<u8>),
    Ready,
    End,
    /// The session ended (or failed to start) abnormally; carries a human-readable reason.
    Error(String),
}

pub enum ExecSessionRequest {
    Input(Vec<u8>),
    Resize(u16, u16),
    Abort,
}

#[tauri::command]
pub async fn pod_exec_write_stdin(
    app: tauri::AppHandle,
    exec_session_id: ExecSessionId,
    buf: Vec<u8>,
) -> Result<(), ExecSessionError> {
    let consoles_state = app.state::<ExecSessions>();
    consoles_state
        .send(&exec_session_id, ExecSessionRequest::Input(buf))
        .await
}

#[tauri::command]
pub async fn pod_exec_abort_session(
    app: tauri::AppHandle,
    exec_session_id: ExecSessionId,
) -> Result<(), ExecSessionError> {
    let consoles_state = app.state::<ExecSessions>();
    consoles_state
        .send(&exec_session_id, ExecSessionRequest::Abort)
        .await
}

#[tauri::command]
pub async fn pod_exec_resize_terminal(
    app: tauri::AppHandle,
    exec_session_id: ExecSessionId,
    columns: u16,
    rows: u16,
) -> Result<(), ExecSessionError> {
    let consoles_state = app.state::<ExecSessions>();
    consoles_state
        .send(&exec_session_id, ExecSessionRequest::Resize(columns, rows))
        .await
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn pod_exec_start_session(
    app: tauri::AppHandle,
    context_source: KubeContextSource,
    pod_namespace: &str,
    pod_name: &str,
    container: &str,
    session_event_channel: Channel<ExecSessionEvent>,
) -> Result<ExecSessionId, BackendError> {
    let clusters = app.state::<ClusterStateRegistry>();
    let channel_tasks = app.state::<ChannelTasks>();
    let consoles_state = app.state::<ExecSessions>();
    let client = clusters.client_for(&context_source)?;

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
        // These streams are expected to be present for an interactive TTY attach, but a
        // missing one is a failed session, not a reason to panic the worker.
        let (Some(stdout), Some(mut terminal_stdin_writer), Some(mut terminal_size_writer)) = (
            attached_process.stdout(),
            attached_process.stdin(),
            attached_process.terminal_size(),
        ) else {
            let _ = session_event_channel
                .send(ExecSessionEvent::Error("exec streams not connected".to_owned()));
            let _ = session_event_channel.send(ExecSessionEvent::End);
            return;
        };

        let mut terminal_stdout_stream = ReaderStream::new(stdout);

        // If the frontend already closed the channel, there's nobody to serve.
        if session_event_channel.send(ExecSessionEvent::Ready).is_err() {
            return;
        }

        // `Some(reason)` => surface an Error event before ending; `None` => clean end.
        let mut error: Option<String> = None;

        loop {
            tokio::select! {
                upstream_output = terminal_stdout_stream.try_next() => {
                    match upstream_output {
                        // Process exited / stdout closed: normal end of session.
                        Ok(None) => break,
                        Ok(Some(stdout)) => {
                            // Frontend went away: stop quietly.
                            if session_event_channel.send(ExecSessionEvent::Bytes(stdout.to_vec())).is_err() {
                                break;
                            }
                        },
                        Err(e) => {
                            error = Some(format!("error reading from container: {e}"));
                            break;
                        },
                    }
                },
                downstream_request = request_rx.recv() => {
                    // Request channel closed: the session was torn down.
                    let Some(request) = downstream_request else { break; };

                    match request {
                        ExecSessionRequest::Input(buf) => {
                            let write_result = match terminal_stdin_writer.write_all(&buf).await {
                                Ok(()) => terminal_stdin_writer.flush().await,
                                Err(e) => Err(e),
                            };
                            if let Err(e) = write_result {
                                error = Some(format!("error writing to container: {e}"));
                                break;
                            }
                        },
                        ExecSessionRequest::Resize(columns, rows) => {
                            info!("Resizing to {}x{}", columns, rows);
                            // A full/closed resize channel is non-fatal — keep the session alive.
                            if let Err(e) = terminal_size_writer.try_send(TerminalSize {
                                height: rows,
                                width: columns,
                            }) {
                                tracing::warn!("Failed to send terminal resize: {e}");
                            }
                        },
                        ExecSessionRequest::Abort => {
                            info!("Aborting exec session");
                            attached_process.abort();
                        },
                    }
                },
            };
        }

        info!("End of loop");

        if let Some(error) = error {
            let _ = session_event_channel.send(ExecSessionEvent::Error(error));
        }
        let _ = session_event_channel.send(ExecSessionEvent::End);
    };

    channel_tasks.submit(channel_id, exec_task)?;

    Ok(exec_session_id)
}
