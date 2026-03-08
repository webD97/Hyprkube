use rhai::{CustomType, Dynamic, EvalAltResult, TypeBuilder};

use crate::scripting::types::ColumnTemplate;

#[derive(Clone, Debug, rhai::CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ResourcePresentation {
    #[rhai_type(readonly)]
    pub title: String,

    #[rhai_type(readonly)]
    pub matcher: Option<rhai::FnPtr>,

    #[rhai_type(readonly)]
    pub columns: Vec<ColumnTemplate>,
}

impl ResourcePresentation {
    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn(
            "ResourcePresentation",
            <rhai::Map as TryInto<Self>>::try_into,
        );
    }
}

impl TryFrom<rhai::Map> for ResourcePresentation {
    type Error = Box<EvalAltResult>;

    fn try_from(mut value: rhai::Map) -> Result<Self, Self::Error> {
        let title = value
            .remove("title")
            .ok_or_else(|| "ResourcePresentation: missing `title`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ResourcePresentation: `title` must be a string".to_owned())?;

        let matcher = value
            .remove("matcher")
            .map(|v| {
                v.try_cast::<rhai::FnPtr>()
                    .ok_or("ResourcePresentation: `matcher` must be a function".to_owned())
            })
            .transpose()?;

        let columns = value
            .remove("columns")
            .ok_or_else(|| "ResourcePresentation: missing `columns`".to_owned())?
            .as_array_mut()
            .map_err(|_| {
                "ResourcePresentation: `columns` must be an array of ColumnTemplate".to_owned()
            })?
            .clone()
            .into_iter()
            .map(|template| template.cast::<ColumnTemplate>())
            .collect();

        Ok(Self {
            title,
            matcher,
            columns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_all_keys_given() {
        let map = rhai::Map::from_iter([
            ("title".into(), "My presentation".into()),
            ("matcher".into(), rhai::FnPtr::new("x").unwrap().into()),
            (
                "columns".into(),
                [ColumnTemplate {
                    title: "My column".into(),
                    render: rhai::FnPtr::new("y").unwrap(),
                }]
                .to_vec()
                .into(),
            ),
        ]);

        let section: ResourcePresentation = map.try_into().unwrap();

        assert_eq!("My presentation", section.title);
        assert!(section.matcher.is_some());
    }

    #[test]
    pub fn test_optional_keys_defaults() {
        let map = rhai::Map::from_iter([
            ("title".into(), "My presentation".into()),
            ("columns".into(), rhai::Array::new().into()),
        ]);

        let section: ResourcePresentation = map.try_into().unwrap();

        assert!(section.matcher.is_none());
    }

    #[test]
    pub fn test_err_on_missing_required_keys() {
        let without_title =
            rhai::Map::from_iter([("columns".into(), Vec::<Dynamic>::new().into())]);
        let without_columns = rhai::Map::from_iter([("title".into(), "My presentation".into())]);

        assert!(TryInto::<ResourcePresentation>::try_into(without_title).is_err());
        assert!(TryInto::<ResourcePresentation>::try_into(without_columns).is_err());
    }
}
