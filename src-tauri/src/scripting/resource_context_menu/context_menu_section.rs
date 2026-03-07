use std::sync::Arc;

use kube::api::GroupVersionKind;

use crate::scripting::types;

pub struct ContextMenuSection {
    pub title: Option<String>,
    pub matcher: Option<rhai::FnPtr>,
    pub items: rhai::FnPtr,
    pub ast: Arc<rhai::AST>,
}

impl ContextMenuSection {
    /// Calls the `matcher` function defined in the script. If no matcher is defined, we assume that this menu section
    /// applies to every single Kubernetes resource. This is useful for general purpose actions like 'Delete'.
    pub fn matches_gvk(
        &self,
        engine: &rhai::Engine,
        ast: &rhai::AST,
        gvk: GroupVersionKind,
    ) -> Result<bool, Box<rhai::EvalAltResult>> {
        self.matcher
            .as_ref()
            .map(|matcher| matcher.call::<bool>(engine, ast, (gvk.group, gvk.version, gvk.kind)))
            .unwrap_or(Ok(true))
    }

    /// Calls the `items` function defined in the script with the given Kubernetes object. The script may inspect the
    /// object to customize the items per resource. This is useful to render repeating elements like an action per
    /// container within a Pod.
    pub fn render_items_for(
        &self,
        engine: &rhai::Engine,
        ast: &rhai::AST,
        obj: rhai::Dynamic,
    ) -> Result<Vec<types::MenuItem>, Box<rhai::EvalAltResult>> {
        Ok(self
            .items
            .call::<rhai::Array>(engine, ast, (obj.clone(),))?
            .into_iter()
            .filter(|i| !i.is_unit())
            .flat_map(|dynamic| {
                let type_name = dynamic.type_name();
                let something: Option<types::MenuItem> = dynamic
                    .try_into()
                    .map_err(|_| {
                        tracing::warn!("Unsupported menu item: {type_name}");
                    })
                    .ok();

                something
            })
            .collect())
    }
}
