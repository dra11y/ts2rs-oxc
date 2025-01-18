use std::path::PathBuf;

use derive_builder::Builder;
use lazy_static::lazy_static;
use oxc_parser::ParseOptions;
use oxc_resolver::{EnforceExtension, ResolveOptions};

lazy_static! {
    static ref DEFAULT_PARSE_OPTIONS: ParseOptions = ParseOptions {
        parse_regular_expression: false,
        preserve_parens: false,
        ..ParseOptions::default()
    };
    static ref DEFAULT_RESOLVE_OPTIONS: ResolveOptions = ResolveOptions {
        extensions: vec![".d.ts".into(), ".ts".into(), "".into()],
        main_fields: vec!["types".into(), "typings".into()],
        enforce_extension: EnforceExtension::Enabled,
        ..ResolveOptions::default()
    };
}

#[derive(Debug, Clone, Builder)]
#[builder(default, build_fn(validate = "Self::validate"))]
pub struct TypeScriptOptions {
    pub ignore_unimplemented: bool,
    pub parse_options: ParseOptions,
    pub resolve_options: ResolveOptions,
    pub entrypoints: Vec<PathBuf>,
}

impl TypeScriptOptionsBuilder {
    fn validate(&self) -> Result<(), String> {
        if self.entrypoints.clone().unwrap_or_default().is_empty() {
            return Err("At least one entrypoint must be specified".into());
        }
        Ok(())
    }
}

impl Default for TypeScriptOptions {
    fn default() -> Self {
        Self {
            ignore_unimplemented: true,
            parse_options: *DEFAULT_PARSE_OPTIONS,
            resolve_options: DEFAULT_RESOLVE_OPTIONS.clone(),
            entrypoints: vec![],
        }
    }
}
