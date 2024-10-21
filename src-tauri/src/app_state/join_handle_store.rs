use std::collections::HashMap;

use serde::Serialize;
use tauri::Emitter;

pub struct JoinHandleStore {
    pub handles: HashMap<u32, Vec<tauri::async_runtime::JoinHandle<()>>>,
    to_kill: Vec<u32>,
    app_handle: tauri::AppHandle,
}

#[derive(Serialize, Clone)]
struct Stats {
    handles: usize,
    channels: usize,
}

impl Drop for JoinHandleStore {
    fn drop(&mut self) {
        self.abort_all();
    }
}

impl JoinHandleStore {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            handles: HashMap::default(),
            to_kill: Vec::default(),
            app_handle,
        }
    }

    fn emit_stats(&self) {
        self.app_handle
            .emit(
                "join_handle_store_stats",
                Stats {
                    handles: self.handles.values().flat_map(|vec| vec.iter()).count(),
                    channels: self.handles.len(),
                },
            )
            .unwrap();
    }

    pub fn insert(&mut self, channel_id: u32, handle: tauri::async_runtime::JoinHandle<()>) {
        let channel_handles = self.handles.entry(channel_id).or_insert(Vec::new());

        // Early kill
        if self.to_kill.contains(&channel_id) {
            println!(
                "Aborting handle of channel {} due to kill list.",
                &channel_id
            );

            handle.abort();

            self.emit_stats();
        } else {
            // We can keep it
            channel_handles.push(handle);
        }

        self.emit_stats();

        // Clean up
        self.handles.retain(|_, v| !v.is_empty());
    }

    pub fn abort_all(&mut self) {
        let keys: Vec<u32> = self.handles.keys().cloned().collect();
        for channel_id in keys {
            self.abort(channel_id.to_owned());
        }
    }

    pub fn abort(&mut self, channel_id: u32) {
        let channel_handles = self.handles.remove(&channel_id);

        if let Some(channel_handles) = channel_handles {
            // Can be aborted right now
            if channel_handles.len() < 1 {
                return;
            }

            println!("Killing handles for channel {}", &channel_id);

            for handle in channel_handles {
                handle.abort();
            }
        } else {
            // Abort later
            println!("Nothing to kill yet, adding {} to kill list", &channel_id);
            self.to_kill.push(channel_id);
        }

        // Clean up
        self.handles.retain(|_, v| !v.is_empty());

        self.emit_stats();
    }
}
