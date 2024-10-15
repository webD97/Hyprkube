use kube::api::GroupVersionKind;
use serde::Serialize;
use thiserror::Error;

use crate::frontend_types::FrontendValue;

use super::resource_view_definition::{
    ColumnDefinion, InvalidViewDefinition, ResourceViewDefinition,
};

pub struct ResourceView {
    engine: rhai::Engine,
    ast: rhai::AST,
    definition: ResourceViewDefinition,
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

impl ResourceView {
    pub fn new(script: &str) -> Result<Self, ResourceViewError> {
        let mut engine = rhai::Engine::new();
        engine
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

    pub fn get_gvk(&self) -> Option<GroupVersionKind> {
        let (group, version) = self
            .definition
            .match_api_version
            .split_once("/")
            .or(Some(("", self.definition.match_api_version.as_str())))?;
        Some(GroupVersionKind {
            group: group.into(),
            version: version.into(),
            kind: self.definition.match_kind.clone(),
        })
    }

    pub fn render_titles(&self) -> Vec<String> {
        self.definition
            .columns
            .iter()
            .map(|column| column.title.clone())
            .collect()
    }

    pub fn render_columns<T>(&self, obj: &T) -> Vec<Result<FrontendValue, String>>
    where
        T: kube::Resource + Clone + Serialize + 'static,
    {
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
                            let dyn_value: Vec<rhai::Dynamic> = dyn_value.into_array().unwrap();

                            let (value, color) = match dyn_value.len() {
                                0 => ("".into(), "".into()),
                                1 => (dyn_value.get(0).unwrap().to_string(), "".into()),
                                _ => (
                                    dyn_value.get(0).unwrap().to_string(),
                                    dyn_value.get(1).unwrap().to_string(),
                                ),
                            };

                            return FrontendValue::ColoredString(value, color);
                        }

                        FrontendValue::PlainString(dyn_value.to_string())
                    })
            })
            .collect()
    }
}
