use std::{any::type_name_of_val, collections::HashMap, path::PathBuf};

use oxc_ast::ast::{TSLiteral, TSType};
use oxc_span::Span;
use serde::Serialize;

use crate::rs_types::*;

pub(crate) fn make_rs_type(
    ts_type: &TSType,
    source: &str,
    // imported_types: &HashMap<String, (PathBuf, String)>,
) -> RSType {
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
                TSLiteral::BooleanLiteral(boolean) => RSEnumVariant::BooleanLiteral(boolean.value),
                TSLiteral::NullLiteral(_) => RSEnumVariant::NullLiteral,
                TSLiteral::NumericLiteral(numeric) => {
                    RSEnumVariant::NumericLiteral(numeric.raw.into())
                }
                TSLiteral::BigIntLiteral(bigint) => {
                    RSEnumVariant::NumericLiteral(bigint.raw.clone().into_string())
                }
                TSLiteral::RegExpLiteral(value) => unimplemented_variant(value, value.span, source),
                TSLiteral::StringLiteral(string) => {
                    RSEnumVariant::StringLiteral(string.value.clone().into_string())
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
            RSType::EnumVariant(RSEnumVariant::RSType(Box::new(element_type)))
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
            println!("TSType::TSTypeReference {:#?}", reference);
            if let Some(params) = &reference.type_parameters {
                return make_union_or_option_type(&make_rs_types(params.params.iter(), source));
            }
            RSType::Reference(RSReference::Unresolved {
                name: reference.type_name.to_string(),
                module_specifier: None,
            })
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
    RSType::Enum(RSEnum { option, variants })
}

fn unimplemented_variant<T: Serialize>(value: &T, span: Span, source: &str) -> RSEnumVariant {
    RSEnumVariant::Unimplemented(
        extract_type_name(value),
        span.source_text(source).to_string(),
    )
}

fn extract_type_name<T: Serialize>(value: &T) -> String {
    type_name_of_val(value)
        .split("::")
        .last()
        .unwrap()
        .replace('>', "")
        .to_string()
}

fn unimplemented_type<T: Serialize>(value: &T, span: Span, source: &str) -> RSType {
    RSType::Unimplemented(
        extract_type_name(value),
        span.source_text(source).to_string(),
    )
}
