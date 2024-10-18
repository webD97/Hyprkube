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
}

impl RendererRegistry {
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

        // Fallback renderer that can be used for any resource but with little information
        renderers
            .entry(GroupVersionKind::gvk("*", "*", "*"))
            .or_insert_with(Vec::new)
            .push(Box::new(FallbackRenderer {}));

        RendererRegistry {
            mappings: renderers,
        }
    }

    pub fn get_renderer(&self, gvk: &GroupVersionKind) -> &Box<dyn ResourceRenderer> {
        self.mappings
            .get(gvk)
            .or_else(|| self.mappings.get(&GroupVersionKind::gvk("*", "*", "*")))
            .unwrap()
            .first()
            .unwrap()
    }
}
