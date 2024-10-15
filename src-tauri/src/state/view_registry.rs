use std::{collections::HashMap, path::PathBuf, sync::Arc};

use kube::api::GroupVersionKind;
use scan_dir::ScanDir;

use crate::resource_views::ResourceView;

const FALLBACK_SCRIPT: &str = include_str!("./fallback_view.rhai");

#[derive(Default)]
pub struct ViewRegistry {
    views: HashMap<GroupVersionKind, Vec<Arc<ResourceView>>>,
}

impl ViewRegistry {
    fn import_script(&mut self, contents: &str) {
        let view = ResourceView::new(contents).unwrap();

        let gvk = match view.get_gvk() {
            None => {
                eprintln!("Script does not contain a valid GVK");
                return;
            }
            Some(gvk) => gvk,
        };

        self.views.entry(gvk).or_insert(vec![]).push(Arc::new(view));
    }

    fn scan_directory(&mut self, path: &PathBuf) {
        let view_scripts: Vec<PathBuf> = ScanDir::files()
            .walk(path, |iter| {
                iter.filter(|&(_, ref name)| name.ends_with(".rhai"))
                    .map(|(ref entry, _)| entry.path())
                    .collect()
            })
            .unwrap();

        for script_path in view_scripts {
            let script = std::fs::read_to_string(&script_path).unwrap();
            self.import_script(script.as_str());
        }
    }

    fn scan_config_directory(&mut self) {
        let config_dir = dirs::config_dir();

        if config_dir.is_none() {
            eprintln!("Cannot scan config dir for custom views because its location is not known. This might be an unsupported platform.");
            return;
        }

        let mut config_dir = config_dir.unwrap();
        config_dir.push("hyprkube");
        let config_dir = config_dir;

        let mut views_dir = config_dir.clone();
        views_dir.push("views");
        let views_dir = views_dir;

        if !views_dir.exists() {
            let create_result = std::fs::create_dir_all(&views_dir);

            if create_result.is_err() {
                eprintln!(
                    "Failed to create directory {:?} for custom view scripts: {:?}",
                    views_dir, create_result
                );
                return;
            }
        }

        self.scan_directory(&views_dir);
    }

    /// Scans all known config directories for custom view scripts
    pub fn scan_directories(&mut self) {
        self.import_script(FALLBACK_SCRIPT);
        self.scan_config_directory();
    }

    pub fn get_default_for_gvk(&self, gvk: &GroupVersionKind) -> Option<&Arc<ResourceView>> {
        let empty_gvk = &GroupVersionKind {
            group: "".into(),
            version: "".into(),
            kind: "".into(),
        };

        if !self.views.contains_key(gvk) || self.views.get(gvk).unwrap().len() < 1 {
            return self.views.get(empty_gvk).unwrap().get(0);
        }

        self.views.get(gvk).unwrap().get(0)
    }
}
