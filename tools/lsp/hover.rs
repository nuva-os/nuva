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

// ! stopinformation

use super::{TextDocument, Position, SemanticAnalyzer, Hover, HoverContents, MarkupContent, MarkupKind, Range};

/// stopinformation
pub fn provide_hover(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Option<Hover> {
 // Getcurrentpositionplacement symbolsignal
 let symbol = analyzer.get_symbol(&doc.uri, position)?;
 
 // Buildstopinside
 let mut content = String::new();
 
 // addPlussymbolsignalTypesumname
 content.push_str(&format!("{} **{}**", symbol_kind_str(symbol.kind), symbol.name));
 
 // addPlusfineinformation
 if let Some(ref detail) = symbol.detail {
 content.push_str(&format!("

{}", detail));
 }
 
 // addPlusDocumentation
 if let Some(ref docs) = symbol.documentation {
 content.push_str(&format!("

---

{}", docs));
 }
 
 Some(Hover {
 contents: HoverContents::Markup(MarkupContent {
 kind: MarkupKind::Markdown,
 value: content,
 }),
 range: Some(symbol.range),
 })
}

/// symbolsignalTypeString
fn symbol_kind_str(kind: super::SymbolKind) -> &'static str {
 match kind {
 super::SymbolKind::File => "file",
 super::SymbolKind::Module => "module",
 super::SymbolKind::Namespace => "namespace",
 super::SymbolKind::Package => "package",
 super::SymbolKind::Class => "class",
 super::SymbolKind::Method => "method",
 super::SymbolKind::Property => "property",
 super::SymbolKind::Field => "field",
 super::SymbolKind::Constructor => "constructor",
 super::SymbolKind::Enum => "enum",
 super::SymbolKind::Interface => "interface",
 super::SymbolKind::Function => "function",
 super::SymbolKind::Variable => "variable",
 super::SymbolKind::Constant => "const",
 super::SymbolKind::String => "string",
 super::SymbolKind::Number => "number",
 super::SymbolKind::Boolean => "bool",
 super::SymbolKind::Array => "array",
 super::SymbolKind::Object => "object",
 super::SymbolKind::Key => "key",
 super::SymbolKind::Null => "null",
 super::SymbolKind::EnumMember => "enum member",
 super::SymbolKind::Struct => "struct",
 super::SymbolKind::Event => "event",
 super::SymbolKind::Operator => "operator",
 super::SymbolKind::TypeParameter => "type parameter",
 }
}

/// GetTypeinformation
pub fn get_type_info(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Option<TypeInfo> {
 let symbol = analyzer.get_symbol(&doc.uri, position)?;
 
 Some(TypeInfo {
 name: symbol.name.clone(),
 kind: symbol.kind,
 detail: symbol.detail.clone(),
 })
}

/// typeinformation
#[derive(Debug, Clone)]
pub struct TypeInfo {
 pub name: String,
 pub kind: super::SymbolKind,
 pub detail: Option<String>,
}

/// GetFunctionSignature
pub fn get_function_signature(doc: &TextDocument, position: Position, analyzer: &SemanticAnalyzer) -> Option<FunctionSignature> {
 let symbol = analyzer.get_symbol(&doc.uri, position)?;
 
 // TODO: Parse function signature
 
 Some(FunctionSignature {
 name: symbol.name.clone(),
 parameters: vec![],
 return_type: None,
 documentation: symbol.documentation.clone(),
 })
}

/// FunctionSignature
#[derive(Debug, Clone)]
pub struct FunctionSignature {
 pub name: String,
 pub parameters: Vec<Parameter>,
 pub return_type: Option<String>,
 pub documentation: Option<String>,
}

/// parameter
#[derive(Debug, Clone)]
pub struct Parameter {
 pub name: String,
 pub type_: String,
 pub documentation: Option<String>,
}