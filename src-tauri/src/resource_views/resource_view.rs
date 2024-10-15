use kube::api::GroupVersionKind;
use rhai::{CustomType, TypeBuilder};
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
            .build_type::<ColoredString>()
            .build_type::<ColoredBox>()
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

    pub fn render_columns<T>(&self, obj: &T) -> Vec<Result<Vec<FrontendValue>, String>>
    where
        T: kube::Resource + Clone + Serialize,
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

                        vec![FrontendValue::PlainString(dyn_value.to_string())]
                    })
            })
            .collect()
    }
}

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColoredString {
    pub string: String,
    pub color: String,
}

impl ColoredString {
    pub fn new(string: String, color: String) -> Self {
        ColoredString { string, color }
    }

    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder.with_fn("ColoredString", |string, color| Self::new(string, color));
    }
}

#[derive(Clone, Serialize, CustomType)]
#[rhai_type(extra = Self::build_extra)]
pub struct ColoredBox {
    pub color: String,
}

impl ColoredBox {
    pub fn new(string: String) -> Self {
        ColoredBox { color: string }
    }

    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder.with_fn("ColoredBox", |string| Self::new(string));
    }
}
