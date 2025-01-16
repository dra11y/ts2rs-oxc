use std::{
    any::type_name_of_val,
    collections::{HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
};

use codegen::Scope;
use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::{Expression, TSAnyKeyword, TSLiteral, TSType, TSTypeName, TSTypeReference};
use oxc_resolver::Resolver;
use oxc_span::Span;
use serde::Serialize;

use crate::{
    hashable_set::HashableSet,
    rs_types::{RSEnum, RSEnumVariant, RSPrimitive, RSReference, RSType},
};

use super::{make_rs_type, options::TypeScriptOptions};

// /// Represents an imported/exported name and whether it was changed or not.
// #[derive(Clone, Debug, Hash, PartialEq, Eq)]
// pub enum OriginalName {
//     /// Import examples:
//     /// import { OriginalName } from "source"
//     /// import { OriginalName as LocalName } from "source"
//     ///
//     /// Export examples:
//     /// export { OriginalName as LocalName }
//     /// export { OriginalName as LocalName } from "source"
//     Renamed(String),
//     /// import { OriginalName } from "source"
//     /// export { OriginalName } // from current module
//     Unchanged,
//     /// import * as local_ns from "source"
//     Namespace,
// }

// #[derive(Clone, Debug, Hash, PartialEq, Eq)]
// pub struct TypeMapping {
//     /// The path to the module where the type is originally defined.
//     /// Some(PathBuf) if the type is imported or re-exported, and None if it's a local type.
//     pub original_module: Option<PathBuf>,
//     /// The original name of the type in its original module.
//     /// Some(String) for named imports/exports and None for default or namespace imports.
//     pub original_name: OriginalName,
//     /// The name used in the current module.
//     pub local_name: String,
//     /// The name under which the type is exported.
//     /// Defaults to local_name.
//     pub public_name: String,
// }

/// Visits and tracks types for a **single** TypeScript module.
pub struct TypeScriptToRustVisitor<'a> {
    /// The path to the current module.
    pub path: PathBuf,
    /// The resolver used to resolve import/export specifiers in this module.
    pub resolver: Resolver,
    /// The codegen scope used to generate Rust code.
    pub scope: codegen::Scope,
    /// The types defined in this module (key = actual TS type name).
    /// Types that reference imports from other modules are of type:
    /// [`RSReference`] ([`RSType::Reference`]).
    pub local_types: HashMap<String, RSType>,
    /// The source text of the current module (for debugging unimplemented types).
    pub source: String,
    /// The options used to configure the TypeScript to Rust conversion.
    pub options: TypeScriptOptions,
    pub(crate) allocator: &'a Allocator,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> TypeScriptToRustVisitor<'a> {
    pub fn resolve_module(&mut self, specifier: &str) -> PathBuf {
        println!("resolve_module: {:?}", specifier);
        let current_dir = self.path.parent().expect("get current module directory");
        let resolution = self
            .resolver
            .resolve(current_dir, specifier)
            .unwrap_or_else(|error| {
                let note = match error {
                    oxc_resolver::ResolveError::NotFound(_) |
                    oxc_resolver::ResolveError::MatchedAliasNotFound(_, _) |
                    oxc_resolver::ResolveError::PackageImportNotDefined(_, _) => {
                        "\n\nDid you run npm install in the source folder?\n\n"
                    },
                    _ => "",
                };
                panic!(
                    "ERROR: {error:?} in resolve_module(specifier: \"{specifier}\")\nfrom path: {current_dir:?}{note}"
                )
            });
        resolution.full_path()
    }

    pub fn new(
        path: PathBuf,
        resolver: Resolver,
        source_text: String,
        options: TypeScriptOptions,
        allocator: &'a Allocator,
    ) -> Self {
        Self {
            path,
            resolver,
            options,
            source: source_text,
            allocator,
            scope: Scope::new(),
            local_types: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub(crate) fn expression_to_ts_type(
        expr: &Expression<'a>,
        allocator: &'a Allocator,
    ) -> TSType<'a> {
        match expr {
            Expression::Identifier(ident) => TSType::TSTypeReference(oxc_allocator::Box::new_in(
                TSTypeReference {
                    type_name: TSTypeName::IdentifierReference(ident.clone_in(allocator)),
                    type_parameters: None,
                    span: ident.span,
                },
                allocator,
            )),
            _ => todo!(),
        }
    }

    pub fn make_rs_type(
        &self,
        ts_type: &TSType,
        // source: &str,
        // imported_types: &HashMap<String, (PathBuf, String)>,
    ) -> RSType {
        let rs_type = match ts_type {
            TSType::TSAnyKeyword(_) => RSType::JSONValue,
            TSType::TSBigIntKeyword(_) => RSType::Primitive(RSPrimitive::I128),
            TSType::TSBooleanKeyword(_) => RSType::Primitive(RSPrimitive::Bool),
            TSType::TSIntrinsicKeyword(value) => self.unimplemented_type(value, value.span),
            TSType::TSNeverKeyword(_) => RSType::Unit,
            TSType::TSNullKeyword(_) => RSType::NullOrUndefined,
            TSType::TSNumberKeyword(_) => RSType::Primitive(RSPrimitive::F64),
            TSType::TSObjectKeyword(_) => RSType::JSONValue,
            TSType::TSStringKeyword(_) => RSType::Primitive(RSPrimitive::String),
            TSType::TSSymbolKeyword(_) => RSType::Primitive(RSPrimitive::String),
            TSType::TSUndefinedKeyword(_) => RSType::NullOrUndefined,
            TSType::TSUnknownKeyword(_) => RSType::JSONValue,
            TSType::TSVoidKeyword(_) => RSType::Unit,
            TSType::TSArrayType(array) => {
                let element_type = self.make_rs_type(&array.element_type);
                RSType::Vec(Box::new(element_type))
            }
            TSType::TSConditionalType(value) => self.unimplemented_type(value, value.span),
            TSType::TSConstructorType(value) => self.unimplemented_type(value, value.span),
            TSType::TSFunctionType(value) => self.unimplemented_type(value, value.span),
            TSType::TSImportType(value) => self.unimplemented_type(value, value.span),
            TSType::TSIndexedAccessType(value) => self.unimplemented_type(value, value.span),
            TSType::TSInferType(value) => self.unimplemented_type(value, value.span),
            TSType::TSIntersectionType(value) => self.unimplemented_type(value, value.span),
            TSType::TSLiteralType(literal) => {
                let variant = match &literal.literal {
                    TSLiteral::BooleanLiteral(boolean) => RSEnumVariant::BoolLiteral(boolean.value),
                    TSLiteral::NullLiteral(_) => RSEnumVariant::NullLiteral,
                    TSLiteral::NumericLiteral(numeric) => RSEnumVariant::NumericLiteral(
                        numeric.value,
                        numeric.raw.clone().map(|r| r.to_string()),
                    ),
                    TSLiteral::BigIntLiteral(bigint) => {
                        RSEnumVariant::BigIntLiteral(bigint.raw.to_string(), bigint.base)
                    }
                    TSLiteral::RegExpLiteral(value) => {
                        self.unimplemented_variant(value, value.span)
                    }
                    TSLiteral::StringLiteral(string) => {
                        RSEnumVariant::StringLiteral(string.value.to_string())
                    }
                    TSLiteral::TemplateLiteral(value) => {
                        self.unimplemented_variant(value, value.span)
                    }
                    TSLiteral::UnaryExpression(value) => {
                        self.unimplemented_variant(value, value.span)
                    }
                };
                RSType::EnumVariant(variant)
            }
            TSType::TSMappedType(_) => RSType::JSONValue,
            TSType::TSNamedTupleMember(named_tuple_member) => {
                // TODO: Make union type! Needs testing!
                let element_ts_type = named_tuple_member.element_type.to_ts_type();
                let element_type = self.make_rs_type(element_ts_type);
                RSType::EnumVariant(RSEnumVariant::RSType(Box::new(element_type)))
            }
            TSType::TSQualifiedName(value) => self.unimplemented_type(value, value.span),
            TSType::TSTemplateLiteralType(value) => self.unimplemented_type(value, value.span),
            TSType::TSThisType(value) => self.unimplemented_type(value, value.span),
            TSType::TSTupleType(tuple) => {
                let types = tuple
                    .element_types
                    .iter()
                    .filter_map(|t| self.make_rs_type(t.as_ts_type()))
                    .collect::<Vec<RSType>>();
                RSType::Vec(Box::new(self.make_union_or_option_type(&types)))
            }
            TSType::TSTypeLiteral(lit) => {
                // unimplemented!("TSTypeLiteral: {:#?}", literal)
                // e.g. { [key: string]: string }
                RSType::JSONValue
            }
            TSType::TSTypeOperatorType(value) => self.unimplemented_type(value, value.span),
            TSType::TSTypePredicate(value) => self.unimplemented_type(value, value.span),
            TSType::TSTypeQuery(value) => self.unimplemented_type(value, value.span),
            TSType::TSTypeReference(reference) => {
                let base = RSType::Reference(RSReference {
                    local_name: reference.type_name.to_string(),
                    original_name: reference.type_name.to_string(),
                    module: self.path.clone(),
                });
                println!(
                    "------------------------\nTSTypeReference: base: {:#?}",
                    base
                );
                match &reference.type_parameters {
                    Some(params) => {
                        let params = params
                            .params
                            .iter()
                            .map(|ts_type| {
                                println!("param: {:#?}", self.make_rs_type(ts_type));
                                self.make_rs_type(ts_type)
                            })
                            .collect::<Vec<_>>();
                        RSType::ParameterizedType(Box::new(base), params)
                    }
                    None => base,
                }
            }
            TSType::TSUnionType(union) => {
                let members = union
                    .types
                    .iter()
                    .map(|t| self.make_rs_type(t))
                    .collect::<Vec<_>>();
                self.make_union_or_option_type(&members)
            }
            TSType::TSParenthesizedType(value) => {
                self.make_rs_type(value.type_annotation.without_parenthesized())
            }
            TSType::JSDocNullableType(value) => self.unimplemented_type(value, value.span),
            TSType::JSDocNonNullableType(value) => self.unimplemented_type(value, value.span),
            TSType::JSDocUnknownType(value) => self.unimplemented_type(value, value.span),
        };

        rs_type
    }

    fn make_rs_types(
        &self,
        types: impl Iterator<Item = &'a TSType<'a>>,
        source: &str,
    ) -> Vec<RSType> {
        types.map(|t| self.make_rs_type(t)).collect()
    }

    fn make_union_or_option_type(&self, types: &[RSType]) -> RSType {
        // println!("make_union_or_option_type {:#?}", types);
        let mut option = false;
        let variants: Vec<RSType> = types
            .iter()
            .filter_map(|t| match t {
                RSType::NullOrUndefined => {
                    option = true;
                    None
                }
                RSType::EnumVariant(RSEnumVariant::NullLiteral) => {
                    option = true;
                    None
                }
                RSType::Unit => None,
                _ => Some(t.clone()),
            })
            .collect();
        let rs_type = RSType::Enum(RSEnum { variants });
        if option {
            RSType::Option(Box::new(rs_type))
        } else {
            rs_type
        }
    }

    fn unimplemented_variant<T: Serialize>(&self, value: &T, span: Span) -> RSEnumVariant {
        RSEnumVariant::Unimplemented(
            self.extract_type_name(value),
            span.source_text(&self.source).to_string(),
        )
    }

    fn extract_type_name<T: Serialize>(&self, value: &T) -> String {
        type_name_of_val(value)
            .split("::")
            .last()
            .expect("extract_type_name")
            .replace('>', "")
            .to_string()
    }

    fn unimplemented_type<T: Serialize>(&self, value: &T, span: Span) -> RSType {
        RSType::Unimplemented(
            self.extract_type_name(value),
            span.source_text(&self.source).to_string(),
        )
    }
}
