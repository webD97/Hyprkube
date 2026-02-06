use rhai::{CustomType, Dynamic, EvalAltResult, Position, TypeBuilder};

#[derive(Clone, rhai::CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ResourceRef {
    pub api_version: String,
    pub kind: String,
    pub namespace: Option<String>,
    pub name: String,
}

impl ResourceRef {
    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("ResourceRef", <rhai::Map as TryInto<Self>>::try_into);
    }
}

impl TryFrom<rhai::Map> for ResourceRef {
    type Error = Box<EvalAltResult>;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        let api_version = value
            .get("apiVersion")
            .ok_or_else(|| "ResourceRef: missing `apiVersion`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ResourceRef: `apiVersion` must be a string".to_owned())?;

        let kind = value
            .get("kind")
            .ok_or_else(|| "ResourceRef: missing `kind`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ResourceRef: `kind` must be a string".to_owned())?;

        let name = value
            .get("name")
            .ok_or_else(|| "ResourceRef: missing `name`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ResourceRef: `name` must be a string".to_owned())?;

        let namespace = match value.get("namespace") {
            None => None,
            Some(v) if v.is_unit() => None,
            Some(v) => Some(
                v.clone()
                    .into_string()
                    .map_err(|_| "ResourceRef: `namespace` must be a string or ()".to_owned())?
                    .to_string(),
            ),
        };

        Ok(Self {
            api_version,
            kind,
            name,
            namespace,
        })
    }
}
