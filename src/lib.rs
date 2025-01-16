#![allow(unused, dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use builder::{TypeScriptToRustBuilder, options::TypeScriptOptions};
use oxc_allocator::Allocator;
use oxc_ast::Visit;
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

mod builder;
mod hashable_set;
mod rs_types;
mod string_utils;
mod typescript_type_id;

mod tests;

pub fn run() -> Result<(), String> {
    // let entrypoint = PathBuf::from_str("examples/axe/axe-types.ts").expect("path");
    let entrypoint =
        PathBuf::from_str("examples/axe/node_modules/axe-core/axe.d.ts").expect("path");
    let options = TypeScriptOptions::default();
    let mut builder = TypeScriptToRustBuilder::new(options);
    builder.visit_module(&entrypoint);
    // builder.resolve_references();

    // let references = resolve_references(&mut visitor.type_map);

    // println!("RESOLVED:\n\n{:#?}", visitor.type_map);
    // println!("REFERENCES:\n\n");
    // for reference in references {
    //     println!("{:?}", reference);
    // }

    Ok(())
}
