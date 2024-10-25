use tauri::State;

use crate::{app_state::JoinHandleStoreState, frontend_types::BackendError};

#[tauri::command]
pub async fn cleanup_channel(
    join_handle_store: State<'_, JoinHandleStoreState>,
    channel: tauri::ipc::Channel<()>,
) -> Result<(), BackendError> {
    let mut join_handle_store = join_handle_store.lock().await;
    join_handle_store.abort(channel.id());

    Ok(())
}
