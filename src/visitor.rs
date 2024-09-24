use std::{any::type_name_of_val, collections::HashMap};

use codegen::Scope;
use oxc_ast::{
    ast::{TSArrayType, TSLiteral, TSSignature, TSType},
    Visit,
};
use oxc_span::Span;
use serde::Serialize;

use crate::rs_types::{EnumVariant, RSEnum, RSPrimitive, RSStruct, RSType, RSTypeMap};

#[derive(Debug)]
pub(crate) struct TypeScriptToRustVisitor {
    #[allow(unused)]
    pub(crate) scope: Scope,
    pub(crate) type_map: RSTypeMap,
    pub(crate) source_text: String,
}

impl Default for TypeScriptToRustVisitor {
    fn default() -> Self {
        Self {
            scope: Scope::new(),
            type_map: HashMap::new(),
            source_text: String::new(),
        }
    }
}

impl TypeScriptToRustVisitor {
    pub fn new(source_text: String) -> Self {
        Self {
            source_text,
            ..Default::default()
        }
    }

    #[allow(unused)]
    pub fn new_with_scope(scope: Scope, source_text: String) -> Self {
        Self {
            scope,
            source_text,
            ..Default::default()
        }
    }
}

/// Resolves all types referenced to look up later.
pub(crate) fn resolve_references(types: &mut RSTypeMap) {
    let keys: Vec<_> = types.keys().cloned().collect();

    for name in keys {
        if let Some(RSType::Reference(reference)) = types.get(&name) {
            if let Some(referenced_type) = types.get(reference).cloned() {
                if let Some(rs_type) = types.get_mut(&name) {
                    *rs_type = referenced_type;
                }
            } else {
                println!("Reference not found: {}", reference);
            }
        }
    }
}

fn extract_type_name<T: Serialize>(value: &T) -> String {
    type_name_of_val(value)
        .split("::")
        .last()
        .unwrap()
        .replace('>', "")
        .to_string()
}

fn unimplemented_variant<T: Serialize>(value: &T, span: Span, source: &str) -> EnumVariant {
    EnumVariant::Unimplemented(
        extract_type_name(value),
        span.source_text(source).to_string(),
    )
}

fn unimplemented_type<T: Serialize>(value: &T, span: Span, source: &str) -> RSType {
    RSType::Unimplemented(
        extract_type_name(value),
        span.source_text(source).to_string(),
    )
}

fn make_rs_type(ts_type: &TSType, source: &str) -> RSType {
    let rs_type = match ts_type {
        TSType::TSAnyKeyword(_) => RSType::JSONValue,
        TSType::TSBigIntKeyword(_) => RSType::Primitive(RSPrimitive::I128),
        TSType::TSBooleanKeyword(_) => RSType::Primitive(RSPrimitive::Bool),
        TSType::TSIntrinsicKeyword(value) => unimplemented_type(value, value.span, source),
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
            let element_type = make_rs_type(&array.element_type, source);
            RSType::Vec(Box::new(element_type))
        }
        TSType::TSConditionalType(value) => unimplemented_type(value, value.span, source),
        TSType::TSConstructorType(value) => unimplemented_type(value, value.span, source),
        TSType::TSFunctionType(value) => unimplemented_type(value, value.span, source),
        TSType::TSImportType(value) => unimplemented_type(value, value.span, source),
        TSType::TSIndexedAccessType(value) => unimplemented_type(value, value.span, source),
        TSType::TSInferType(value) => unimplemented_type(value, value.span, source),
        TSType::TSIntersectionType(value) => unimplemented_type(value, value.span, source),
        TSType::TSLiteralType(literal) => {
            let variant = match &literal.literal {
                TSLiteral::BooleanLiteral(boolean) => EnumVariant::BooleanLiteral(boolean.value),
                TSLiteral::NullLiteral(_) => EnumVariant::NullLiteral,
                TSLiteral::NumericLiteral(numeric) => {
                    EnumVariant::NumericLiteral(numeric.raw.into())
                }
                TSLiteral::BigIntLiteral(bigint) => {
                    EnumVariant::NumericLiteral(bigint.raw.clone().into_string())
                }
                TSLiteral::RegExpLiteral(value) => unimplemented_variant(value, value.span, source),
                TSLiteral::StringLiteral(string) => {
                    EnumVariant::StringLiteral(string.value.clone().into_string())
                }
                TSLiteral::TemplateLiteral(value) => {
                    unimplemented_variant(value, value.span, source)
                }
                TSLiteral::UnaryExpression(value) => {
                    unimplemented_variant(value, value.span, source)
                }
            };
            RSType::EnumVariant(variant)
        }
        TSType::TSMappedType(_) => RSType::JSONValue,
        TSType::TSNamedTupleMember(named_tuple_member) => {
            // TODO: Make union type! Needs testing!
            let element_ts_type = named_tuple_member.element_type.to_ts_type();
            let element_type = make_rs_type(element_ts_type, source);
            RSType::EnumVariant(EnumVariant::RSType(Box::new(element_type)))
        }
        TSType::TSQualifiedName(value) => unimplemented_type(value, value.span, source),
        TSType::TSTemplateLiteralType(value) => unimplemented_type(value, value.span, source),
        TSType::TSThisType(value) => unimplemented_type(value, value.span, source),
        TSType::TSTupleType(tuple) => {
            let variants: Vec<RSType> = make_rs_types(
                tuple.element_types.iter().filter_map(|t| {
                    if t.is_ts_type() {
                        Some(t.to_ts_type())
                    } else {
                        None
                    }
                }),
                source,
            );

            RSType::Vec(Box::new(make_union_or_option_type(&variants)))
        }
        TSType::TSTypeLiteral(_) => {
            // unimplemented!("TSTypeLiteral: {:#?}", literal)
            // e.g. { [key: string]: string }
            RSType::JSONValue
        }
        TSType::TSTypeOperatorType(value) => unimplemented_type(value, value.span, source),
        TSType::TSTypePredicate(value) => unimplemented_type(value, value.span, source),
        TSType::TSTypeQuery(value) => unimplemented_type(value, value.span, source),
        TSType::TSTypeReference(reference) => {
            // TSType::TSTypeReference(tstyperef_type) => {
            //     let type_name = tstyperef_type.type_name.to_string().capitalize();
            //     if let Some(type_params) = &tstyperef_type.type_parameters {
            //         if type_params.params.is_empty() {
            //             return type_name;
            //         }
            //         let params = type_params
            //             .params
            //             .iter()
            //             .map(|p| p.rs_type(unions))
            //             .join(", ");
            //         return format!("{}<{}>", type_name, params);
            //     }
            //     type_name
            // }
            if let Some(params) = &reference.type_parameters {
                // let variants: Vec<RSType> = params
                //     .params
                //     .iter()
                //     .map(|t| make_rs_type(t, source))
                //     .collect();

                return make_union_or_option_type(&make_rs_types(params.params.iter(), source));
            }
            RSType::Reference(reference.type_name.to_string())
        }
        TSType::TSUnionType(union) => {
            make_union_or_option_type(&make_rs_types(union.types.iter(), source))
        }
        TSType::TSParenthesizedType(value) => unimplemented_type(value, value.span, source),
        TSType::JSDocNullableType(value) => unimplemented_type(value, value.span, source),
        TSType::JSDocNonNullableType(value) => unimplemented_type(value, value.span, source),
        TSType::JSDocUnknownType(value) => unimplemented_type(value, value.span, source),
    };

    rs_type
}

fn make_rs_types<'a>(types: impl Iterator<Item = &'a TSType<'a>>, source: &str) -> Vec<RSType> {
    types.map(|t| make_rs_type(t, source)).collect()
}

fn make_union_or_option_type(types: &[RSType]) -> RSType {
    println!("make_union_or_option_type {:#?}", types);
    let mut option = false;
    let variants: Vec<RSType> = types
        .iter()
        .filter_map(|t| match t {
            RSType::NullOrUndefined => {
                option = true;
                None
            }
            RSType::EnumVariant(EnumVariant::NullLiteral) => {
                option = true;
                None
            }
            RSType::Unit => None,
            _ => Some(t.clone()),
        })
        .collect();
    RSType::Enum(RSEnum { option, variants })
}

impl<'a> Visit<'a> for TypeScriptToRustVisitor {
    fn visit_ts_type_alias_declaration(&mut self, it: &oxc_ast::ast::TSTypeAliasDeclaration<'a>) {
        let type_name = it.id.name.to_string();
        let rs_type = make_rs_type(&it.type_annotation, &self.source_text);
        println!("\nTYPE: {}: {:#?}", type_name, rs_type);
        self.type_map.insert(type_name, rs_type);
    }

    fn visit_ts_interface_declaration(&mut self, it: &oxc_ast::ast::TSInterfaceDeclaration<'a>) {
        let interface_name = it.id.name.to_string();
        println!("\nINTERFACE: {}", &interface_name);
        let mut fields: HashMap<String, RSType> = HashMap::new();
        for member in &it.body.body {
            let TSSignature::TSPropertySignature(property) = member else {
                continue;
            };
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
            println!(
                "  {}{}: {:?}",
                field_name,
                if property.optional { "?" } else { "" },
                &rs_type
            );
            fields.insert(field_name.to_string(), rs_type);
        }
        self.type_map
            .insert(interface_name, RSType::Struct(RSStruct { fields }));
    }
}
