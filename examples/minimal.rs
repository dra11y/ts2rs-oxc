use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::{Visit, ast, visit::walk};
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
            let module_source = it.source.clone().map(|s| s.value.to_string());

            println!(
                "Found export: {export_kind:?} {exported_name} as {local_name} from {module_source:?}",
            );
        }

        walk::walk_export_named_declaration(self, it);
    }

    fn visit_import_declaration(&mut self, it: &ast::ImportDeclaration<'a>) {
        let Some(specifiers) = &it.specifiers else {
            walk::walk_import_declaration(self, it);
            return;
        };
        for specifier in specifiers {
            if let ast::ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier {
                let imported_name = spec.imported.name().into_string();
                let import_kind = spec.import_kind;
                let local_name = spec.local.name.clone().into_string();
                let module_source = it.source.value.clone().into_string();

                println!(
                    "Found import: {import_kind:?} {imported_name} as {local_name} from {module_source:?}",
                );

                // assert_eq!(import_kind, ast::ImportOrExportKind::Type);
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

    let resolve_options: ResolveOptions = ResolveOptions {
        extensions: vec![".d.ts".into(), ".ts".into()],
        ..ResolveOptions::default()
    };

    let specifier = &format!("./{}", file.to_string_lossy());

    let resolver: oxc_resolver::ResolverGeneric<oxc_resolver::FileSystemOs> =
        Resolver::new(resolve_options);
    let resolution: oxc_resolver::Resolution = resolver.resolve(dir, specifier).expect("resolve");
    println!("resolution: {:#?}", resolution);

    let module_path: std::path::PathBuf = resolution.full_path();

    let source_text: String = fs::read_to_string(&module_path)
        .map_err(|err| format!("{err:?} Not found?: '{entrypoint}'"))?;
    let source_type: SourceType = SourceType::from_path(&module_path).expect("source type");

    let allocator: Allocator = Allocator::default();
    let parser: Parser<'_> =
        Parser::new(&allocator, &source_text, source_type).with_options(ParseOptions {
            parse_regular_expression: false,
            preserve_parens: false,
            ..ParseOptions::default()
        });

    let ret: oxc_parser::ParserReturn<'_> = parser.parse();
    let mut visitor: TypeScriptToRustVisitor = TypeScriptToRustVisitor;
    visitor.visit_program(&ret.program);

    Ok(())
}
