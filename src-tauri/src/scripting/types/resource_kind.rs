use kube::{api::GroupVersionKind, core::gvk::ParseGroupVersionError};
use rhai::{CustomType, EvalAltResult, TypeBuilder};
use serde::Serialize;

#[derive(Clone, Debug, rhai::CustomType, Serialize)]
#[rhai_type(extra = Self::build_extra)]
#[serde(rename_all = "camelCase")]
pub struct ResourceKind {
    #[rhai_type(readonly)]
    pub api_version: String,

    #[rhai_type(readonly)]
    pub kind: String,
}

impl ResourceKind {
    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("ResourceKind", <rhai::Map as TryInto<Self>>::try_into);
        builder.with_fn("ResourceKind", Self::from_parts);
    }

    fn from_parts(api_version: &str, kind: &str) -> Self {
        Self {
            api_version: api_version.to_owned(),
            kind: kind.to_owned(),
        }
    }
}

impl TryFrom<rhai::Map> for ResourceKind {
    type Error = Box<EvalAltResult>;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        let api_version = value
            .get("apiVersion")
            .ok_or_else(|| "ApiVersionKind: missing `apiVersion`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ApiVersionKind: `api_version` must be a string".to_owned())?;

        let kind = value
            .get("kind")
            .ok_or_else(|| "ApiVersionKind: missing `kind`".to_owned())?
            .clone()
            .into_string()
            .map_err(|_| "ApiVersionKind: `kind` must be a string".to_owned())?;

        Ok(ResourceKind { api_version, kind })
    }
}

impl TryFrom<ResourceKind> for kube::api::GroupVersionKind {
    type Error = ParseGroupVersionError;

    fn try_from(value: ResourceKind) -> Result<Self, Self::Error> {
        let gv: kube::core::GroupVersion = value.api_version.parse()?;

        Ok(GroupVersionKind {
            group: gv.group,
            version: gv.version,
            kind: value.kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_from_map_all_keys_given() {
        let map = rhai::Map::from_iter([
            ("apiVersion".into(), "apps/v1".into()),
            ("kind".into(), "Deployment".into()),
        ]);

        let kind: ResourceKind = map.try_into().unwrap();

        assert_eq!("apps/v1", kind.api_version);
        assert_eq!("Deployment", kind.kind);
    }

    #[test]
    pub fn test_from_map_err_on_missing_required_keys() {
        let without_api_version = rhai::Map::from_iter([("kind".into(), "Deployment".into())]);
        let without_kind = rhai::Map::from_iter([("apiVersion".into(), "apps/v1".into())]);

        assert!(TryInto::<ResourceKind>::try_into(without_api_version).is_err());
        assert!(TryInto::<ResourceKind>::try_into(without_kind).is_err());
    }

    #[test]
    pub fn test_from_parts() {
        let kind = ResourceKind::from_parts("apps/v1", "Deployment");

        assert_eq!("apps/v1", kind.api_version);
        assert_eq!("Deployment", kind.kind);
    }
}
