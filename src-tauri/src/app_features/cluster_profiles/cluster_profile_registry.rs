use std::{collections::HashMap, path::PathBuf, sync::Arc};

use scan_dir::ScanDir;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dirs::get_cluster_profiles_dir;

#[derive(Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub struct ClusterProfileId(Uuid);

#[derive(Default)]
pub struct ClusterProfileRegistry {
    profiles: HashMap<ClusterProfileId, (String, PathBuf)>,
}

pub type ClusterProfileRegistryState = Arc<ClusterProfileRegistry>;

impl ClusterProfileRegistry {
    pub fn scan_profiles(&mut self) {
        let profiles_dir = get_cluster_profiles_dir().unwrap();

        let profile_paths: Vec<PathBuf> = ScanDir::dirs()
            .walk(&profiles_dir, |iter| {
                iter.map(|(ref entry, _)| entry.path()).collect()
            })
            .unwrap();

        self.profiles.clear();

        for path in profile_paths {
            let display_name = path.file_name().unwrap().to_string_lossy().to_string();

            self.profiles
                .entry(ClusterProfileId(Uuid::new_v4()))
                .or_insert((display_name, path));
        }
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

    pub fn get_profile_basedir(&self, profile_runtime_id: &ClusterProfileId) -> Option<PathBuf> {
        self.profiles
            .get(&profile_runtime_id)
            .map(|profile| profile.1.clone())
    }
}
