use async_trait::async_trait;
use json_patch::{jsonptr::PointerBuf, PatchOperation, ReplaceOperation};
use kube::{
    api::{DynamicObject, GroupVersionKind, Patch, PatchParams},
    core::GroupVersion,
};
use rhai::{CustomType, Scope, TypeBuilder};
use serde::Serialize;
use tauri::async_runtime;

use crate::menus::MenuAction;

pub(crate) struct RhaiScriptedMenuAction {
    script: String,
    resource: DynamicObject,
}

impl RhaiScriptedMenuAction {
    pub fn new(script: &str, resource: DynamicObject) -> anyhow::Result<Self> {
        Ok(Self {
            script: script.to_owned(),
            resource,
        })
    }
}

#[async_trait]
impl MenuAction for RhaiScriptedMenuAction {
    async fn run(&self, _app: &tauri::AppHandle, client: kube::Client) -> anyhow::Result<()> {
        let mut engine = rhai::Engine::new();

        engine.build_type::<JsonPatch>();
        engine.build_type::<ApiResource>();

        engine.register_fn(
            "kpatch",
            move |resource: ApiResource, patches: rhai::Array| {
                let client = client.clone();

                let patches: Vec<JsonPatch> =
                    patches.into_iter().map(|v| v.cast::<JsonPatch>()).collect();

                // Maybe hacky?
                async_runtime::spawn(async move {
                    use kube::Api;
                    let gv: GroupVersion = resource.api_version.parse().unwrap();
                    let gvk = GroupVersionKind::gvk(&gv.group, &gv.version, &resource.kind);

                    let (api_resource, _) = kube::discovery::oneshot::pinned_kind(&client, &gvk)
                        .await
                        .unwrap();

                    let namespace = &resource.namespace;

                    let api: Api<DynamicObject> =
                        Api::namespaced_with(client, namespace, &api_resource);

                    let patches = Patch::Json::<()>(json_patch::Patch(
                        patches.into_iter().map(|p| p.into()).collect(),
                    ));

                    api.patch(&resource.name, &PatchParams::default(), &patches)
                        .await
                        .unwrap();
                });
            },
        );

        let ast = engine.compile(&self.script)?;

        ///////////

        let mut scope = Scope::new();
        let resource = rhai::serde::to_dynamic(self.resource.clone()).unwrap();
        engine.call_fn::<()>(&mut scope, &ast, "run", (resource,))?;

        Ok(())
    }
}

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct JsonPatch {
    pub operation: String,
    pub path: String,
    pub new_value: serde_json::Value,
}

impl JsonPatch {
    pub fn new(operation: String, path: String, new_value: rhai::Dynamic) -> Self {
        Self {
            operation,
            path,
            new_value: rhai::serde::from_dynamic(&new_value).unwrap(),
        }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("JsonPatch", Self::new);
    }
}

impl From<JsonPatch> for PatchOperation {
    fn from(val: JsonPatch) -> Self {
        match val.operation.as_str() {
            "replace" => PatchOperation::Replace(ReplaceOperation {
                path: PointerBuf::parse(val.path).unwrap(),
                value: val.new_value,
            }),
            _ => panic!("unsupported operation"),
        }
    }
}

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ApiResource {
    pub api_version: String,
    pub kind: String,
    pub namespace: String,
    pub name: String,
}

impl ApiResource {
    pub fn new(object: rhai::Dynamic) -> Self {
        use kube::runtime::reflector::Lookup;

        let resource: DynamicObject = rhai::serde::from_dynamic(&object).unwrap();

        let api_version = resource
            .types
            .as_ref()
            .map(|t| t.api_version.clone())
            .unwrap();

        let kind = resource.types.as_ref().map(|t| t.kind.clone()).unwrap();

        let namespace = resource.namespace().unwrap();
        let name = resource.name().unwrap();

        Self {
            api_version,
            kind,
            namespace: namespace.to_string(),
            name: name.to_string(),
        }
    }

    fn build_extra(builder: &mut rhai::TypeBuilder<Self>) {
        builder.with_fn("ApiResource", Self::new);
    }
}
