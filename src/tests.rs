#![cfg(test)]
use crate::{
    builder::{
        TypeScriptToRustBuilder,
        options::{TypeScriptOptions, TypeScriptOptionsBuilder},
    },
    rs_types::{RSPrimitive, RSReference, RSType},
    typescript_type_id::TypeScriptTypeId,
};
use serde_json::json;
use std::path::PathBuf;

/// Test data representing a complete axe-core test run
const COMPLETE_AXE_RESULTS: &str = r#"{
        "testEngine": {
            "name": "axe-core",
            "version": "4.10.0"
        },
        "testRunner": {
            "name": "axe"
        },
        "testEnvironment": {
            "userAgent": "Mozilla/5.0",
            "windowWidth": 1200,
            "windowHeight": 900,
            "orientationAngle": 0,
            "orientationType": "landscape-primary"
        },
        "timestamp": "2024-03-14T12:00:00.000Z",
        "url": "https://example.com",
        "passes": [{
            "description": "Ensures <img> elements have alternate text",
            "help": "Images must have alternate text",
            "helpUrl": "https://dequeuniversity.com/rules/axe/4.10/image-alt",
            "id": "image-alt",
            "impact": null,
            "tags": ["wcag2a", "wcag111", "cat.text-alternatives", "section508"],
            "nodes": [{
                "html": "<img src='test.png' alt='Test image'>",
                "impact": null,
                "target": ["img"],
                "failureSummary": "",
                "any": [{
                    "id": "has-alt",
                    "data": null,
                    "relatedNodes": []
                }],
                "all": [],
                "none": []
            }]
        }],
        "violations": [],
        "incomplete": [],
        "inapplicable": []
    }"#;

/// Test data for RunOptions with all possible configurations
const COMPLETE_RUN_OPTIONS: &str = r#"{
        "runOnly": {
            "type": "tag",
            "values": ["wcag2a", "wcag2aa"]
        },
        "rules": {
            "image-alt": { "enabled": true },
            "color-contrast": { "enabled": false }
        },
        "reporter": "v2",
        "resultTypes": ["violations", "incomplete"],
        "selectors": true,
        "ancestry": true,
        "xpath": false,
        "absolutePaths": true,
        "frames": true,
        "frameWaitTime": 1000,
        "iframes": true,
        "elementRef": false,
        "performanceTimer": true,
        "pingWaitTime": 500
    }"#;

#[test]
fn test_generate_axe_types() {
    let entrypoint = PathBuf::from("examples/axe/axe-types.ts");
    let canonical_path = entrypoint
        .canonicalize()
        .expect("Failed to canonicalize path");
    let options = TypeScriptOptionsBuilder::default()
        .entrypoints(vec![entrypoint])
        .build()
        .expect("options");
    let mut builder = TypeScriptToRustBuilder::new(options);
    builder.visit_entrypoints();

    // Get the axe.d.ts module path
    let axe_dts_path = PathBuf::from("examples/axe/node_modules/axe-core/axe.d.ts")
        .canonicalize()
        .expect("Failed to canonicalize axe.d.ts path");

    // Verify RunOptions type structure
    let run_options = builder
        .types
        .get(&TypeScriptTypeId {
            module: axe_dts_path.clone(),
            name: "RunOptions".to_string(),
        })
        .expect("RunOptions type not found");

    if let RSType::Struct(struct_type) = run_options {
        assert!(
            struct_type.fields.contains_key("runOnly"),
            "Missing runOnly field"
        );
        assert!(
            struct_type.fields.contains_key("rules"),
            "Missing rules field"
        );
        assert!(
            struct_type.fields.contains_key("reporter"),
            "Missing reporter field"
        );
        assert!(
            struct_type.fields.contains_key("resultTypes"),
            "Missing resultTypes field"
        );
        assert!(
            struct_type.fields.contains_key("selectors"),
            "Missing selectors field"
        );
        assert!(
            struct_type.fields.contains_key("ancestry"),
            "Missing ancestry field"
        );
        assert!(
            struct_type.fields.contains_key("xpath"),
            "Missing xpath field"
        );
        assert!(
            struct_type.fields.contains_key("absolutePaths"),
            "Missing absolutePaths field"
        );
        assert!(
            struct_type.fields.contains_key("frameWaitTime"),
            "Missing frameWaitTime field"
        );
        assert!(
            struct_type.fields.contains_key("iframes"),
            "Missing iframes field"
        );

        // Check runOnly field type (should be an Option of either RunOnly enum or array/string)
        let run_only = struct_type.fields.get("runOnly").expect("runOnly field");
        match run_only {
            RSType::Option(inner) => match &**inner {
                RSType::Enum(enum_type) => {
                    assert_eq!(enum_type.variants.len(), 4, "Should have 4 variants");

                    // Check each variant
                    let variants = &enum_type.variants;
                    match &variants[0] {
                        RSType::Reference(ref_type) => match ref_type {
                            RSReference::Unresolved {
                                local_name: name, ..
                            } => assert_eq!(name, "RunOnly"),
                            RSReference::Resolved { id, .. } => assert_eq!(id.name, "RunOnly"),
                        },
                        _ => panic!("First variant should be RunOnly reference"),
                    }

                    match &variants[1] {
                        RSType::Vec(inner) => match &**inner {
                            RSType::Reference(ref_type) => match ref_type {
                                RSReference::Unresolved {
                                    local_name: name, ..
                                } => {
                                    assert_eq!(name, "TagValue")
                                }
                                RSReference::Resolved { id, .. } => assert_eq!(id.name, "TagValue"),
                            },
                            _ => panic!("Second variant should be TagValue array"),
                        },
                        _ => panic!("Second variant should be array"),
                    }

                    match &variants[2] {
                        RSType::Vec(inner) => match &**inner {
                            RSType::Primitive(RSPrimitive::String) => (),
                            _ => panic!("Third variant should be string array"),
                        },
                        _ => panic!("Third variant should be array"),
                    }

                    match &variants[3] {
                        RSType::Primitive(RSPrimitive::String) => (),
                        _ => panic!("Fourth variant should be string"),
                    }
                }
                _ => panic!("runOnly should be an enum representing the union type"),
            },
            _ => panic!("runOnly should be optional"),
        }

        // Check rules field (should be optional RuleObject)
        let rules = struct_type.fields.get("rules").expect("rules field");
        match rules {
            RSType::Option(inner) => match &**inner {
                RSType::Reference(ref_type) => match ref_type {
                    RSReference::Unresolved {
                        local_name: name, ..
                    } => assert_eq!(name, "RuleObject"),
                    RSReference::Resolved { id, .. } => assert_eq!(id.name, "RuleObject"),
                },
                _ => panic!("rules should be RuleObject"),
            },
            _ => panic!("rules should be optional"),
        }

        // Check resultTypes field (should be optional array of resultGroups)
        let result_types = struct_type
            .fields
            .get("resultTypes")
            .expect("resultTypes field");
        match result_types {
            RSType::Option(inner) => match &**inner {
                RSType::Vec(array_type) => match &**array_type {
                    RSType::Reference(ref_type) => match ref_type {
                        RSReference::Unresolved {
                            local_name: name, ..
                        } => assert_eq!(name, "resultGroups"),
                        RSReference::Resolved { id, .. } => assert_eq!(id.name, "resultGroups"),
                    },
                    _ => panic!("resultTypes should contain resultGroups enum"),
                },
                _ => panic!("resultTypes should be an array"),
            },
            _ => panic!("resultTypes should be optional"),
        }
    } else {
        panic!("RunOptions is not a struct type");
    }

    // Verify AxeResults type structure
    let axe_results = builder
        .types
        .get(&TypeScriptTypeId {
            module: axe_dts_path.clone(),
            name: "AxeResults".to_string(),
        })
        .expect("AxeResults type not found");

    if let RSType::Struct(struct_type) = axe_results {
        println!(
            "DEBUG: Available fields in AxeResults = {:#?}",
            struct_type.fields.keys().collect::<Vec<_>>()
        );

        assert!(
            struct_type.fields.contains_key("passes"),
            "Missing passes field"
        );
        assert!(
            struct_type.fields.contains_key("violations"),
            "Missing violations field"
        );
        assert!(
            struct_type.fields.contains_key("incomplete"),
            "Missing incomplete field"
        );
        assert!(
            struct_type.fields.contains_key("inapplicable"),
            "Missing inapplicable field"
        );
        assert!(
            struct_type.fields.contains_key("toolOptions"),
            "Missing toolOptions field"
        );

        // Check passes field (should be array of Result)
        let passes = struct_type.fields.get("passes").expect("passes field");
        match passes {
            RSType::Vec(array_type) => match &**array_type {
                RSType::Reference(ref_type) => match ref_type {
                    RSReference::Unresolved {
                        local_name: name, ..
                    } => assert_eq!(name, "Result"),
                    RSReference::Resolved { id, .. } => assert_eq!(id.name, "Result"),
                },
                _ => panic!("passes should contain Result type"),
            },
            _ => panic!("passes should be an array"),
        }

        // Check violations field (should be array of Result)
        let violations = struct_type
            .fields
            .get("violations")
            .expect("violations field");
        match violations {
            RSType::Vec(array_type) => match &**array_type {
                RSType::Reference(ref_type) => match ref_type {
                    RSReference::Unresolved {
                        local_name: name, ..
                    } => assert_eq!(name, "Result"),
                    RSReference::Resolved { id, .. } => assert_eq!(id.name, "Result"),
                },
                _ => panic!("violations should contain Result type"),
            },
            _ => panic!("violations should be an array"),
        }

        // Check timestamp field (should be string)
        let timestamp = struct_type
            .fields
            .get("timestamp")
            .expect("timestamp field");
        println!("DEBUG: timestamp type = {:#?}", timestamp);
        match timestamp {
            RSType::Primitive(RSPrimitive::String) => (),
            _ => panic!("timestamp should be string"),
        }

        // Check url field (should be string)
        let url = struct_type.fields.get("url").expect("url field");
        match url {
            RSType::Primitive(RSPrimitive::String) => (),
            _ => panic!("url should be string"),
        }
    } else {
        panic!("AxeResults is not a struct type");
    }

    // The following tests should pass once type generation is implemented:

    // Test RunOptions deserialization
    // let run_options: RunOptions = serde_json::from_str(COMPLETE_RUN_OPTIONS).unwrap();
    // assert_eq!(run_options.reporter.as_deref(), Some("v2"));
    // assert!(matches!(run_options.run_only, Some(RunOnly::Tag(_))));

    // Test AxeResults deserialization
    // let axe_results: AxeResults = serde_json::from_str(COMPLETE_AXE_RESULTS).unwrap();
    // assert!(!axe_results.passes.is_empty());
    // assert_eq!(axe_results.passes[0].id, "image-alt");
}

#[test]
fn test_run_only_variants() {
    let test_cases = [
        (r#"{"runOnly": ["wcag2a", "wcag2aa"]}"#, "Array of tags"),
        (
            r#"{"runOnly": {"type": "tag", "values": ["wcag2a"]}}"#,
            "RunOnly object with tag type",
        ),
        (
            r#"{"runOnly": {"type": "rule", "values": ["image-alt"]}}"#,
            "RunOnly object with rule type",
        ),
        (r#"{"runOnly": "wcag2a"}"#, "Single tag string"),
    ];

    let entrypoint = PathBuf::from("examples/axe/axe-types.ts");
    let canonical_path = entrypoint
        .canonicalize()
        .expect("Failed to canonicalize path");
    let options = TypeScriptOptions::default();
    let mut builder = TypeScriptToRustBuilder::new(options);

    // builder.visit_module(&entrypoint);

    for (json, description) in test_cases {
        // TODO: Once implemented, test each variant:
        // let run_options: RunOptions = serde_json::from_str(json)
        //     .unwrap_or_else(|e| panic!("Failed to parse {description}: {e}"));
        // assert!(matches!(run_options.run_only, Some(_)));
    }
}

#[test]
fn test_impact_values() {
    let test_cases = ["minor", "moderate", "serious", "critical"];

    // TODO: Test ImpactValue enum generation and values
}

#[test]
fn test_result_groups() {
    let test_cases = ["inapplicable", "passes", "incomplete", "violations"];

    // TODO: Test ResultGroups enum generation and values
}
