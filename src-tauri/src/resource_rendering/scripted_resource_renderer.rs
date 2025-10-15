use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::GroupVersionKind;
use thiserror::Error;

use crate::{
    frontend_types::BackendError,
    resource_rendering::scripting::types::{
        ColoredBox, ColoredBoxes, Hyperlink, RelativeTime, ResourceViewField, Text,
    },
};

use super::{
    resource_view_definition::{ColumnDefinion, InvalidViewDefinition, ResourceViewDefinition},
    ResourceRenderer,
};

pub struct ScriptedResourceView {
    engine: rhai::Engine,
    ast: rhai::AST,
    pub definition: ResourceViewDefinition,
}

#[derive(Debug, Error)]
pub enum ResourceViewError {
    #[error("error in view definition")]
    ViewDefinition(#[from] InvalidViewDefinition),

    #[error("error in rhai script")]
    Syntax(#[from] rhai::ParseError),

    #[error("error in rhai script")]
    Runtime(#[from] Box<rhai::EvalAltResult>),
}

impl ScriptedResourceView {
    pub fn new(script: &str) -> Result<Self, ResourceViewError> {
        let mut engine = rhai::Engine::new();
        engine
            .build_type::<Text>()
            .build_type::<Hyperlink>()
            .build_type::<RelativeTime>()
            .build_type::<ColoredBox>()
            .build_type::<ColoredBoxes>()
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

    pub fn column_definitions(&self) -> Vec<super::ResourceColumnDefinition> {
        self.definition
            .columns
            .iter()
            .map(|column| super::ResourceColumnDefinition {
                title: column.title.clone(),
                filterable: column.filterable,
            })
            .collect()
    }

    pub fn render_columns(
        &self,
        obj: &kube::api::DynamicObject,
    ) -> Vec<Result<ResourceViewField, String>> {
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
                    .map(|value| {
                        if value.is::<Text>() {
                            return ResourceViewField::Text(value.cast::<Text>());
                        }

                        if value.is::<Hyperlink>() {
                            return ResourceViewField::Hyperlink(value.cast::<Hyperlink>());
                        }

                        if value.is::<RelativeTime>() {
                            return ResourceViewField::RelativeTime(value.cast::<RelativeTime>());
                        }

                        if value.is::<ColoredBoxes>() {
                            return ResourceViewField::ColoredBoxes(value.cast::<ColoredBoxes>());
                        }

                        ResourceViewField::Text(Text {
                            content: value.to_string(),
                            properties: None,
                        })
                    })
            })
            .collect()
    }
}

impl ResourceRenderer for ScriptedResourceView {
    fn display_name(&self) -> &str {
        self.display_name()
    }

    fn column_definitions(
        &self,
        _gvk: &GroupVersionKind,
        _crd: Option<&CustomResourceDefinition>,
    ) -> Result<Vec<super::ResourceColumnDefinition>, BackendError> {
        Ok(self.column_definitions())
    }

    fn render(
        &self,
        _gvk: &GroupVersionKind,
        _crd: Option<&CustomResourceDefinition>,
        obj: &kube::api::DynamicObject,
    ) -> Result<Vec<Result<ResourceViewField, String>>, BackendError> {
        Ok(self.render_columns(obj))
    }
}
