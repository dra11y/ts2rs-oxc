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
    ast::{self, Program},
    Visit,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_span::SourceType;

mod errors;
mod make_rs_type;
pub(crate) mod options;
mod reference_resolver;
mod visitor;
mod visitor_impl;

use oxc_resolver::{ResolveOptions, Resolver};

use make_rs_type::*;
use reference_resolver::ReferenceResolver;
use visitor::TypeScriptToRustVisitor;

use crate::rs_types::{RSType, RSTypeMap};

#[derive(Debug, Default)]
pub(crate) struct TypeScriptToRustBuilder {
    options: TypeScriptOptions,
    modules: HashMap<PathBuf, RSTypeMap>,
}

impl TypeScriptToRustBuilder {
    pub fn new(options: TypeScriptOptions) -> Self {
        Self {
            options,
            ..Self::default()
        }
    }

    pub fn visit_module<R: AsRef<Path>>(&mut self, path: R) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref().canonicalize()?;

        // Skip module if already processed.
        if self.modules.contains_key(&path) {
            return Ok(());
        }

        self.modules.insert(path.clone(), RSTypeMap::new());

        println!("visit_module: {:?}", path);

        // Read and parse the module
        let source_text = fs::read_to_string(&path)?;
        let source_type = SourceType::from_path(&path)?;
        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, &source_text, source_type)
            .with_options(self.options.parse_options);
        let ret = parser.parse();

        // Create and use the visitor
        let resolver = Resolver::new(self.options.resolve_options.clone());
        let mut visitor = TypeScriptToRustVisitor::new(
            path.clone(),
            resolver,
            source_text.clone(),
            self.options.clone(),
        );

        visitor.visit_program(&ret.program);

        // Store the result
        let mut type_map = self.modules.get_mut(&path).unwrap();
        *type_map = visitor.types.clone();

        // // Recursively visit imported modules
        // for imported_module in visitor.get_imported_modules() {
        //     self.visit_module(&imported_module)?;
        // }

        Ok(())
    }
}
