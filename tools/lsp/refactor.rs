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

// ! repeatfeature

use super::{TextDocument, Range, SemanticAnalyzer, WorkspaceEdit, TextEdit};
use std::collections::HashMap;

/// shoulduserepeat
pub fn apply_refactor(doc: &TextDocument, range: Range, action: &str, analyzer: &SemanticAnalyzer) -> Option<WorkspaceEdit> {
 match action {
 "rename" => rename_symbol(doc, range, analyzer),
 "extract_function" => extract_function(doc, range, analyzer),
 "extract_variable" => extract_variable(doc, range, analyzer),
 "inline_variable" => inline_variable(doc, range, analyzer),
 "organize_imports" => organize_imports(doc, analyzer),
 _ => None,
 }
}

/// repeatnamesymbolsignal
fn rename_symbol(doc: &TextDocument, range: Range, analyzer: &SemanticAnalyzer) -> Option<WorkspaceEdit> {
 // Getcurrentpositionplacement symbolsignal
 let position = range.start;
 let symbol = analyzer.get_symbol(&doc.uri, position)?;
 
 // findplacefinitereferenceparallelgenerateencoding
 let mut changes = HashMap::new();
 let mut edits = Vec::new();
 
 // traverseplacefiniteDocumentation symbolsignalform,findreference
 for (uri, symbols) in &analyzer.symbols {
 // searchDocumentationtextbookinfix reference
 if let Some(text) = analyzer.get_document_text(uri) {
 let mut search_pos = 0;
 while let Some(pos) = text[search_pos..].find(&symbol.name) {
 let abs_pos = search_pos + pos;
 search_pos = abs_pos + 1;
 
 // Computerowsumcolumn
 let mut line = 0;
 let mut character = 0;
 for (i, c) in text.chars().enumerate() {
 if i == abs_pos {
 break;
 }
 if c == '
' {
 line += 1;
 character = 0;
 } else {
 character += 1;
 }
 }
 
 edits.push(TextEdit {
 range: Range {
 start: Position { line, character },
 end: Position { line, character: character + symbol.name.len() as u32 },
 },
 new_text: "$1".to_string(), // positionsymbol,realactualshouldthemakeuseuse newname
 });
 }
 }
 }
 
 if !edits.is_empty() {
 changes.insert(doc.uri.clone(), edits);
 return Some(WorkspaceEdit { changes });
 }
 
 None
}

/// takeFunction
fn extract_function(doc: &TextDocument, range: Range, _analyzer: &SemanticAnalyzer) -> Option<WorkspaceEdit> {
 // Getselectinfix Code
 let start_offset = doc.offset_at(range.start);
 let end_offset = doc.offset_at(range.end);
 
 if start_offset >= end_offset {
 return None;
 }
 
 let selected_text = &doc.text[start_offset..end_offset];
 
 // Analysisselectinfix Code
 // SimplifiedImplementation:AnalysisinputsumoutputVariable
 let mut inputs = Vec::new();
 let mut outputs = Vec::new();
 
 // findcancaninput Variable(simpleformImplementation)
 for word in selected_text.split_whitespace() {
 if word.contains('=') {
 let var = word.split('=').next().unwrap_or("");
 if !var.is_empty() && !inputs.contains(&var) {
 inputs.push(var.to_string());
 }
 }
 }
 
 // generatenewFunction
 let params = if inputs.is_empty() {
 String::new()
 } else {
 format!(", {} ", inputs.join(", "))
 };
 
 let new_function = format!(
 "fn extracted_function({}) {{
 {}
}}

",
 params.trim_matches(|c| c == ','),
 selected_text
 );
 
 // generateencoding
 let mut changes = HashMap::new();
 changes.insert(doc.uri.clone(), vec![
 // inFileopenheaderinsertnewFunction
 TextEdit {
 range: Range::default(),
 new_text: new_function,
 },
 // replaceselectinfixCodeasFunctiontuneuse
 TextEdit {
 range,
 new_text: "extracted_function()".to_string(),
 },
 ]);
 
 Some(WorkspaceEdit { changes })
}

/// takeVariable
fn extract_variable(doc: &TextDocument, range: Range, _analyzer: &SemanticAnalyzer) -> Option<WorkspaceEdit> {
 // Getselectinfix formreachstyle
 let start_offset = doc.offset_at(range.start);
 let end_offset = doc.offset_at(range.end);
 
 if start_offset >= end_offset {
 return None;
 }
 
 let selected_text = &doc.text[start_offset..end_offset];
 
 // generateVariablesoundbright
 let var_decl = format!("let extracted = {};
", selected_text);
 
 // generateencoding
 let mut changes = HashMap::new();
 changes.insert(doc.uri.clone(), vec![
 // incurrentrowprefixinsertVariablesoundbright
 TextEdit {
 range: Range {
 start: super::Position { line: range.start.line, character: 0 },
 end: super::Position { line: range.start.line, character: 0 },
 },
 new_text: var_decl,
 },
 // replaceselectinfixformreachstyleasVariablereference
 TextEdit {
 range,
 new_text: "extracted".to_string(),
 },
 ]);
 
 Some(WorkspaceEdit { changes })
}

/// insideVariable
fn inline_variable(doc: &TextDocument, range: Range, analyzer: &SemanticAnalyzer) -> Option<WorkspaceEdit> {
 // Getcurrentpositionplacement symbolsignal
 let position = range.start;
 let symbol = analyzer.get_symbol(&doc.uri, position)?;
 
 // findVariablefixedmeaningsumplacefinitereference,usefixedmeaningvaluereplace
 let mut changes = HashMap::new();
 let mut edits = Vec::new();
 
 // findVariablefixedmeaningpositionplacement
 let mut definition_text = None;
 if let Some(text) = analyzer.get_document_text(&doc.uri) {
 let mut search_pos = 0;
 while let Some(pos) = text[search_pos..].find(&format!("let {} = ", symbol.name)) {
 let abs_pos = search_pos + pos;
 search_pos = abs_pos + 1;
 
 // findAssignmentformreachstyle
 if let Some(end_pos) = text[abs_pos..].find(';') {
 definition_text = Some(text[abs_pos + symbol.name.len() + 7..abs_pos + end_pos].trim().to_string());
 break;
 }
 }
 }
 
 // makeusefixedmeaningvaluereplaceplacefinitereference
 if let Some(def_text) = definition_text {
 if let Some(text) = analyzer.get_document_text(&doc.uri) {
 let mut search_pos = 0;
 while let Some(pos) = text[search_pos..].find(&symbol.name) {
 let abs_pos = search_pos + pos;
 search_pos = abs_pos + 1;
 
 // Computerowsumcolumn
 let mut line = 0;
 let mut character = 0;
 for (i, c) in text.chars().enumerate() {
 if i == abs_pos {
 break;
 }
 if c == '
' {
 line += 1;
 character = 0;
 } else {
 character += 1;
 }
 }
 
 edits.push(TextEdit {
 range: Range {
 start: Position { line, character },
 end: Position { line, character: character + symbol.name.len() as u32 },
 },
 new_text: def_text.clone(),
 });
 }
 }
 }
 
 if !edits.is_empty() {
 changes.insert(doc.uri.clone(), edits);
 return Some(WorkspaceEdit { changes });
 }
 
 None
}

/// grouporganizeconductenter
fn organize_imports(doc: &TextDocument, _analyzer: &SemanticAnalyzer) -> Option<WorkspaceEdit> {
 // parseconductenterlanguagesentence
 let imports = parse_imports(&doc.text);
 
 // sortsumgorepeat
 let organized = organize_import_list(imports);
 
 // generateencoding
 // Computeconductenterlanguagesentence range
 let mut import_range_start = None;
 let mut import_range_end = None;
 let mut found_import = false;
 
 for (line_num, line) in doc.text.lines().enumerate() {
 let trimmed = line.trim();
 if trimmed.starts_with("use ") {
 if !found_import {
 import_range_start = Some(Position {
 line: line_num as u32,
 character: 0,
 });
 found_import = true;
 }
 import_range_end = Some(Position {
 line: line_num as u32,
 character: line.len() as u32,
 });
 } else if found_import && !trimmed.is_empty() && !trimmed.starts_with("//") {
 // conductenterblockend
 break;
 }
 }
 
 if let (Some(start), Some(end)) = (import_range_start, import_range_end) {
 // generategrouporganizethen conductenterlanguagesentence
 let organized_text = organized
 .iter()
 .map(|import| format!("use {};
", import.module))
 .collect::<Vec<_>>()
 .join("");
 
 let mut changes = HashMap::new();
 changes.insert(doc.uri.clone(), vec![
 TextEdit {
 range: Range { start, end },
 new_text: organized_text,
 },
 ]);
 
 return Some(WorkspaceEdit { changes });
 }
 
 None
}

/// parseconductenterlanguagesentence
fn parse_imports(text: &str) -> Vec<ImportInfo> {
 let mut imports = vec![];
 
 for line in text.lines() {
 let line = line.trim();
 
 if line.starts_with("use ") {
 // parse use languagesentence
 let module = line[4..].trim().trim_end_matches(';');
 imports.push(ImportInfo {
 module: module.to_string(),
 items: vec![],
 });
 }
 }
 
 imports
}

/// grouporganizeconductenterList
fn organize_import_list(mut imports: Vec<ImportInfo>) -> Vec<ImportInfo> {
 // sorting
 imports.sort_by(|a, b| a.module.cmp(&b.module));
 
 // gorepeat
 imports.dedup_by(|a, b| a.module == b.module);
 
 imports
}

/// conductenterinformation
#[derive(Debug, Clone)]
pub struct ImportInfo {
 pub module: String,
 pub items: Vec<String>,
}

/// canuse repeatOperation
pub fn get_refactor_actions(doc: &TextDocument, range: Range, analyzer: &SemanticAnalyzer) -> Vec<RefactorAction> {
 let mut actions = vec![];
 
 // checkiswhethercanwithrepeatname
 if analyzer.get_symbol(&doc.uri, range.start).is_some() {
 actions.push(RefactorAction {
 title: "Rename Symbol".to_string(),
 action: "rename".to_string(),
 kind: "refactor.rename".to_string(),
 });
 }
 
 // checkiswhethercanwithtakeFunction
 if range.start != range.end {
 actions.push(RefactorAction {
 title: "Extract Function".to_string(),
 action: "extract_function".to_string(),
 kind: "refactor.extract.function".to_string(),
 });
 
 actions.push(RefactorAction {
 title: "Extract Variable".to_string(),
 action: "extract_variable".to_string(),
 kind: "refactor.extract.variable".to_string(),
 });
 }
 
 // checkiswhethercanwithgrouporganizeconductenter
 actions.push(RefactorAction {
 title: "Organize Imports".to_string(),
 action: "organize_imports".to_string(),
 kind: "source.organizeImports".to_string(),
 });
 
 actions
}

/// repeatOperation
#[derive(Debug, Clone)]
pub struct RefactorAction {
 pub title: String,
 pub action: String,
 pub kind: String,
}