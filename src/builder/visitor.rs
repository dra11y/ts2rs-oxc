use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
};

use codegen::Scope;
use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::{Expression, TSAnyKeyword, TSType, TSTypeName, TSTypeReference};
use oxc_resolver::Resolver;

use crate::{hashable_set::HashableSet, rs_types::RSType};

use super::{make_rs_type, options::TypeScriptOptions};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OriginalName {
    /// import { OriginalName } from "source"
    /// import { OriginalName as local } from "source"
    /// export { OriginalName }
    /// export { OriginalName as renamed }
    /// export { OriginalName as renamed } from "source"
    Name(String),
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

pub struct TypeScriptToRustVisitor<'a> {
    /// The path to the current module.
    pub path: PathBuf,
    /// The resolver used to resolve import/export specifiers in this module.
    pub resolver: Resolver,
    /// The codegen scope used to generate Rust code.
    pub scope: codegen::Scope,
    /// The type map used to store the types defined in this module.
    pub types: HashMap<String, RSType>,
    /// The type mappings used to store the types imported/exported from other modules.
    pub type_mappings: HashMap<String, TypeMapping>,
    /// The source text of the current module (for debugging unimplemented types).
    pub source_text: String,
    /// The options used to configure the TypeScript to Rust conversion.
    pub options: TypeScriptOptions,
    pub(crate) allocator: &'a Allocator,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> TypeScriptToRustVisitor<'a> {
    pub fn resolve_module(&mut self, specifier: &str) -> PathBuf {
        println!("resolve_module: {:?}", specifier);
        let current_dir = self.path.parent().expect("get current module directory");
        let resolution = self
            .resolver
            .resolve(current_dir, specifier)
            .unwrap_or_else(|error| {
                let note = match error {
                    oxc_resolver::ResolveError::NotFound(_) |
                    oxc_resolver::ResolveError::MatchedAliasNotFound(_, _) |
                    oxc_resolver::ResolveError::PackageImportNotDefined(_, _) => {
                        "\n\nDid you run npm install in the source folder?\n\n"
                    },
                    _ => "",
                };
                panic!(
                    "ERROR: {error:?} in resolve_module(specifier: \"{specifier}\")\nfrom path: {current_dir:?}{note}"
                )
            });
        resolution.full_path()
    }

    pub fn new(
        path: PathBuf,
        resolver: Resolver,
        source_text: String,
        options: TypeScriptOptions,
        allocator: &'a Allocator,
    ) -> Self {
        Self {
            path,
            resolver,
            options,
            source_text,
            allocator,
            scope: Scope::new(),
            types: HashMap::new(),
            type_mappings: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub(crate) fn expression_to_ts_type(
        expr: &Expression<'a>,
        allocator: &'a Allocator,
    ) -> TSType<'a> {
        match expr {
            Expression::Identifier(ident) => TSType::TSTypeReference(oxc_allocator::Box::new_in(
                TSTypeReference {
                    type_name: TSTypeName::IdentifierReference(ident.clone_in(allocator)),
                    type_parameters: None,
                    span: ident.span,
                },
                allocator,
            )),
            _ => todo!(),
        }
    }

    pub(crate) fn ts_type_to_rs_type(ts_type: &TSType<'_>, source_text: &str) -> RSType {
        make_rs_type(ts_type, source_text)
    }
}
