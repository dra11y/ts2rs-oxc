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
use oxc_ast::ast::{self, Program, TSType};
use oxc_ast_visit::Visit as _;
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
    rs_types::{RSEnum, RSEnumVariant, RSReference, RSStruct, RSType},
    typescript_type_id::TypeScriptTypeId,
};
use reference_resolver::resolve_type;

/// The TypeScript to Rust builder that keeps track of
/// the TypeScript modules and their types across modules.
#[derive(Default)]
pub struct TypeScriptToRustBuilder {
    /// The options used to configure the TypeScript to Rust conversion.
    pub options: TypeScriptOptions,
    /// Visited modules
    pub modules: HashSet<PathBuf>,
    /// The TypeScript modules and their types.
    pub types: HashMap<TypeScriptTypeId, RSType>,
    allocator: Allocator,
}

impl TypeScriptToRustBuilder {
    pub fn new(options: TypeScriptOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    pub fn visit_entrypoints(&mut self) -> Result<(), Box<dyn Error>> {
        for entrypoint in &self.options.entrypoints.clone() {
            self.visit_module(entrypoint)?;
        }

        self.resolve_references();

        Ok(())
    }

    /// Visits a TypeScript module and its dependencies.
    fn visit_module<R: AsRef<Path>>(&mut self, path: R) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref().canonicalize()?;

        // Skip module if already processed.
        if self.modules.contains(&path) {
            return Ok(());
        }

        self.modules.insert(path.clone());

        // Read and parse the module
        let source_text = fs::read_to_string(&path)?;
        let source_type = SourceType::from_path(&path)?;
        let parser = Parser::new(&self.allocator, &source_text, source_type)
            .with_options(self.options.parse_options);
        let ret = parser.parse();

        // NOTHING USEFUL IN SEMANTICS?
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
        let module_types: HashMap<TypeScriptTypeId, RSType> = visitor
            .local_types
            .iter()
            .map(|t| {
                (
                    TypeScriptTypeId {
                        module: path.clone(),
                        name: t.0.clone(),
                    },
                    t.1.clone(),
                )
            })
            .collect();
        self.types.extend(module_types);
        Ok(())
    }

    fn resolve_references(&mut self) {
        let keys: Vec<_> = self.types.keys().cloned().collect();

        for id in keys {
            if let Some(rs_type) = self.types.get(&id).cloned() {
                let resolved_type = self.resolve_type(&rs_type);
                if rs_type != resolved_type {
                    if let Some(mut_ref_type) = self.types.get_mut(&id) {
                        *mut_ref_type = resolved_type;
                    }
                }
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn resolve_type(&self, rs_type: &RSType) -> RSType {
        match rs_type {
            RSType::Reference(reference) => match reference {
                RSReference::Resolved { .. } => rs_type.clone(),
                RSReference::Unresolved {
                    local_name,
                    original_name,
                    resolved_module,
                } => {
                    let id = TypeScriptTypeId {
                        module: resolved_module.clone(),
                        name: original_name.clone(),
                    };
                    let resolved_type = self.types.get(&id);

                    match resolved_type {
                        Some(rs_type) => RSType::Reference(RSReference::Resolved { id }),
                        None => {
                            //
                            panic!(
                                "Failed to resolve reference: {reference:#?} \n\n{:#?}",
                                self.types
                            )
                        }
                    }
                }
            },
            RSType::Vec(inner) => RSType::Vec(Box::new(self.resolve_type(inner))),
            RSType::Option(inner) => RSType::Option(Box::new(self.resolve_type(inner))),
            RSType::Enum(RSEnum { variants }) => {
                let variants = variants
                    .iter()
                    .map(|variant| self.resolve_type(variant))
                    .collect();
                RSType::Enum(RSEnum { variants })
            }
            RSType::Struct(RSStruct { fields }) => {
                let fields = fields
                    .iter()
                    .map(|(field_name, field_type)| {
                        (field_name.clone(), self.resolve_type(field_type))
                    })
                    .collect();
                RSType::Struct(RSStruct { fields })
            }
            RSType::EnumVariant(variant) => rs_type.clone(),
            // RSType::EnumVariant(variant) => {
            //     panic!("Unexpected in resolve_type: RSType::EnumVariant({variant:#?})")
            // }
            RSType::Primitive(_) => rs_type.clone(),
            RSType::Tuple(vec) => todo!(),
            RSType::ParameterizedType(rstype, vec) => {
                let resolved_type = self.resolve_type(rstype);
                let resolved_params = vec.iter().map(|inner| self.resolve_type(inner)).collect();
                RSType::ParameterizedType(Box::new(resolved_type), resolved_params)
            }
            RSType::JSONValue => rs_type.clone(),
            RSType::NullOrUndefined => rs_type.clone(),
            RSType::Unit => rs_type.clone(),
            RSType::Unimplemented(_, _) => rs_type.clone(),
        }
    }
}
