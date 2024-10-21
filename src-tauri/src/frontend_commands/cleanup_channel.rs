use std::sync::{Arc, Mutex};

use tauri::State;

use crate::{app_state::JoinHandleStore, frontend_types::BackendError};

#[tauri::command]
pub async fn cleanup_channel(
    // channel_handles: State<'_, Mutex<HashMap<u32, Vec<tauri::async_runtime::JoinHandle<()>>>>>,
    join_handle_store: State<'_, Arc<Mutex<JoinHandleStore>>>,
    channel: tauri::ipc::Channel<()>,
) -> Result<(), BackendError> {
    // let mut channel_handles = channel_handles
    //     .lock()
    //     .map_err(|e| BackendError::Generic(e.to_string()))?;

    // if let Some(handles) = channel_handles.remove(&channel.id()) {
    //     println!(
    //         "Aborting {} JoinHandles for channel {}",
    //         handles.len(),
    //         channel.id()
    //     );
    //     for handle in &handles {
    //         handle.abort();
    //     }
    // } else {
    //     println!("Nothing to abort for channel {}", channel.id());
    // }

    let mut join_handle_store = join_handle_store.lock().unwrap();
    join_handle_store.abort(channel.id());

    println!(
        "There are {} channels with handles",
        join_handle_store.handles.len()
    );

    Ok(())
}
