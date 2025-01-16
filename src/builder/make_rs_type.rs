use std::{any::type_name_of_val, collections::HashMap, path::PathBuf};

use oxc_ast::ast::{TSLiteral, TSType};
use oxc_span::Span;
use serde::Serialize;

use crate::rs_types::*;
