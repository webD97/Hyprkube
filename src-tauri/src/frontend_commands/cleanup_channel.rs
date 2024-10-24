use std::sync::Arc;
use tauri::async_runtime::Mutex;

use tauri::State;

use crate::{app_state::JoinHandleStore, frontend_types::BackendError};

#[tauri::command]
pub async fn cleanup_channel(
    join_handle_store: State<'_, Arc<Mutex<JoinHandleStore>>>,
    channel: tauri::ipc::Channel<()>,
) -> Result<(), BackendError> {
    let mut join_handle_store = join_handle_store.lock().await;
    join_handle_store.abort(channel.id());

    Ok(())
}
