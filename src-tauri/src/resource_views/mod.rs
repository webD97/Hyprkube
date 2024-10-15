use kube::api::GroupVersionKind;
use rhai::{CustomType, Dynamic, EvalAltResult, FnPtr, TypeBuilder};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ViewDefinitionError {
    #[error("the field `{0}` is not present")]
    MissingField(String),
    #[error("the field did not expect the type `{0}`")]
    IncorrectType(String),
    #[error("incorrect type for `{0}`, expected `{1}`")]
    TypeMismatch(String, String),
}

#[derive(Clone, CustomType, Debug)]
pub struct ColumnDefinion {
    pub title: String,
    pub accessor: FnPtr,
}

impl Into<Dynamic> for ColumnDefinion {
    fn into(self) -> Dynamic {
        Dynamic::from(self)
    }
}

impl TryFrom<rhai::Map> for ColumnDefinion {
    type Error = ViewDefinitionError;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        Ok(Self {
            title: value
                .get("title")
                .ok_or(ViewDefinitionError::MissingField(".title".into()))?
                .clone()
                .into_string()
                .map_err(|e| ViewDefinitionError::IncorrectType(e.into()))?,
            accessor: value
                .get("accessor")
                .ok_or(ViewDefinitionError::MissingField(".accessor".into()))?
                .clone()
                .try_cast::<FnPtr>()
                .ok_or(ViewDefinitionError::TypeMismatch(
                    ".accessor".into(),
                    "FnPtr".into(),
                ))?,
        })
    }
}

#[derive(Clone, CustomType, Debug)]
pub struct ResourceViewDefinition {
    pub name: String,
    pub match_api_version: String,
    pub match_kind: String,
    pub columns: Vec<ColumnDefinion>,
}

impl Into<Dynamic> for ResourceViewDefinition {
    fn into(self) -> Dynamic {
        Dynamic::from(self)
    }
}

impl TryFrom<rhai::Map> for ResourceViewDefinition {
    type Error = ViewDefinitionError;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value
                .get("name")
                .ok_or(ViewDefinitionError::MissingField(".matchKind".into()))?
                .clone()
                .into_string()
                .map_err(|e| ViewDefinitionError::IncorrectType(e.into()))?,
            match_kind: value
                .get("matchKind")
                .ok_or(ViewDefinitionError::MissingField(".matchKind".into()))?
                .clone()
                .into_string()
                .map_err(|e| ViewDefinitionError::IncorrectType(e.into()))?,
            match_api_version: value
                .get("matchApiVersion")
                .ok_or(ViewDefinitionError::MissingField(".matchApiVersion".into()))?
                .clone()
                .into_string()
                .map_err(|e| ViewDefinitionError::IncorrectType(e.into()))?,
            columns: value
                .get("columns")
                .ok_or(ViewDefinitionError::MissingField(".columns".into()))?
                .clone()
                .into_typed_array::<rhai::Map>()
                .map_err(|e| ViewDefinitionError::IncorrectType(e.into()))?
                .into_iter()
                .map(|v| -> Result<ColumnDefinion, ViewDefinitionError> { Ok(v.try_into())? })
                .collect::<Result<Vec<ColumnDefinion>, ViewDefinitionError>>()?,
        })
    }
}

pub struct ResourceView {
    engine: rhai::Engine,
    ast: rhai::AST,
    definition: ResourceViewDefinition,
}

#[derive(Debug, Error)]
pub enum ResourceViewError {
    #[error("error in view definition")]
    ViewDefinitionError(#[from] ViewDefinitionError),

    #[error("error in rhai script")]
    SyntaxError(#[from] rhai::ParseError),

    #[error("error in rhai script")]
    RuntimeError(#[from] Box<EvalAltResult>),
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

    pub fn render_columns<T>(&self, obj: &T) -> Vec<Result<String, String>>
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
                    .call::<String>(&self.engine, &self.ast, (obj_as_map.clone(),))
                    .map_err(|e| e.to_string())
            })
            .collect()
    }
}
