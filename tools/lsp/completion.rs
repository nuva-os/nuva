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

// ! Codepatchall

use super::{TextDocument, Position, SemanticAnalyzer, CompletionList, CompletionItem, CompletionItemKind};

/// Codepatchall
pub fn provide_completions(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> CompletionList {
 let mut items = vec![];
 
 // Getcurrentpositionplacement context
 let context = analyze_context(doc, position);
 
 match context {
 CompletionContext::Expression => {
 // formreachstylecontext: patchallVariable、Function、Typeetc
 items.extend(complete_variables(doc, position, analyzer));
 items.extend(complete_functions(doc, position, analyzer));
 items.extend(complete_types(doc, position, analyzer));
 items.extend(complete_keywords());
 }
 CompletionContext::Type => {
 // Typecontext: patchallTypename
 items.extend(complete_types(doc, position, analyzer));
 }
 CompletionContext::Member { base } => {
 // successaccesscontext: patchallsuccess
 items.extend(complete_members(doc, position, analyzer, &base));
 }
 CompletionContext::Import => {
 // conductentercontext: patchallModulename
 items.extend(complete_modules(doc, position, analyzer));
 }
 CompletionContext::Attribute => {
 // Propertycontext: patchallPropertyname
 items.extend(complete_attributes());
 }
 _ => {}
 }
 
 CompletionList {
 is_incomplete: false,
 items,
 }
}

/// patchallcontext
#[derive(Debug, Clone)]
pub enum CompletionContext {
 Unknown,
 Expression,
 Type,
 Member { base: String },
 Import,
 Attribute,
 Statement,
}

/// Analysispatchallcontext
fn analyze_context(doc: &TextDocument, position: Position) -> CompletionContext {
 // Getcurrentrow
 let line_start = doc.text.lines()
 .take(position.line as usize)
 .map(|l| l.len() + 1)
 .sum::<usize>();
 
 let line = doc.text.lines()
 .nth(position.line as usize)
 .unwrap_or("");
 
 let prefix = &line[..position.character.min(line.len() as u32) as usize];
 
 // checkiswhetherissuccessaccess
 if let Some(dot_pos) = prefix.rfind('.') {
 let base = &prefix[..dot_pos];
 return CompletionContext::Member { base: base.to_string() };
 }
 
 // checkiswhetherisconductenterlanguagesentence
 if prefix.trim().starts_with("use ") || prefix.trim().starts_with("import ") {
 return CompletionContext::Import;
 }
 
 // checkiswhetherisProperty
 if prefix.trim().starts_with('#') {
 return CompletionContext::Attribute;
 }
 
 // defaultasformreachstylecontext
 CompletionContext::Expression
}

/// patchallVariable
fn complete_variables(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Vec<CompletionItem> {
 let mut items = Vec::new();
 
 // secondarysymbolsignalformGetVariable
 if let Some(symbols) = analyzer.symbols.get(&doc.uri) {
 for symbol in symbols {
 if symbol.kind == super::SymbolKind::Variable {
 items.push(CompletionItem {
 label: symbol.name.clone(),
 kind: CompletionItemKind::Variable,
 detail: symbol.detail.clone(),
 documentation: symbol.documentation.clone(),
 insert_text: Some(symbol.name.clone()),
 sort_text: Some(symbol.name.clone()),
 filter_text: Some(symbol.name.clone()),
 });
 }
 }
 }
 
 items
}

/// patchallFunction
fn complete_functions(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Vec<CompletionItem> {
 let mut items = Vec::new();
 
 // secondarysymbolsignalformGetFunction
 if let Some(symbols) = analyzer.symbols.get(&doc.uri) {
 for symbol in symbols {
 if symbol.kind == super::SymbolKind::Function {
 items.push(CompletionItem {
 label: symbol.name.clone(),
 kind: CompletionItemKind::Function,
 detail: symbol.detail.clone(),
 documentation: symbol.documentation.clone(),
 insert_text: Some(format!("{}($1)", symbol.name)),
 sort_text: Some(symbol.name.clone()),
 filter_text: Some(symbol.name.clone()),
 });
 }
 }
 }
 
 // addPlusinsideplacementFunction
 items.push(CompletionItem {
 label: "println".to_string(),
 kind: CompletionItemKind::Function,
 detail: Some("fn println(msg: &str)".to_string()),
 documentation: Some("Print a line to stdout".to_string()),
 insert_text: Some("println!($1)".to_string()),
 sort_text: Some("println".to_string()),
 filter_text: Some("println".to_string()),
 });
 
 items
}

/// patchallType
fn complete_types(_doc: &TextDocument, _position: Position, _analyzer: &SemanticAnalyzer) -> Vec<CompletionItem> {
 vec![
 CompletionItem {
 label: "i32".to_string(),
 kind: CompletionItemKind::Class,
 detail: Some("32-bit signed integer".to_string()),
 documentation: None,
 insert_text: Some("i32".to_string()),
 sort_text: Some("i32".to_string()),
 filter_text: Some("i32".to_string()),
 },
 CompletionItem {
 label: "u32".to_string(),
 kind: CompletionItemKind::Class,
 detail: Some("32-bit unsigned integer".to_string()),
 documentation: None,
 insert_text: Some("u32".to_string()),
 sort_text: Some("u32".to_string()),
 filter_text: Some("u32".to_string()),
 },
 CompletionItem {
 label: "String".to_string(),
 kind: CompletionItemKind::Class,
 detail: Some("String type".to_string()),
 documentation: None,
 insert_text: Some("String".to_string()),
 sort_text: Some("String".to_string()),
 filter_text: Some("String".to_string()),
 },
 ]
}

/// patchallclosekeycharacter
fn complete_keywords() -> Vec<CompletionItem> {
 let keywords = [
 "fn", "let", "const", "mut", "if", "else", "match", "while", "for", "loop",
 "return", "break", "continue", "struct", "enum", "impl", "trait", "type",
 "pub", "mod", "use", "self", "super", "true", "false", "async", "await",
 ];
 
 keywords.iter().map(|&kw| CompletionItem {
 label: kw.to_string(),
 kind: CompletionItemKind::Keyword,
 detail: None,
 documentation: None,
 insert_text: Some(kw.to_string()),
 sort_text: Some(format!("zzz_{}", kw)),
 filter_text: Some(kw.to_string()),
 }).collect()
}

/// patchallsuccess
fn complete_members(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer, base: &str) -> Vec<CompletionItem> {
 let mut items = Vec::new();
 
 // rootevidencebaseTypeGetsuccess
 if let Some(symbols) = analyzer.symbols.get(&doc.uri) {
 for symbol in symbols {
 if symbol.kind == super::SymbolKind::Type && symbol.name == base {
 // findtoTypefixedmeaning,Returnitssuccess
 // SimplifiedImplementation:Returnsomeconstant success
 items.push(CompletionItem {
 label: "new".to_string(),
 kind: CompletionItemKind::Method,
 detail: Some("fn new() -> Self".to_string()),
 documentation: Some("Create a new instance".to_string()),
 insert_text: Some("new()".to_string()),
 sort_text: Some("new".to_string()),
 filter_text: Some("new".to_string()),
 });
 
 items.push(CompletionItem {
 label: "clone".to_string(),
 kind: CompletionItemKind::Method,
 detail: Some("fn clone(&self) -> Self".to_string()),
 documentation: Some("Clone the instance".to_string()),
 insert_text: Some("clone()".to_string()),
 sort_text: Some("clone".to_string()),
 filter_text: Some("clone".to_string()),
 });
 }
 }
 }
 
 items
}

/// patchallModule
fn complete_modules(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Vec<CompletionItem> {
 let mut items = Vec::new();
 
 // secondarysymbolsignalformGetModule
 if let Some(symbols) = analyzer.symbols.get(&doc.uri) {
 for symbol in symbols {
 if symbol.kind == super::SymbolKind::Module {
 items.push(CompletionItem {
 label: symbol.name.clone(),
 kind: CompletionItemKind::Module,
 detail: symbol.detail.clone(),
 documentation: symbol.documentation.clone(),
 insert_text: Some(symbol.name.clone()),
 sort_text: Some(symbol.name.clone()),
 filter_text: Some(symbol.name.clone()),
 });
 }
 }
 }
 
 // addPlusinsideplacementModule
 items.push(CompletionItem {
 label: "std".to_string(),
 kind: CompletionItemKind::Module,
 detail: Some("Standard library".to_string()),
 documentation: None,
 insert_text: Some("std".to_string()),
 sort_text: Some("std".to_string()),
 filter_text: Some("std".to_string()),
 });
 
 items.push(CompletionItem {
 label: "core".to_string(),
 kind: CompletionItemKind::Module,
 detail: Some("Core library".to_string()),
 documentation: None,
 insert_text: Some("core".to_string()),
 sort_text: Some("core".to_string()),
 filter_text: Some("core".to_string()),
 });
 
 items
}

/// patchallProperty
fn complete_attributes() -> Vec<CompletionItem> {
 let attrs = [
 ("derive", "Derive trait implementations"),
 ("inline", "Hint to inline the function"),
 ("no_mangle", "Disable name mangling"),
 ("cfg", "Conditional compilation"),
 ("test", "Mark as test function"),
 ("doc", "Documentation"),
 ];
 
 attrs.iter().map(|&(name, desc)| CompletionItem {
 label: format!("#[{}]", name),
 kind: CompletionItemKind::Property,
 detail: Some(desc.to_string()),
 documentation: None,
 insert_text: Some(format!("#[{}]", name)),
 sort_text: Some(name.to_string()),
 filter_text: Some(name.to_string()),
 }).collect()
}