use tauri::State;

use crate::{app_state::JoinHandleStoreState, frontend_types::BackendError};

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn cleanup_channel(
    join_handle_store: State<'_, JoinHandleStoreState>,
    channel: tauri::ipc::Channel<()>,
) -> Result<(), BackendError> {
    crate::internal::tracing::set_span_request_id();
    join_handle_store.abort(&channel.id());

    Ok(())
}
