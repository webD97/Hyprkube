use std::{collections::HashMap, path::PathBuf, sync::Arc};

use kube::api::GroupVersionKind;
use rust_embed::Embed;
use scan_dir::ScanDir;

use tauri::{AppHandle, Manager as _};
use uuid::Uuid;

use crate::{
    app_state::KubernetesClientRegistryState,
    resource_rendering::{CrdRenderer, FallbackRenderer, ResourceRenderer, ScriptedResourceView},
};

#[derive(Embed)]
#[folder = "views/"]
struct BuiltinScripts;

pub type RendererRegistryState = Arc<RendererRegistry>;

pub struct RendererRegistry {
    pub mappings: HashMap<GroupVersionKind, Vec<Box<dyn ResourceRenderer>>>,
    generic_renderer: Box<dyn ResourceRenderer>,
    crd_renderer: Box<dyn ResourceRenderer>,
    app_handle: tauri::AppHandle,
}

impl RendererRegistry {
    const EMPTY_VEC: &Vec<Box<dyn ResourceRenderer>> = &Vec::new();

    pub fn new_state(app_handle: tauri::AppHandle) -> RendererRegistryState {
        Arc::new(Self::new(app_handle))
    }

    pub fn new(app_handle: tauri::AppHandle) -> RendererRegistry {
        let mut renderers: HashMap<GroupVersionKind, Vec<Box<dyn ResourceRenderer>>> =
            HashMap::new();

        let views_dir = get_views_dir(&app_handle).unwrap();

        let walk_result: Vec<PathBuf> = ScanDir::files()
            .walk(&views_dir, |iter| {
                iter.filter(|(_, name)| name.ends_with(".rhai"))
                    .map(|(ref entry, _)| entry.path())
                    .collect()
            })
            .unwrap();

        let view_scripts = walk_result
            .iter()
            .map(|path| std::fs::read_to_string(path).unwrap());

        let builtin_scripts = BuiltinScripts::iter().map(|path| {
            let script_bytes = BuiltinScripts::get(&path).unwrap().data;
            std::str::from_utf8(script_bytes.as_ref())
                .unwrap()
                .to_owned()
        });

        for script in builtin_scripts.chain(view_scripts) {
            let view = ScriptedResourceView::new(script.as_str()).unwrap();

            let (group, version) = view
                .definition
                .match_api_version
                .split_once("/")
                .unwrap_or(("", view.definition.match_api_version.as_str()));

            let gvk = GroupVersionKind {
                group: group.into(),
                version: version.into(),
                kind: view.definition.match_kind.clone(),
            };

            println!("Found view {:?} for {:?}", view.definition.name, gvk);

            renderers.entry(gvk).or_default().push(Box::new(view));
        }

        RendererRegistry {
            mappings: renderers,
            generic_renderer: Box::new(FallbackRenderer {}),
            crd_renderer: Box::new(CrdRenderer::default()),
            app_handle,
        }
    }

    /// Returns the names of all available renderers for the given GVK
    pub async fn get_renderers(
        &self,
        kube_client_id: &Uuid,
        gvk: &GroupVersionKind,
    ) -> Vec<String> {
        let renderers = self.mappings.get(gvk).unwrap_or(Self::EMPTY_VEC);

        let kubernetes_client_registry = self.app_handle.state::<KubernetesClientRegistryState>();

        let registered = kubernetes_client_registry
            .get_cluster(kube_client_id)
            .unwrap();

        let crds: Vec<&GroupVersionKind> = registered.2.crds.keys().collect();

        renderers
            .iter()
            .map(|v| v.display_name().to_owned())
            .chain(match crds.contains(&gvk) {
                true => Some(self.crd_renderer.display_name().to_owned()),
                false => None,
            })
            .chain(Some(self.generic_renderer.display_name().to_owned()))
            .collect()
    }

    pub async fn get_renderer(
        &self,
        gvk: &GroupVersionKind,
        view_name: &str,
    ) -> &Box<dyn ResourceRenderer> {
        if view_name == self.generic_renderer.display_name() {
            return &self.generic_renderer;
        }
        if view_name == self.crd_renderer.display_name() {
            return &self.crd_renderer;
        }

        let specific_view = self
            .mappings
            .get(gvk)
            .unwrap_or(Self::EMPTY_VEC)
            .iter()
            .find(|view| view.display_name() == view_name);

        match specific_view {
            Some(view) => view.to_owned(),
            None => {
                eprintln!(
                    "View {:?} not found for {:?}, returning fallback.",
                    &view_name, &gvk
                );
                &self.generic_renderer
            }
        }
    }
}

fn get_views_dir(app: &AppHandle) -> Option<PathBuf> {
    let mut views_dir = app.path().app_data_dir().unwrap();
    views_dir.push("views");

    if !views_dir.exists() {
        match std::fs::create_dir_all(&views_dir) {
            Ok(()) => (),
            Err(error) => {
                eprintln!(
                    "Failed to create directory {:?} for view scripts: {:?}",
                    views_dir, error
                );
                return None;
            }
        }
    }

    Some(views_dir)
}
