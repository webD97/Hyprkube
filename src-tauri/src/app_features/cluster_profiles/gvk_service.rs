use std::sync::Arc;

use kube::api::GroupVersionKind;
use serde::Serialize;
use tauri::{AppHandle, Emitter as _, Manager, Wry};
use tauri_plugin_store::{Store, StoreExt as _};

use super::cluster_profile_registry::{ClusterProfileId, ClusterProfileRegistryState};

pub struct GvkService {
    app: AppHandle,
}

impl GvkService {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn get_tauri_store(&self, profile: &ClusterProfileId) -> Result<Arc<Store<Wry>>, self::Error> {
        let profile_registry = self.app.state::<ClusterProfileRegistryState>();
        let mut profile_directory = profile_registry.get_profile_basedir(profile).unwrap();
        profile_directory.push("pinned_gvks.json");

        Ok(self.app.store(profile_directory)?)
    }

    pub fn list_pinned_gvks(
        &self,
        profile: &ClusterProfileId,
    ) -> Result<Vec<GroupVersionKind>, self::Error> {
        let store = self.get_tauri_store(profile)?;

        if store.is_empty() || !store.has("items") {
            return Ok(Vec::new());
        }

        let items = store.get("items").unwrap();
        let items = serde_json::from_value(items).unwrap();

        Ok(items)
    }

    pub fn add_pinned_gvk(
        &self,
        profile: &ClusterProfileId,
        gvk: GroupVersionKind,
    ) -> Result<(), self::Error> {
        let mut pinned = self.list_pinned_gvks(profile)?;

        if pinned.contains(&gvk) {
            return Ok(());
        }

        pinned.push(gvk.clone());

        let store = self.get_tauri_store(profile)?;
        store.set("items", serde_json::to_value(pinned.clone())?);
        store.save()?;

        self.app.emit(
            "hyprkube://pinned-gvks-changed",
            PinnedGvksChanged {
                cluster_profile: profile.to_owned(),
                gvks: pinned,
            },
        )?;

        Ok(())
    }

    pub fn remove_pinned_gvk(
        &self,
        profile: &ClusterProfileId,
        gvk: &GroupVersionKind,
    ) -> Result<(), self::Error> {
        let mut pinned = self.list_pinned_gvks(profile)?;

        if !pinned.contains(&gvk) {
            return Ok(());
        }

        pinned.retain(|g| g != gvk);

        let store = self.get_tauri_store(profile)?;
        store.set("items", serde_json::to_value(pinned.clone())?);
        store.save()?;

        self.app.emit(
            "hyprkube://pinned-gvks-changed",
            PinnedGvksChanged {
                cluster_profile: profile.to_owned(),
                gvks: pinned,
            },
        )?;

        Ok(())
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PinnedGvksChanged {
    cluster_profile: ClusterProfileId,
    gvks: Vec<GroupVersionKind>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TauriError(#[from] tauri::Error),

    #[error(transparent)]
    TauriStoreError(#[from] tauri_plugin_store::Error),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
