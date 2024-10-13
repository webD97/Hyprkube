use std::{collections::HashMap, sync::Mutex};

use serde::Serialize;
use tauri::Manager as _;
use uuid::Uuid;

use crate::{app_state::KubernetesClientRegistry, frontend_types::BackendError};

#[derive(Serialize)]
pub struct DiscoveryResult {
    pub gvks: HashMap<String, Vec<(String, String)>>
}

#[tauri::command]
pub async fn kube_discover(app: tauri::AppHandle, client_id: Uuid) -> Result<DiscoveryResult, BackendError> {
    let client = {
        let client_registry = app.state::<Mutex<KubernetesClientRegistry>>();
        let client_registry = client_registry
            .lock()
            .map_err(|x| BackendError::Generic(x.to_string()))?;
        
        client_registry.try_clone(&client_id)?
    };

    let discovery = kube::Discovery::new(client).run().await?;

    let mut result = DiscoveryResult {
        gvks: HashMap::new()
    };

    for group in discovery.groups() {
        for (ar, capabilities) in group.recommended_resources() {
            if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                continue;
            }

            let g = ar.group;
            let v = ar.version;
            let k = ar.kind;

            if !result.gvks.contains_key(&g) {
                result.gvks.insert(g.clone(), vec![]);
            }

            result.gvks.get_mut(&g).unwrap().push((k, v));
        }
    }

    Ok(result)
}
