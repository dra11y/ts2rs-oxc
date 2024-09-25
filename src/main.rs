#![allow(unused, dead_code)]

use std::{fs, path::Path};

use builder::{options::TypeScriptOptions, TypeScriptToRustBuilder};
use oxc_allocator::Allocator;
use oxc_ast::Visit;
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_span::SourceType;

mod builder;
mod hashable_set;
mod rs_types;
mod string_utils;

fn main() -> Result<(), String> {
    let entrypoint = String::from("examples/axe/axe-types.ts");
    let path = Path::new(&entrypoint).canonicalize().unwrap();

    let options = TypeScriptOptions::default();
    let mut builder = TypeScriptToRustBuilder::new(options);
    builder.visit_module(path);

    // let references = resolve_references(&mut visitor.type_map);

    // println!("RESOLVED:\n\n{:#?}", visitor.type_map);
    // println!("REFERENCES:\n\n");
    // for reference in references {
    //     println!("{:?}", reference);
    // }

    Ok(())
}
