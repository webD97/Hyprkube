use kube::api::{DynamicObject, GroupVersionKind};

use crate::resource_menu::api::HyprkubeMenuItem;

pub trait DynamicResourceMenuProvider<C> {
    /// Whether or a not a menu provider should be called for a given GVK
    fn matches(&self, gvk: &GroupVersionKind) -> bool;

    /// Build a menu for a specific resource
    fn build(&self, gvk: &GroupVersionKind, resource: &DynamicObject) -> Vec<HyprkubeMenuItem<C>>;
}
