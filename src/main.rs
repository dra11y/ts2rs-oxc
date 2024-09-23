use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::Visit;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;
use visitor::{resolve_references, TypeScriptToRustVisitor};

mod capitalize;
mod hashable_set;
mod rs_types;
mod visitor;

fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();

    let name = args
        .subcommand()
        .ok()
        .flatten()
        .unwrap_or_else(|| String::from("examples/axe.d.ts"));

    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Not found: '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, &source_text, source_type).with_options(ParseOptions {
        parse_regular_expression: false,
        preserve_parens: false,
        ..ParseOptions::default()
    });

    let ret = parser.parse();

    let mut visitor = TypeScriptToRustVisitor::new(source_text.clone());
    visitor.visit_program(&ret.program);

    resolve_references(&mut visitor.types);

    println!("RESOLVED:\n\n{:#?}", visitor.types);

    Ok(())
}
