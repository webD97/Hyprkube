use crate::scripting::types::ActionButton;

#[derive(Debug, Clone)]
pub enum MenuItem {
    ActionButton(ActionButton),
}

impl From<ActionButton> for MenuItem {
    fn from(value: ActionButton) -> Self {
        MenuItem::ActionButton(value)
    }
}
