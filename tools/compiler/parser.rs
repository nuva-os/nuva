/* * Nuva OS - Tools - Compiler
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

// ! Nuva languagelanguageSyntax Analysisdevice

use crate::Nuva_compiler::lexer::{Lexer, Token, TokenKind};
use crate::Nuva_compiler::ast::*;
use alloc::boxed::Box;

/// Syntax Analyzer
pub struct Parser {
 lexer: Lexer,
 current: Token,
 previous: Token,
 had_error: bool,
}

impl Parser {
 pub fn new() -> Self {
 Self {
 lexer: Lexer::new(),
 current: Token::new(TokenKind::Eof, b"", 0, 0),
 previous: Token::new(TokenKind::Eof, b"", 0, 0),
 had_error: false,
 }
 }

 pub fn init(&mut self, source: &[u8]) {
 self.lexer.init(source);
 self.current = self.lexer.next_token();
 self.had_error = false;
 }

 /// parseModule
 pub fn parse_module(&mut self, name: &[u8]) -> Module {
 let mut module = Module::new(name);
 
 while !self.check(TokenKind::Eof) {
 if let Some(decl) = self.parse_declaration() {
 module.add_decl(decl);
 } else {
 self.advance();
 }
 }
 
 module
 }

 /// parseDeclaration
 fn parse_declaration(&mut self) -> Option<Decl> {
 match self.current.kind {
 TokenKind::KwFunc => self.parse_func_decl(),
 TokenKind::KwClass => self.parse_class_decl(),
 TokenKind::KwStruct => self.parse_struct_decl(),
 TokenKind::KwEnum => self.parse_enum_decl(),
 TokenKind::KwProtocol => self.parse_protocol_decl(),
 TokenKind::KwExtension => self.parse_extension_decl(),
 TokenKind::KwImport => self.parse_import_decl(),
 _ => None,
 }
 }

 /// parseFunctionDeclaration
 fn parse_func_decl(&mut self) -> Option<Decl> {
 self.advance(); // consume 'func'
 
 let name = self.parse_identifier()?;
 let name_len = name.len() as u8;
 let mut name_buf = [0u8; 64];
 name_buf[..name.len()].copy_from_slice(name);
 
 // Parameter
 let mut params = [Parameter; 8];
 let mut num_params = 0u8;
 
 if self.match_token(TokenKind::LParen) {
 while !self.check(TokenKind::RParen) && num_params < 8 {
 if let Some(param) = self.parse_parameter() {
 params[num_params as usize] = param;
 num_params += 1;
 }
 
 if !self.match_token(TokenKind::Comma) {
 break;
 }
 }
 self.expect(TokenKind::RParen);
 }
 
 // returnType
 let return_type = if self.match_token(TokenKind::Arrow) {
 Some(self.parse_type()?)
 } else {
 None
 };
 
 // FunctionVolume
 let body = if self.check(TokenKind::LBrace) {
 Some(self.parse_block()?)
 } else {
 None
 };
 
 Some(Decl::Func {
 name: name_buf,
 name_len,
 params,
 num_params,
 return_type,
 body,
 is_async: false,
 throws: false,
 is_static: false,
 access: AccessLevel::Internal,
 loc: SourceLocation::default(),
 })
 }

 /// parseParameter
 fn parse_parameter(&mut self) -> Option<Parameter> {
 let mut external_name = [0u8; 64];
 let mut external_name_len = 0u8;
 let mut internal_name = [0u8; 64];
 let mut internal_name_len = 0u8;
 
 // Exteriorname
 if self.check_identifier() {
 let name = self.parse_identifier()?;
 external_name[..name.len()].copy_from_slice(name);
 external_name_len = name.len() as u8;
 
 // Interiorname
 if self.check_identifier() {
 let name = self.parse_identifier()?;
 internal_name[..name.len()].copy_from_slice(name);
 internal_name_len = name.len() as u8;
 } else {
 internal_name = external_name;
 internal_name_len = external_name_len;
 }
 }
 
 // Typenotesolve
 self.expect(TokenKind::Colon);
 let type_annotation = self.parse_type()?;
 
 // Defaultvalue
 let default_value = if self.match_token(TokenKind::Eq) {
 Some(self.parse_expression()?)
 } else {
 None
 };
 
 Some(Parameter {
 external_name,
 external_name_len,
 internal_name,
 internal_name_len,
 type_annotation,
 default_value,
 is_inout: false,
 })
 }

 /// parseClassDeclaration
 fn parse_class_decl(&mut self) -> Option<Decl> {
 self.advance(); // consume 'class'
 
 let name = self.parse_identifier()?;
 let mut name_buf = [0u8; 64];
 name_buf[..name.len()].copy_from_slice(name);
 let name_len = name.len() as u8;
 
 // ParentClass
 let super_class = if self.match_token(TokenKind::Colon) {
 Some(self.parse_type()?)
 } else {
 None
 };
 
 // Member
 self.expect(TokenKind::LBrace);
 let mut members = [Decl; 64];
 let mut num_members = 0u8;
 
 while !self.check(TokenKind::RBrace) && num_members < 64 {
 if let Some(member) = self.parse_declaration() {
 members[num_members as usize] = member;
 num_members += 1;
 } else {
 self.advance();
 }
 }
 self.expect(TokenKind::RBrace);
 
 Some(Decl::Class {
 name: name_buf,
 name_len,
 super_class,
 protocols: [TypeExpr; 8],
 num_protocols: 0,
 members,
 num_members,
 access: AccessLevel::Internal,
 loc: SourceLocation::default(),
 })
 }

 /// parseStructDeclaration
 fn parse_struct_decl(&mut self) -> Option<Decl> {
 self.advance(); // consume 'struct'
 
 let name = self.parse_identifier()?;
 let mut name_buf = [0u8; 64];
 name_buf[..name.len()].copy_from_slice(name);
 let name_len = name.len() as u8;
 
 self.expect(TokenKind::LBrace);
 let mut members = [Decl; 64];
 let mut num_members = 0u8;
 
 while !self.check(TokenKind::RBrace) && num_members < 64 {
 if let Some(member) = self.parse_declaration() {
 members[num_members as usize] = member;
 num_members += 1;
 } else {
 self.advance();
 }
 }
 self.expect(TokenKind::RBrace);
 
 Some(Decl::Struct {
 name: name_buf,
 name_len,
 protocols: [TypeExpr; 8],
 num_protocols: 0,
 members,
 num_members,
 access: AccessLevel::Internal,
 loc: SourceLocation::default(),
 })
 }

 /// parseEnumDeclaration
 fn parse_enum_decl(&mut self) -> Option<Decl> {
 self.advance(); // consume 'enum'
 
 let name = self.parse_identifier()?;
 let mut name_buf = [0u8; 64];
 name_buf[..name.len()].copy_from_slice(name);
 let name_len = name.len() as u8;
 
 // RawType
 let raw_type = if self.match_token(TokenKind::Colon) {
 Some(self.parse_type()?)
 } else {
 None
 };
 
 self.expect(TokenKind::LBrace);
 let mut cases = [EnumCase; 32];
 let mut num_cases = 0u8;
 
 while self.match_token(TokenKind::KwCase) && num_cases < 32 {
 let case_name = self.parse_identifier()?;
 let mut case_name_buf = [0u8; 64];
 case_name_buf[..case_name.len()].copy_from_slice(case_name);
 
 cases[num_cases as usize] = EnumCase {
 name: case_name_buf,
 name_len: case_name.len() as u8,
 associated_values: [TypeExpr; 4],
 num_associated: 0,
 raw_value: None,
 };
 num_cases += 1;
 }
 
 self.expect(TokenKind::RBrace);
 
 Some(Decl::Enum {
 name: name_buf,
 name_len,
 raw_type,
 cases,
 num_cases,
 members: [Decl; 32],
 num_members: 0,
 access: AccessLevel::Internal,
 loc: SourceLocation::default(),
 })
 }

 /// parseProtocolDeclaration
 fn parse_protocol_decl(&mut self) -> Option<Decl> {
 self.advance(); // consume 'protocol'
 
 let name = self.parse_identifier()?;
 let mut name_buf = [0u8; 64];
 name_buf[..name.len()].copy_from_slice(name);
 let name_len = name.len() as u8;
 
 self.expect(TokenKind::LBrace);
 let mut requirements = [Decl; 32];
 let mut num_requirements = 0u8;
 
 while !self.check(TokenKind::RBrace) && num_requirements < 32 {
 if let Some(req) = self.parse_declaration() {
 requirements[num_requirements as usize] = req;
 num_requirements += 1;
 } else {
 self.advance();
 }
 }
 self.expect(TokenKind::RBrace);
 
 Some(Decl::Protocol {
 name: name_buf,
 name_len,
 parent_protocols: [TypeExpr; 8],
 num_parents: 0,
 requirements,
 num_requirements,
 access: AccessLevel::Internal,
 loc: SourceLocation::default(),
 })
 }

 /// parseScalingDeclaration
 fn parse_extension_decl(&mut self) -> Option<Decl> {
 self.advance(); // consume 'extension'
 
 let extended_type = self.parse_type()?;
 
 self.expect(TokenKind::LBrace);
 let mut members = [Decl; 32];
 let mut num_members = 0u8;
 
 while !self.check(TokenKind::RBrace) && num_members < 32 {
 if let Some(member) = self.parse_declaration() {
 members[num_members as usize] = member;
 num_members += 1;
 } else {
 self.advance();
 }
 }
 self.expect(TokenKind::RBrace);
 
 Some(Decl::Extension {
 extended_type,
 protocols: [TypeExpr; 8],
 num_protocols: 0,
 members,
 num_members,
 access: AccessLevel::Internal,
 loc: SourceLocation::default(),
 })
 }

 /// parseconductenterDeclaration
 fn parse_import_decl(&mut self) -> Option<Decl> {
 self.advance(); // consume 'import'
 
 let module = self.parse_identifier()?;
 let mut module_buf = [0u8; 128];
 module_buf[..module.len()].copy_from_slice(module);
 
 Some(Decl::Import {
 module: module_buf,
 module_len: module.len() as u8,
 loc: SourceLocation::default(),
 })
 }

 /// parseformreachstyle
 fn parse_expression(&mut self) -> Option<Expr> {
 self.parse_assignment()
 }

 /// parseAssignmentformreachstyle
 fn parse_assignment(&mut self) -> Option<Expr> {
 let expr = self.parse_or()?;
 
 if let Some(op) = BinaryOp::from_token(self.current.kind) {
 match op {
 BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign 
 | BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::ModAssign => {
 self.advance();
 let right = self.parse_assignment()?;
 return Some(Expr::Binary {
 op,
 left: Box::new(expr),
 right: Box::new(right),
 loc: SourceLocation::default(),
 });
 }
 _ => {}
 }
 }
 
 Some(expr)
 }

 /// parse or formreachstyle
 fn parse_or(&mut self) -> Option<Expr> {
 let mut left = self.parse_and()?;
 
 while self.match_token(TokenKind::PipePipe) {
 let right = self.parse_and()?;
 left = Expr::Binary {
 op: BinaryOp::Or,
 left: Box::new(left),
 right: Box::new(right),
 loc: SourceLocation::default(),
 };
 }
 
 Some(left)
 }

 /// parse and formreachstyle
 fn parse_and(&mut self) -> Option<Expr> {
 let mut left = self.parse_equality()?;
 
 while self.match_token(TokenKind::AmpAmp) {
 let right = self.parse_equality()?;
 left = Expr::Binary {
 op: BinaryOp::And,
 left: Box::new(left),
 right: Box::new(right),
 loc: SourceLocation::default(),
 };
 }
 
 Some(left)
 }

 /// parsemutualetcityformreachstyle
 fn parse_equality(&mut self) -> Option<Expr> {
 let mut left = self.parse_comparison()?;
 
 while matches!(self.current.kind, TokenKind::EqEq | TokenKind::BangEq) {
 let op = BinaryOp::from_token(self.current.kind).unwrap();
 self.advance();
 let right = self.parse_comparison()?;
 left = Expr::Binary {
 op,
 left: Box::new(left),
 right: Box::new(right),
 loc: SourceLocation::default(),
 };
 }
 
 Some(left)
 }

 /// parseCompareformreachstyle
 fn parse_comparison(&mut self) -> Option<Expr> {
 let mut left = self.parse_term()?;
 
 while matches!(self.current.kind, TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq) {
 let op = BinaryOp::from_token(self.current.kind).unwrap();
 self.advance();
 let right = self.parse_term()?;
 left = Expr::Binary {
 op,
 left: Box::new(left),
 right: Box::new(right),
 loc: SourceLocation::default(),
 };
 }
 
 Some(left)
 }

 /// parsePlusMinusformreachstyle
 fn parse_term(&mut self) -> Option<Expr> {
 let mut left = self.parse_factor()?;
 
 while matches!(self.current.kind, TokenKind::Plus | TokenKind::Minus) {
 let op = BinaryOp::from_token(self.current.kind).unwrap();
 self.advance();
 let right = self.parse_factor()?;
 left = Expr::Binary {
 op,
 left: Box::new(left),
 right: Box::new(right),
 loc: SourceLocation::default(),
 };
 }
 
 Some(left)
 }

 /// parseMultiplyDivideformreachstyle
 fn parse_factor(&mut self) -> Option<Expr> {
 let mut left = self.parse_unary()?;
 
 while matches!(self.current.kind, TokenKind::Star | TokenKind::Slash | TokenKind::Percent) {
 let op = BinaryOp::from_token(self.current.kind).unwrap();
 self.advance();
 let right = self.parse_unary()?;
 left = Expr::Binary {
 op,
 left: Box::new(left),
 right: Box::new(right),
 loc: SourceLocation::default(),
 };
 }
 
 Some(left)
 }

 /// parseaformreachstyle
 fn parse_unary(&mut self) -> Option<Expr> {
 if let Some(op) = UnaryOp::from_token(self.current.kind) {
 self.advance();
 let operand = self.parse_unary()?;
 return Some(Expr::Unary {
 op,
 operand: Box::new(operand),
 loc: SourceLocation::default(),
 });
 }
 
 self.parse_primary()
 }

 /// parsebasebookformreachstyle
 fn parse_primary(&mut self) -> Option<Expr> {
 match self.current.kind {
 TokenKind::IntegerLiteral => {
 let value = self.parse_integer_literal();
 self.advance();
 Some(Expr::IntegerLiteral {
 value,
 loc: SourceLocation::default(),
 })
 }
 TokenKind::FloatLiteral => {
 let value = self.parse_float_literal();
 self.advance();
 Some(Expr::FloatLiteral {
 value,
 loc: SourceLocation::default(),
 })
 }
 TokenKind::StringLiteral => {
 let value = self.current.lexeme;
 let mut value_buf = [0u8; 256];
 let len = value.len().min(255);
 value_buf[..len].copy_from_slice(&value[..len]);
 self.advance();
 Some(Expr::StringLiteral {
 value: value_buf,
 value_len: len as u8,
 loc: SourceLocation::default(),
 })
 }
 TokenKind::BoolLiteral => {
 let value = self.current.lexeme == b"true";
 self.advance();
 Some(Expr::BoolLiteral {
 value,
 loc: SourceLocation::default(),
 })
 }
 TokenKind::NilLiteral => {
 self.advance();
 Some(Expr::NilLiteral {
 loc: SourceLocation::default(),
 })
 }
 TokenKind::Identifier => {
 let name = self.parse_identifier()?;
 let mut name_buf = [0u8; 64];
 name_buf[..name.len()].copy_from_slice(name);
 self.advance();
 Some(Expr::Identifier {
 name: name_buf,
 name_len: name.len() as u8,
 loc: SourceLocation::default(),
 })
 }
 TokenKind::LParen => {
 self.advance();
 let expr = self.parse_expression()?;
 self.expect(TokenKind::RParen);
 Some(expr)
 }
 _ => None,
 }
 }

 /// parseType
 fn parse_type(&mut self) -> Option<TypeExpr> {
 let name = self.parse_identifier()?;
 let mut name_buf = [0u8; 64];
 name_buf[..name.len()].copy_from_slice(name);
 self.advance();
 
 // optionalType
 if self.match_token(TokenKind::Question) {
 return Some(TypeExpr::Optional {
 inner: Box::new(TypeExpr::Simple {
 name: name_buf,
 name_len: name.len() as u8,
 }),
 });
 }
 
 Some(TypeExpr::Simple {
 name: name_buf,
 name_len: name.len() as u8,
 })
 }

 /// parseBlocklanguagesentence
 fn parse_block(&mut self) -> Option<Stmt> {
 self.expect(TokenKind::LBrace);
 
 let mut stmts = [Stmt; 64];
 let mut num_stmts = 0u8;
 
 while !self.check(TokenKind::RBrace) && num_stmts < 64 {
 if let Some(stmt) = self.parse_statement() {
 stmts[num_stmts as usize] = stmt;
 num_stmts += 1;
 } else {
 self.advance();
 }
 }
 
 self.expect(TokenKind::RBrace);
 
 Some(Stmt::Block {
 stmts,
 num_stmts,
 loc: SourceLocation::default(),
 })
 }

 /// parselanguagesentence
 fn parse_statement(&mut self) -> Option<Stmt> {
 match self.current.kind {
 TokenKind::KwVar | TokenKind::KwLet => self.parse_var_decl(),
 TokenKind::KwIf => self.parse_if_stmt(),
 TokenKind::KwWhile => self.parse_while_stmt(),
 TokenKind::KwFor => self.parse_for_stmt(),
 TokenKind::KwReturn => self.parse_return_stmt(),
 TokenKind::KwBreak => {
 self.advance();
 Some(Stmt::Break { loc: SourceLocation::default() })
 }
 TokenKind::KwContinue => {
 self.advance();
 Some(Stmt::Continue { loc: SourceLocation::default() })
 }
 TokenKind::LBrace => self.parse_block(),
 _ => {
 let expr = self.parse_expression()?;
 Some(Stmt::Expr {
 expr,
 loc: SourceLocation::default(),
 })
 }
 }
 }

 /// parseVariableDeclaration
 fn parse_var_decl(&mut self) -> Option<Stmt> {
 let is_mutable = self.current.kind == TokenKind::KwVar;
 self.advance();
 
 let name = self.parse_identifier()?;
 let mut name_buf = [0u8; 64];
 name_buf[..name.len()].copy_from_slice(name);
 self.advance();
 
 let type_annotation = if self.match_token(TokenKind::Colon) {
 Some(self.parse_type()?)
 } else {
 None
 };
 
 let init = if self.match_token(TokenKind::Eq) {
 Some(self.parse_expression()?)
 } else {
 None
 };
 
 Some(Stmt::VarDecl {
 name: name_buf,
 name_len: name.len() as u8,
 type_annotation,
 init,
 is_mutable,
 loc: SourceLocation::default(),
 })
 }

 /// parse if languagesentence
 fn parse_if_stmt(&mut self) -> Option<Stmt> {
 self.advance(); // consume 'if'
 
 let condition = self.parse_expression()?;
 let then_branch = self.parse_block()?;
 
 let else_branch = if self.match_token(TokenKind::KwElse) {
 if self.check(TokenKind::LBrace) {
 Some(Box::new(self.parse_block()?))
 } else if self.check(TokenKind::KwIf) {
 Some(Box::new(self.parse_if_stmt()?))
 } else {
 None
 }
 } else {
 None
 };
 
 Some(Stmt::If {
 condition,
 then_branch: Box::new(then_branch),
 else_branch,
 loc: SourceLocation::default(),
 })
 }

 /// parse while languagesentence
 fn parse_while_stmt(&mut self) -> Option<Stmt> {
 self.advance(); // consume 'while'
 
 let condition = self.parse_expression()?;
 let body = self.parse_block()?;
 
 Some(Stmt::While {
 condition,
 body: Box::new(body),
 loc: SourceLocation::default(),
 })
 }

 /// parse for languagesentence
 fn parse_for_stmt(&mut self) -> Option<Stmt> {
 self.advance(); // consume 'for'
 
 let var_name = self.parse_identifier()?;
 let mut var_name_buf = [0u8; 64];
 var_name_buf[..var_name.len()].copy_from_slice(var_name);
 self.advance();
 
 self.expect(TokenKind::KwIn);
 let iterable = self.parse_expression()?;
 let body = self.parse_block()?;
 
 Some(Stmt::ForIn {
 var_name: var_name_buf,
 var_name_len: var_name.len() as u8,
 iterable,
 body: Box::new(body),
 loc: SourceLocation::default(),
 })
 }

 /// parse return languagesentence
 fn parse_return_stmt(&mut self) -> Option<Stmt> {
 self.advance(); // consume 'return'
 
 let value = if !self.check(TokenKind::Semicolon) && !self.check(TokenKind::RBrace) {
 Some(self.parse_expression()?)
 } else {
 None
 };
 
 Some(Stmt::Return {
 value,
 loc: SourceLocation::default(),
 })
 }

 // auxiliaryMethod
 fn advance(&mut self) {
 self.previous = self.current.clone();
 self.current = self.lexer.next_token();
 }

 fn check(&self, kind: TokenKind) -> bool {
 self.current.kind == kind
 }

 fn check_identifier(&self) -> bool {
 self.current.kind == TokenKind::Identifier
 }

 fn match_token(&mut self, kind: TokenKind) -> bool {
 if self.check(kind) {
 self.advance();
 true
 } else {
 false
 }
 }

 fn expect(&mut self, kind: TokenKind) -> bool {
 if self.check(kind) {
 self.advance();
 true
 } else {
 self.had_error = true;
 false
 }
 }

 fn parse_identifier(&self) -> Option<&[u8]> {
 if self.check_identifier() {
 Some(self.current.lexeme())
 } else {
 None
 }
 }

 fn parse_integer_literal(&self) -> u64 {
 let lexeme = self.current.lexeme();
 let mut value = 0u64;
 for &b in lexeme {
 if b.is_ascii_digit() {
 value = value * 10 + (b - b'0') as u64;
 }
 }
 value
 }

 fn parse_float_literal(&self) -> f64 {
 let lexeme = self.current.lexeme();
 let mut value = 0.0f64;
 let mut decimal = false;
 let mut divisor = 1.0f64;
 
 for &b in lexeme {
 if b == b'.' {
 decimal = true;
 } else if b.is_ascii_digit() {
 let digit = (b - b'0') as f64;
 if decimal {
 divisor *= 10.0;
 value += digit / divisor;
 } else {
 value = value * 10.0 + digit;
 }
 }
 }
 value
 }
}