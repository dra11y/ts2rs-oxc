use std::collections::{HashMap, HashSet};

use convert_case::{Case, Casing};
use oxc_ast::{
    Visit,
    ast::{
        self, Expression, IdentifierReference, ImportDeclarationSpecifier, ImportDefaultSpecifier,
        ImportNamespaceSpecifier, ImportOrExportKind, ImportSpecifier, ModuleExportName,
        StringLiteral, TSType, TSTypeName,
    },
    visit::walk,
};
use oxc_span::{Atom, Span};
use oxc_syntax::scope::ScopeFlags;

use crate::{hashable_set::HashableSet, rs_types::*, string_utils::StringUtils};

use super::{TypeScriptToRustVisitor, make_rs_type};

impl<'a> Visit<'a> for TypeScriptToRustVisitor<'a> {
    fn visit_import_declaration(&mut self, it: &ast::ImportDeclaration<'a>) {
        let Some(specs) = &it.specifiers else {
            walk::walk_import_declaration(self, it);
            return;
        };

        let module = self.resolve_module(it.source.value.as_str());

        for spec in specs {
            let (imported_name, local_name) = match spec {
                ast::ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                    let local_name = spec.local.name.to_string();
                    let imported_name = match spec.imported.identifier_name() {
                        None => local_name.clone(),
                        Some(name) => name.to_string(),
                    };

                    // TODO: buggy: spec.import_kind doesn't resolve to [`ImportOrExportKind::Type`]

                    (imported_name, local_name)
                }
                ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                    todo!("handle ImportDefaultSpecifier");
                    // let local_name = spec.local.name.clone().into_string();
                    // (local_name, local_name)
                }
                ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                    todo!("handle ImportNamespaceSpecifier");
                    // let local_name = spec.local.name.clone().into_string();
                    // println!("import * as {} from {:?}", local_name, &module_specifier);
                    // (local_name, local_name)
                }
            };

            self.local_types.insert(
                local_name.clone(),
                RSType::Reference(RSReference {
                    local_name,
                    original_name: imported_name,
                    module: module.clone(),
                }),
            );
        }

        walk::walk_import_declaration(self, it);
    }

    fn visit_export_named_declaration(&mut self, it: &ast::ExportNamedDeclaration<'a>) {
        for spec in &it.specifiers {
            let exported_name = spec.exported.name().into_string();
            let local_name = spec.local.name().into_string();
            let module = it
                .source
                .clone()
                .map(|s| self.resolve_module(&s.value))
                .unwrap_or_else(|| self.path.clone());

            self.local_types.insert(
                exported_name.clone(),
                RSType::Reference(RSReference {
                    original_name: local_name.clone(),
                    local_name,
                    module: module.clone(),
                }),
            );
        }

        walk::walk_export_named_declaration(self, it);
    }

    fn visit_ts_type_alias_declaration(&mut self, it: &ast::TSTypeAliasDeclaration<'a>) {
        let type_name = it.id.name.to_string();
        let rs_type = self.make_rs_type(&it.type_annotation);
        self.local_types.insert(type_name, rs_type);
    }

    fn visit_ts_interface_declaration(&mut self, it: &ast::TSInterfaceDeclaration<'a>) {
        let interface_name = it.id.name.to_string();
        let mut fields: HashMap<String, RSType> = HashMap::new();

        // Handle extended interfaces
        if let Some(extends) = &it.extends {
            for heritage in extends {
                // Convert Expression to TSType using existing helper
                let ts_type = Self::expression_to_ts_type(&heritage.expression, self.allocator);
                let rs_type = self.make_rs_type(&ts_type);

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
            let rs_type = self.make_rs_type(ts_type);

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

        self.local_types
            .insert(interface_name, RSType::Struct(RSStruct { fields }));
    }
}
