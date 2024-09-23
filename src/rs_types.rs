use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub(crate) type RSTypeMap = HashMap<String, RSType>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum RSPrimitive {
    String,
    I32,
    I128,
    Bool,
    F64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RSEnum {
    pub(crate) variants: Vec<RSType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RSStruct {
    pub(crate) fields: HashMap<String, RSType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum EnumVariant {
    RSType(Box<RSType>),
    StringLiteral(String),
    BooleanLiteral(bool),
    NullLiteral,
    NumericLiteral(String),
    Unimplemented(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum RSType {
    Primitive(bool, RSPrimitive),
    Reference(bool, String),
    Enum(bool, RSEnum),
    Struct(bool, RSStruct),
    EnumVariant(bool, EnumVariant),
    Vec(bool, Box<RSType>),
    JSONValue(bool),
    Unit(bool),
    Unimplemented(String, String),
}
