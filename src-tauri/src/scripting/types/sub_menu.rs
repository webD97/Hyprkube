use rhai::{CustomType, EvalAltResult, TypeBuilder};

use crate::scripting::types;

#[derive(Clone, Debug, rhai::CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct SubMenu {
    #[rhai_type(readonly)]
    pub title: String,

    #[rhai_type(readonly)]
    pub items: Vec<types::MenuItem>,
}

impl SubMenu {
    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("SubMenu", <rhai::Map as TryInto<Self>>::try_into);
    }
}

impl TryFrom<rhai::Map> for SubMenu {
    type Error = Box<EvalAltResult>;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        let title = value
            .get("title")
            .ok_or_else(|| "SubMenu: missing `title`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "SubMenu: `title` must be a string".to_owned())?;

        let items = value
            .get("items")
            .ok_or_else(|| "ActionButton: missing `title`".to_owned())?
            .as_array_ref()
            .map_err(|_| "SubMenu: `items` must be an array".to_owned())?
            .clone()
            .into_iter()
            .flat_map(|i| i.try_into().ok())
            .collect();

        Ok(Self { title, items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_all_keys_given() {
        let map = rhai::Map::from_iter([
            ("title".into(), "Amazing submenu".into()),
            ("items".into(), Vec::<rhai::Dynamic>::new().into()),
        ]);

        let submenu: SubMenu = map.try_into().unwrap();

        assert_eq!("Amazing submenu", submenu.title);
    }

    #[test]
    pub fn test_err_on_missing_required_keys() {
        let without_title =
            rhai::Map::from_iter([("items".into(), Vec::<rhai::Dynamic>::new().into())]);
        let without_items = rhai::Map::from_iter([("title".into(), "Something".into())]);

        assert!(TryInto::<SubMenu>::try_into(without_title).is_err());
        assert!(TryInto::<SubMenu>::try_into(without_items).is_err());
    }
}
