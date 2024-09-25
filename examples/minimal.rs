use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::{ast, visit::walk, Visit};
use oxc_parser::{ParseOptions, Parser};
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_span::SourceType;

struct TypeScriptToRustVisitor;

impl<'a> Visit<'a> for TypeScriptToRustVisitor {
    fn visit_export_named_declaration(&mut self, it: &ast::ExportNamedDeclaration<'a>) {
        for spec in &it.specifiers {
            let exported_name = spec.exported.name().into_string();
            let export_kind = spec.export_kind;
            let local_name = spec.local.name().into_string();

            println!(
                "Found export: {:?} {} as {} from {:?}",
                export_kind, exported_name, local_name, it.source
            );

            assert_eq!(export_kind, ast::ImportOrExportKind::Type);
        }

        walk::walk_export_named_declaration(self, it);
    }

    fn visit_import_declaration(&mut self, it: &ast::ImportDeclaration<'a>) {
        let Some(specifiers) = &it.specifiers else {
            return;
        };

        for specifier in specifiers {
            if let ast::ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier {
                let imported_name = spec.imported.name().into_string();
                let import_kind = spec.import_kind;
                let local_name = spec.local.name.clone().into_string();
                let module_source = it.source.value.clone().into_string();

                println!(
                    "Found import: {:?} {} as {} from {}",
                    import_kind, imported_name, local_name, module_source
                );

                assert_eq!(import_kind, ast::ImportOrExportKind::Type);
            }
        }
        walk::walk_import_declaration(self, it);
    }
}

fn main() -> Result<(), String> {
    let entrypoint = String::from("examples/axe/axe-types.ts");

    let path = Path::new(&entrypoint).canonicalize().unwrap();
    assert!(&path.is_absolute(), "{path:?} must be an absolute path.");
    let dir = path
        .parent()
        .expect("Failed to get parent directory of path");
    let file = path.file_name().expect("Failed to get file name of path");

    let resolve_options = ResolveOptions {
        extensions: vec![".d.ts".into(), ".ts".into()],
        ..ResolveOptions::default()
    };

    let specifier = &format!("./{}", file.to_string_lossy());

    let resolver = Resolver::new(resolve_options);
    let resolution = resolver.resolve(dir, specifier).expect("resolve");
    println!("resolution: {:#?}", resolution);

    let path = resolution.full_path();

    let source_text =
        fs::read_to_string(&path).map_err(|_| format!("Not found: '{entrypoint}'"))?;
    let source_type = SourceType::from_path(&path).unwrap();

    let allocator = Allocator::default();
    let parser = Parser::new(&allocator, &source_text, source_type).with_options(ParseOptions {
        parse_regular_expression: false,
        preserve_parens: false,
        ..ParseOptions::default()
    });

    let ret = parser.parse();

    let mut visitor = TypeScriptToRustVisitor;
    visitor.visit_program(&ret.program);

    Ok(())
}
