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

// ! Nuva languagelanguageSemantic Analysisdevice

use crate::Nuva_compiler::ast::*;
use core::sync::atomic::{AtomicU32, Ordering};
use alloc::boxed::Box;

/// Type
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
 Void,
 Int,
 Float,
 Bool,
 String,
 Nil,
 UserDefined {
 name: [u8; 64],
 name_len: u8,
 },
 Optional {
 inner: Box<Type>,
 },
 Array {
 element: Box<Type>,
 },
 Function {
 params: [Type; 8],
 num_params: u8,
 return_type: Box<Type>,
 },
 Error,
}

/// SignType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
 Variable,
 Function,
 Class,
 Struct,
 Enum,
 Protocol,
 TypeAlias,
 Module,
}

/// Sign
#[derive(Debug, Clone)]
pub struct Symbol {
 pub name: [u8; 64],
 pub name_len: u8,
 pub kind: SymbolKind,
 pub type_info: Type,
 pub scope_level: u32,
 pub is_mutable: bool,
}

impl Symbol {
 pub fn new(name: &[u8], kind: SymbolKind, type_info: Type, scope_level: u32) -> Self {
 let mut name_buf = [0u8; 64];
 let len = name.len().min(63);
 name_buf[..len].copy_from_slice(&name[..len]);
 
 Self {
 name: name_buf,
 name_len: len as u8,
 kind,
 type_info,
 scope_level,
 is_mutable: false,
 }
 }
}

/// makeuseField
pub struct Scope {
 pub level: u32,
 pub symbols: [Option<Symbol>; 64],
 pub num_symbols: AtomicU32,
 pub parent: Option<u32>,
}

impl Scope {
 pub fn new(level: u32, parent: Option<u32>) -> Self {
 Self {
 level,
 symbols: [None; 64],
 num_symbols: AtomicU32::new(0),
 parent,
 }
 }

 pub fn add_symbol(&mut self, symbol: Symbol) -> bool {
 let idx = self.num_symbols.load(Ordering::Relaxed) as usize;
 if idx < 64 {
 self.symbols[idx] = Some(symbol);
 self.num_symbols.fetch_add(1, Ordering::Relaxed);
 return true;
 }
 false
 }

 pub fn lookup(&self, name: &[u8]) -> Option<&Symbol> {
 for i in 0..self.num_symbols.load(Ordering::Relaxed) as usize {
 if let Some(ref symbol) = self.symbols[i] {
 if &symbol.name[..symbol.name_len as usize] == name {
 return Some(symbol);
 }
 }
 }
 None
 }
}

/// languagemeaningError
#[derive(Debug, Clone)]
pub struct SemanticError {
 pub message: [u8; 256],
 pub message_len: u8,
 pub line: u32,
 pub column: u32,
}

impl SemanticError {
 pub fn new(message: &[u8], line: u32, column: u32) -> Self {
 let mut msg_buf = [0u8; 256];
 let len = message.len().min(255);
 msg_buf[..len].copy_from_slice(&message[..len]);
 
 Self {
 message: msg_buf,
 message_len: len as u8,
 line,
 column,
 }
 }
}

/// Semantic Analysisdevice
pub struct SemanticAnalyzer {
 scopes: [Option<Scope>; 32],
 num_scopes: AtomicU32,
 current_scope: Option<u32>,
 errors: [Option<SemanticError>; 64],
 num_errors: AtomicU32,
}

impl SemanticAnalyzer {
 pub fn new() -> Self {
 Self {
 scopes: [None; 32],
 num_scopes: AtomicU32::new(0),
 current_scope: None,
 errors: [None; 64],
 num_errors: AtomicU32::new(0),
 }
 }

 pub fn init(&mut self) {
 // CreateGlobalmakeuseField
 self.push_scope();
 
 // addPlusinsideplacementType
 self.add_builtin_types();
 }

 /// AnalysisModule
 pub fn analyze_module(&mut self, module: &Module) -> bool {
 // aiterate: receivecollectionplacefiniteDeclaration
 for i in 0..module.num_declarations as usize {
 self.collect_declaration(&module.declarations[i]);
 }
 
 // seconditerate: AnalysisDeclarationVolume
 for i in 0..module.num_declarations as usize {
 self.analyze_declaration(&module.declarations[i]);
 }
 
 self.num_errors.load(Ordering::Relaxed) == 0
 }

 /// receivecollectionDeclaration
 fn collect_declaration(&mut self, decl: &Decl) {
 match decl {
 Decl::Func { name, name_len, .. } => {
 let symbol = Symbol::new(
 &name[..*name_len as usize],
 SymbolKind::Function,
 Type::Void,
 self.current_scope_level(),
 );
 self.add_symbol_to_current_scope(symbol);
 }
 Decl::Class { name, name_len, .. } => {
 let symbol = Symbol::new(
 &name[..*name_len as usize],
 SymbolKind::Class,
 Type::Void,
 self.current_scope_level(),
 );
 self.add_symbol_to_current_scope(symbol);
 }
 Decl::Struct { name, name_len, .. } => {
 let symbol = Symbol::new(
 &name[..*name_len as usize],
 SymbolKind::Struct,
 Type::Void,
 self.current_scope_level(),
 );
 self.add_symbol_to_current_scope(symbol);
 }
 Decl::Enum { name, name_len, .. } => {
 let symbol = Symbol::new(
 &name[..*name_len as usize],
 SymbolKind::Enum,
 Type::Void,
 self.current_scope_level(),
 );
 self.add_symbol_to_current_scope(symbol);
 }
 Decl::Protocol { name, name_len, .. } => {
 let symbol = Symbol::new(
 &name[..*name_len as usize],
 SymbolKind::Protocol,
 Type::Void,
 self.current_scope_level(),
 );
 self.add_symbol_to_current_scope(symbol);
 }
 _ => {}
 }
 }

 /// AnalysisDeclaration
 fn analyze_declaration(&mut self, decl: &Decl) {
 match decl {
 Decl::Func { params, num_params, body, .. } => {
 self.push_scope();
 
 // addParameter
 for i in 0..*num_params as usize {
 let param = &params[i];
 let type_info = self.type_expr_to_type(&param.type_annotation);
 let symbol = Symbol::new(
 &param.internal_name[..param.internal_name_len as usize],
 SymbolKind::Variable,
 type_info,
 self.current_scope_level(),
 );
 self.add_symbol_to_current_scope(symbol);
 }
 
 // AnalysisFunctionVolume
 if let Some(body) = body {
 self.analyze_statement(body);
 }
 
 self.pop_scope();
 }
 Decl::Class { members, num_members, .. } => {
 self.push_scope();
 
 for i in 0..*num_members as usize {
 self.analyze_declaration(&members[i]);
 }
 
 self.pop_scope();
 }
 Decl::Struct { members, num_members, .. } => {
 self.push_scope();
 
 for i in 0..*num_members as usize {
 self.analyze_declaration(&members[i]);
 }
 
 self.pop_scope();
 }
 _ => {}
 }
 }

 /// Analysislanguagesentence
 fn analyze_statement(&mut self, stmt: &Stmt) {
 match stmt {
 Stmt::VarDecl { name, name_len, type_annotation, init, is_mutable, .. } => {
 let type_info = if let Some(ty) = type_annotation {
 self.type_expr_to_type(ty)
 } else if let Some(init_expr) = init {
 self.analyze_expression(init_expr)
 } else {
 Type::Error
 };
 
 let mut symbol = Symbol::new(
 &name[..*name_len as usize],
 SymbolKind::Variable,
 type_info,
 self.current_scope_level(),
 );
 symbol.is_mutable = *is_mutable;
 self.add_symbol_to_current_scope(symbol);
 }
 Stmt::If { condition, then_branch, else_branch, .. } => {
 let cond_type = self.analyze_expression(condition);
 if cond_type != Type::Bool && cond_type != Type::Error {
 self.add_error(b"Condition must be a boolean expression", 0, 0);
 }
 
 self.analyze_statement(then_branch);
 if let Some(else_stmt) = else_branch {
 self.analyze_statement(else_stmt);
 }
 }
 Stmt::While { condition, body, .. } => {
 let cond_type = self.analyze_expression(condition);
 if cond_type != Type::Bool && cond_type != Type::Error {
 self.add_error(b"Condition must be a boolean expression", 0, 0);
 }
 
 self.analyze_statement(body);
 }
 Stmt::ForIn { var_name, var_name_len, iterable, body, .. } => {
 let iter_type = self.analyze_expression(iterable);
 
 self.push_scope();
 
 let element_type = match iter_type {
 Type::Array { element } => *element,
 _ => Type::Error,
 };
 
 let symbol = Symbol::new(
 &var_name[..*var_name_len as usize],
 SymbolKind::Variable,
 element_type,
 self.current_scope_level(),
 );
 self.add_symbol_to_current_scope(symbol);
 
 self.analyze_statement(body);
 
 self.pop_scope();
 }
 Stmt::Block { stmts, num_stmts, .. } => {
 self.push_scope();
 
 for i in 0..*num_stmts as usize {
 self.analyze_statement(&stmts[i]);
 }
 
 self.pop_scope();
 }
 Stmt::Expr { expr, .. } => {
 self.analyze_expression(expr);
 }
 Stmt::Return { value, .. } => {
 if let Some(expr) = value {
 self.analyze_expression(expr);
 }
 }
 _ => {}
 }
 }

 /// Analysisformreachstyle
 fn analyze_expression(&mut self, expr: &Expr) -> Type {
 match expr {
 Expr::IntegerLiteral { .. } => Type::Int,
 Expr::FloatLiteral { .. } => Type::Float,
 Expr::BoolLiteral { .. } => Type::Bool,
 Expr::StringLiteral { .. } => Type::String,
 Expr::NilLiteral { .. } => Type::Nil,
 Expr::Identifier { name, name_len, .. } => {
 self.lookup_symbol(&name[..*name_len as usize])
 .map(|s| s.type_info.clone())
 .unwrap_or_else(|| {
 self.add_error(b"Undefined identifier", 0, 0);
 Type::Error
 })
 }
 Expr::Binary { op, left, right, .. } => {
 let left_type = self.analyze_expression(left);
 let right_type = self.analyze_expression(right);
 
 match op {
 BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
 if left_type == Type::Int && right_type == Type::Int {
 Type::Int
 } else if left_type == Type::Float || right_type == Type::Float {
 Type::Float
 } else {
 self.add_error(b"Invalid operand types for arithmetic operation", 0, 0);
 Type::Error
 }
 }
 BinaryOp::And | BinaryOp::Or => {
 if left_type == Type::Bool && right_type == Type::Bool {
 Type::Bool
 } else {
 self.add_error(b"Logical operators require boolean operands", 0, 0);
 Type::Error
 }
 }
 BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
 Type::Bool
 }
 _ => Type::Error,
 }
 }
 Expr::Unary { op, operand, .. } => {
 let operand_type = self.analyze_expression(operand);
 
 match op {
 UnaryOp::Neg => {
 if operand_type == Type::Int || operand_type == Type::Float {
 operand_type
 } else {
 self.add_error(b"Cannot negate non-numeric type", 0, 0);
 Type::Error
 }
 }
 UnaryOp::Not => {
 if operand_type == Type::Bool {
 Type::Bool
 } else {
 self.add_error(b"Cannot apply 'not' to non-boolean type", 0, 0);
 Type::Error
 }
 }
 _ => Type::Error,
 }
 }
 Expr::Call { callee, args, num_args, .. } => {
 let callee_type = self.analyze_expression(callee);
 
 for i in 0..*num_args as usize {
 self.analyze_expression(&args[i]);
 }
 
 match callee_type {
 Type::Function { return_type, .. } => *return_type,
 _ => {
 self.add_error(b"Cannot call non-function type", 0, 0);
 Type::Error
 }
 }
 }
 Expr::ArrayLiteral { elements, num_elements, .. } => {
 let mut element_type = Type::Void;
 
 for i in 0..*num_elements as usize {
 let ty = self.analyze_expression(&elements[i]);
 if i == 0 {
 element_type = ty;
 } else if ty != element_type {
 self.add_error(b"Array elements must have the same type", 0, 0);
 }
 }
 
 Type::Array {
 element: Box::new(element_type),
 }
 }
 _ => Type::Error,
 }
 }

 /// TypeformreachstylebranchType
 fn type_expr_to_type(&self, type_expr: &TypeExpr) -> Type {
 match type_expr {
 TypeExpr::Simple { name, name_len } => {
 match &name[..*name_len as usize] {
 b"Int" => Type::Int,
 b"Float" => Type::Float,
 b"Bool" => Type::Bool,
 b"String" => Type::String,
 b"Void" => Type::Void,
 name => Type::UserDefined {
 name: {
 let mut buf = [0u8; 64];
 buf[..name.len()].copy_from_slice(name);
 buf
 },
 name_len: name.len() as u8,
 },
 }
 }
 TypeExpr::Optional { inner } => {
 Type::Optional {
 inner: Box::new(self.type_expr_to_type(inner)),
 }
 }
 TypeExpr::Array { element, .. } => {
 Type::Array {
 element: Box::new(self.type_expr_to_type(element)),
 }
 }
 _ => Type::Error,
 }
 }

 // makeuseFieldmanagementadministration
 fn push_scope(&mut self) {
 let level = self.num_scopes.load(Ordering::Relaxed);
 let parent = self.current_scope;
 
 if level < 32 {
 self.scopes[level as usize] = Some(Scope::new(level, parent));
 self.current_scope = Some(level);
 self.num_scopes.fetch_add(1, Ordering::Relaxed);
 }
 }

 fn pop_scope(&mut self) {
 if let Some(current) = self.current_scope {
 if let Some(ref scope) = self.scopes[current as usize] {
 self.current_scope = scope.parent;
 }
 }
 }

 fn current_scope_level(&self) -> u32 {
 self.current_scope.unwrap_or(0)
 }

 fn add_symbol_to_current_scope(&mut self, symbol: Symbol) {
 if let Some(current) = self.current_scope {
 if let Some(ref mut scope) = self.scopes[current as usize] {
 scope.add_symbol(symbol);
 }
 }
 }

 fn lookup_symbol(&self, name: &[u8]) -> Option<&Symbol> {
 let mut current = self.current_scope;
 
 while let Some(idx) = current {
 if let Some(ref scope) = self.scopes[idx as usize] {
 if let Some(symbol) = scope.lookup(name) {
 return Some(symbol);
 }
 current = scope.parent;
 } else {
 break;
 }
 }
 
 None
 }

 fn add_builtin_types(&mut self) {
 let builtins = [
 ("Int", Type::Int),
 ("Float", Type::Float),
 ("Bool", Type::Bool),
 ("String", Type::String),
 ("Void", Type::Void),
 ];
 
 for (name, type_info) in builtins {
 let symbol = Symbol::new(name.as_bytes(), SymbolKind::TypeAlias, type_info, 0);
 self.add_symbol_to_current_scope(symbol);
 }
 }

 fn add_error(&mut self, message: &[u8], line: u32, column: u32) {
 let idx = self.num_errors.load(Ordering::Relaxed) as usize;
 if idx < 64 {
 self.errors[idx] = Some(SemanticError::new(message, line, column));
 self.num_errors.fetch_add(1, Ordering::Relaxed);
 }
 }

 pub fn has_errors(&self) -> bool {
 self.num_errors.load(Ordering::Relaxed) > 0
 }

 pub fn get_errors(&self) -> &[Option<SemanticError>] {
 &self.errors[..self.num_errors.load(Ordering::Relaxed) as usize]
 }
}