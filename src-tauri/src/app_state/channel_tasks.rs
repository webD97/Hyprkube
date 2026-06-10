use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, RwLock},
};

use futures::future::{AbortHandle, Abortable};
use serde::Serialize;
use tauri::{async_runtime::spawn, AppHandle, Emitter, Runtime, Wry};
use tracing::{debug, error};
use tracing_futures::Instrument;

use crate::app_state::ManagedState;

/// State of a single tracked channel.
///
/// Both running tasks and pending kill-requests live in the same map so that
/// `submit` and `abort` can be serialized against a single lock, closing the
/// submit/abort race window.
enum TaskSlot {
    /// The task is running and can be aborted via this handle.
    Running(AbortHandle),
    /// An `abort` arrived before the task was submitted; the next `submit` for
    /// this channel must be rejected instead of started.
    PendingKill,
}

/// Registry of abortable background tasks keyed by frontend channel id.
///
/// Generic over the Tauri runtime purely so tests can drive it with
/// `MockRuntime`; production always uses the `Wry` default.
pub struct ChannelTasks<R: Runtime = Wry> {
    tasks: Arc<RwLock<HashMap<u32, TaskSlot>>>,
    app_handle: Arc<AppHandle<R>>,
}

impl ManagedState for ChannelTasks<Wry> {
    type WrappedState = Arc<ChannelTasks<Wry>>;

    fn build(app: tauri::AppHandle) -> Self::WrappedState {
        Arc::new(ChannelTasks::new(app))
    }
}

#[derive(Serialize, Clone)]
struct Stats {
    handles: usize,
}

#[derive(Debug)]
pub struct Rejected;

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

impl<R: Runtime> ChannelTasks<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::default())),
            app_handle: Arc::new(app_handle),
        }
    }

    pub fn submit<F>(&self, channel_id: u32, future: F) -> Result<(), Rejected>
    where
        F: Future<Output: BackgroundTaskOutput + Send + 'static> + Send + 'static,
    {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let abortable = Abortable::new(future, abort_registration);

        // Check kill-list and register the handle under a single write lock so a
        // concurrent `abort` can't slip in between the check and the insert.
        let join_handle = {
            let mut tasks = self.tasks.write().unwrap();

            // An `abort` for this channel arrived before we got here: honor it
            // and drop the request instead of starting the task.
            if matches!(tasks.get(&channel_id), Some(TaskSlot::PendingKill)) {
                debug!("Ignoring future of channel {}", &channel_id);
                tasks.remove(&channel_id);
                return Err(Rejected);
            }

            let join_handle = spawn(abortable.in_current_span());
            tasks.insert(channel_id, TaskSlot::Running(abort_handle));

            Self::emit_stats(&self.app_handle, &tasks);

            join_handle
        };

        let tasks = Arc::clone(&self.tasks);
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

                let mut tasks = tasks.write().unwrap();
                tasks.remove(&channel_id);
                Self::emit_stats(&app_handle, &tasks);
            }
            .in_current_span(),
        );

        Ok(())
    }

    pub fn abort_all(&self) {
        let channels: Vec<u32> = {
            let tasks = self.tasks.read().unwrap();
            tasks.keys().copied().collect()
        };

        for channel_id in channels {
            self.abort(&channel_id);
        }
    }

    pub fn abort(&self, channel_id: &u32) {
        let mut tasks = self.tasks.write().unwrap();

        match tasks.get(channel_id) {
            Some(TaskSlot::Running(abort_handle)) => {
                debug!("Trying to kill channel {}", channel_id);
                abort_handle.abort();
            }
            // Already queued for a kill; nothing to do.
            Some(TaskSlot::PendingKill) => {}
            // Task not submitted yet: remember to reject it when it arrives.
            None => {
                debug!("Channel {} now on kill list", &channel_id);
                tasks.insert(*channel_id, TaskSlot::PendingKill);
            }
        }
    }

    fn emit_stats(app_handle: &AppHandle<R>, tasks: &HashMap<u32, TaskSlot>) {
        let handles = tasks
            .values()
            .filter(|slot| matches!(slot, TaskSlot::Running(_)))
            .count();

        if let Err(err) = app_handle.emit("join_handle_store_stats", Stats { handles }) {
            error!("Failed to emit task stats: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        time::Duration,
    };

    use tauri::test::{mock_app, MockRuntime};
    use tokio::sync::Notify;

    use super::*;

    fn channel_tasks() -> ChannelTasks<MockRuntime> {
        // `mock_app` is leaked so the underlying runtime/app stays alive for the
        // whole test; the handle clone is what `ChannelTasks` needs.
        let app = Box::leak(Box::new(mock_app()));
        ChannelTasks::new(app.handle().clone())
    }

    /// (b) submit→abort race: an `abort` that arrives before `submit` must
    /// reject the task, and the task body must never run.
    #[tokio::test]
    async fn abort_before_submit_rejects_and_never_runs() {
        let tasks = channel_tasks();

        tasks.abort(&42);

        let ran = Arc::new(AtomicBool::new(false));
        let result = {
            let ran = Arc::clone(&ran);
            tasks.submit(42, async move {
                ran.store(true, Ordering::SeqCst);
            })
        };

        assert!(result.is_err(), "submit after abort must be rejected");

        // Give any erroneously-spawned task a chance to run before asserting.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!ran.load(Ordering::SeqCst), "rejected task must never run");

        // The PendingKill marker is consumed by the rejected submit, so a fresh
        // submit for the same id succeeds.
        let ran_again = Arc::new(AtomicBool::new(false));
        let result = {
            let ran_again = Arc::clone(&ran_again);
            tasks.submit(42, async move {
                ran_again.store(true, Ordering::SeqCst);
            })
        };
        assert!(result.is_ok(), "submit after the marker is consumed must succeed");
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(ran_again.load(Ordering::SeqCst), "second task should run");
    }

    /// A running task must be cancelled (its future dropped) when aborted.
    #[tokio::test(flavor = "multi_thread")]
    async fn submit_then_abort_cancels_running_task() {
        struct DropFlag(Arc<AtomicBool>);
        impl Drop for DropFlag {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let tasks = channel_tasks();

        let dropped = Arc::new(AtomicBool::new(false));
        let started = Arc::new(Notify::new());

        {
            let dropped = Arc::clone(&dropped);
            let started = Arc::clone(&started);
            tasks
                .submit(7, async move {
                    let _guard = DropFlag(dropped);
                    started.notify_one();
                    // Never completes on its own; only an abort ends it.
                    futures::future::pending::<()>().await;
                })
                .expect("submit should succeed");
        }

        // Make sure the task actually started before aborting it.
        started.notified().await;
        tasks.abort(&7);

        // Poll until the guard drops (task cancelled), with a bounded timeout.
        for _ in 0..200 {
            if dropped.load(Ordering::SeqCst) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            dropped.load(Ordering::SeqCst),
            "aborting must cancel (drop) the running future"
        );
    }

    /// (a) Hammer `submit` and `abort` from many tasks concurrently. The old
    /// `try_read().unwrap()` would panic whenever a writer held the lock; the
    /// single-lock design must stay panic-free regardless of interleaving.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_submit_and_abort_do_not_panic() {
        let tasks = Arc::new(channel_tasks());

        let mut joins = Vec::new();
        for id in 0..200u32 {
            let submit_tasks = Arc::clone(&tasks);
            joins.push(tokio::spawn(async move {
                let _ = submit_tasks.submit(id, futures::future::pending::<()>());
            }));

            let abort_tasks = Arc::clone(&tasks);
            joins.push(tokio::spawn(async move {
                abort_tasks.abort(&id);
            }));
        }

        for join in joins {
            join.await.expect("worker task panicked");
        }

        // Cleaning everyone up must also stay panic-free.
        tasks.abort_all();
    }
}
