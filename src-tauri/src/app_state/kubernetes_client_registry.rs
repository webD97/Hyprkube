use std::collections::HashMap;

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{GroupVersionKind, ListParams};
use serde::Serialize;
use uuid::Uuid;

use crate::frontend_types::BackendError;

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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryResult {
    pub gvks: HashMap<String, DiscoveredGroup>,
    pub crd_apigroups: Vec<String>,
    pub builtin_apigroups: Vec<String>,
    pub crds: HashMap<GroupVersionKind, CustomResourceDefinition>,
}

pub struct KubernetesClientRegistry {
    pub registered: HashMap<Uuid, (kube::Client, DiscoveryResult)>,
}

impl KubernetesClientRegistry {
    pub fn new() -> KubernetesClientRegistry {
        KubernetesClientRegistry {
            registered: HashMap::new(),
        }
    }

    pub async fn manage(&mut self, client: kube::Client) -> Result<Uuid, BackendError> {
        let id = Uuid::new_v4();

        let discovery = Self::run_discovery(client.clone()).await?;

        self.registered.insert(id, (client, discovery));

        Ok(id)
    }

    pub fn try_clone(&self, id: &Uuid) -> Result<kube::Client, BackendError> {
        self.registered
            .get(id)
            .map(|(client, _)| client.clone())
            .ok_or(BackendError::Generic(format!(
                "Kubernetes client with id {id} not found."
            )))
    }

    async fn run_discovery(client: kube::Client) -> Result<DiscoveryResult, BackendError> {
        let discovery = kube::Discovery::new(client.clone()).run().await?;

        let mut result = DiscoveryResult {
            gvks: HashMap::new(),
            crd_apigroups: vec![],
            builtin_apigroups: vec![],
            crds: HashMap::new(),
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

                if is_crd {
                    let crd = &crds
                        .iter()
                        .find(|crd| crd.spec.group == ar.group && crd.spec.names.kind == ar.kind)
                        .unwrap();

                    let gvk = GroupVersionKind {
                        group: crd.spec.group.clone(),
                        version: crd.spec.versions.first().unwrap().name.clone(),
                        kind: crd.spec.names.kind.clone(),
                    };

                    result.crds.insert(gvk, (*crd).to_owned());
                }

                result
                    .gvks
                    .get_mut(&ar.group)
                    .unwrap()
                    .kinds
                    .push(DiscoveredResource {
                        kind: ar.kind.clone(),
                        version: ar.version.clone(),
                    });
            }
        }

        Ok(result)
    }
}
