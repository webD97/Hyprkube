use crate::{
    app_state::{ChannelTasks, StateFacade},
    frontend_types::BackendError,
};

#[tauri::command]
#[tracing::instrument(skip_all, fields(request_id = tracing::field::Empty))]
pub async fn cleanup_channel(
    app: tauri::AppHandle,
    channel: tauri::ipc::Channel<()>,
) -> Result<(), BackendError> {
    crate::internal::tracing::set_span_request_id();
    let channel_tasks = app.state::<ChannelTasks>();

    channel_tasks.abort(&channel.id());

    Ok(())
}
