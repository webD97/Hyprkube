use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use kube::api::GroupVersionKind;
use notify_debouncer_mini::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer,
};
use scan_dir::ScanDir;
use serde::Serialize;
use tauri::Emitter;

use crate::{dirs::get_views_dir, frontend_types::FrontendValue, resource_views::ResourceView};

const FALLBACK_SCRIPT: &str = include_str!("./fallback_view.rhai");

pub struct ViewRegistry {
    views: Arc<Mutex<HashMap<GroupVersionKind, HashMap<PathBuf, ResourceView>>>>,
    script_watchers: HashMap<PathBuf, Debouncer<RecommendedWatcher>>,
    app: tauri::AppHandle,
}

impl ViewRegistry {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self {
            views: Arc::new(Mutex::new(HashMap::new())),
            script_watchers: HashMap::new(),
            app,
        }
    }

    fn import_script(
        view_map: Arc<Mutex<HashMap<GroupVersionKind, HashMap<PathBuf, ResourceView>>>>,
        contents: &str,
        source_path: PathBuf,
    ) -> Result<(GroupVersionKind, String), &str> {
        let view: ResourceView = ResourceView::new(contents).unwrap();

        let gvk = match view.get_gvk() {
            None => return Err("Script does not contain a valid GVK"),
            Some(gvk) => gvk,
        };

        let view_name = view.get_display_name().to_owned();

        view_map
            .lock()
            .unwrap()
            .entry(gvk.clone())
            .or_insert(HashMap::new())
            .insert(source_path, view);

        Ok((gvk, view_name))
    }

    fn scan_directory(&mut self, path: PathBuf) {
        let view_scripts: Vec<PathBuf> = ScanDir::files()
            .walk(&path, |iter| {
                iter.filter(|&(_, ref name)| name.ends_with(".rhai"))
                    .map(|(ref entry, _)| entry.path())
                    .collect()
            })
            .unwrap();

        for script_path in view_scripts {
            let the_map = self.views.clone();
            let the_app = self.app.clone();

            let mut watcher =
                new_debouncer(Duration::from_secs(1), move |res: DebounceEventResult| {
                    match res {
                        Ok(events) => events.iter().for_each(|e| {
                            println!("Event {:?} for {:?}", e.kind, e.path);

                            let script = std::fs::read_to_string(&e.path).unwrap();

                            let (gvk, name) = Self::import_script(
                                the_map.clone(),
                                script.as_str(),
                                e.path.clone(),
                            )
                            .unwrap();

                            the_app
                                .emit("view_definition_changed".into(), (gvk, name))
                                .unwrap();
                        }),
                        Err(e) => println!("Error {:?}", e),
                    };
                })
                .unwrap();

            watcher
                .watcher()
                .watch(&script_path, RecursiveMode::NonRecursive)
                .unwrap();

            self.script_watchers.insert(script_path.clone(), watcher);

            let script = std::fs::read_to_string(&script_path).unwrap();

            Self::import_script(self.views.clone(), script.as_str(), script_path).unwrap();
        }
    }

    /// Scans all known config directories for custom view scripts
    pub fn scan_directories(&mut self) {
        Self::import_script(self.views.clone(), FALLBACK_SCRIPT, PathBuf::new()).unwrap();
        let views_dir = get_views_dir().unwrap();
        self.scan_directory(views_dir);
    }

    pub fn render_default_column_titles_for_gvk(&self, gvk: &GroupVersionKind) -> Vec<String> {
        let current_views = self.views.lock().unwrap();

        if !current_views.contains_key(gvk) || current_views.get(gvk).unwrap().len() < 1 {
            println!("Using fallback columns for {:?}", gvk);

            let empty_gvk = &GroupVersionKind {
                group: "".into(),
                version: "".into(),
                kind: "".into(),
            };

            let fallback = current_views
                .get(empty_gvk)
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .1;
            return fallback.render_titles();
        }

        current_views
            .get(gvk)
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .1
            .render_titles()
    }

    pub fn render_default_view_for_gvk<T>(
        &self,
        gvk: &GroupVersionKind,
        obj: &T,
    ) -> Vec<Result<Vec<FrontendValue>, String>>
    where
        T: kube::Resource + Clone + Serialize,
    {
        let current_views = self.views.lock().unwrap();

        if !current_views.contains_key(gvk) || current_views.get(gvk).unwrap().len() < 1 {
            println!("Using fallback view for {:?}", gvk);

            let empty_gvk = &GroupVersionKind {
                group: "".into(),
                version: "".into(),
                kind: "".into(),
            };

            let fallback = current_views
                .get(empty_gvk)
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .1;
            return fallback.render_columns(obj);
        }

        current_views
            .get(gvk)
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .1
            .render_columns(obj)
    }
}
