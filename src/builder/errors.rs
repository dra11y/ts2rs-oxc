use std::{error::Error, fmt};

use oxc_diagnostics::OxcDiagnostic;

// Custom error type to hold `Vec<OxcDiagnostic>`
#[derive(Debug)]
pub struct DiagnosticsError {
    pub diagnostics: Vec<OxcDiagnostic>,
}

// Implement `Display` for `DiagnosticsError`
impl fmt::Display for DiagnosticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in &self.diagnostics {
            writeln!(f, "{}", diagnostic)?;
        }
        Ok(())
    }
}

// Implement `Error` for `DiagnosticsError`
impl Error for DiagnosticsError {}
