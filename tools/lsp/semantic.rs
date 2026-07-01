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

// ! languagemeaninghighbright

use super::{TextDocument, SemanticAnalyzer, SemanticTokens, Position, Range};
use alloc::vec;
use alloc::vec::Vec;

/// languagemeaningTokenType
#[derive(Debug, Clone, Copy)]
pub enum SemanticTokenType {
 Namespace,
 Type,
 Class,
 Enum,
 Interface,
 Struct,
 TypeParameter,
 Parameter,
 Variable,
 Property,
 EnumMember,
 Event,
 Function,
 Method,
 Macro,
 Keyword,
 Modifier,
 Comment,
 String,
 Number,
 Regexp,
 Operator,
}

impl SemanticTokenType {
 pub fn as_str(&self) -> &'static str {
 match self {
 Self::Namespace => "namespace",
 Self::Type => "type",
 Self::Class => "class",
 Self::Enum => "enum",
 Self::Interface => "interface",
 Self::Struct => "struct",
 Self::TypeParameter => "typeParameter",
 Self::Parameter => "parameter",
 Self::Variable => "variable",
 Self::Property => "property",
 Self::EnumMember => "enumMember",
 Self::Event => "event",
 Self::Function => "function",
 Self::Method => "method",
 Self::Macro => "macro",
 Self::Keyword => "keyword",
 Self::Modifier => "modifier",
 Self::Comment => "comment",
 Self::String => "string",
 Self::Number => "number",
 Self::Regexp => "regexp",
 Self::Operator => "operator",
 }
 }
}

/// languagemeaningTokenModifier
#[derive(Debug, Clone, Copy)]
pub enum SemanticTokenModifier {
 Declaration,
 Definition,
 Readonly,
 Static,
 Deprecated,
 Abstract,
 Async,
 Modification,
 Documentation,
 DefaultLibrary,
}

impl SemanticTokenModifier {
 pub fn as_str(&self) -> &'static str {
 match self {
 Self::Declaration => "declaration",
 Self::Definition => "definition",
 Self::Readonly => "readonly",
 Self::Static => "static",
 Self::Deprecated => "deprecated",
 Self::Abstract => "abstract",
 Self::Async => "async",
 Self::Modification => "modification",
 Self::Documentation => "documentation",
 Self::DefaultLibrary => "defaultLibrary",
 }
 }
}

/// ComputelanguagemeaningToken
pub fn compute_tokens(doc: &TextDocument, analyzer: &SemanticAnalyzer) -> SemanticTokens {
 let mut tokens = vec![];
 let mut builder = SemanticTokensBuilder::new();
 
 // parseDocumentationparallelgenerateToken
 for token in tokenize_document(doc) {
 builder.push(
 token.delta_line,
 token.delta_start,
 token.type_ as u32,
 token.modifiers,
 );
 }
 
 SemanticTokens {
 data: builder.data,
 }
}

/// TokenDocumentation
fn tokenize_document(doc: &TextDocument) -> Vec<SemanticToken> {
 let mut tokens = vec![];
 let mut line = 0u32;
 let mut character = 0u32;
 let mut prev_line = 0u32;
 let mut prev_char = 0u32;
 
 // simpleform wordlawAnalysis
 let mut current_token = String::new();
 let mut token_start = Position::default();
 
 for (i, c) in doc.text.char_indices() {
 if c == '
' {
 // ProcesscurrentToken
 if !current_token.is_empty() {
 if let Some(token) = classify_token(&current_token, token_start, line, character) {
 tokens.push(token);
 }
 current_token.clear();
 }
 
 line += 1;
 character = 0;
 continue;
 }
 
 if c.is_whitespace() {
 // ProcesscurrentToken
 if !current_token.is_empty() {
 if let Some(token) = classify_token(&current_token, token_start, line, character) {
 tokens.push(token);
 }
 current_token.clear();
 }
 character += 1;
 continue;
 }
 
 // startnewToken
 if current_token.is_empty() {
 token_start = Position { line, character };
 }
 
 current_token.push(c);
 character += 1;
 }
 
 // ProcessmostthenitemToken
 if !current_token.is_empty() {
 if let Some(token) = classify_token(&current_token, token_start, line, character) {
 tokens.push(token);
 }
 }
 
 // Computeincreasequantification
 let mut result = vec![];
 let mut prev_line = 0u32;
 let mut prev_char = 0u32;
 
 for mut token in tokens {
 let delta_line = token.line - prev_line;
 let delta_start = if delta_line == 0 {
 token.start - prev_char
 } else {
 token.start
 };
 
 token.delta_line = delta_line;
 token.delta_start = delta_start;
 
 prev_line = token.line;
 prev_char = token.start;
 
 result.push(token);
 }
 
 result
}

/// classifyToken
fn classify_token(text: &str, start: Position, line: u32, end_char: u32) -> Option<SemanticToken> {
 let type_ = if is_keyword(text) {
 SemanticTokenType::Keyword as u32
 } else if is_type(text) {
 SemanticTokenType::Type as u32
 } else if is_function(text) {
 SemanticTokenType::Function as u32
 } else if is_number(text) {
 SemanticTokenType::Number as u32
 } else if is_string(text) {
 SemanticTokenType::String as u32
 } else {
 SemanticTokenType::Variable as u32
 };
 
 Some(SemanticToken {
 line: start.line,
 start: start.character,
 length: end_char - start.character,
 type_,
 modifiers: 0,
 delta_line: 0,
 delta_start: 0,
 })
}

/// checkiswhetherisclosekeycharacter
fn is_keyword(text: &str) -> bool {
 matches!(text,
 "fn" | "let" | "const" | "mut" | "if" | "else" | "match" | "while" |
 "for" | "loop" | "return" | "break" | "continue" | "struct" | "enum" |
 "impl" | "trait" | "type" | "pub" | "mod" | "use" | "self" | "super" |
 "true" | "false" | "async" | "await"
 )
}

/// checkiswhetherisType
fn is_type(text: &str) -> bool {
 matches!(text,
 "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
 "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
 "f32" | "f64" | "bool" | "char" | "str" | "String"
 ) || text.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}

/// checkiswhetherisFunction
fn is_function(text: &str) -> bool {
 text.ends_with('(') || text == "println" || text == "print"
}

/// checkiswhetherisnumbercharacter
fn is_number(text: &str) -> bool {
 text.parse::<f64>().is_ok()
}

/// checkiswhetherisString
fn is_string(text: &str) -> bool {
 text.starts_with('"') && text.ends_with('"')
}

/// languagemeaningToken
#[derive(Debug, Clone)]
pub struct SemanticToken {
 pub line: u32,
 pub start: u32,
 pub length: u32,
 pub type_: u32,
 pub modifiers: u32,
 pub delta_line: u32,
 pub delta_start: u32,
}

/// languagemeaningTokenBuilddevice
pub struct SemanticTokensBuilder {
 pub data: Vec<u32>,
}

impl SemanticTokensBuilder {
 pub fn new() -> Self {
 Self { data: vec![] }
 }

 pub fn push(&mut self, delta_line: u32, delta_start: u32, type_: u32, modifiers: u32) {
 self.data.push(delta_line);
 self.data.push(delta_start);
 self.data.push(type_);
 self.data.push(modifiers);
 }
}

impl Default for SemanticTokensBuilder {
 fn default() -> Self {
 Self::new()
 }
}