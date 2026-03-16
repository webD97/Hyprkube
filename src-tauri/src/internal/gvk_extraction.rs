use kube::{
    api::{GroupVersionKind, TypeMeta},
    core::{gvk::ParseGroupVersionError, GroupVersion},
};

pub trait GvkExtraction {
    fn try_gvk(&self) -> Result<GroupVersionKind, ParseGroupVersionError>;
}

impl GvkExtraction for TypeMeta {
    fn try_gvk(&self) -> Result<GroupVersionKind, ParseGroupVersionError> {
        let gv = self.api_version.parse::<GroupVersion>()?;

        Ok(GroupVersionKind::gvk(&gv.group, &gv.version, &self.kind))
    }
}

#[cfg(test)]
mod tests {
    use kube::api::TypeMeta;

    use super::GvkExtraction as _;

    #[test]
    fn test_extract_core_group() {
        let meta = TypeMeta {
            api_version: "v1".into(),
            kind: "Pod".into(),
        };

        let gvk = meta.try_gvk().unwrap();

        assert_eq!("", gvk.group);
        assert_eq!("v1", gvk.version);
        assert_eq!("Pod", gvk.kind);
    }

    #[test]
    fn test_extract_other_group() {
        let meta = TypeMeta {
            api_version: "acme.com/v1".into(),
            kind: "GiantRubberBand".into(),
        };

        let gvk = meta.try_gvk().unwrap();

        assert_eq!("acme.com", gvk.group);
        assert_eq!("v1", gvk.version);
        assert_eq!("GiantRubberBand", gvk.kind);
    }
}
