use std::collections::{HashMap, HashSet};

use convert_case::{Case, Casing};
use oxc_ast::{
    Visit,
    ast::{self, Expression, IdentifierReference, StringLiteral, TSType, TSTypeName},
    visit::walk,
};
use oxc_span::{Atom, Span};
use oxc_syntax::scope::ScopeFlags;

use crate::{
    builder::visitor::{OriginalName, TypeMapping},
    hashable_set::HashableSet,
    rs_types::*,
    string_utils::StringUtils,
};

use super::{TypeScriptToRustVisitor, make_rs_type};

impl<'a> Visit<'a> for TypeScriptToRustVisitor<'a> {
    fn visit_import_declaration(&mut self, it: &ast::ImportDeclaration<'a>) {
        let module_specifier = it.source.value.to_string();
        let module_path = self
            .resolver
            .resolve(
                self.path.parent().expect("get current dir"),
                &module_specifier,
            )
            .expect("resolve module");
        let module_path = module_path.full_path();

        if let Some(specs) = &it.specifiers {
            for spec in specs {
                let (imported_name, local_name) = match spec {
                    ast::ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        let imported_name = spec.imported.name().into_string();
                        let local_name = spec.local.name.clone().into_string();

                        println!(
                            "import ({:?}) {} as {} from {:?}",
                            spec.import_kind, imported_name, local_name, &module_specifier
                        );

                        (OriginalName::Name(imported_name), local_name)
                    }
                    ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        let local_name = spec.local.name.clone().into_string();

                        println!("import {} from {:?}", local_name, &module_specifier);

                        (OriginalName::Default, local_name)
                    }
                    ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        let local_name = spec.local.name.clone().into_string();

                        println!("import * as {} from {:?}", local_name, &module_specifier);

                        (OriginalName::Namespace, local_name)
                    }
                };

                let mapping = TypeMapping {
                    original_module: Some(
                        module_specifier.parse().expect("invalid module specifier"),
                    ),
                    original_name: imported_name,
                    local_name: local_name.clone(),
                    public_name: local_name.clone(),
                };

                self.type_mappings.insert(local_name.clone(), mapping);
            }
        }
        walk::walk_import_declaration(self, it);
    }

    fn visit_export_named_declaration(&mut self, it: &ast::ExportNamedDeclaration<'a>) {
        for spec in &it.specifiers {
            let exported_name = spec.exported.name().into_string();
            let local_name = spec.local.name().into_string();
            let module_specifier = it.source.clone().map(|s| self.resolve_module(&s.value));

            println!(
                "export {} as {} from {:?}",
                exported_name, local_name, &module_specifier
            );

            let mapping = TypeMapping {
                original_module: module_specifier,
                original_name: OriginalName::Name(local_name.clone()),
                local_name: local_name.clone(),
                public_name: exported_name,
            };

            self.type_mappings.insert(local_name, mapping);
        }

        walk::walk_export_named_declaration(self, it);
    }

    fn visit_ts_type_alias_declaration(&mut self, it: &ast::TSTypeAliasDeclaration<'a>) {
        let type_name = it.id.name.to_string();
        let rs_type = make_rs_type(&it.type_annotation, &self.source_text);
        // println!("\nTYPE: {}: {:#?}", type_name, rs_type);
        self.types.insert(type_name, rs_type);
    }

    fn visit_ts_interface_declaration(&mut self, it: &ast::TSInterfaceDeclaration<'a>) {
        let interface_name = it.id.name.to_string();
        // println!("\nINTERFACE: {}", &interface_name);
        let mut fields: HashMap<String, RSType> = HashMap::new();

        // Handle extended interfaces
        if let Some(extends) = &it.extends {
            println!("{interface_name} extends:");
            for heritage in extends {
                // Convert Expression to TSType using existing helper
                let ts_type = Self::expression_to_ts_type(&heritage.expression, self.allocator);
                let rs_type = Self::ts_type_to_rs_type(&ts_type, &self.source_text);
                println!("HERITAGE base_type: {:?}", rs_type);

                if let RSType::Struct(base_struct) = rs_type {
                    fields.extend(base_struct.fields);
                }
            }
        }

        // Add interface's own fields
        for member in &it.body.body {
            let ast::TSSignature::TSPropertySignature(property) = member else {
                continue;
            };
            if property.computed {
                continue;
            }
            let Some(field_name) = property.key.name() else {
                continue;
            };
            let ts_type = match &property.type_annotation {
                Some(type_annotation) => &type_annotation.type_annotation,
                None => continue,
            };
            let rs_type = make_rs_type(ts_type, &self.source_text);
            let rs_type = match rs_type {
                RSType::Option(inner) => RSType::Option(inner),
                RSType::Enum(rs_enum) => match property.optional || rs_enum.option {
                    true => RSType::Option(Box::new(RSType::Enum(rs_enum))),
                    false => RSType::Enum(rs_enum),
                },
                _ => match property.optional {
                    true => RSType::Option(Box::new(rs_type)),
                    false => rs_type,
                },
            };

            if self.options.ignore_unimplemented {
                if let RSType::Unimplemented(_, _) = rs_type {
                    continue;
                }
            }

            fields.insert(field_name.to_string(), rs_type);
        }

        if fields.is_empty() {
            return;
        }

        self.types
            .insert(interface_name, RSType::Struct(RSStruct { fields }));
    }
}
