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


use super::token::{Token, TokenType, Keyword, TokenValue};

/// Lexical Analyzer
pub struct Lexer {
 /// sourceCode
 source: &'static str,
 /// CurrentPosition
 position: usize,
 /// Currentrow
 line: u32,
 /// Currentcolumn
 column: u32,
 /// iftoreachfinalTail
 at_end: bool,
}

impl Lexer {
 /// CreatenewLexical Analyzer
 pub fn new(source: &'static str) -> Self {
 Lexer {
 source,
 position: 0,
 line: 1,
 column: 1,
 at_end: false,
 }
 }
 
 /// GetNext Token
 pub fn next_token(&mut self) -> Token {
 // jumpoveremptywhiteCharacter
 self.skip_whitespace();
 
 // CheckiftoreachfinalTail
 if self.at_end {
 return Token::new(TokenType::Eof, self.line, self.column);
 }
 
 let start_line = self.line;
 let start_column = self.column;
 let start_pos = self.position;
 
 // GetCurrentCharacter
 let ch = self.advance();
 
 // RootevidenceCharacterTypegenerate Token
 match ch {
 // formCharacter Token
 '(' => Token::new(TokenType::LeftParen, start_line, start_column),
 ')' => Token::new(TokenType::RightParen, start_line, start_column),
 '[' => Token::new(TokenType::LeftBracket, start_line, start_column),
 ']' => Token::new(TokenType::RightBracket, start_line, start_column),
 '{' => Token::new(TokenType::LeftBrace, start_line, start_column),
 '}' => Token::new(TokenType::RightBrace, start_line, start_column),
 ',' => Token::new(TokenType::Comma, start_line, start_column),
 ';' => Token::new(TokenType::Semicolon, start_line, start_column),
 ':' => {
 if self.match_char(':') {
 Token::new(TokenType::DoubleColon, start_line, start_column)
 } else {
 Token::new(TokenType::Colon, start_line, start_column)
 }
 }
 '.' => Token::new(TokenType::Dot, start_line, start_column),
 
 // Operator
 '+' => {
 if self.match_char('=') {
 Token::new(TokenType::PlusAssign, start_line, start_column)
 } else {
 Token::new(TokenType::Plus, start_line, start_column)
 }
 }
 '-' => {
 if self.match_char('>') {
 Token::new(TokenType::Arrow, start_line, start_column)
 } else if self.match_char('=') {
 Token::new(TokenType::MinusAssign, start_line, start_column)
 } else {
 Token::new(TokenType::Minus, start_line, start_column)
 }
 }
 '*' => {
 if self.match_char('=') {
 Token::new(TokenType::StarAssign, start_line, start_column)
 } else {
 Token::new(TokenType::Star, start_line, start_column)
 }
 }
 '/' => {
 if self.match_char('=') {
 Token::new(TokenType::SlashAssign, start_line, start_column)
 } else if self.match_char('/') {
 // formrowComment
 self.skip_line_comment();
 self.next_token()
 } else if self.match_char('*') {
 // manyrowComment
 self.skip_block_comment();
 self.next_token()
 } else {
 Token::new(TokenType::Slash, start_line, start_column)
 }
 }
 '%' => Token::new(TokenType::Percent, start_line, start_column),
 
 '=' => {
 if self.match_char('=') {
 Token::new(TokenType::Equal, start_line, start_column)
 } else {
 Token::new(TokenType::Assign, start_line, start_column)
 }
 }
 '!' => {
 if self.match_char('=') {
 Token::new(TokenType::NotEqual, start_line, start_column)
 } else {
 Token::new(TokenType::Not, start_line, start_column)
 }
 }
 '<' => {
 if self.match_char('=') {
 Token::new(TokenType::LessEqual, start_line, start_column)
 } else if self.match_char('<') {
 Token::new(TokenType::LeftShift, start_line, start_column)
 } else {
 Token::new(TokenType::Less, start_line, start_column)
 }
 }
 '>' => {
 if self.match_char('=') {
 Token::new(TokenType::GreaterEqual, start_line, start_column)
 } else if self.match_char('>') {
 Token::new(TokenType::RightShift, start_line, start_column)
 } else {
 Token::new(TokenType::Greater, start_line, start_column)
 }
 }
 '&' => {
 if self.match_char('&') {
 Token::new(TokenType::And, start_line, start_column)
 } else {
 Token::new(TokenType::BitAnd, start_line, start_column)
 }
 }
 '|' => {
 if self.match_char('>') {
 Token::new(TokenType::Pipeline, start_line, start_column)
 } else if self.match_char('|') {
 Token::new(TokenType::Or, start_line, start_column)
 } else {
 Token::new(TokenType::BitOr, start_line, start_column)
 }
 }
 '^' => Token::new(TokenType::BitXor, start_line, start_column),
 '~' => Token::new(TokenType::BitNot, start_line, start_column),
 
 // String
 '"' => self.read_string(start_line, start_column),
 
 // Character
 '\'' => self.read_char(start_line, start_column),
 
 // numberWord
 '0'..='9' => {
 self.position = start_pos;
 self.column = start_column;
 self.read_number()
 }
 
 // IdentifiersumKeyword
 'a'..='z' | 'A'..='Z' | '_' => {
 self.position = start_pos;
 self.column = start_column;
 self.read_identifier()
 }
 
 // Character
 _ => Token::error("Unexpected character", start_line, start_column),
 }
 }
 
 /// ReadString
 fn read_string(&mut self, start_line: u32, start_column: u32) -> Token {
 let start_pos = self.position;
 while !self.at_end {
 let ch = self.advance();
 if ch == '"' {
 let len = (self.position - start_pos) as u32;
 let text = &self.source[start_pos..self.position - 1];
 return Token::string(text, start_line, start_column, len);
 }
 if ch == '\\' {
 if self.at_end {
 break;
 }
 self.advance();
 }
 if ch == '\n' {
 self.line += 1;
 self.column = 0;
 }
 }
 Token::error("Unterminated string", start_line, start_column)
 }
 
 /// ReadCharacter
 fn read_char(&mut self, start_line: u32, start_column: u32) -> Token {
 if self.at_end {
 return Token::error("Unterminated character", start_line, start_column);
 }
 let ch = self.advance();
 let value = if ch == '\\' {
 if self.at_end {
 return Token::error("Unterminated character escape", start_line, start_column);
 }
 self.read_escape()
 } else {
 ch
 };
 if self.at_end || self.peek() != '\'' {
 return Token::error("Unterminated character", start_line, start_column);
 }
 self.advance();
 Token {
 token_type: TokenType::Char,
 keyword: None,
 value: TokenValue::Char(value),
 line: start_line,
 column: start_column,
 length: 3,
 }
 }
 
 /// Read escape character after backslash
 fn read_escape(&mut self) -> char {
 let ch = self.advance();
 match ch {
 'n' => '\n',
 't' => '\t',
 'r' => '\r',
 '0' => '\0',
 '\\' => '\\',
 '\'' => '\'',
 '"' => '"',
 _ => ch,
 }
 }
 
 /// ReadnumberWord
 fn read_number(&mut self) -> Token {
 let start_pos = self.position;
 let start_column = self.column;
 
 if self.peek() == '0' {
 self.advance();
 let next = self.peek();
 if next == 'x' || next == 'X' {
 self.advance();
 return self.read_hex_number(start_pos, self.line, start_column);
 }
 if next == 'b' || next == 'B' {
 self.advance();
 return self.read_binary_number(start_pos, self.line, start_column);
 }
 if next == 'o' || next == 'O' {
 self.advance();
 return self.read_octal_number(start_pos, self.line, start_column);
 }
 self.position = start_pos;
 self.column = start_column;
 }
 
 let mut is_float = false;
 while !self.at_end {
 let ch = self.peek();
 if ch == '.' && !is_float {
 is_float = true;
 self.advance();
 continue;
 }
 if ch >= '0' && ch <= '9' {
 self.advance();
 continue;
 }
 if ch == 'e' || ch == 'E' {
 is_float = true;
 self.advance();
 if !self.at_end && (self.peek() == '+' || self.peek() == '-') {
 self.advance();
 }
 continue;
 }
 break;
 }
 
 let text = &self.source[start_pos..self.position];
 let len = (self.position - start_pos) as u32;
 
 if is_float {
 match text.parse::<f64>() {
 Ok(v) => Token::float(v, self.line, start_column, len),
 Err(_) => Token::error("Invalid float literal", self.line, start_column),
 }
 } else {
 match text.parse::<i64>() {
 Ok(v) => Token::integer(v, self.line, start_column, len),
 Err(_) => Token::error("Invalid integer literal", self.line, start_column),
 }
 }
 }
 
 /// Read hexadecimal number (after 0x prefix)
 fn read_hex_number(&mut self, start_pos: usize, start_line: u32, start_column: u32) -> Token {
 let num_start = self.position;
 while !self.at_end {
 let ch = self.peek();
 if (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F') || ch == '_' {
 self.advance();
 } else {
 break;
 }
 }
 if self.position == num_start {
 return Token::error("Invalid hex literal", start_line, start_column);
 }
 let text = &self.source[num_start..self.position];
 let len = (self.position - start_pos) as u32;
 let mut value: i64 = 0;
 for ch in text.chars() {
 if ch == '_' { continue; }
 value = value.wrapping_mul(16);
 if ch >= '0' && ch <= '9' {
 value = value.wrapping_add((ch as i64) - ('0' as i64));
 } else if ch >= 'a' && ch <= 'f' {
 value = value.wrapping_add((ch as i64) - ('a' as i64) + 10);
 } else {
 value = value.wrapping_add((ch as i64) - ('A' as i64) + 10);
 }
 }
 Token::integer(value, start_line, start_column, len)
 }
 
 /// Read binary number (after 0b prefix)
 fn read_binary_number(&mut self, start_pos: usize, start_line: u32, start_column: u32) -> Token {
 let num_start = self.position;
 while !self.at_end {
 let ch = self.peek();
 if ch == '0' || ch == '1' || ch == '_' {
 self.advance();
 } else {
 break;
 }
 }
 if self.position == num_start {
 return Token::error("Invalid binary literal", start_line, start_column);
 }
 let text = &self.source[num_start..self.position];
 let len = (self.position - start_pos) as u32;
 let mut value: i64 = 0;
 for ch in text.chars() {
 if ch == '_' { continue; }
 value = value.wrapping_shl(1);
 if ch == '1' {
 value = value.wrapping_add(1);
 }
 }
 Token::integer(value, start_line, start_column, len)
 }
 
 /// Read octal number (after 0o prefix)
 fn read_octal_number(&mut self, start_pos: usize, start_line: u32, start_column: u32) -> Token {
 let num_start = self.position;
 while !self.at_end {
 let ch = self.peek();
 if (ch >= '0' && ch <= '7') || ch == '_' {
 self.advance();
 } else {
 break;
 }
 }
 if self.position == num_start {
 return Token::error("Invalid octal literal", start_line, start_column);
 }
 let text = &self.source[num_start..self.position];
 let len = (self.position - start_pos) as u32;
 let mut value: i64 = 0;
 for ch in text.chars() {
 if ch == '_' { continue; }
 value = value.wrapping_mul(8).wrapping_add((ch as i64) - ('0' as i64));
 }
 Token::integer(value, start_line, start_column, len)
 }
 
 /// ReadIdentifier
 fn read_identifier(&mut self) -> Token {
 let start_pos = self.position;
 let start_column = self.column;
 
 while !self.at_end {
 let ch = self.peek();
 if Self::is_identifier_char(ch) {
 self.advance();
 } else {
 break;
 }
 }
 
 let text = &self.source[start_pos..self.position];
 let len = (self.position - start_pos) as u32;
 
 let keyword = Self::lookup_keyword(text);
 if let Some(kw) = keyword {
 Token::keyword(kw, self.line, start_column, len)
 } else {
 Token::identifier(text, self.line, start_column, len)
 }
 }
 
 /// Check if character is valid in identifier
 fn is_identifier_char(ch: char) -> bool {
 (ch >= 'a' && ch <= 'z')
 || (ch >= 'A' && ch <= 'Z')
 || (ch >= '0' && ch <= '9')
 || ch == '_'
 }
 
 /// Lookup keyword from identifier text
 fn lookup_keyword(text: &str) -> Option<Keyword> {
 match text {
 "let" => Some(Keyword::Let),
 "var" => Some(Keyword::Var),
 "const" => Some(Keyword::Const),
 "fn" => Some(Keyword::Fn),
 "struct" => Some(Keyword::Struct),
 "enum" => Some(Keyword::Enum),
 "trait" => Some(Keyword::Trait),
 "impl" => Some(Keyword::Impl),
 "type" => Some(Keyword::Type),
 "if" => Some(Keyword::If),
 "else" => Some(Keyword::Else),
 "match" => Some(Keyword::Match),
 "while" => Some(Keyword::While),
 "for" => Some(Keyword::For),
 "in" => Some(Keyword::In),
 "loop" => Some(Keyword::Loop),
 "break" => Some(Keyword::Break),
 "continue" => Some(Keyword::Continue),
 "return" => Some(Keyword::Return),
 "i8" | "i16" | "i32" | "i64" | "i128" => Some(Keyword::Int),
 "u8" | "u16" | "u32" | "u64" | "u128" => Some(Keyword::Uint),
 "f32" | "f64" => Some(Keyword::Float),
 "bool" => Some(Keyword::Bool),
 "char" => Some(Keyword::Char),
 "str" => Some(Keyword::Str),
 "pub" => Some(Keyword::Pub),
 "priv" => Some(Keyword::Priv),
 "mut" => Some(Keyword::Mut),
 "static" => Some(Keyword::Static),
 "async" => Some(Keyword::Async),
 "await" => Some(Keyword::Await),
 "component" => Some(Keyword::Component),
 "signal" => Some(Keyword::Signal),
 "effect" => Some(Keyword::Effect),
 "reactive" => Some(Keyword::Reactive),
 "resource" => Some(Keyword::Resource),
 "with" => Some(Keyword::With),
 "use" => Some(Keyword::Use),
 "mod" => Some(Keyword::Mod),
 "self" => Some(Keyword::Self_),
 "super" => Some(Keyword::Super),
 "true" => Some(Keyword::True),
 "false" => Some(Keyword::False),
 "None" => Some(Keyword::None),
 "Some" => Some(Keyword::Some),
 _ => None,
 }
 }
 
 /// jumpoveremptywhiteCharacter
 fn skip_whitespace(&mut self) {
 while !self.at_end {
 match self.peek() {
 ' ' | '\t' | '\r' => {
 self.advance();
 }
 '
' => {
 self.line += 1;
 self.column = 0;
 self.advance();
 }
 _ => break,
 }
 }
 }
 
 /// jumpoverformrowComment
 fn skip_line_comment(&mut self) {
 while !self.at_end && self.peek() != '
' {
 self.advance();
 }
 }
 
 /// jumpovermanyrowComment
 fn skip_block_comment(&mut self) {
 while !self.at_end {
 if self.peek() == '*' && self.peek_next() == '/' {
 self.advance();
 self.advance();
 break;
 }
 if self.peek() == '
' {
 self.line += 1;
 self.column = 0;
 }
 self.advance();
 }
 }
 
 /// prefixenteraitemCharacter
 fn advance(&mut self) -> char {
 let ch = self.peek();
 self.position += ch.len_utf8();
 self.column += 1;
 
 if self.position >= self.source.len() {
 self.at_end = true;
 }
 
 ch
 }
 
 /// inspectionCurrentCharacter
 fn peek(&self) -> char {
 if self.at_end {
 return '\0';
 }
 
 self.source[self.position..].chars().next().unwrap_or('\0')
 }
 
 /// inspectionNextCharacter
 fn peek_next(&self) -> char {
 if self.position >= self.source.len() {
 return '\0';
 }
 
 let mut chars = self.source[self.position..].chars();
 chars.next();
 chars.next().unwrap_or('\0')
 }
 
 /// MatchCharacter
 fn match_char(&mut self, expected: char) -> bool {
 if self.at_end || self.peek() != expected {
 return false;
 }
 
 self.advance();
 true
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_lexer_new() {
 let lexer = Lexer::new("let x = 1;");
 assert_eq!(lexer.position, 0);
 assert_eq!(lexer.line, 1);
 assert_eq!(lexer.column, 1);
 assert!(!lexer.at_end);
 }

 #[test]
 fn test_lexer_empty() {
 let mut lexer = Lexer::new("");
 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Eof);
 }

 #[test]
 fn test_lexer_single_char_tokens() {
 let mut lexer = Lexer::new("()[]{};,.");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::LeftParen);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::RightParen);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::LeftBracket);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::RightBracket);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::LeftBrace);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::RightBrace);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Comma);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Semicolon);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Dot);
 }

 #[test]
 fn test_lexer_operators() {
 let mut lexer = Lexer::new("+-*/%");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Plus);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Minus);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Star);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Slash);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Percent);
 }

 #[test]
 fn test_lexer_comparison() {
 let mut lexer = Lexer::new("==!=<><=>=");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Equal);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::NotEqual);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Less);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Greater);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::LessEqual);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::GreaterEqual);
 }

 #[test]
 fn test_lexer_logical() {
 let mut lexer = Lexer::new("&&||!");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::And);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Or);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Not);
 }

 #[test]
 fn test_lexer_assignment() {
 let mut lexer = Lexer::new("=+=-=*=/=");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Assign);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::PlusAssign);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::MinusAssign);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::StarAssign);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::SlashAssign);
 }

 #[test]
 fn test_lexer_arrow() {
 let mut lexer = Lexer::new("->");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Arrow);
 }

 #[test]
 fn test_lexer_double_colon() {
 let mut lexer = Lexer::new("::");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::DoubleColon);
 }

 #[test]
 fn test_lexer_whitespace() {
 let mut lexer = Lexer::new(" + - ");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Plus);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Minus);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Eof);
 }

 #[test]
 fn test_lexer_line_tracking() {
 let mut lexer = Lexer::new("+
-");

 let token = lexer.next_token();
 assert_eq!(token.line, 1);

 let token = lexer.next_token();
 assert_eq!(token.line, 2);
 }

 #[test]
 fn test_lexer_pipeline() {
 let mut lexer = Lexer::new("|>");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Pipeline);
 }

 #[test]
 fn test_lexer_pipeline_in_expression() {
 let mut lexer = Lexer::new("x |> f |> g");

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Identifier);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Pipeline);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Identifier);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Pipeline);

 let token = lexer.next_token();
 assert_eq!(token.token_type, TokenType::Identifier);
 }
}