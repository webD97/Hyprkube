use std::sync::Mutex;

use tauri::Manager as _;

use crate::app_state::AppState;

#[tauri::command]
pub fn cleanup_channel(app: tauri::AppHandle, id: u32) {
    println!("Clean up channel {id}");

    let app_state = app.state::<Mutex<AppState>>();
    let mut app_state = app_state.lock().unwrap();

    if !app_state.channel_handlers.contains_key(&id) {
        return;
    }

    let handler = app_state.channel_handlers.get(&id).unwrap();
    handler.abort();

    app_state.channel_handlers.remove(&id);
}
