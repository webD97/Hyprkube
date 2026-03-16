use kube::api::{GroupVersionKind, TypeMeta};

pub trait GvkExtraction {
    fn gvk(&self) -> GroupVersionKind;
}

impl GvkExtraction for TypeMeta {
    fn gvk(&self) -> GroupVersionKind {
        let (group, version) = match self.api_version.split_once('/') {
            Some((g, v)) => (g, v),
            None => ("", self.api_version.as_str()),
        };

        GroupVersionKind::gvk(group, version, &self.kind)
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

        let gvk = meta.gvk();

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

        let gvk = meta.gvk();

        assert_eq!("acme.com", gvk.group);
        assert_eq!("v1", gvk.version);
        assert_eq!("GiantRubberBand", gvk.kind);
    }
}
