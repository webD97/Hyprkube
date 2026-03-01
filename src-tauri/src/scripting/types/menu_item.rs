use crate::scripting::types::{ActionButton, SubMenu};

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
