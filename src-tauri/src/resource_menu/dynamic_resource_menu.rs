use kube::api::{DynamicObject, GroupVersionKind};

use crate::menus::HyprkubeMenuItem;

pub trait DynamicResourceMenuProvider {
    /// Whether or a not a menu provider should be called for a given GVK
    fn matches(&self, gvk: &GroupVersionKind) -> bool;

    /// Build a menu for a specific resource
    fn build(
        &self,
        gvk: &GroupVersionKind,
        resource: &DynamicObject,
        tab_id: String,
    ) -> anyhow::Result<Vec<HyprkubeMenuItem>>;
}
