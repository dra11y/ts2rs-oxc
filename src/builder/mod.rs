use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{Debug, Display},
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use codegen::Scope;
use errors::DiagnosticsError;
use itertools::Itertools;
use lazy_static::lazy_static;
use options::TypeScriptOptions;
use oxc_allocator::Allocator;
use oxc_ast::{
    Visit,
    ast::{self, Program, TSType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_semantic::{SemanticBuilder, Stats};
use oxc_span::SourceType;

mod errors;
mod make_rs_type;
pub mod options;
mod reference_resolver;
mod visitor;
mod visitor_impl;

use oxc_resolver::{ResolveContext, ResolveOptions, Resolver};

use make_rs_type::*;
use reference_resolver::ReferenceResolver;
use visitor::TypeScriptToRustVisitor;

use crate::{
    rs_types::{RSReference, RSType},
    typescript_type_id::TypeScriptTypeId,
};
use reference_resolver::resolve_type;

#[derive(Debug)]
pub enum ConversionError {
    UnsupportedType(String),
}

/// The TypeScript to Rust builder that keeps track of
/// the TypeScript modules and their types across modules.
#[derive(Default)]
pub struct TypeScriptToRustBuilder {
    /// The options used to configure the TypeScript to Rust conversion.
    options: TypeScriptOptions,
    /// Visited modules
    modules: HashSet<PathBuf>,
    /// The TypeScript modules and their types.
    types: HashMap<TypeScriptTypeId, RSType>,
    allocator: Allocator,
}

impl TypeScriptToRustBuilder {
    pub fn new(options: TypeScriptOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    /// Visits a TypeScript module and its dependencies.
    pub fn visit_module<R: AsRef<Path>>(&mut self, path: R) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref().canonicalize()?;

        // // Skip module if already processed.
        // if self.types.contains_key(&path) {
        //     return Ok(());
        // }

        // println!("visit_module: {:?}", path);

        // self.types.insert(path.clone(), HashMap::new());

        // Read and parse the module
        let source_text = fs::read_to_string(&path)?;
        let source_type = SourceType::from_path(&path)?;
        let parser = Parser::new(&self.allocator, &source_text, source_type)
            .with_options(self.options.parse_options);
        let ret = parser.parse();

        // NOTHING USEFUL?
        // let semantic = SemanticBuilder::new()
        //     .with_cfg(false)
        //     .with_build_jsdoc(false)
        //     .with_check_syntax_error(false)
        //     .with_scope_tree_child_ids(true)
        //     .build(&ret.program);

        // Create and use the visitor
        let resolver = Resolver::new(self.options.resolve_options.clone());
        let mut visitor = TypeScriptToRustVisitor::new(
            path.clone(),
            resolver,
            source_text.clone(),
            self.options.clone(),
            &self.allocator,
        );

        visitor.visit_program(&ret.program);

        // Store the result
        // let mut type_map = self.types.get_mut(&path).unwrap();
        // *type_map = visitor.types.clone();

        // // Resolve dependencies
        // for mapping in visitor.local_types.values() {
        //     if let Some(original_module_path) = &mapping.original_module {
        //         // Recursively visit the original module
        //         self.visit_module(original_module_path)?;
        //     }
        // }

        // After all modules have been visited, resolve type references
        // self.resolve_type_references(&path)?;

        Ok(())
    }

    // pub fn get_types(&self) -> &HashMap<PathBuf, HashMap<String, RSType>> {
    //     &self.types
    // }

    // pub fn resolve_references(&mut self) {
    //     let mut references: HashSet<RSReference> = HashSet::new();

    //     for (module_path, types) in self.modules.iter_mut() {
    //         for (_type_name, rs_type) in types.iter_mut() {
    //             let resolved_type = resolve_type(rs_type, types, &mut references);
    //             *rs_type = resolved_type;
    //         }
    //     }
    // }

    /// Resolves module specifiers to absolute paths.
    fn resolve_module(&self, specifier: &str) -> PathBuf {
        // Implementation for resolving module paths
        // This could use the resolver from `TypeScriptToRustVisitor` or another mechanism
        PathBuf::from(specifier) // Placeholder implementation
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TypeId {
    module_path: PathBuf,
    name: String,
}

#[derive(Debug, Clone)]
pub struct TypeReference<'a> {
    type_id: TypeId,
    type_parameters: Vec<&'a TSType<'a>>,
}

#[derive(Debug)]
pub struct TypeRegistry<'a> {
    types: HashMap<TypeId, RSType>,
    unresolved_references: HashMap<TypeId, Vec<TypeReference<'a>>>,
}

impl<'a> TypeRegistry<'a> {
    pub fn convert_type(
        &mut self,
        ts_type: &'a TSType<'a>,
        current_module: &Path,
    ) -> Result<RSType, ConversionError> {
        match ts_type {
            TSType::TSTypeReference(reference) => {
                let type_id = TypeId {
                    module_path: current_module.to_path_buf(),
                    name: reference.type_name.to_string(),
                };

                if let Some(resolved) = self.types.get(&type_id) {
                    Ok(resolved.clone())
                } else {
                    let type_ref = TypeReference {
                        type_id: type_id.clone(),
                        type_parameters: reference
                            .type_parameters
                            .as_ref()
                            .map(|params| params.params.iter().collect())
                            .unwrap_or_default(),
                    };

                    self.unresolved_references
                        .entry(type_id.clone())
                        .or_default()
                        .push(type_ref.clone());

                    todo!()
                    // Ok(RSType::Reference(RSReference::Unresolved {
                    //     local_name: type_id.name,
                    //     module_specifier: Some(type_id.module_path.to_string_lossy().into()),
                    // }))
                }
            }
            // ... other type conversions ...
            _ => Err(ConversionError::UnsupportedType(format!("{:?}", ts_type))),
        }
    }
}
