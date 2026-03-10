use rhai::{CustomType, EvalAltResult, TypeBuilder};

use crate::scripting::types::ResourceRef;

#[derive(Clone, Debug, rhai::CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ResourceSubMenu {
    #[rhai_type(readonly)]
    pub title: String,

    #[rhai_type(readonly)]
    pub resource_ref: ResourceRef,
}

impl ResourceSubMenu {
    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("ResourceSubMenu", <rhai::Map as TryInto<Self>>::try_into);
    }
}

impl TryFrom<rhai::Map> for ResourceSubMenu {
    type Error = Box<EvalAltResult>;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        let title = value
            .get("title")
            .ok_or_else(|| "ResourceSubMenu: missing `title`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ResourceSubMenu: `title` must be a string".to_owned())?;

        let resource_ref = value
            .get("resource_ref")
            .ok_or_else(|| "ActionButton: missing `resource_ref`".to_owned())?
            .clone()
            .try_cast::<ResourceRef>()
            .ok_or("ResourceSubMenu: `resource_ref` must be a ResourceRef".to_owned())?;
        Ok(Self {
            title,
            resource_ref,
        })
    }
}
