/*
 * Nuva OS - SystemLibrary - Lang
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


use super::token::Token;
use super::lexer::Lexer;

/// scanresult
pub struct ScanResult {
 /// Token Array
 pub tokens: &'static [Token],
 /// Errorcount
 pub error_count: u32,
 /// ifSuccess
 pub success: bool,
}

/// scandevice
pub struct Scanner {
 /// Lexical Analyzer
 lexer: Option<Lexer>,
 /// Token Buffer
 token_buffer: [Option<Token>; 1024],
 /// Token count
 token_count: u32,
 /// Errorcount
 error_count: u32,
}

impl Scanner {
 pub const fn new() -> Self {
 Scanner {
 lexer: None,
 token_buffer: [None; 1024],
 token_count: 0,
 error_count: 0,
 }
 }
 
 /// Initializescandevice
 pub fn init(&mut self, source: &'static str) {
 self.lexer = Some(Lexer::new(source));
 self.token_count = 0;
 self.error_count = 0;
 }
 
 /// scanplacefinite Token
 pub fn scan_all(&mut self) -> ScanResult {
 // ClearBuffer
 self.token_count = 0;
 self.error_count = 0;
 
 if let Some(ref mut lexer) = self.lexer {
 loop {
 let token = lexer.next_token();
 
 // CheckError
 if token.token_type == super::token::TokenType::Error {
 self.error_count += 1;
 }
 
 // CheckiftoreachfinalTail
 let is_eof = token.token_type == super::token::TokenType::Eof;
 
 // addPlustoBuffer
 if (self.token_count as usize) < self.token_buffer.len() {
 self.token_buffer[self.token_count as usize] = Some(token);
 self.token_count += 1;
 }
 
 if is_eof {
 break;
 }
 }
 }
 
 ScanResult {
 tokens: &[],
 error_count: self.error_count,
 success: self.error_count == 0,
 }
 }
 
 /// Get Token count
 pub fn get_token_count(&self) -> u32 {
 self.token_count
 }
 
 /// GetErrorcount
 pub fn get_error_count(&self) -> u32 {
 self.error_count
 }
 
 /// Get Token
 pub fn get_token(&self, index: u32) -> Option<&Token> {
 if (index as usize) < self.token_buffer.len() {
 self.token_buffer[index as usize].as_ref()
 } else {
 None
 }
 }
}

/// Globalscandevice
static mut SCANNER: Scanner = Scanner::new();

pub fn get_scanner() -> &'static mut Scanner {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut SCANNER }
}

pub fn init_scanner() {
 log_info!("Scanner initialized");
}