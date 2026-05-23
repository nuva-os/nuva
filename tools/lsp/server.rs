/* * Nuva OS - Tools - Lsp
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

//! LSP Server

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// LSP Version
pub const LSP_VERSION: &str = "3.16.0";

/// ClientcanForce
#[derive(Debug, Clone)]
pub struct ClientCapabilities {
 pub text_document: TextDocumentClientCapabilities,
 pub workspace: WorkspaceClientCapabilities,
}

#[derive(Debug, Clone, Default)]
pub struct TextDocumentClientCapabilities {
 pub completion: CompletionCapabilities,
 pub hover: HoverCapabilities,
 pub definition: DefinitionCapabilities,
 pub references: ReferencesCapabilities,
 pub document_symbol: DocumentSymbolCapabilities,
 pub publish_diagnostics: PublishDiagnosticsCapabilities,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionCapabilities {
 pub completion_item: CompletionItemCapabilities,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionItemCapabilities {
 pub snippet_support: bool,
 pub documentation_format: [u8; 16],
 pub documentation_format_len: u8,
}

#[derive(Debug, Clone, Default)]
pub struct HoverCapabilities {
 pub content_format: [u8; 16],
 pub content_format_len: u8,
}

#[derive(Debug, Clone, Default)]
pub struct DefinitionCapabilities {
 pub link_support: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ReferencesCapabilities {}

#[derive(Debug, Clone, Default)]
pub struct DocumentSymbolCapabilities {
 pub hierarchical_document_symbol_support: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PublishDiagnosticsCapabilities {
 pub related_information: bool,
 pub version_support: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceClientCapabilities {
 pub symbol: SymbolCapabilities,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolCapabilities {}

/// Position
#[derive(Debug, Clone, Copy)]
pub struct Position {
 pub line: u32,
 pub character: u32,
}

impl Position {
 pub fn new(line: u32, character: u32) -> Self {
 Self { line, character }
 }
}

/// Range
#[derive(Debug, Clone, Copy)]
pub struct Range {
 pub start: Position,
 pub end: Position,
}

impl Range {
 pub fn new(start: Position, end: Position) -> Self {
 Self { start, end }
 }
}

/// textencoding
#[derive(Debug, Clone)]
pub struct TextEdit {
 pub range: Range,
 pub new_text: [u8; 256],
 pub new_text_len: u8,
}

impl TextEdit {
 pub fn new(range: Range, text: &[u8]) -> Self {
 let mut buf = [0u8; 256];
 let len = text.len().min(255);
 buf[..len].copy_from_slice(&text[..len]);
 
 Self {
 range,
 new_text: buf,
 new_text_len: len as u8,
 }
 }
}

/// break
#[derive(Debug, Clone)]
pub struct Diagnostic {
 pub range: Range,
 pub message: [u8; 256],
 pub message_len: u8,
 pub severity: DiagnosticSeverity,
 pub code: [u8; 32],
 pub code_len: u8,
 pub source: [u8; 32],
 pub source_len: u8,
}

/// breakstrictrepeatprocessDegree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DiagnosticSeverity {
 Error = 1,
 Warning = 2,
 Information = 3,
 Hint = 4,
}

impl Diagnostic {
 pub fn error(range: Range, message: &[u8]) -> Self {
 let mut msg_buf = [0u8; 256];
 let len = message.len().min(255);
 msg_buf[..len].copy_from_slice(&message[..len]);
 
 Self {
 range,
 message: msg_buf,
 message_len: len as u8,
 severity: DiagnosticSeverity::Error,
 code: [0; 32],
 code_len: 0,
 source: *b"Nuva\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
 source_len: 4,
 }
 }

 pub fn warning(range: Range, message: &[u8]) -> Self {
 let mut d = Self::error(range, message);
 d.severity = DiagnosticSeverity::Warning;
 d
 }

 pub fn hint(range: Range, message: &[u8]) -> Self {
 let mut d = Self::error(range, message);
 d.severity = DiagnosticSeverity::Hint;
 d
 }
}

/// patchallproject
#[derive(Debug, Clone)]
pub struct CompletionItem {
 pub label: [u8; 64],
 pub label_len: u8,
 pub kind: CompletionItemKind,
 pub detail: [u8; 128],
 pub detail_len: u8,
 pub documentation: [u8; 512],
 pub documentation_len: u16,
 pub insert_text: [u8; 256],
 pub insert_text_len: u8,
 pub sort_text: [u8; 16],
 pub sort_text_len: u8,
}

/// patchallprojectType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompletionItemKind {
 Text = 1,
 Method = 2,
 Function = 3,
 Constructor = 4,
 Field = 5,
 Variable = 6,
 Class = 7,
 Interface = 8,
 Module = 9,
 Property = 10,
 Keyword = 11,
 Snippet = 15,
 Enum = 13,
 EnumMember = 22,
 Struct = 22,
 TypeParameter = 25,
}

impl CompletionItem {
 pub fn new(label: &[u8], kind: CompletionItemKind) -> Self {
 let mut label_buf = [0u8; 64];
 let len = label.len().min(63);
 label_buf[..len].copy_from_slice(&label[..len]);
 
 Self {
 label: label_buf,
 label_len: len as u8,
 kind,
 detail: [0; 128],
 detail_len: 0,
 documentation: [0; 512],
 documentation_len: 0,
 insert_text: label_buf,
 insert_text_len: len as u8,
 sort_text: [0; 16],
 sort_text_len: 0,
 }
 }

 pub fn function(name: &[u8]) -> Self {
 Self::new(name, CompletionItemKind::Function)
 }

 pub fn class(name: &[u8]) -> Self {
 Self::new(name, CompletionItemKind::Class)
 }

 pub fn variable(name: &[u8]) -> Self {
 Self::new(name, CompletionItemKind::Variable)
 }

 pub fn keyword(name: &[u8]) -> Self {
 Self::new(name, CompletionItemKind::Keyword)
 }

 pub fn set_detail(&mut self, detail: &[u8]) {
 let len = detail.len().min(127);
 self.detail[..len].copy_from_slice(&detail[..len]);
 self.detail_len = len as u8;
 }

 pub fn set_documentation(&mut self, doc: &[u8]) {
 let len = doc.len().min(511);
 self.documentation[..len].copy_from_slice(&doc[..len]);
 self.documentation_len = len as u16;
 }
}

/// patchallList
#[derive(Debug)]
pub struct CompletionList {
 pub items: [CompletionItem; 128],
 pub num_items: u8,
 pub is_incomplete: bool,
}

impl CompletionList {
 pub fn new() -> Self {
 Self {
 items: [CompletionItem::new(b"", CompletionItemKind::Text); 128],
 num_items: 0,
 is_incomplete: false,
 }
 }

 pub fn add(&mut self, item: CompletionItem) {
 if self.num_items < 128 {
 self.items[self.num_items as usize] = item;
 self.num_items += 1;
 }
 }
}

/// PositionInfo
#[derive(Debug, Clone)]
pub struct Location {
 pub uri: [u8; 256],
 pub uri_len: u8,
 pub range: Range,
}

impl Location {
 pub fn new(uri: &[u8], range: Range) -> Self {
 let mut uri_buf = [0u8; 256];
 let len = uri.len().min(255);
 uri_buf[..len].copy_from_slice(&uri[..len]);
 
 Self {
 uri: uri_buf,
 uri_len: len as u8,
 range,
 }
 }
}

/// SignInfo
#[derive(Debug, Clone)]
pub struct SymbolInformation {
 pub name: [u8; 64],
 pub name_len: u8,
 pub kind: SymbolKind,
 pub location: Location,
 pub container_name: [u8; 64],
 pub container_name_len: u8,
}

/// SignType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SymbolKind {
 File = 1,
 Module = 2,
 Namespace = 3,
 Package = 4,
 Class = 5,
 Method = 6,
 Property = 7,
 Field = 8,
 Constructor = 9,
 Enum = 10,
 Interface = 11,
 Function = 12,
 Variable = 13,
 Constant = 14,
 Struct = 23,
}

/// Documentation
#[derive(Debug)]
pub struct TextDocument {
 pub uri: [u8; 256],
 pub uri_len: u8,
 pub language_id: [u8; 32],
 pub language_id_len: u8,
 pub version: AtomicU32,
 pub content: [u8; 65536],
 pub content_len: AtomicU32,
}

impl TextDocument {
 pub fn new(uri: &[u8], language_id: &[u8]) -> Self {
 let mut uri_buf = [0u8; 256];
 let uri_len = uri.len().min(255);
 uri_buf[..uri_len].copy_from_slice(&uri[..uri_len]);
 
 let mut lang_buf = [0u8; 32];
 let lang_len = language_id.len().min(31);
 lang_buf[..lang_len].copy_from_slice(&language_id[..lang_len]);
 
 Self {
 uri: uri_buf,
 uri_len: uri_len as u8,
 language_id: lang_buf,
 language_id_len: lang_len as u8,
 version: AtomicU32::new(0),
 content: [0; 65536],
 content_len: AtomicU32::new(0),
 }
 }

 pub fn set_content(&mut self, content: &[u8]) {
 let len = content.len().min(65535);
 self.content[..len].copy_from_slice(&content[..len]);
 self.content_len.store(len as u32, Ordering::Release);
 self.version.fetch_add(1, Ordering::Release);
 }

 pub fn get_content(&self) -> &[u8] {
 &self.content[..self.content_len.load(Ordering::Acquire) as usize]
 }

 pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
 let content = self.get_content();
 let mut new_content = [0u8; 65536];
 let mut new_len = 0;
 
 // Applicationchangeupdate
 if let Some(range) = change.range {
 // Rangechangeupdate
 let start_offset = self.position_to_offset(range.start);
 let end_offset = self.position_to_offset(range.end);
 
 // Copychangeupdateprefix inside
 new_content[..start_offset].copy_from_slice(&content[..start_offset]);
 new_len = start_offset;
 
 // Insertnewinside
 let text_len = change.text_len as usize;
 new_content[new_len..new_len + text_len].copy_from_slice(&change.text[..text_len]);
 new_len += text_len;
 
 // Copychangeupdatethen inside
 let remaining = content.len() - end_offset;
 new_content[new_len..new_len + remaining].copy_from_slice(&content[end_offset..]);
 new_len += remaining;
 } else {
 // alltextReplace
 let len = change.text_len as usize;
 new_content[..len].copy_from_slice(&change.text[..len]);
 new_len = len;
 }
 
 self.content[..new_len].copy_from_slice(&new_content[..new_len]);
 self.content_len.store(new_len as u32, Ordering::Release);
 self.version.fetch_add(1, Ordering::Release);
 }

 fn position_to_offset(&self, pos: Position) -> usize {
 let content = self.get_content();
 let mut offset = 0;
 let mut line = 0;
 
 while offset < content.len() && line < pos.line {
 if content[offset] == b'
' {
 line += 1;
 }
 offset += 1;
 }
 
 offset + pos.character as usize
 }
}

/// DocumentationchangeupdateEvent
#[derive(Debug, Clone)]
pub struct TextDocumentContentChangeEvent {
 pub range: Option<Range>,
 pub range_length: Option<u32>,
 pub text: [u8; 65536],
 pub text_len: u16,
}

/// LSP Server
pub struct LspServer {
 documents: [Option<TextDocument>; 64],
 num_documents: AtomicU32,
 capabilities: ClientCapabilities,
 initialized: AtomicU32,
}

impl LspServer {
 pub fn new() -> Self {
 Self {
 documents: [None; 64],
 num_documents: AtomicU32::new(0),
 capabilities: ClientCapabilities {
 text_document: TextDocumentClientCapabilities::default(),
 workspace: WorkspaceClientCapabilities::default(),
 },
 initialized: AtomicU32::new(0),
 }
 }

 pub fn initialize(&mut self, capabilities: ClientCapabilities) {
 self.capabilities = capabilities;
 self.initialized.store(1, Ordering::Release);
 }

 pub fn is_initialized(&self) -> bool {
 self.initialized.load(Ordering::Relaxed) != 0
 }

 /// OpenDocumentation
 pub fn open_document(&mut self, uri: &[u8], language_id: &[u8], content: &[u8]) {
 let mut doc = TextDocument::new(uri, language_id);
 doc.set_content(content);
 
 let idx = self.num_documents.load(Ordering::Relaxed) as usize;
 if idx < 64 {
 self.documents[idx] = Some(doc);
 self.num_documents.fetch_add(1, Ordering::Release);
 }
 }

 /// CloseDocumentation
 pub fn close_document(&mut self, uri: &[u8]) {
 for i in 0..self.num_documents.load(Ordering::Relaxed) as usize {
 if let Some(ref doc) = self.documents[i] {
 if &doc.uri[..doc.uri_len as usize] == uri {
 self.documents[i] = None;
 return;
 }
 }
 }
 }

 /// GetDocumentation
 pub fn get_document(&self, uri: &[u8]) -> Option<&TextDocument> {
 for i in 0..self.num_documents.load(Ordering::Relaxed) as usize {
 if let Some(ref doc) = self.documents[i] {
 if &doc.uri[..doc.uri_len as usize] == uri {
 return Some(doc);
 }
 }
 }
 None
 }

 /// GetDocumentation(canchange)
 pub fn get_document_mut(&mut self, uri: &[u8]) -> Option<&mut TextDocument> {
 for i in 0..self.num_documents.load(Ordering::Relaxed) as usize {
 if let Some(ref mut doc) = self.documents[i] {
 if &doc.uri[..doc.uri_len as usize] == uri {
 return Some(doc);
 }
 }
 }
 None
 }

 /// UpdateDocumentation
 pub fn update_document(&mut self, uri: &[u8], change: &TextDocumentContentChangeEvent) {
 if let Some(doc) = self.get_document_mut(uri) {
 doc.apply_change(change);
 }
 }
}