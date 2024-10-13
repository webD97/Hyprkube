use std::collections::HashMap;

use crate::frontend_types::BackendError;

#[tauri::command]
pub async fn kube_discover(app: tauri::AppHandle) -> Result<HashMap<String, Vec<(String, String)>>, BackendError> {
    let client = crate::app_state::clone_client(&app)?;
    let discovery = kube::Discovery::new(client).run().await?;

    let mut kinds = HashMap::<String, Vec<(String, String)>>::new();

    for group in discovery.groups() {
        for (ar, capabilities) in group.recommended_resources() {
            if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                continue;
            }

            let g = ar.group;
            let v = ar.version;
            let k = ar.kind;

            if !kinds.contains_key(&g) {
                kinds.insert(g.clone(), vec![]);
            }

            kinds.get_mut(&g).unwrap().push((k, v));
        }
    }

    Ok(kinds)
}
