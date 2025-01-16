use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TypeScriptTypeId {
    pub module: PathBuf,
    pub name: String,
}
