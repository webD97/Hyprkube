use std::collections::HashMap;

use uuid::Uuid;

use crate::frontend_types::BackendError;

pub struct KubernetesClientRegistry {
    pub registered: HashMap<Uuid, Box<kube::Client>>,
}

impl KubernetesClientRegistry {
    pub fn new() -> KubernetesClientRegistry {
        KubernetesClientRegistry {
            registered: HashMap::new(),
        }
    }

    pub fn insert(&mut self, client: kube::Client) -> Uuid {
        let id = Uuid::new_v4();

        self.registered.insert(id, Box::new(client));

        id
    }

    pub fn try_clone(&self, id: &Uuid) -> Result<kube::Client, BackendError> {
        self.registered
            .get(id)
            .map(|client| *client.clone())
            .ok_or(BackendError::Generic(format!(
                "Kubernetes client with id {id} not found."
            )))
    }
}

#[derive(Default)]
pub struct JoinHandleStore {
    pub handles: HashMap<u32, Vec<tauri::async_runtime::JoinHandle<()>>>,
    to_kill: Vec<u32>,
}

impl JoinHandleStore {
    pub fn insert(&mut self, channel_id: u32, handle: tauri::async_runtime::JoinHandle<()>) {
        let channel_handles = self.handles.entry(channel_id).or_insert(Vec::new());

        // Early kill
        if self.to_kill.contains(&channel_id) {
            println!(
                "Aborting handle of channel {} due to kill list.",
                &channel_id
            );

            handle.abort();

            return;
        }

        // We can keep it
        channel_handles.push(handle);
    }

    pub fn abort(&mut self, channel_id: u32) {
        // let mut handles = self.handles.lock().unwrap();
        let channel_handles = self.handles.remove(&channel_id);

        // Kill now
        if let Some(channel_handles) = channel_handles {
            if channel_handles.len() < 1 {
                return;
            }

            println!("Killing handles for channel {}", &channel_id);

            for handle in channel_handles {
                handle.abort();
            }

            return;
        }

        // Kill later
        println!("Nothing to kill yet, adding {} to kill list", &channel_id);
        self.to_kill.push(channel_id);
    }
}
