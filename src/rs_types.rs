use std::{collections::HashMap, path::PathBuf};

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

impl RSPrimitive {
    #[allow(unused)]
    pub(crate) fn name(&self) -> String {
        match self {
            RSPrimitive::String => "String".to_string(),
            RSPrimitive::I32 => "i32".to_string(),
            RSPrimitive::I128 => "i128".to_string(),
            RSPrimitive::Bool => "bool".to_string(),
            RSPrimitive::F64 => "f64".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RSEnum {
    pub(crate) option: bool,
    pub(crate) variants: Vec<RSType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RSStruct {
    pub(crate) fields: HashMap<String, RSType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum RSEnumVariant {
    RSType(Box<RSType>),
    StringLiteral(String),
    BooleanLiteral(bool),
    NullLiteral,
    NumericLiteral(String),
    Unimplemented(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub(crate) enum RSReference {
    Unresolved {
        name: String,
        module_specifier: Option<String>,
    },
    Resolved {
        name: String,
        module_path: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum RSType {
    Primitive(RSPrimitive),
    Reference(RSReference),
    Enum(RSEnum),
    Struct(RSStruct),
    EnumVariant(RSEnumVariant),
    Vec(Box<RSType>),
    Option(Box<RSType>),
    JSONValue,
    NullOrUndefined,
    Unit,
    Unimplemented(String, String),
}

impl RSType {
    #[allow(unused)]
    pub(crate) fn name(&self) -> String {
        match self {
            RSType::Primitive(p) => p.name(),
            RSType::Reference(r) => format!("REF<{:?}>", r),
            RSType::Enum(e) => format!("{:?}", e),
            RSType::Struct(s) => format!("{:?}", s),
            RSType::EnumVariant(v) => format!("{:?}", v),
            RSType::Vec(v) => format!("Vec<{}>", v.name()),
            RSType::Option(o) => format!("Option<{}>", o.name()),
            RSType::JSONValue => "serde_json::Value".to_string(),
            RSType::NullOrUndefined => "Option<()>".to_string(),
            RSType::Unit => "()".to_string(),
            RSType::Unimplemented(t, n) => format!("Unimplemented<{}, {}>", t, n),
        }
    }
}
