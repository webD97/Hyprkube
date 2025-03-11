use kube::api::GroupVersionKind;

pub trait ToFlatString {
    fn to_flat_string(&self) -> String;
}

impl ToFlatString for GroupVersionKind {
    fn to_flat_string(&self) -> String {
        format!("{}/{}/{}", self.group, self.kind, self.version)
    }
}
