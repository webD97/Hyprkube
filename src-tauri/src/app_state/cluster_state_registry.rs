use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    app_state::ManagedState,
    cluster_discovery::{ClusterDiscovery, ClusterState},
    frontend_commands::KubeContextSource,
    frontend_types::BackendError,
    scripting::{
        resource_context_menu_facade::ResourceContextMenuFacade,
        resource_presentation_facade::ResourcePresentationFacade,
    },
};

pub struct ClusterStateRegistry {
    clusters: RwLock<HashMap<KubeContextSource, Arc<ClusterState>>>,
}

impl ManagedState for ClusterStateRegistry {
    type WrappedState = Arc<ClusterStateRegistry>;

    fn build(_: tauri::AppHandle) -> Self::WrappedState {
        Arc::new(ClusterStateRegistry::new())
    }
}

impl ClusterStateRegistry {
    pub fn new() -> Self {
        Self {
            clusters: RwLock::new(HashMap::new()),
        }
    }

    fn get_state(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<Arc<ClusterState>, BackendError> {
        Ok(self
            .clusters
            .read()
            .unwrap()
            .get(context_source)
            .ok_or_else(|| BackendError::Unmanaged(context_source.to_owned()))?
            .clone())
    }

    /// Returns a cloned client for the given KubeContextSource if such a managed cluster exists.
    pub fn client_for(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<kube::Client, BackendError> {
        Ok(self
            .clusters
            .read()
            .unwrap()
            .get(context_source)
            .ok_or_else(|| BackendError::Unmanaged(context_source.to_owned()))?
            .client
            .to_owned())
    }

    pub fn contextmenu_scripting_for(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<Arc<ResourceContextMenuFacade>, BackendError> {
        Ok(self
            .get_state(context_source)?
            .context_menu_facade
            .as_ref()
            .ok_or_else(|| BackendError::IncompleteClusterDiscovery(context_source.to_owned()))?
            .clone())
    }

    pub fn presentation_scripting_for(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<Arc<ResourcePresentationFacade>, BackendError> {
        Ok(self
            .get_state(context_source)?
            .resource_presentation_facade
            .as_ref()
            .ok_or_else(|| BackendError::IncompleteClusterDiscovery(context_source.to_owned()))?
            .clone())
    }

    pub fn discovery_for(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<Arc<ClusterDiscovery>, BackendError> {
        Ok(self.get_state(context_source)?.discovery.clone())
    }

    pub fn discovery_cache_for(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<Arc<kube::Discovery>, BackendError> {
        Ok(self
            .get_state(context_source)?
            .kube_discovery
            .as_ref()
            .ok_or_else(|| BackendError::IncompleteClusterDiscovery(context_source.to_owned()))?
            .clone())
    }

    pub fn manage(&self, state: ClusterState) {
        let mut contexts = self.clusters.write().unwrap();
        contexts
            .entry(state.context_source.clone())
            .insert_entry(Arc::new(state));
    }

    pub fn unmanage(
        &self,
        context_source: &KubeContextSource,
    ) -> Result<ClusterState, BackendError> {
        let mut contexts = self.clusters.write().unwrap();

        let x = contexts
            .remove(context_source)
            .ok_or_else(|| BackendError::Unmanaged(context_source.to_owned()))?;

        Ok(Arc::into_inner(x).expect("must be the only reference"))
    }
}
