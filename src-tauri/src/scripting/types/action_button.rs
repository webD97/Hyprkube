use rhai::{CustomType, EvalAltResult, TypeBuilder};

#[derive(Clone, Debug, rhai::CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ActionButton {
    #[rhai_type(readonly)]
    pub title: String,

    #[rhai_type(readonly)]
    pub action: rhai::FnPtr,

    #[rhai_type(readonly)]
    pub dangerous: bool,
}

impl ActionButton {
    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("ActionButton", <rhai::Map as TryInto<Self>>::try_into);
    }
}

impl TryFrom<rhai::Map> for ActionButton {
    type Error = Box<EvalAltResult>;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        let title = value
            .get("title")
            .ok_or_else(|| "ActionButton: missing `title`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ActionButton: `title` must be a string".to_owned())?;

        let action = value
            .get("action")
            .ok_or_else(|| "ActionButton: missing `action`".to_owned())?
            .clone()
            .try_cast::<rhai::FnPtr>()
            .ok_or("ActionButton: `action` must be a function".to_owned())?;

        let dangerous = value
            .get("dangerous")
            .unwrap_or(&rhai::Dynamic::from_bool(false))
            .clone()
            .try_cast::<bool>()
            .ok_or("ActionButton: `dangerous` must be either bool or ()")?;

        Ok(Self {
            title,
            action,
            dangerous,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_all_keys_given() {
        let map = rhai::Map::from_iter([
            ("title".into(), "Fancy button".into()),
            ("dangerous".into(), true.into()),
            ("action".into(), rhai::FnPtr::new("x").unwrap().into()),
        ]);

        let button: ActionButton = map.try_into().unwrap();

        assert_eq!("Fancy button", button.title);
        assert!(button.dangerous);
    }

    #[test]
    pub fn test_optional_keys_defaults() {
        let map = rhai::Map::from_iter([
            ("title".into(), "Fancy button".into()),
            ("action".into(), rhai::FnPtr::new("x").unwrap().into()),
        ]);

        let button: ActionButton = map.try_into().unwrap();

        assert!(!button.dangerous);
    }

    #[test]
    pub fn test_err_on_missing_required_keys() {
        let without_title =
            rhai::Map::from_iter([("action".into(), rhai::FnPtr::new("x").unwrap().into())]);
        let without_action =
            rhai::Map::from_iter([("action".into(), rhai::FnPtr::new("x").unwrap().into())]);

        assert!(TryInto::<ActionButton>::try_into(without_title).is_err());
        assert!(TryInto::<ActionButton>::try_into(without_action).is_err());
    }
}
