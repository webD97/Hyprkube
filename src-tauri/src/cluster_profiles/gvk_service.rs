use std::sync::Arc;

use kube::api::GroupVersionKind;
use serde::Serialize;
use tauri::{AppHandle, Emitter as _};

use crate::persistence;

use super::cluster_profile_registry::ClusterProfileId;

pub struct GvkService {
    app: AppHandle,
    repository: Arc<persistence::Repository>,
}

const PERISTENCE_KEY_PINNED_GVKS: &str = "pinned_gvks";

impl GvkService {
    pub fn new(app: AppHandle, repository: Arc<persistence::Repository>) -> Self {
        Self { app, repository }
    }

    pub fn list_pinned_gvks(
        &self,
        profile: &ClusterProfileId,
    ) -> Result<Vec<GroupVersionKind>, self::Error> {
        let items = self.repository.read_key(
            &persistence::Context::PerClusterProfile(profile.clone()),
            PERISTENCE_KEY_PINNED_GVKS,
        )?;

        if items.is_none() {
            return Ok(Vec::new());
        }

        let items = serde_json::from_value(items.unwrap())?;

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

        self.repository.set_key(
            &persistence::Context::PerClusterProfile(profile.clone()),
            PERISTENCE_KEY_PINNED_GVKS,
            serde_json::to_value(pinned.clone())?,
        )?;

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

        self.repository.set_key(
            &persistence::Context::PerClusterProfile(profile.clone()),
            PERISTENCE_KEY_PINNED_GVKS,
            serde_json::to_value(pinned.clone())?,
        )?;

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

    #[error(transparent)]
    RepositoryError(#[from] persistence::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
