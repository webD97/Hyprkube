use std::collections::HashMap;

use serde_json::json;

use crate::{
    internal::mini_id::random_id,
    scripting::{
        resource_context_menu::{FrontendMenuItem, FrontendMenuItemKind},
        types::{ActionButton, SubMenu},
    },
};

#[derive(Debug, Clone)]
pub enum MenuItem {
    ActionButton(ActionButton),
    SubMenu(SubMenu),
}

impl From<ActionButton> for MenuItem {
    fn from(value: ActionButton) -> Self {
        MenuItem::ActionButton(value)
    }
}

impl From<SubMenu> for MenuItem {
    fn from(value: SubMenu) -> Self {
        MenuItem::SubMenu(value)
    }
}

impl TryFrom<rhai::Dynamic> for MenuItem {
    type Error = ();

    fn try_from(value: rhai::Dynamic) -> Result<Self, Self::Error> {
        if value.is::<ActionButton>() {
            let item = value.cast::<ActionButton>();
            Ok(MenuItem::ActionButton(item))
        } else if value.is::<SubMenu>() {
            let item = value.cast::<SubMenu>();
            Ok(MenuItem::SubMenu(item))
        } else {
            Err(())
        }
    }
}

pub type ActionId = String;

impl MenuItem {
    /// Transforms a MenuItem into a FrontendMenuItem. Special care is taken for callbacks (used in [ActionButton]):
    /// Callbacks' [rhai::FnPtr]s are assigned an [ActionId] which the frontend can later refer to in order to invoke
    /// the callback.
    pub fn transform_for_frontend(item: Self) -> (FrontendMenuItem, Vec<(ActionId, rhai::FnPtr)>) {
        match item {
            Self::ActionButton(action_button) => {
                let action_id = random_id(5);

                let frontend_item = FrontendMenuItem::new(
                    FrontendMenuItemKind::ActionButton,
                    Some(HashMap::from_iter([
                        ("title", json!(action_button.title)),
                        ("dangerous", json!(action_button.dangerous)),
                        ("confirm", json!(action_button.confirm)),
                        ("actionRef", json!(action_id.clone())),
                    ])),
                );

                (frontend_item, vec![(action_id, action_button.action)])
            }

            Self::SubMenu(submenu) => {
                let (sub_items, actions): (Vec<_>, Vec<_>) = submenu
                    .items
                    .into_iter()
                    .map(Self::transform_for_frontend)
                    .unzip();

                let actions = actions.into_iter().flatten().collect();

                let frontend_item = FrontendMenuItem::new(
                    FrontendMenuItemKind::SubMenu,
                    Some(HashMap::from_iter([
                        ("title", json!(submenu.title)),
                        ("items", json!(sub_items)),
                    ])),
                );

                (frontend_item, actions)
            }
        }
    }
}
