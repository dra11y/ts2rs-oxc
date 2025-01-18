use std::collections::{HashMap, HashSet};

use crate::rs_types::*;

use super::TypeScriptToRustVisitor;

pub trait ReferenceResolver {
    fn resolve_references(&mut self) -> HashSet<RSReference>;
}

impl ReferenceResolver for TypeScriptToRustVisitor<'_> {
    fn resolve_references(&mut self) -> HashSet<RSReference> {
        // self.types.iter().map(|(k, v)| {
        //     let resolved_type = resolve_type(&rs_type, &self.types, &mut references);
        //     if let Some(mut_ref_type) = self.types.get_mut(&name) {
        //         *mut_ref_type = resolved_type;
        //     }
        // }).collect();

        let keys: Vec<_> = self.local_types.keys().cloned().collect();

        let mut references: HashSet<RSReference> = HashSet::new();

        for name in keys {
            if let Some(rs_type) = self.local_types.get(&name).cloned() {
                let resolved_type = resolve_type(&rs_type, &self.local_types, &mut references);
                if let Some(mut_ref_type) = self.local_types.get_mut(&name) {
                    *mut_ref_type = resolved_type;
                }
            }
        }

        references
    }
}

pub(crate) fn resolve_type(
    rs_type: &RSType,
    type_map: &HashMap<String, RSType>,
    references: &mut HashSet<RSReference>,
) -> RSType {
    match rs_type {
        RSType::Reference(reference) => RSType::Reference(reference.clone()),
        // Recursively resolve contained types for Vec and Option
        RSType::Vec(inner) => RSType::Vec(Box::new(resolve_type(inner, type_map, references))),
        RSType::Option(inner) => {
            RSType::Option(Box::new(resolve_type(inner, type_map, references)))
        }
        RSType::Enum(RSEnum { variants }) => {
            let variants = variants
                .iter()
                .map(|variant| resolve_type(variant, type_map, references))
                .collect();
            RSType::Enum(RSEnum { variants })
        }
        RSType::Struct(RSStruct { fields }) => {
            let fields = fields
                .iter()
                .map(|(field_name, field_type)| {
                    (
                        field_name.clone(),
                        resolve_type(field_type, type_map, references),
                    )
                })
                .collect();
            RSType::Struct(RSStruct { fields })
        }
        _ => rs_type.clone(),
    }
}
