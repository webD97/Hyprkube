use async_trait::async_trait;
use kube::api::GroupVersionKind;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    frontend_types::FrontendValue,
    resource_rendering::{ColoredBox, ColoredString, RelativeTime},
};

use super::{
    resource_view_definition::{ColumnDefinion, InvalidViewDefinition, ResourceViewDefinition},
    Hyperlink, ResourceRenderer,
};

pub struct ScriptedResourceView {
    engine: rhai::Engine,
    ast: rhai::AST,
    pub definition: ResourceViewDefinition,
}

#[derive(Debug, Error)]
pub enum ResourceViewError {
    #[error("error in view definition")]
    ViewDefinitionError(#[from] InvalidViewDefinition),

    #[error("error in rhai script")]
    SyntaxError(#[from] rhai::ParseError),

    #[error("error in rhai script")]
    RuntimeError(#[from] Box<rhai::EvalAltResult>),
}

impl ScriptedResourceView {
    pub fn new(script: &str) -> Result<Self, ResourceViewError> {
        let mut engine = rhai::Engine::new();
        engine
            .build_type::<ColoredString>()
            .build_type::<ColoredBox>()
            .build_type::<Hyperlink>()
            .build_type::<RelativeTime>()
            .register_type_with_name::<ColumnDefinion>("Column")
            .register_type_with_name::<ResourceViewDefinition>("ResourceView");

        let engine = engine;
        let ast = engine.compile(script)?;
        let definition = engine.eval_ast::<rhai::Map>(&ast)?.try_into()?;

        Ok(Self {
            engine,
            ast,
            definition,
        })
    }

    pub fn display_name(&self) -> &str {
        self.definition.name.as_str()
    }

    pub fn render_titles(&self) -> Vec<String> {
        self.definition
            .columns
            .iter()
            .map(|column| column.title.clone())
            .collect()
    }

    pub fn render_columns(
        &self,
        obj: &kube::api::DynamicObject,
    ) -> Vec<Result<Vec<FrontendValue>, String>> {
        let obj_as_map: rhai::Dynamic =
            rhai::serde::to_dynamic(obj).expect("failed to convert Kubernetes resource to dynamic");

        self.definition
            .columns
            .iter()
            .map(|column| {
                column
                    .accessor
                    .call::<rhai::Dynamic>(&self.engine, &self.ast, (obj_as_map.clone(),))
                    .map_err(|e| e.to_string())
                    .map(|dyn_value| {
                        // Poor man's coloring: 0 -> value, 1 -> color
                        if dyn_value.is::<Vec<rhai::Dynamic>>() {
                            return dyn_value
                                .into_array()
                                .unwrap()
                                .iter()
                                .map(|value| {
                                    if value.is::<ColoredString>() {
                                        return FrontendValue::ColoredString(value.clone().cast());
                                    }

                                    if value.is::<ColoredBox>() {
                                        return FrontendValue::ColoredBox(value.clone().cast());
                                    }

                                    if value.is::<Hyperlink>() {
                                        return FrontendValue::Hyperlink(value.clone().cast());
                                    }

                                    if value.is::<RelativeTime>() {
                                        return FrontendValue::RelativeTime(value.clone().cast());
                                    }

                                    FrontendValue::PlainString(value.to_string())
                                })
                                .collect();
                        }

                        if dyn_value.is::<ColoredString>() {
                            return vec![FrontendValue::ColoredString(dyn_value.clone().cast())];
                        }

                        if dyn_value.is::<ColoredBox>() {
                            return vec![FrontendValue::ColoredBox(dyn_value.clone().cast())];
                        }

                        if dyn_value.is::<Hyperlink>() {
                            return vec![FrontendValue::Hyperlink(dyn_value.clone().cast())];
                        }

                        if dyn_value.is::<RelativeTime>() {
                            return vec![FrontendValue::RelativeTime(dyn_value.clone().cast())];
                        }

                        vec![FrontendValue::PlainString(dyn_value.to_string())]
                    })
            })
            .collect()
    }
}

#[async_trait]
impl ResourceRenderer for ScriptedResourceView {
    fn display_name(&self) -> &str {
        self.display_name()
    }

    async fn titles(
        &self,
        _app_handle: tauri::AppHandle,
        _client_id: &Uuid,
        _gvk: &GroupVersionKind,
    ) -> Vec<String> {
        self.render_titles()
    }

    async fn render(
        &self,
        _app_handle: tauri::AppHandle,
        _client_id: &Uuid,
        _gvk: &GroupVersionKind,
        obj: &kube::api::DynamicObject,
    ) -> Vec<Result<Vec<FrontendValue>, String>> {
        self.render_columns(obj)
    }
}
