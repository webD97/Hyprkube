use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Store, StoreExt};

use crate::cluster_profiles;

#[allow(dead_code)]
pub enum Context {
    ApplicationWide,
    PerClusterProfile(cluster_profiles::ClusterProfileId),
    PerCluster(String),
}

pub struct Repository {
    app: AppHandle,
}

impl Repository {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn read_key(
        &self,
        context: &Context,
        key: &str,
    ) -> Result<Option<serde_json::Value>, self::Error> {
        Ok(self.get_tauri_store(context)?.get(key))
    }

    pub fn set_key(
        &self,
        context: &Context,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), self::Error> {
        let store = self.get_tauri_store(context)?;
        store.set(key, value);
        store.save()?;
        Ok(())
    }

    fn get_tauri_store(&self, context: &Context) -> Result<Arc<Store<Wry>>, self::Error> {
        let filename = match context {
            Context::ApplicationWide => "persistence/settings".to_owned(),
            Context::PerCluster(c) => format!("persistence/clusters/{}", c),
            Context::PerClusterProfile(p) => format!("persistence/profiles/{}", p),
        };

        Ok(self.app.store(filename)?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TauriStoreError(#[from] tauri_plugin_store::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
