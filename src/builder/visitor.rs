use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
};

use codegen::Scope;
use oxc_resolver::Resolver;

use crate::{hashable_set::HashableSet, rs_types::RSTypeMap};

use super::options::TypeScriptOptions;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OriginalName {
    /// import { OriginalName } from "source"
    /// import { OriginalName as local } from "source"
    /// export { OriginalName }
    /// export { OriginalName as renamed }
    /// export { OriginalName as renamed } from "source"
    Named(String),
    /// import local from "source"
    Default,
    /// import * as local from "source"
    Namespace,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TypeMapping {
    /// The path to the module where the type is originally defined.
    /// Some(PathBuf) if the type is imported or re-exported, and None if it's a local type.
    pub original_module: Option<PathBuf>,
    /// The original name of the type in its original module.
    /// Some(String) for named imports/exports and None for default or namespace imports.
    pub original_name: OriginalName,
    /// The name used in the current module.
    pub local_name: String,
    /// The name under which the type is exported.
    /// Defaults to local_name.
    pub public_name: String,
}

pub(crate) struct TypeScriptToRustVisitor {
    /// The path to the current module.
    pub(super) path: PathBuf,
    /// The resolver used to resolve import/export specifiers in this module.
    pub(super) resolver: Resolver,
    /// The codegen scope used to generate Rust code.
    pub(super) scope: codegen::Scope,
    /// The type map used to store the types defined in this module.
    pub(super) types: RSTypeMap,
    /// The type mappings used to store the types imported/exported from other modules.
    pub(super) type_mappings: HashMap<String, TypeMapping>,
    /// The source text of the current module (for debugging unimplemented types).
    pub(super) source_text: String,
    /// The options used to configure the TypeScript to Rust conversion.
    pub(super) options: TypeScriptOptions,
}

impl TypeScriptToRustVisitor {
    pub(super) fn resolve_module(&mut self, specifier: &str) -> PathBuf {
        println!("resolve_module: {:?}", specifier);
        let current_dir = self
            .path
            .parent()
            .expect("Failed to get current module directory");
        let resolution = self
            .resolver
            .resolve(current_dir, specifier)
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to resolve module specifier: {} from path: {:?}",
                    specifier, current_dir
                )
            });
        resolution.full_path()
    }

    pub fn new(
        path: PathBuf,
        resolver: Resolver,
        source_text: String,
        options: TypeScriptOptions,
    ) -> Self {
        Self {
            path,
            resolver,
            options,
            source_text,
            ..Self::default()
        }
    }
}

impl Default for TypeScriptToRustVisitor {
    fn default() -> Self {
        Self {
            path: PathBuf::default(),
            resolver: Resolver::default(),
            scope: Scope::new(),
            types: RSTypeMap::default(),
            type_mappings: HashMap::default(),
            source_text: String::default(),
            options: TypeScriptOptions::default(),
        }
    }
}
