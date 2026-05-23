/*
 * Nuva OS
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// ! LSP break

use super::{Range, Diagnostic, DiagnosticSeverity, Location, DiagnosticRelatedInformation};

/// breakManager
pub struct DiagnosticManager {
 /// breakSet
 diagnostics: Vec<Diagnostic>,
}

impl DiagnosticManager {
 pub fn new() -> Self {
 Self {
 diagnostics: vec![],
 }
 }

 /// addPlusbreak
 pub fn add(&mut self, diagnostic: Diagnostic) {
 self.diagnostics.push(diagnostic);
 }

 /// addPlusError
 pub fn add_error(&mut self, range: Range, message: &str) {
 self.add(Diagnostic {
 range,
 severity: DiagnosticSeverity::Error,
 code: None,
 source: Some("nuva".to_string()),
 message: message.to_string(),
 related_information: vec![],
 });
 }

 /// addPluswarning
 pub fn add_warning(&mut self, range: Range, message: &str) {
 self.add(Diagnostic {
 range,
 severity: DiagnosticSeverity::Warning,
 code: None,
 source: Some("nuva".to_string()),
 message: message.to_string(),
 related_information: vec![],
 });
 }

 /// addPlusinformation
 pub fn add_info(&mut self, range: Range, message: &str) {
 self.add(Diagnostic {
 range,
 severity: DiagnosticSeverity::Information,
 code: None,
 source: Some("nuva".to_string()),
 message: message.to_string(),
 related_information: vec![],
 });
 }

 /// addPlus
 pub fn add_hint(&mut self, range: Range, message: &str) {
 self.add(Diagnostic {
 range,
 severity: DiagnosticSeverity::Hint,
 code: None,
 source: Some("nuva".to_string()),
 message: message.to_string(),
 related_information: vec![],
 });
 }

 /// Getplacefinitebreak
 pub fn get_all(&self) -> &[Diagnostic] {
 &self.diagnostics
 }

 /// clearDividebreak
 pub fn clear(&mut self) {
 self.diagnostics.clear();
 }

 /// bystrictrepeatprocessmeasurementFilter
 pub fn filter_by_severity(&self, severity: DiagnosticSeverity) -> Vec<&Diagnostic> {
 self.diagnostics.iter()
 .filter(|d| d.severity == severity)
 .collect()
 }

 /// GetErrorcount
 pub fn error_count(&self) -> usize {
 self.filter_by_severity(DiagnosticSeverity::Error).len()
 }

 /// Getwarningcount
 pub fn warning_count(&self) -> usize {
 self.filter_by_severity(DiagnosticSeverity::Warning).len()
 }
}

impl Default for DiagnosticManager {
 fn default() -> Self {
 Self::new()
 }
}

/// createlanguagelawErrorbreak
pub fn syntax_error(range: Range, message: &str) -> Diagnostic {
 Diagnostic {
 range,
 severity: DiagnosticSeverity::Error,
 code: Some("E0001".to_string()),
 source: Some("nuva".to_string()),
 message: message.to_string(),
 related_information: vec![],
 }
}

/// createTypeErrorbreak
pub fn type_error(range: Range, expected: &str, actual: &str) -> Diagnostic {
 Diagnostic {
 range,
 severity: DiagnosticSeverity::Error,
 code: Some("E0002".to_string()),
 source: Some("nuva".to_string()),
 message: format!("Type mismatch: expected '{}', found '{}'", expected, actual),
 related_information: vec![],
 }
}

/// createUndefinedVariablebreak
pub fn undefined_variable(range: Range, name: &str) -> Diagnostic {
 Diagnostic {
 range,
 severity: DiagnosticSeverity::Error,
 code: Some("E0003".to_string()),
 source: Some("nuva".to_string()),
 message: format!("Undefined variable: '{}'", name),
 related_information: vec![],
 }
}

/// createUndefinedFunctionbreak
pub fn undefined_function(range: Range, name: &str) -> Diagnostic {
 Diagnostic {
 range,
 severity: DiagnosticSeverity::Error,
 code: Some("E0004".to_string()),
 source: Some("nuva".to_string()),
 message: format!("Undefined function: '{}'", name),
 related_information: vec![],
 }
}

/// createParametercountnotMatchbreak
pub fn argument_count_mismatch(range: Range, expected: usize, actual: usize) -> Diagnostic {
 Diagnostic {
 range,
 severity: DiagnosticSeverity::Error,
 code: Some("E0005".to_string()),
 source: Some("nuva".to_string()),
 message: format!("Argument count mismatch: expected {}, found {}", expected, actual),
 related_information: vec![],
 }
}

/// createmakeuseVariablewarning
pub fn unused_variable(range: Range, name: &str) -> Diagnostic {
 Diagnostic {
 range,
 severity: DiagnosticSeverity::Warning,
 code: Some("W0001".to_string()),
 source: Some("nuva".to_string()),
 message: format!("Unused variable: '{}'", name),
 related_information: vec![],
 }
}

/// createmakeuseconductenterwarning
pub fn unused_import(range: Range, name: &str) -> Diagnostic {
 Diagnostic {
 range,
 severity: DiagnosticSeverity::Warning,
 code: Some("W0002".to_string()),
 source: Some("nuva".to_string()),
 message: format!("Unused import: '{}'", name),
 related_information: vec![],
 }
}

/// createDeprecatedwarning
pub fn deprecated(range: Range, name: &str, message: &str) -> Diagnostic {
 Diagnostic {
 range,
 severity: DiagnosticSeverity::Warning,
 code: Some("W0003".to_string()),
 source: Some("nuva".to_string()),
 message: format!("'{}' is deprecated: {}", name, message),
 related_information: vec![],
 }
}