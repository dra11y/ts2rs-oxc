use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TypeScriptTypeId {
    pub module: PathBuf,
    pub name: String,
}
