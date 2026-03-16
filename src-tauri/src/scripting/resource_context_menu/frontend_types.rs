use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[allow(unused)]
pub enum FrontendMenuItemKind {
    ActionButton,
    SubMenu,
    ResourceSubMenu,
    Separator,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendMenuItem {
    kind: FrontendMenuItemKind,
    data: Option<HashMap<&'static str, serde_json::Value>>,
}

impl FrontendMenuItem {
    pub fn new(
        kind: FrontendMenuItemKind,
        data: Option<HashMap<&'static str, serde_json::Value>>,
    ) -> Self {
        Self { kind, data }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendMenuSection {
    title: Option<String>,
    items: Vec<FrontendMenuItem>,
}

impl FrontendMenuSection {
    pub fn new(title: Option<String>, items: Vec<FrontendMenuItem>) -> Self {
        Self { title, items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MenuBlueprint {
    id: String,
    items: Vec<FrontendMenuSection>,
}

impl MenuBlueprint {
    pub fn new(id: String, items: Vec<FrontendMenuSection>) -> Self {
        Self { id, items }
    }
}
