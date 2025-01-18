use std::{collections::HashMap, path::PathBuf};

use oxc_ast::ast::BigintBase;
use serde::{Deserialize, Serialize, Serializer};

// pub type RSTypeMap = HashMap<String, RSType>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RSPrimitive {
    String,
    I32,
    I128,
    Bool,
    F64,
}

impl RSPrimitive {
    #[allow(unused)]
    pub fn name(&self) -> String {
        match self {
            RSPrimitive::String => "String".to_string(),
            RSPrimitive::I32 => "i32".to_string(),
            RSPrimitive::I128 => "i128".to_string(),
            RSPrimitive::Bool => "bool".to_string(),
            RSPrimitive::F64 => "f64".to_string(),
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub struct RSEnum {
//     pub variants: Vec<RSType>,
// }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RSStruct {
    pub fields: HashMap<String, RSType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RSEnumVariant {
    RSType(Box<RSType>),
    BoolLiteral(bool),
    NumericLiteral(f64, Option<String>),
    BigIntLiteral(String, #[serde(with = "bigint_base")] BigintBase),
    StringLiteral(String),
    NullLiteral,
    Unimplemented(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RSTupleElement {
    Type(Box<RSType>),
    OptionalType(Box<RSType>),
    RestType(Box<RSType>),
}

mod bigint_base {
    use oxc_ast::ast::BigintBase;
    use serde::{
        Deserialize as _, Serializer,
        de::{DeserializeOwned, Deserializer},
    };

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BigintBase, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "decimal" => BigintBase::Decimal,
            "binary" => BigintBase::Binary,
            "octal" => BigintBase::Octal,
            "hex" => BigintBase::Hex,
            _ => unreachable!(),
        })
    }

    pub fn serialize<S>(x: &BigintBase, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(match x {
            BigintBase::Decimal => "decimal",
            BigintBase::Binary => "binary",
            BigintBase::Octal => "octal",
            BigintBase::Hex => "hex",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct RSReference {
    pub local_name: String,
    pub original_name: String,
    pub module: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RSEnum {
    pub variants: Vec<RSType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RSType {
    Primitive(RSPrimitive),
    Reference(RSReference),
    Struct(RSStruct),
    Enum(RSEnum),
    EnumVariant(RSEnumVariant),
    Tuple(Vec<RSTupleElement>),
    Vec(Box<RSType>),
    Option(Box<RSType>),
    ParameterizedType(Box<RSType>, Vec<RSType>),
    JSONValue,
    NullOrUndefined,
    Unit,
    Unimplemented(String, String),
}

impl RSType {
    pub fn name(&self) -> String {
        match self {
            RSType::Primitive(p) => p.name(),
            RSType::Reference(r) => format!("RSReference<{}>", r.local_name),
            RSType::Enum(e) => format!("{:?}", e),
            RSType::EnumVariant(v) => format!("{:?}", v),
            RSType::Struct(s) => format!("{:?}", s),
            RSType::Tuple(t) => format!("{:?}", t),
            RSType::Vec(v) => format!("Vec<{}>", v.name()),
            RSType::Option(o) => format!("Option<{}>", o.name()),
            RSType::JSONValue => "serde_json::Value".to_string(),
            RSType::NullOrUndefined => "Option<()>".to_string(),
            RSType::Unit => "()".to_string(),
            RSType::Unimplemented(t, n) => format!("Unimplemented<{}, {}>", t, n),
            RSType::ParameterizedType(base, params) => todo!(),
        }
    }
}
