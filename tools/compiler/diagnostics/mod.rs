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

// ! breakEngineModule
/*!*/
// ! good Errorinformationsumfixrestorebuild

pub mod suggestions;

use std::path::PathBuf;
use std::fmt;

/// breakEngine
pub struct DiagnosticEngine {
 /// Error codeMap
 error_codes: std::collections::HashMap<String, ErrorInfo>,
 /// languagelanguage
 language: Language,
}

impl DiagnosticEngine {
 pub fn new() -> Self {
 let mut engine = Self {
 error_codes: std::collections::HashMap::new(),
 language: Language::English,
 };
 
 engine.register_builtin_errors();
 engine
 }

 /// Settingslanguagelanguage
 pub fn with_language(mut self, lang: Language) -> Self {
 self.language = lang;
 self
 }

 /// RegisterinsideplacementError
 fn register_builtin_errors(&mut self) {
 // languagelawError
 self.register_error(ErrorInfo {
 code: "E0001".to_string(),
 title: "Syntax error".to_string(),
 description: "The input could not be parsed due to a syntax error.".to_string(),
 suggestion: Some("Check the syntax and ensure all tokens are correctly placed.".to_string()),
 doc_url: Some("https://docs.nuva.io/errors/E0001".to_string()),
 });

 // typeerror
 self.register_error(ErrorInfo {
 code: "E0002".to_string(),
 title: "Type mismatch".to_string(),
 description: "Expected type does not match the actual type.".to_string(),
 suggestion: Some("Ensure the expression has the correct type.".to_string()),
 doc_url: Some("https://docs.nuva.io/errors/E0002".to_string()),
 });

 // UndefinedVariable
 self.register_error(ErrorInfo {
 code: "E0003".to_string(),
 title: "Undefined variable".to_string(),
 description: "The variable is not defined in the current scope.".to_string(),
 suggestion: Some("Check if the variable name is correct or if it needs to be declared.".to_string()),
 doc_url: Some("https://docs.nuva.io/errors/E0003".to_string()),
 });

 // UndefinedFunction
 self.register_error(ErrorInfo {
 code: "E0004".to_string(),
 title: "Undefined function".to_string(),
 description: "The function is not defined.".to_string(),
 suggestion: Some("Check if the function name is correct or if it needs to be imported.".to_string()),
 doc_url: Some("https://docs.nuva.io/errors/E0004".to_string()),
 });

 // ParametercountnotMatch
 self.register_error(ErrorInfo {
 code: "E0005".to_string(),
 title: "Argument count mismatch".to_string(),
 description: "The number of arguments does not match the function signature.".to_string(),
 suggestion: Some("Check the function signature and provide the correct number of arguments.".to_string()),
 doc_url: Some("https://docs.nuva.io/errors/E0005".to_string()),
 });
 }

 /// RegisterError
 fn register_error(&mut self, info: ErrorInfo) {
 self.error_codes.insert(info.code.clone(), info);
 }

 /// createbreak
 pub fn create_diagnostic(&self, level: DiagnosticLevel, code: &str, span: Span) -> Diagnostic {
 let info = self.error_codes.get(code).cloned();
 
 Diagnostic {
 level,
 code: code.to_string(),
 span,
 message: info.as_ref().map(|i| i.title.clone()).unwrap_or_default(),
 description: info.as_ref().map(|i| i.description.clone()),
 suggestion: info.as_ref().and_then(|i| i.suggestion.clone()),
 doc_url: info.and_then(|i| i.doc_url),
 notes: vec![],
 }
 }

 /// formatbreak
 pub fn format_diagnostic(&self, diag: &Diagnostic, source: &str) -> String {
 let mut output = String::new();
 
 // ErrorlevelcategorysumCode
 let level_str = match diag.level {
 DiagnosticLevel::Error => "\x1b[31merror\x1b[0m",
 DiagnosticLevel::Warning => "\x1b[33mwarning\x1b[0m",
 DiagnosticLevel::Note => "\x1b[34mnote\x1b[0m",
 DiagnosticLevel::Help => "\x1b[32mhelp\x1b[0m",
 };
 
 output.push_str(&format!("{}[{}]: {}
", level_str, diag.code, diag.message));
 
 // positionplacementinformation
 if let Some(ref file) = diag.span.file {
 output.push_str(&format!(" --> {}:{}:{}
", 
 file.display(),
 diag.span.start_line,
 diag.span.start_column
 ));
 }
 
 // Codesliceparagraph
 if !source.is_empty() {
 output.push_str(&self.format_code_snippet(diag, source));
 }
 
 // description
 if let Some(ref desc) = diag.description {
 output.push_str(&format!("
 {}
", desc));
 }
 
 // build
 if let Some(ref suggestion) = diag.suggestion {
 output.push_str(&format!("
\x1b[32m suggestion\x1b[0m: {}
", suggestion));
 }
 
 // Documentationlinkaccept
 if let Some(ref url) = diag.doc_url {
 output.push_str(&format!("
 See {} for more information.
", url));
 }
 
 // appendPluscomment
 for note in &diag.notes {
 output.push_str(&format!("
 {}: {}", note.label, note.message));
 }
 
 output
 }

 /// formatCodesliceparagraph
 fn format_code_snippet(&self, diag: &Diagnostic, source: &str) -> String {
 let lines: Vec<&str> = source.lines().collect();
 let mut output = String::new();
 
 let start_line = diag.span.start_line.saturating_sub(1) as usize;
 let end_line = diag.span.end_line.saturating_sub(1) as usize;
 
 let context_before = 2;
 let context_after = 2;
 
 let show_start = start_line.saturating_sub(context_before);
 let show_end = (end_line + context_after).min(lines.len() - 1);
 
 let line_num_width = (show_end + 1).to_string().len();
 
 for i in show_start..=show_end {
 let line_num = i + 1;
 let line = lines.get(i).unwrap_or(&"");
 
 let prefix = if i >= start_line && i <= end_line {
 ">"
 } else {
 " "
 };
 
 output.push_str(&format!(
 "{} {:>width$} | {}
",
 prefix, line_num, line, width = line_num_width
 ));
 
 // displayErrorpositionplacement
 if i == start_line {
 let mut underline = String::new();
 for _ in 0..diag.span.start_column.saturating_sub(1) as usize {
 underline.push(' ');
 }
 let len = if start_line == end_line {
 (diag.span.end_column - diag.span.start_column) as usize
 } else {
 line.len() - diag.span.start_column as usize + 1
 };
 for _ in 0..len.max(1) {
 underline.push('^');
 }
 
 output.push_str(&format!(
 " {:>width$} | \x1b[31m{}\x1b[0m
",
 "", underline, width = line_num_width
 ));
 }
 }
 
 output
 }
}

impl Default for DiagnosticEngine {
 fn default() -> Self {
 Self::new()
 }
}

/// break
#[derive(Debug, Clone)]
pub struct Diagnostic {
 /// levelcategory
 pub level: DiagnosticLevel,
 /// Error code
 pub code: String,
 /// positionplacement
 pub span: Span,
 /// Message
 pub message: String,
 /// description
 pub description: Option<String>,
 /// build
 pub suggestion: Option<String>,
 /// Documentationlinkaccept
 pub doc_url: Option<String>,
 /// appendPluscomment
 pub notes: Vec<Note>,
}

impl Diagnostic {
 /// addPluscomment
 pub fn note(mut self, label: &str, message: &str) -> Self {
 self.notes.push(Note {
 label: label.to_string(),
 message: message.to_string(),
 });
 self
 }
}

/// breaklevelcategory
#[derive(Debug, Clone, Copy)]
pub enum DiagnosticLevel {
 Error,
 Warning,
 Note,
 Help,
}

/// sourcecodepositionplacement
#[derive(Debug, Clone)]
pub struct Span {
 pub file: Option<PathBuf>,
 pub start_line: u32,
 pub start_column: u32,
 pub end_line: u32,
 pub end_column: u32,
}

impl Default for Span {
 fn default() -> Self {
 Self {
 file: None,
 start_line: 1,
 start_column: 1,
 end_line: 1,
 end_column: 1,
 }
 }
}

impl Span {
 pub fn new(file: PathBuf, start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
 Self {
 file: Some(file),
 start_line,
 start_column,
 end_line,
 end_column,
 }
 }

 pub fn point(file: PathBuf, line: u32, column: u32) -> Self {
 Self::new(file, line, column, line, column)
 }
}

/// comment
#[derive(Debug, Clone)]
pub struct Note {
 pub label: String,
 pub message: String,
}

/// errorinformation
#[derive(Debug, Clone)]
pub struct ErrorInfo {
 pub code: String,
 pub title: String,
 pub description: String,
 pub suggestion: Option<String>,
 pub doc_url: Option<String>,
}

/// languagelanguage
#[derive(Debug, Clone, Copy)]
pub enum Language {
 English,
 Chinese,
}