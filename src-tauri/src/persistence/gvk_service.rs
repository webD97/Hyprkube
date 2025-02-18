use std::sync::Arc;

use kube::api::GroupVersionKind;
use serde::Serialize;
use tauri::{AppHandle, Emitter as _};

use crate::{cluster_profiles::ClusterProfileId, persistence};

pub struct GvkService {
    app: AppHandle,
    repository: Arc<persistence::Repository>,
}

const PERISTENCE_KEY_PINNED_GVKS: &str = "pinned_gvks";
const PERISTENCE_KEY_HIDDEN_GVKS: &str = "hidden_gvks";

impl GvkService {
    pub fn new(app: AppHandle, repository: Arc<persistence::Repository>) -> Self {
        Self { app, repository }
    }

    pub fn list_pinned_gvks(
        &self,
        profile: &ClusterProfileId,
    ) -> Result<Vec<GroupVersionKind>, self::Error> {
        self.list_gvks(profile, PERISTENCE_KEY_PINNED_GVKS)
    }

    pub fn add_pinned_gvk(
        &self,
        profile: &ClusterProfileId,
        gvk: GroupVersionKind,
    ) -> Result<(), self::Error> {
        self.add_gvk(profile, gvk, PERISTENCE_KEY_PINNED_GVKS)
            .map(|gvks| {
                self.app
                    .emit(
                        "hyprkube://pinned-gvks-changed",
                        PinnedGvksChanged {
                            cluster_profile: profile.to_owned(),
                            gvks,
                        },
                    )
                    .unwrap();
            })
    }

    pub fn remove_pinned_gvk(
        &self,
        profile: &ClusterProfileId,
        gvk: &GroupVersionKind,
    ) -> Result<(), self::Error> {
        self.remove_gvk(profile, gvk, PERISTENCE_KEY_PINNED_GVKS)
            .map(|gvks| {
                self.app
                    .emit(
                        "hyprkube://pinned-gvks-changed",
                        PinnedGvksChanged {
                            cluster_profile: profile.to_owned(),
                            gvks,
                        },
                    )
                    .unwrap();
            })
    }

    pub fn list_hidden_gvks(
        &self,
        profile: &ClusterProfileId,
    ) -> Result<Vec<GroupVersionKind>, self::Error> {
        self.list_gvks(profile, PERISTENCE_KEY_HIDDEN_GVKS)
    }

    pub fn add_hidden_gvk(
        &self,
        profile: &ClusterProfileId,
        gvk: GroupVersionKind,
    ) -> Result<(), self::Error> {
        self.add_gvk(profile, gvk, PERISTENCE_KEY_HIDDEN_GVKS)
            .map(|gvks| {
                self.app
                    .emit(
                        "hyprkube://hidden-gvks-changed",
                        PinnedGvksChanged {
                            cluster_profile: profile.to_owned(),
                            gvks,
                        },
                    )
                    .unwrap();
            })
    }

    pub fn remove_hidden_gvk(
        &self,
        profile: &ClusterProfileId,
        gvk: &GroupVersionKind,
    ) -> Result<(), self::Error> {
        self.remove_gvk(profile, gvk, PERISTENCE_KEY_HIDDEN_GVKS)
            .map(|gvks| {
                self.app
                    .emit(
                        "hyprkube://hidden-gvks-changed",
                        PinnedGvksChanged {
                            cluster_profile: profile.to_owned(),
                            gvks,
                        },
                    )
                    .unwrap();
            })
    }

    fn list_gvks(
        &self,
        profile: &ClusterProfileId,
        persistence_key: &str,
    ) -> Result<Vec<GroupVersionKind>, self::Error> {
        let items = self.repository.read_key(
            &persistence::Context::PerClusterProfile(profile.clone()),
            persistence_key,
        )?;

        if items.is_none() {
            return Ok(Vec::new());
        }

        let items = serde_json::from_value(items.unwrap())?;

        Ok(items)
    }

    fn add_gvk(
        &self,
        profile: &ClusterProfileId,
        gvk: GroupVersionKind,
        persistence_key: &str,
    ) -> Result<Vec<GroupVersionKind>, self::Error> {
        let mut pinned = self.list_gvks(profile, persistence_key)?;

        if pinned.contains(&gvk) {
            return Ok(pinned);
        }

        pinned.push(gvk.clone());

        self.repository.set_key(
            &persistence::Context::PerClusterProfile(profile.clone()),
            persistence_key,
            serde_json::to_value(pinned.clone())?,
        )?;

        Ok(pinned)
    }

    pub fn remove_gvk(
        &self,
        profile: &ClusterProfileId,
        gvk: &GroupVersionKind,
        persistence_key: &str,
    ) -> Result<Vec<GroupVersionKind>, self::Error> {
        let mut pinned = self.list_gvks(profile, persistence_key)?;

        if !pinned.contains(gvk) {
            return Ok(pinned);
        }

        pinned.retain(|g| g != gvk);

        self.repository.set_key(
            &persistence::Context::PerClusterProfile(profile.clone()),
            persistence_key,
            serde_json::to_value(pinned.clone())?,
        )?;

        Ok(pinned)
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
    Tauri(#[from] tauri::Error),

    #[error(transparent)]
    TauriStore(#[from] tauri_plugin_store::Error),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Repository(#[from] persistence::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
