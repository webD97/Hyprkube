use std::sync::{Arc, Mutex};

use tauri::State;

use crate::{app_state::JoinHandleStore, frontend_types::BackendError};

#[tauri::command]
pub async fn cleanup_channel(
    // channel_handles: State<'_, Mutex<HashMap<u32, Vec<tauri::async_runtime::JoinHandle<()>>>>>,
    join_handle_store: State<'_, Arc<Mutex<JoinHandleStore>>>,
    channel: tauri::ipc::Channel<()>,
) -> Result<(), BackendError> {
    let mut join_handle_store = join_handle_store.lock().unwrap();
    join_handle_store.abort(channel.id());

    Ok(())
}
