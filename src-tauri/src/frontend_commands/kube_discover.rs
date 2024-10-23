use std::{collections::HashMap, sync::Arc};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{GroupVersionKind, ListParams};
use serde::Serialize;
use tauri::{Manager as _, State};
use uuid::Uuid;

use crate::{
    app_state::KubernetesClientRegistryState, frontend_types::BackendError,
    resource_rendering::RendererRegistry,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredGroup {
    pub name: String,
    pub is_crd: bool,
    pub kinds: Vec<DiscoveredResource>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredResource {
    pub version: String,
    pub kind: String,
    pub views: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryResult {
    pub gvks: HashMap<String, DiscoveredGroup>,
    pub crd_apigroups: Vec<String>,
    pub builtin_apigroups: Vec<String>,
}

#[tauri::command]
pub async fn kube_discover(
    app: tauri::AppHandle,
    view_registry: State<'_, Arc<RendererRegistry>>,
    client_id: Uuid,
) -> Result<DiscoveryResult, BackendError> {
    let client = {
        let client_registry = app.state::<KubernetesClientRegistryState>();
        let client_registry = client_registry.lock().await;

        client_registry.try_clone(&client_id)?
    };

    let discovery = kube::Discovery::new(client.clone()).run().await?;

    let mut result = DiscoveryResult {
        gvks: HashMap::new(),
        crd_apigroups: vec![],
        builtin_apigroups: vec![],
    };

    let api: kube::Api<CustomResourceDefinition> = kube::Api::all(client.clone());
    let crds = api.list(&ListParams::default()).await?.items;

    for crd in &crds {
        if !result.crd_apigroups.contains(&crd.spec.group) {
            result.crd_apigroups.push(crd.spec.group.clone());
        }
    }

    for group in discovery.groups() {
        for (ar, capabilities) in group.recommended_resources() {
            if !capabilities.supports_operation(kube::discovery::verbs::WATCH) {
                continue;
            }

            let is_crd = result.crd_apigroups.contains(&ar.group);

            if !is_crd && !result.builtin_apigroups.contains(&ar.group) {
                result.builtin_apigroups.push(ar.group);
                continue;
            }

            if !result.gvks.contains_key(&ar.group) {
                result.gvks.insert(
                    ar.group.clone(),
                    DiscoveredGroup {
                        name: ar.group.clone(),
                        kinds: vec![],
                        is_crd,
                    },
                );
            }

            let gvk = GroupVersionKind::gvk(&ar.group, &ar.version, &ar.kind);
            let views = view_registry.get_renderers(&client_id, &gvk).await;

            result
                .gvks
                .get_mut(&ar.group)
                .unwrap()
                .kinds
                .push(DiscoveredResource {
                    kind: ar.kind.clone(),
                    version: ar.version.clone(),
                    views,
                });
        }
    }

    Ok(result)
}
