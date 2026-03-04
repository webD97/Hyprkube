use kube::api::GroupVersionKind;
use rhai::plugin::*;
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FrontendTriggerResourceEdit {
    pub gvk: GroupVersionKind,
    pub namespace: String,
    pub name: String,
    pub tab_id: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FrontendTriggerPickNamespace {
    pub namespace: String,
    pub tab_id: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FrontendTriggerExec {
    pub namespace: String,
    pub name: String,
    pub container: String,
    pub tab_id: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FrontendTriggerLogView {
    pub namespace: String,
    pub name: String,
    pub container: String,
    pub tab_id: String,
}

#[export_module]
pub mod frontend_rhai {
    use std::sync::Arc;

    use kube::core::GroupVersion;
    use serde::Serialize;
    use tauri::Emitter as _;

    use crate::scripting::{resource_context_menu_facade::CallbackContext, types::ResourceRef};

    #[rhai_fn(return_raw)]
    pub fn open_resource_editor(
        ctx: Arc<CallbackContext>,
        resource: ResourceRef,
    ) -> Result<(), Box<rhai::EvalAltResult>> {
        let gv: GroupVersion = resource.api_version.parse().unwrap();
        let frontend_tab = ctx.frontend_tab.to_owned();

        emit(
            ctx,
            "hyprkube:menu:resource:trigger_edit",
            FrontendTriggerResourceEdit {
                gvk: GroupVersionKind::gvk(&gv.group, &gv.version, &resource.kind),
                namespace: resource.namespace.unwrap_or_default(),
                name: resource.name,
                tab_id: frontend_tab,
            },
        )
    }

    #[rhai_fn(return_raw)]
    pub fn exec_shell(
        ctx: Arc<CallbackContext>,
        namespace: &str,
        name: &str,
        container: &str,
    ) -> Result<(), Box<rhai::EvalAltResult>> {
        let frontend_tab = ctx.frontend_tab.to_owned();

        emit(
            ctx,
            "hyprkube:menu:resource:trigger_exec",
            FrontendTriggerExec {
                namespace: namespace.to_owned(),
                name: name.to_owned(),
                container: container.to_owned(),
                tab_id: frontend_tab,
            },
        )
    }

    #[rhai_fn(return_raw)]
    pub fn open_logs(
        ctx: Arc<CallbackContext>,
        namespace: &str,
        name: &str,
        container: &str,
    ) -> Result<(), Box<rhai::EvalAltResult>> {
        let frontend_tab = ctx.frontend_tab.to_owned();

        emit(
            ctx,
            "hyprkube:menu:resource:trigger_exec",
            FrontendTriggerLogView {
                namespace: namespace.to_owned(),
                name: name.to_owned(),
                container: container.to_owned(),
                tab_id: frontend_tab,
            },
        )
    }

    #[rhai_fn(return_raw)]
    pub fn pick_namespace(
        ctx: Arc<CallbackContext>,
        namespace: &str,
    ) -> Result<(), Box<rhai::EvalAltResult>> {
        let frontend_tab = ctx.frontend_tab.to_owned();

        emit(
            ctx,
            "hyprkube:menu:resource:pick_namespace",
            FrontendTriggerPickNamespace {
                namespace: namespace.to_owned(),
                tab_id: frontend_tab,
            },
        )
    }

    fn emit<T: Serialize + Clone>(
        ctx: Arc<CallbackContext>,
        event: &str,
        payload: T,
    ) -> Result<(), Box<rhai::EvalAltResult>> {
        ctx.app_handle
            .emit(event, payload)
            .map_err(|e| e.to_string().into())
    }
}
