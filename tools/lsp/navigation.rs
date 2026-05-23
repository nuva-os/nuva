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

// ! Codeconduct

use super::{TextDocument, Position, SemanticAnalyzer, Location, Range};

/// jumpbranchtofixedmeaning
pub fn goto_definition(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Option<Location> {
 // Getcurrentpositionplacement symbolsignal
 let symbol = analyzer.get_symbol(&doc.uri, position)?;
 
 // findfixedmeaningpositionplacement
 // searchsymbolsignalform,findtofixedmeaningpositionplacement
 for (uri, symbols) in &analyzer.symbols {
 for s in symbols {
 if s.name == symbol.name && s.kind == symbol.kind {
 // checkiswhetherisfixedmeaning(constantfixedmeaningisitemexit positionplacement)
 return Some(Location {
 uri: uri.clone(),
 range: s.range,
 });
 }
 }
 }
 
 Some(Location {
 uri: doc.uri.clone(),
 range: symbol.range,
 })
}

/// jumpbranchtoTypefixedmeaning
pub fn goto_type_definition(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Option<Location> {
 // Getcurrentpositionplacement symbolsignal
 let symbol = analyzer.get_symbol(&doc.uri, position)?;
 
 // findTypefixedmeaningpositionplacement
 // ifsymbolsignalisType,Returnitsfixedmeaningpositionplacement
 if symbol.kind == super::SymbolKind::Type || symbol.kind == super::SymbolKind::Struct || symbol.kind == super::SymbolKind::Enum {
 return Some(Location {
 uri: doc.uri.clone(),
 range: symbol.range,
 });
 }
 
 // ifsymbolsignalisVariableorFunction,finditsTypefixedmeaning
 if let Some(type_name) = symbol.detail.as_ref() {
 for (uri, symbols) in &analyzer.symbols {
 for s in symbols {
 if s.name == type_name && (s.kind == super::SymbolKind::Type || s.kind == super::SymbolKind::Struct) {
 return Some(Location {
 uri: uri.clone(),
 range: s.range,
 });
 }
 }
 }
 }
 
 None
}

/// findreference
pub fn find_references(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Vec<Location> {
 // Getcurrentpositionplacement symbolsignal
 let symbol = match analyzer.get_symbol(&doc.uri, position) {
 Some(s) => s,
 None => return vec![],
 };
 
 // findplacefinitereference
 let mut references = Vec::new();
 
 // traverseplacefiniteDocumentation symbolsignalform
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
 
 references.push(Location {
 uri: uri.clone(),
 range: Range {
 start: Position { line, character },
 end: Position { line, character: character + symbol.name.len() as u32 },
 },
 });
 }
 }
 }
 
 references
}

/// findImplementation
pub fn find_implementations(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Vec<Location> {
 // Getcurrentpositionplacement symbolsignal
 let symbol = match analyzer.get_symbol(&doc.uri, position) {
 Some(s) => s,
 None => return vec![],
 };
 
 // findplacefiniteImplementation
 let mut implementations = Vec::new();
 
 // ifis trait or interface,finditsImplementation
 if symbol.kind == super::SymbolKind::Interface || symbol.kind == super::SymbolKind::Type {
 for (uri, symbols) in &analyzer.symbols {
 for s in symbols {
 // find impl block
 if s.kind == super::SymbolKind::Method || s.kind == super::SymbolKind::Function {
 if let Some(detail) = s.detail.as_ref() {
 if detail.contains(&symbol.name) {
 implementations.push(Location {
 uri: uri.clone(),
 range: s.range,
 });
 }
 }
 }
 }
 }
 }
 
 implementations
}

/// Documentationsymbolsignal
pub fn document_symbols(doc: &TextDocument, analyzer: &SemanticAnalyzer) -> Vec<DocumentSymbol> {
 // GetDocumentationinfix placefinitesymbolsignal
 let mut symbols = Vec::new();
 
 if let Some(doc_symbols) = analyzer.symbols.get(&doc.uri) {
 for symbol in doc_symbols {
 symbols.push(DocumentSymbol {
 name: symbol.name.clone(),
 detail: symbol.detail.clone(),
 kind: symbol.kind,
 range: symbol.range,
 selection_range: symbol.selection_range,
 children: Vec::new(),
 });
 }
 }
 
 symbols
}

/// workmakezonesymbolsignal
pub fn workspace_symbols(query: &str, analyzer: &SemanticAnalyzer) -> Vec<SymbolInformation> {
 // searchworkmakezonesymbolsignal
 let mut symbols = Vec::new();
 
 // traverseplacefiniteDocumentation symbolsignalform
 for (uri, doc_symbols) in &analyzer.symbols {
 for symbol in doc_symbols {
 // modelMatchQueryString
 if symbol.name.to_lowercase().contains(&query.to_lowercase()) {
 symbols.push(SymbolInformation {
 name: symbol.name.clone(),
 kind: symbol.kind,
 location: Location {
 uri: uri.clone(),
 range: symbol.range,
 },
 container_name: None,
 });
 }
 }
 }
 
 symbols
}

/// tuneuselayertime
pub fn call_hierarchy(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Option<CallHierarchyItem> {
 // Getcurrentpositionplacement symbolsignal
 let symbol = analyzer.get_symbol(&doc.uri, position)?;
 
 Some(CallHierarchyItem {
 name: symbol.name.clone(),
 kind: symbol.kind,
 uri: doc.uri.clone(),
 range: symbol.range,
 selection_range: symbol.selection_range,
 })
}

/// Gettuneuseer
pub fn call_hierarchy_incoming(item: &CallHierarchyItem, analyzer: &SemanticAnalyzer) -> Vec<CallHierarchyIncomingCall> {
 let mut calls = Vec::new();
 
 // findtuneusecurrentFunction Function
 for (uri, symbols) in &analyzer.symbols {
 for symbol in symbols {
 if symbol.kind == super::SymbolKind::Function {
 // checkFunctionvolumeiswhethertuneuse currentFunction
 if let Some(detail) = symbol.detail.as_ref() {
 if detail.contains(&item.name) {
 calls.push(CallHierarchyIncomingCall {
 from: CallHierarchyItem {
 name: symbol.name.clone(),
 kind: symbol.kind,
 uri: uri.clone(),
 range: symbol.range,
 selection_range: symbol.selection_range,
 },
 from_ranges: vec![symbol.range],
 });
 }
 }
 }
 }
 }
 
 calls
}

/// Getbytuneuseer
pub fn call_hierarchy_outgoing(item: &CallHierarchyItem, analyzer: &SemanticAnalyzer) -> Vec<CallHierarchyOutgoingCall> {
 let mut calls = Vec::new();
 
 // findcurrentFunctiontuneuse Function
 if let Some(symbols) = analyzer.symbols.get(&item.uri) {
 for symbol in symbols {
 if symbol.name == item.name && symbol.kind == super::SymbolKind::Function {
 // checkFunctionvolume,taketuneuse Function
 if let Some(detail) = symbol.detail.as_ref() {
 // SimplifiedImplementation:secondary detail infixtakeFunctiontuneuse
 // realactualImplementationneedwantparseFunctionvolume
 for (uri, other_symbols) in &analyzer.symbols {
 for other_symbol in other_symbols {
 if other_symbol.kind == super::SymbolKind::Function {
 if detail.contains(&other_symbol.name) {
 calls.push(CallHierarchyOutgoingCall {
 to: CallHierarchyItem {
 name: other_symbol.name.clone(),
 kind: other_symbol.kind,
 uri: uri.clone(),
 range: other_symbol.range,
 selection_range: other_symbol.selection_range,
 },
 from_ranges: vec![symbol.range],
 });
 }
 }
 }
 }
 }
 }
 }
 }
 
 calls
}

/// Documentationsymbolsignal
#[derive(Debug, Clone)]
pub struct DocumentSymbol {
 pub name: String,
 pub detail: Option<String>,
 pub kind: super::SymbolKind,
 pub range: Range,
 pub selection_range: Range,
 pub children: Vec<DocumentSymbol>,
}

/// symbolsignalinformation
#[derive(Debug, Clone)]
pub struct SymbolInformation {
 pub name: String,
 pub kind: super::SymbolKind,
 pub location: Location,
 pub container_name: Option<String>,
}

/// tuneuselayertimeproject
#[derive(Debug, Clone)]
pub struct CallHierarchyItem {
 pub name: String,
 pub kind: super::SymbolKind,
 pub uri: String,
 pub range: Range,
 pub selection_range: Range,
}

/// entertuneuse
#[derive(Debug, Clone)]
pub struct CallHierarchyIncomingCall {
 pub from: CallHierarchyItem,
 pub from_ranges: Vec<Range>,
}

/// exittuneuse
#[derive(Debug, Clone)]
pub struct CallHierarchyOutgoingCall {
 pub to: CallHierarchyItem,
 pub from_ranges: Vec<Range>,
}