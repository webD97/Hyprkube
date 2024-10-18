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
