use k8s_openapi::api::core::v1::Pod;
use tauri::State;

use crate::{
    app_state::{ClientId, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn list_pod_container_names(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    client_id: ClientId,
    namespace: &str,
    name: &str,
) -> Result<Vec<String>, BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;
    let pods: kube::Api<Pod> = kube::Api::namespaced(client, namespace);

    let pod = pods.get(name).await?;

    Ok(pod
        .spec
        .expect("spec exists")
        .containers
        .iter()
        .map(|container| container.name.clone())
        .collect())
}
