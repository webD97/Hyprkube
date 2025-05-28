use std::{collections::HashMap, fs::OpenOptions, path::PathBuf, sync::Arc};

use scan_dir::ScanDir;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{error, info};

#[derive(Eq, Hash, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct ClusterProfileId(String);

impl std::fmt::Display for ClusterProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct ClusterProfileRegistry {
    app_handle: AppHandle,
    profiles: HashMap<ClusterProfileId, (String, PathBuf)>,
}

pub type ClusterProfileRegistryState = Arc<ClusterProfileRegistry>;

impl ClusterProfileRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            profiles: HashMap::new(),
        }
    }

    pub fn ensure_default_profile(&self) -> Result<(), std::io::Error> {
        let profiles_dir = {
            let mut path = get_cluster_profiles_dir(&self.app_handle).unwrap();
            path.push("_default");
            path
        };

        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(profiles_dir)?;

        Ok(())
    }

    pub fn scan_profiles(&mut self) {
        let profiles_dir = get_cluster_profiles_dir(&self.app_handle).unwrap();

        let profile_paths: Vec<PathBuf> = ScanDir::files()
            .walk(&profiles_dir, |iter| {
                iter.map(|(ref entry, _)| entry.path()).collect()
            })
            .unwrap();

        self.profiles.clear();

        for path in profile_paths {
            let display_name = path.file_name().unwrap().to_string_lossy().to_string();

            self.profiles
                .entry(ClusterProfileId(display_name.clone()))
                .or_insert((display_name, path));
        }

        info!("Profiles: {:?}", self.profiles);
    }

    pub fn get_profiles(&self) -> Vec<(ClusterProfileId, String)> {
        self.profiles
            .iter()
            .map(|profile| {
                let id = profile.0.clone();
                let display_name = (profile.1).0.clone();

                (id, display_name)
            })
            .collect()
    }
}

fn get_cluster_profiles_dir(app: &AppHandle) -> Option<PathBuf> {
    let mut cluster_profiles_dir = app.path().app_data_dir().unwrap();
    cluster_profiles_dir.push("persistence/profiles");

    if !cluster_profiles_dir.exists() {
        match std::fs::create_dir_all(&cluster_profiles_dir) {
            Ok(()) => (),
            Err(error) => {
                error!(
                    "Failed to create directory {:?} for cluster profiles: {:?}",
                    cluster_profiles_dir, error
                );
                return None;
            }
        }
    }

    Some(cluster_profiles_dir)
}
