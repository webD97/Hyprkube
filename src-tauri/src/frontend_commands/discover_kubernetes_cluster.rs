use std::{collections::HashMap, sync::Arc};

use kube::api::GroupVersionKind;
use serde::Serialize;

use crate::{
    app_state::KubernetesClientRegistryState,
    frontend_types::{BackendError, DiscoveredCluster},
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
pub async fn discover_kubernetes_cluster(
    client_registry: tauri::State<'_, KubernetesClientRegistryState>,
    view_registry: tauri::State<'_, Arc<RendererRegistry>>,
) -> Result<DiscoveredCluster, BackendError> {
    let client = kube::Client::try_default().await?;
    let (client_id, internal_discovery) = client_registry.lock().await.manage(client).await?;

    let mut gvks: HashMap<String, DiscoveredGroup> = HashMap::new();

    for (name, group) in internal_discovery.gvks {
        let mut discovered_kinds: Vec<DiscoveredResource> = Vec::new();

        for k in group.kinds {
            let gvk = GroupVersionKind::gvk(&name, &k.version, &k.kind);
            let views = view_registry.get_renderers(&client_id, &gvk).await;

            discovered_kinds.push(DiscoveredResource {
                version: k.version,
                kind: k.kind,
                views,
            });
        }

        let discovered_group = DiscoveredGroup {
            name: name.clone(),
            is_crd: group.is_crd,
            kinds: discovered_kinds,
        };

        gvks.insert(name.to_owned(), discovered_group);
    }

    let discovery = DiscoveryResult {
        builtin_apigroups: internal_discovery.builtin_apigroups,
        crd_apigroups: internal_discovery.crd_apigroups,
        gvks,
    };

    Ok(DiscoveredCluster {
        client_id,
        discovery,
    })
}
