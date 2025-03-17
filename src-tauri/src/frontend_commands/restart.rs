use k8s_openapi::api::apps::v1::{Deployment, StatefulSet};
use tauri::State;

use crate::{
    app_state::{ClientId, KubernetesClientRegistryState},
    frontend_types::BackendError,
};

#[tauri::command]
pub async fn restart_deployment(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    client_id: ClientId,
    namespace: &str,
    name: &str,
) -> Result<(), BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;
    let api: kube::Api<Deployment> = kube::Api::namespaced(client, namespace);

    api.restart(name).await?;

    Ok(())
}

#[tauri::command]
pub async fn restart_statefulset(
    client_registry_arc: State<'_, KubernetesClientRegistryState>,
    client_id: ClientId,
    namespace: &str,
    name: &str,
) -> Result<(), BackendError> {
    let client = client_registry_arc.try_clone(&client_id)?;
    let api: kube::Api<StatefulSet> = kube::Api::namespaced(client, namespace);

    api.restart(name).await?;

    Ok(())
}
