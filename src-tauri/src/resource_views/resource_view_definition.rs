use rhai::{CustomType, TypeBuilder};

/// An error in the user-supplied object map that prevents it from being
/// used as a custom view
#[derive(Debug, thiserror::Error)]
pub enum InvalidViewDefinition {
    #[error("the required field `{0}` is not present")]
    MissingField(String),
    #[error("incorrect type for `{0}`, expected `{1}`")]
    TypeMismatch(&'static str, &'static str),
}

#[derive(Clone, rhai::CustomType, Debug)]
pub struct ResourceViewDefinition {
    pub name: String,
    pub match_api_version: String,
    pub match_kind: String,
    pub columns: Vec<ColumnDefinion>,
}

impl TryFrom<rhai::Map> for ResourceViewDefinition {
    type Error = InvalidViewDefinition;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value
                .get("name")
                .ok_or(InvalidViewDefinition::MissingField(".matchKind".into()))?
                .clone()
                .into_string()
                .map_err(|_e| InvalidViewDefinition::TypeMismatch(".name", "str"))?,
            match_kind: value
                .get("matchKind")
                .ok_or(InvalidViewDefinition::MissingField(".matchKind".into()))?
                .clone()
                .into_string()
                .map_err(|_e| InvalidViewDefinition::TypeMismatch(".matchKind", "str"))?,
            match_api_version: value
                .get("matchApiVersion")
                .ok_or(InvalidViewDefinition::MissingField(
                    ".matchApiVersion".into(),
                ))?
                .clone()
                .into_string()
                .map_err(|_e| InvalidViewDefinition::TypeMismatch(".matchApiVersion", "str"))?,
            columns: value
                .get("columns")
                .ok_or(InvalidViewDefinition::MissingField(".columns".into()))?
                .clone()
                .into_typed_array::<rhai::Map>()
                .map_err(|_e| InvalidViewDefinition::TypeMismatch(".columns", "map"))?
                .into_iter()
                .map(|v| -> Result<ColumnDefinion, InvalidViewDefinition> { Ok(v.try_into())? })
                .collect::<Result<Vec<ColumnDefinion>, InvalidViewDefinition>>()?,
        })
    }
}

#[derive(Clone, rhai::CustomType, Debug)]
pub struct ColumnDefinion {
    pub title: String,
    pub accessor: rhai::FnPtr,
}

impl TryFrom<rhai::Map> for ColumnDefinion {
    type Error = InvalidViewDefinition;

    fn try_from(value: rhai::Map) -> Result<Self, Self::Error> {
        Ok(Self {
            title: value
                .get("title")
                .ok_or(InvalidViewDefinition::MissingField(".title".into()))?
                .clone()
                .into_string()
                .map_err(|_e| InvalidViewDefinition::TypeMismatch(".title", "str"))?,
            accessor: value
                .get("accessor")
                .ok_or(InvalidViewDefinition::MissingField(".accessor".into()))?
                .clone()
                .try_cast::<rhai::FnPtr>()
                .ok_or(InvalidViewDefinition::TypeMismatch(".accessor", "FnPtr"))?,
        })
    }
}
