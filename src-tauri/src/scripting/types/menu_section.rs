use rhai::{CustomType, Dynamic, EvalAltResult, TypeBuilder};

#[derive(Clone, Debug, rhai::CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct MenuSection {
    #[rhai_type(readonly)]
    pub title: Option<String>,

    #[rhai_type(readonly)]
    pub matcher: Option<rhai::FnPtr>,

    #[rhai_type(readonly)]
    pub items: rhai::FnPtr,
}

impl MenuSection {
    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("MenuSection", <rhai::Map as TryInto<Self>>::try_into);
    }
}

impl TryFrom<rhai::Map> for MenuSection {
    type Error = Box<EvalAltResult>;

    fn try_from(mut value: rhai::Map) -> Result<Self, Self::Error> {
        let title: Option<String> = value
            .remove("title")
            .map(|v| {
                v.into_string()
                    .map_err(|_| "MenuSection: `title` must be a string".to_owned())
            })
            .transpose()?;

        let matcher = value
            .remove("matcher")
            .map(|v| {
                v.try_cast::<rhai::FnPtr>()
                    .ok_or("MenuSection: `matcher` must be a function".to_owned())
            })
            .transpose()?;

        let items = value
            .remove("items")
            .ok_or_else(|| "MenuSection: missing `items`".to_owned())?
            .try_cast::<rhai::FnPtr>()
            .ok_or("MenuSection: `items` must be a function".to_owned())?;

        Ok(Self {
            title,
            matcher,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_all_keys_given() {
        let map = rhai::Map::from_iter([
            ("title".into(), "Fancy section".into()),
            ("matcher".into(), rhai::FnPtr::new("x").unwrap().into()),
            ("items".into(), rhai::FnPtr::new("y").unwrap().into()),
        ]);

        let section: MenuSection = map.try_into().unwrap();

        assert_eq!(Some("Fancy section"), section.title.as_deref());
        assert!(section.matcher.is_some());
    }

    #[test]
    pub fn test_optional_keys_defaults() {
        let map = rhai::Map::from_iter([("items".into(), rhai::FnPtr::new("y").unwrap().into())]);

        let section: MenuSection = map.try_into().unwrap();

        assert!(section.title.is_none());
        assert!(section.matcher.is_none());
    }

    #[test]
    pub fn test_err_on_missing_required_keys() {
        let without_items = rhai::Map::from_iter([("items".into(), Vec::<Dynamic>::new().into())]);

        assert!(TryInto::<MenuSection>::try_into(without_items).is_err());
    }
}
