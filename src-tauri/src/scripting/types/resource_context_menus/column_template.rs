use rhai::{CustomType, EvalAltResult, TypeBuilder};

#[derive(Clone, Debug, rhai::CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColumnTemplate {
    #[rhai_type(readonly)]
    pub title: String,

    #[rhai_type(readonly)]
    pub render: rhai::FnPtr,
}

impl ColumnTemplate {
    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("ColumnTemplate", <rhai::Map as TryInto<Self>>::try_into);
    }
}

impl TryFrom<rhai::Map> for ColumnTemplate {
    type Error = Box<EvalAltResult>;

    fn try_from(mut value: rhai::Map) -> Result<Self, Self::Error> {
        let title = value
            .remove("title")
            .ok_or_else(|| "ColumnTemplate: missing `title`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ColumnTemplate: `title` must be a string".to_owned())?;

        let render = value
            .remove("render")
            .ok_or_else(|| "ColumnTemplate: missing `render`".to_owned())?
            .try_cast::<rhai::FnPtr>()
            .ok_or("ColumnTemplate: `render` must be a function".to_owned())?;

        Ok(Self { title, render })
    }
}

#[cfg(test)]
mod tests {
    use rhai::Dynamic;

    use super::*;

    #[test]
    pub fn test_all_keys_given() {
        let map = rhai::Map::from_iter([
            ("title".into(), "Column A".into()),
            ("render".into(), rhai::FnPtr::new("y").unwrap().into()),
        ]);

        let section: ColumnTemplate = map.try_into().unwrap();

        assert_eq!("Column A", section.title);
    }

    #[test]
    pub fn test_err_on_missing_required_keys() {
        let without_title = rhai::Map::from_iter([("render".into(), Vec::<Dynamic>::new().into())]);
        let without_render = rhai::Map::from_iter([("title".into(), "Column A".into())]);

        assert!(TryInto::<ColumnTemplate>::try_into(without_title).is_err());
        assert!(TryInto::<ColumnTemplate>::try_into(without_render).is_err());
    }
}
