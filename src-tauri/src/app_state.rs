use std::collections::HashMap;

use uuid::Uuid;

use crate::frontend_types::BackendError;

pub struct AppState {
    pub channel_handlers: HashMap<u32, tokio::task::JoinHandle<()>>,
    pub podlog_stream_handles: HashMap<u32, tokio::task::JoinHandle<()>>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            channel_handlers: HashMap::new(),
            podlog_stream_handles: HashMap::new(),
        }
    }
}

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
