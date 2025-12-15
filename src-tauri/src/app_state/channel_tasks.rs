use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use futures::future::{AbortHandle, Abortable};
use serde::Serialize;
use tauri::{async_runtime::spawn, Emitter};
use tracing::{debug, error};
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
#[derive(Debug)]
pub struct Rejected;

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

pub trait BackgroundTaskOutput {
    fn handle(self);
}

impl BackgroundTaskOutput for anyhow::Result<()> {
    fn handle(self) {
        if let Err(err) = self {
            tracing::error!("Task failed: {err}");
        }
    }
}

impl BackgroundTaskOutput for () {
    fn handle(self) {}
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

    pub fn submit<F>(&self, channel_id: u32, future: F) -> Result<(), Rejected>
    where
        F: Future<Output: BackgroundTaskOutput + Send + 'static> + Send + 'static,
    {
        // Check if we can already kill this task
        if self.to_kill.try_read().unwrap().contains(&channel_id) {
            debug!("Ignoring future of channel {}", &channel_id);
            self.to_kill.write().unwrap().retain(|&el| el != channel_id);
            return Err(Rejected);
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
                match join_handle.await {
                    Ok(Ok(output)) => {
                        debug!("Task for channel {channel_id} ended naturally");
                        output.handle();
                    }
                    Ok(Err(_)) => debug!("Task for channel {channel_id} was aborted"),
                    Err(e) => error!("Task for channel {channel_id} panicked: {e}"),
                }

                let mut handles = handles.write().unwrap();

                handles.remove(&channel_id);

                app_handle
                    .emit("join_handle_store_stats", Stats::from(handles))
                    .unwrap();
            }
            .in_current_span(),
        );

        Ok(())
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
            debug!("Trying to kill channel {}", channel_id);
            abort_handle.abort();
        } else {
            debug!("Channel {} now on kill list", &channel_id);
            self.to_kill.write().unwrap().push(channel_id.to_owned());
        }
    }
}
