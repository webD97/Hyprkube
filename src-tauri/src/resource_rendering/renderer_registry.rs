use std::{collections::HashMap, path::PathBuf};

use kube::api::GroupVersionKind;
use rust_embed::Embed;
use scan_dir::ScanDir;

use crate::{dirs::get_views_dir, resource_rendering::ScriptedResourceView};

use super::{fallback_resource_renderer::FallbackRenderer, ResourceRenderer};

#[derive(Embed)]
#[folder = "views/"]
struct BuiltinScripts;

pub struct RendererRegistry {
    pub mappings: HashMap<GroupVersionKind, Vec<Box<dyn ResourceRenderer>>>,
    generic_renderer: Box<dyn ResourceRenderer>,
}

impl RendererRegistry {
    const EMPTY_VEC: &Vec<Box<dyn ResourceRenderer>> = &Vec::new();

    pub fn new() -> RendererRegistry {
        let mut renderers: HashMap<GroupVersionKind, Vec<Box<dyn ResourceRenderer>>> =
            HashMap::new();

        let views_dir = get_views_dir().unwrap();

        let walk_result: Vec<PathBuf> = ScanDir::files()
            .walk(&views_dir, |iter| {
                iter.filter(|&(_, ref name)| name.ends_with(".rhai"))
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
                .or(Some(("", view.definition.match_api_version.as_str())))
                .unwrap();

            let gvk = GroupVersionKind {
                group: group.into(),
                version: version.into(),
                kind: view.definition.match_kind.clone(),
            };

            println!("Found view {:?} for {:?}", view.definition.name, gvk);

            renderers
                .entry(gvk)
                .or_insert_with(Vec::new)
                .push(Box::new(view));
        }

        RendererRegistry {
            mappings: renderers,
            generic_renderer: Box::new(FallbackRenderer {}),
        }
    }

    /// Returns the names of all available renderers for the given GVK
    pub fn get_renderers(&self, gvk: &GroupVersionKind) -> Vec<String> {
        let renderers = self.mappings.get(gvk).or(Some(Self::EMPTY_VEC)).unwrap();

        renderers
            .iter()
            .map(|v| v.display_name().to_owned())
            .chain(std::iter::once(
                self.generic_renderer.display_name().to_owned(),
            ))
            .collect()
    }

    pub fn get_renderer(
        &self,
        gvk: &GroupVersionKind,
        view_name: &str,
    ) -> &Box<dyn ResourceRenderer> {
        let specific_view = self
            .mappings
            .get(gvk)
            .or(Some(Self::EMPTY_VEC))
            .unwrap()
            .iter()
            .find(|view| view.display_name() == view_name);

        match specific_view {
            Some(view) => return view.to_owned(),
            None => return &self.generic_renderer,
        }
    }
}
