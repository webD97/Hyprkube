use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use futures::future::{AbortHandle, Abortable};
use serde::Serialize;
use tauri::{async_runtime::spawn, Emitter};
use tracing::info;
use tracing_futures::Instrument;

pub type JoinHandleStoreState = Arc<ChannelTasks>;

pub struct ChannelTasks {
    handles: Arc<RwLock<HashMap<u32, AbortHandle>>>,
    to_kill: RwLock<Vec<u32>>,
    app_handle: Arc<tauri::AppHandle>,
}

#[derive(Serialize, Clone)]
struct Stats {
    handles: usize,
}

impl From<RwLockWriteGuard<'_, HashMap<u32, AbortHandle>>> for Stats {
    fn from(value: RwLockWriteGuard<'_, HashMap<u32, AbortHandle>>) -> Self {
        Self {
            handles: value.len(),
        }
    }
}

impl From<RwLockReadGuard<'_, HashMap<u32, AbortHandle>>> for Stats {
    fn from(value: RwLockReadGuard<'_, HashMap<u32, AbortHandle>>) -> Self {
        Self {
            handles: value.len(),
        }
    }
}

impl ChannelTasks {
    pub fn new_state(app_handle: tauri::AppHandle) -> JoinHandleStoreState {
        Arc::new(ChannelTasks::new(app_handle))
    }

    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            handles: Arc::new(RwLock::new(HashMap::default())),
            to_kill: RwLock::new(Vec::default()),
            app_handle: Arc::new(app_handle),
        }
    }

    pub fn submit<F>(&self, channel_id: u32, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // Check if we can already kill this task
        if self.to_kill.try_read().unwrap().contains(&channel_id) {
            info!("Ignoring future of channel {}", &channel_id);
            self.to_kill.write().unwrap().retain(|&el| el != channel_id);
            return;
        }

        // Task may run, track it
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let abortable = Abortable::new(future, abort_registration);

        let join_handle = spawn(abortable.in_current_span());

        {
            let mut handles = self.handles.write().unwrap();
            handles.insert(channel_id, abort_handle);

            self.app_handle
                .emit("join_handle_store_stats", Stats::from(handles))
                .unwrap();
        }

        let handles = Arc::clone(&self.handles);
        let app_handle = Arc::clone(&self.app_handle);

        // Wait for completion, then remove the task from our tracking
        spawn(
            async move {
                match join_handle.await.unwrap() {
                    Ok(_) => info!("Task for channel {} ended naturally", &channel_id),
                    Err(_) => info!("Task for channel {} was aborted", &channel_id),
                }

                let mut handles = handles.write().unwrap();

                handles.remove(&channel_id);

                app_handle
                    .emit("join_handle_store_stats", Stats::from(handles))
                    .unwrap();
            }
            .in_current_span(),
        );
    }

    pub fn abort_all(&self) {
        let channels: Vec<u32> = {
            let handles = self.handles.read().unwrap();
            handles.keys().cloned().collect()
        };

        for channel_id in channels {
            self.abort(&channel_id);
        }
    }

    pub fn abort(&self, channel_id: &u32) {
        let handles = self.handles.read().unwrap();

        if let Some(abort_handle) = handles.get(channel_id) {
            info!("Trying to kill channel {}", channel_id);
            abort_handle.abort();
        } else {
            info!("Channel {} now on kill list", &channel_id);
            self.to_kill.write().unwrap().push(channel_id.to_owned());
        }
    }
}
