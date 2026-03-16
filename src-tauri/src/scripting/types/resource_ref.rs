use rhai::{CustomType, Dynamic, EvalAltResult, TypeBuilder};
use serde::Serialize;

#[derive(Clone, Debug, rhai::CustomType, Serialize)]
#[rhai_type(extra = Self::build_extra)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRef {
    #[rhai_type(readonly)]
    pub api_version: String,

    #[rhai_type(readonly)]
    pub kind: String,

    #[rhai_type(readonly)]
    pub namespace: Option<String>,

    #[rhai_type(readonly)]
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
        if value.contains_key("apiVersion")
            && value.contains_key("kind")
            && value.contains_key("metadata")
        {
            try_from_resource(value)
        } else {
            try_from_parts(value)
        }
    }
}

fn try_from_resource(value: rhai::Map) -> Result<ResourceRef, Box<EvalAltResult>> {
    let api_version = value
        .get("apiVersion")
        .ok_or_else(|| "ResourceRef: missing `apiVersion`".to_owned())?
        .clone()
        .into_string()
        .map_err(|_| "ResourceRef: `api_version` must be a string".to_owned())?;

    let kind = value
        .get("kind")
        .ok_or_else(|| "ResourceRef: missing `kind`".to_owned())?
        .clone()
        .into_string()
        .map_err(|_| "ResourceRef: `kind` must be a string".to_owned())?;

    let namespace = value
        .get("metadata")
        .ok_or_else(|| "ResourceRef: missing `metadata`".to_owned())?
        .clone()
        .try_cast::<rhai::Map>()
        .ok_or_else(|| "ResourceRef: `metadata` must be a map".to_owned())?
        .get("namespace")
        .ok_or_else(|| "ResourceRef: missing `metadata.namespace`".to_owned())?
        .clone()
        .into_string()
        .map_err(|_| "ResourceRef: `metadata.namespace` must be a string".to_owned())?;

    let name = value
        .get("metadata")
        .ok_or_else(|| "ResourceRef: missing `metadata`".to_owned())?
        .clone()
        .try_cast::<rhai::Map>()
        .ok_or_else(|| "ResourceRef: `metadata` must be a map".to_owned())?
        .get("name")
        .ok_or_else(|| "ResourceRef: missing `metadata.name`".to_owned())?
        .clone()
        .into_string()
        .map_err(|_| "ResourceRef: `metadata.name` must be a string".to_owned())?;

    Ok(ResourceRef {
        api_version,
        kind,
        name,
        namespace: Some(namespace),
    })
}

fn try_from_parts(value: rhai::Map) -> Result<ResourceRef, Box<EvalAltResult>> {
    let api_version = value
        .get("api_version")
        .ok_or_else(|| "ResourceRef: missing `api_version`".to_owned())?
        .clone()
        .into_string()
        .map_err(|_| "ResourceRef: `api_version` must be a string".to_owned())?;

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

    Ok(ResourceRef {
        api_version,
        kind,
        name,
        namespace,
    })
}
