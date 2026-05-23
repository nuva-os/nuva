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


use super::ast::*;
use crate::nuva_lang::lexer::token::{Token, TokenType, Keyword, TokenValue};
use super::precedence::{get_binary_precedence, get_unary_precedence, precedence};

/// Syntax Analyzer
pub struct Parser {
 /// Token Array
 tokens: &'static [Token],
 /// CurrentPosition
 position: usize,
 /// Errorcount
 error_count: u32,
 /// Whether inside async context (for await validation)
 in_async_context: bool,
}

impl Parser {
 /// CreatenewSyntax Analyzer
 pub fn new(tokens: &'static [Token]) -> Self {
 Parser {
 tokens,
 position: 0,
 error_count: 0,
 in_async_context: false,
 }
 }
 
 /// parseprocessorder
 pub fn parse(&mut self) -> Option<Program> {
 let mut declarations = Vec::new();
 
 while !self.is_at_end() {
 if let Some(decl) = self.parse_declaration() {
 declarations.push(decl);
 }
 }
 
 Some(Program { declarations })
 }
 
 /// parseDeclaration
 fn parse_declaration(&mut self) -> Option<AstNode> {
 let token = self.peek()?;
 
 match token.token_type {
 TokenType::Keyword => {
 match token.keyword? {
 Keyword::Fn => self.parse_function_def(),
 Keyword::Struct => self.parse_struct_def(),
 Keyword::Enum => self.parse_enum_def(),
 Keyword::Trait => self.parse_trait_def(),
 Keyword::Impl => self.parse_impl_block(),
 Keyword::Let | Keyword::Var | Keyword::Const => self.parse_var_decl(),
 Keyword::Component => self.parse_component_def(),
 Keyword::Signal => self.parse_signal_decl(),
 Keyword::Effect => self.parse_effect_decl(),
 Keyword::Async => self.parse_async_def(),
 Keyword::Resource => self.parse_resource_decl(),
 _ => None,
 }
 }
 _ => None,
 }
 }
 
 /// parseFunctionDefinition
 fn parse_function_def(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'fn'
 let name = self.expect_identifier()?;
 let type_params = self.parse_type_params()?;
 let params = self.parse_param_list()?;
 let return_type = self.parse_return_type()?;
 let body = self.parse_expr()?;
 Some(AstNode::FunctionDef(FunctionDef {
 name, type_params, params, return_type, body,
 is_async: false, is_pub: false, is_pure: false, is_inline: false,
 lifetimes: Vec::new(),
 }))
 }
 
 /// parseStructDefinition
 fn parse_struct_def(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'struct'
 let name = self.expect_identifier()?;
 let type_params = self.parse_type_params()?;
 let fields = self.parse_field_list()?;
 Some(AstNode::StructDef(StructDef {
 name, type_params, fields, is_pub: false, derive: Vec::new(),
 }))
 }
 
 /// parseEnumDefinition
 fn parse_enum_def(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'enum'
 let name = self.expect_identifier()?;
 let type_params = self.parse_type_params()?;
 let variants = self.parse_variant_list()?;
 Some(AstNode::EnumDef(EnumDef {
 name, type_params, variants, is_pub: false, derive: Vec::new(),
 }))
 }
 
 /// parseTraitDefinition
 fn parse_trait_def(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'trait'
 let name = self.expect_identifier()?;
 let type_params = self.parse_type_params()?;
 let methods = self.parse_method_list()?;
 Some(AstNode::TraitDef(TraitDef {
 name, type_params, methods, is_pub: false, assoc_types: Vec::new(),
 }))
 }
 
 /// parseImplementationBlock
 fn parse_impl_block(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'impl'
 let type_params = self.parse_type_params()?;
 let trait_type = if self.match_keyword(Keyword::For) {
 let name = self.expect_identifier()?;
 Some(Type { name, type_args: Vec::new(), is_mut_ref: false, is_ref: false, lifetime: None })
 } else {
 None
 };
 let target_name = self.expect_identifier()?;
 let target_type = Type { name: target_name, type_args: Vec::new(), is_mut_ref: false, is_ref: false, lifetime: None };
 let methods = self.parse_method_list()?;
 Some(AstNode::ImplBlock(ImplBlock {
 target_type, trait_type, methods, type_params,
 }))
 }
 
 /// parseVariableDeclaration
 fn parse_var_decl(&mut self) -> Option<AstNode> {
 let token = self.advance()?;
 let is_mut = token.keyword == Some(Keyword::Var) || (token.keyword == Some(Keyword::Let) && self.match_keyword(Keyword::Mut));
 let is_const = token.keyword == Some(Keyword::Const);
 let name = self.expect_identifier()?;
 let var_type = if self.match_token(TokenType::Colon) {
 Some(self.parse_type()?)
 } else {
 None
 };
 self.expect_token(TokenType::Assign);
 let init = self.parse_expr()?;
 Some(AstNode::VarDecl(VarDecl { name, var_type, init, is_mut, is_const }))
 }
 
 /// Parse component definition (declarative UI)
 fn parse_component_def(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'component'
 let name = self.expect_identifier()?;
 let type_params = self.parse_type_params()?;
 let params = self.parse_param_list()?;
 let mut signals = Vec::new();
 let mut effects = Vec::new();
 self.expect_token(TokenType::LeftBrace);
 while let Some(token) = self.peek() {
 if token.token_type == TokenType::RightBrace { break; }
 match token.keyword {
 Some(Keyword::Signal) => {
 if let Some(AstNode::SignalDecl(s)) = self.parse_signal_decl() {
 signals.push(s);
 }
 }
 Some(Keyword::Effect) => {
 if let Some(AstNode::EffectDecl(e)) = self.parse_effect_decl() {
 effects.push(e);
 }
 }
 _ => break,
 }
 }
 let body = self.parse_block_inner()?;
 Some(AstNode::ComponentDef(ComponentDef {
 name, type_params, params, signals, effects, body, is_pub: false,
 }))
 }
 
 /// Parse signal declaration (reactive)
 fn parse_signal_decl(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'signal'
 let name = self.expect_identifier()?;
 let signal_type = if self.match_token(TokenType::Colon) {
 self.parse_type()?
 } else {
 Type { name: "_", type_args: Vec::new(), is_mut_ref: false, is_ref: false, lifetime: None }
 };
 self.expect_token(TokenType::Assign);
 let initial = self.parse_expr()?;
 Some(AstNode::SignalDecl(SignalDecl { name, signal_type, initial }))
 }
 
 /// Parse effect declaration (reactive)
 fn parse_effect_decl(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'effect'
 let body = self.parse_expr()?;
 Some(AstNode::EffectDecl(EffectDecl { body, dependencies: Vec::new() }))
 }
 
 /// Parse async function definition
 fn parse_async_def(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'async'
 self.expect_keyword(Keyword::Fn);
 let name = self.expect_identifier()?;
 let type_params = self.parse_type_params()?;
 let params = self.parse_param_list()?;
 let return_type = self.parse_return_type()?;
 let prev_async = self.in_async_context;
 self.in_async_context = true;
 let body = self.parse_expr()?;
 self.in_async_context = prev_async;
 Some(AstNode::FunctionDef(FunctionDef {
 name, type_params, params, return_type, body,
 is_async: true, is_pub: false, is_pure: false, is_inline: false,
 lifetimes: Vec::new(),
 }))
 }
 
 /// Parse resource declaration (declarative resource management)
 fn parse_resource_decl(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'resource'
 let name = self.expect_identifier()?;
 let resource_type = if self.match_token(TokenType::Colon) {
 self.parse_type()?
 } else {
 Type { name: "_", type_args: Vec::new(), is_mut_ref: false, is_ref: false, lifetime: None }
 };
 self.expect_token(TokenType::Assign);
 let acquire = self.parse_expr()?;
 self.expect_keyword(Keyword::With);
 let release = self.parse_expr()?;
 Some(AstNode::ResourceDecl(ResourceDecl { name, resource_type, acquire, release }))
 }
 
 /// parseformreachstyle
 pub fn parse_expr(&mut self) -> Option<Expr> {
 self.parse_expr_precedence(0)
 }
 
 /// parseexpfixedPriority formreachstyle (Pratt parser)
 fn parse_expr_precedence(&mut self, min_precedence: u8) -> Option<Expr> {
 let mut left = self.parse_prefix()?;
 
 while let Some(token) = self.peek() {
 let op = match token.token_type {
 TokenType::Plus => BinaryOp::Add,
 TokenType::Minus => BinaryOp::Sub,
 TokenType::Star => BinaryOp::Mul,
 TokenType::Slash => BinaryOp::Div,
 TokenType::Percent => BinaryOp::Mod,
 TokenType::Equal => BinaryOp::Equal,
 TokenType::NotEqual => BinaryOp::NotEqual,
 TokenType::Less => BinaryOp::Less,
 TokenType::LessEqual => BinaryOp::LessEqual,
 TokenType::Greater => BinaryOp::Greater,
 TokenType::GreaterEqual => BinaryOp::GreaterEqual,
 TokenType::And => BinaryOp::And,
 TokenType::Or => BinaryOp::Or,
 TokenType::BitAnd => BinaryOp::BitAnd,
 TokenType::BitOr => BinaryOp::BitOr,
 TokenType::BitXor => BinaryOp::BitXor,
 TokenType::LeftShift => BinaryOp::LeftShift,
 TokenType::RightShift => BinaryOp::RightShift,
 TokenType::Pipeline => BinaryOp::Pipeline,
 _ => break,
 };
 
 let prec = get_binary_precedence(op);
 if prec.value <= min_precedence { break; }
 
 self.advance();
 let right = self.parse_expr_precedence(prec.value)?;
 left = Expr {
 kind: ExprKind::Binary { left: Box::new(left), op, right: Box::new(right) },
 ty: None,
 };
 }
 
 Some(left)
 }
 
 /// Parse prefix expression (unary, literals, identifiers, grouping)
 fn parse_prefix(&mut self) -> Option<Expr> {
 let token = self.peek()?.clone();
 
 match token.token_type {
 TokenType::Integer => {
 self.advance();
 if let TokenValue::Integer(v) = token.value {
 Some(Expr { kind: ExprKind::Literal(Literal::Integer(v)), ty: None })
 } else {
 Some(Expr { kind: ExprKind::Literal(Literal::Integer(0)), ty: None })
 }
 }
 TokenType::Float => {
 self.advance();
 if let TokenValue::Float(v) = token.value {
 Some(Expr { kind: ExprKind::Literal(Literal::Float(v)), ty: None })
 } else {
 Some(Expr { kind: ExprKind::Literal(Literal::Float(0.0)), ty: None })
 }
 }
 TokenType::String => {
 self.advance();
 if let TokenValue::String(s) = token.value {
 Some(Expr { kind: ExprKind::Literal(Literal::String(s)), ty: None })
 } else {
 Some(Expr { kind: ExprKind::Literal(Literal::String("")), ty: None })
 }
 }
 TokenType::Char => {
 self.advance();
 if let TokenValue::Char(c) = token.value {
 Some(Expr { kind: ExprKind::Literal(Literal::Char(c)), ty: None })
 } else {
 Some(Expr { kind: ExprKind::Literal(Literal::Char('\0')), ty: None })
 }
 }
 TokenType::Identifier => {
 self.advance();
 if let TokenValue::Identifier(name) = token.value {
 let mut expr = Expr { kind: ExprKind::Identifier(name), ty: None };
 if let Some(t) = self.peek() {
 if t.token_type == TokenType::LeftParen {
 let args = self.parse_call_args()?;
 expr = Expr { kind: ExprKind::Call { callee: Box::new(expr), args }, ty: None };
 }
 }
 Some(expr)
 } else {
 None
 }
 }
 TokenType::Keyword => {
 match token.keyword {
 Some(Keyword::True) => { self.advance(); Some(Expr { kind: ExprKind::Literal(Literal::Bool(true)), ty: None }) }
 Some(Keyword::False) => { self.advance(); Some(Expr { kind: ExprKind::Literal(Literal::Bool(false)), ty: None }) }
 Some(Keyword::None) => { self.advance(); Some(Expr { kind: ExprKind::Literal(Literal::None), ty: None }) }
 Some(Keyword::Some) => { self.advance(); let inner = self.parse_expr()?; Some(Expr { kind: ExprKind::Call { callee: Box::new(Expr { kind: ExprKind::Identifier("Some"), ty: None }), args: vec![inner] }, ty: None }) }
 Some(Keyword::Await) => {
 if !self.in_async_context { self.error_count += 1; }
 self.advance();
 let operand = self.parse_expr()?;
 Some(Expr { kind: ExprKind::Await(Box::new(operand)), ty: None })
 }
 Some(Keyword::If) => self.parse_if_expr(),
 Some(Keyword::Match) => self.parse_match_expr(),
 Some(Keyword::Fn) => self.parse_closure(),
 Some(Keyword::With) => self.parse_with_block_expr(),
 _ => None,
 }
 }
 TokenType::Minus => {
 self.advance();
 let operand = self.parse_prefix()?;
 Some(Expr { kind: ExprKind::Unary { op: UnaryOp::Neg, operand: Box::new(operand) }, ty: None })
 }
 TokenType::Not => {
 self.advance();
 let operand = self.parse_prefix()?;
 Some(Expr { kind: ExprKind::Unary { op: UnaryOp::Not, operand: Box::new(operand) }, ty: None })
 }
 TokenType::BitNot => {
 self.advance();
 let operand = self.parse_prefix()?;
 Some(Expr { kind: ExprKind::Unary { op: UnaryOp::BitNot, operand: Box::new(operand) }, ty: None })
 }
 TokenType::LeftParen => {
 self.advance();
 let expr = self.parse_expr()?;
 self.expect_token(TokenType::RightParen);
 Some(expr)
 }
 TokenType::LeftBracket => {
 self.advance();
 let mut elements = Vec::new();
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::RightBracket { break; }
 elements.push(self.parse_expr()?);
 if let Some(t) = self.peek() {
 if t.token_type == TokenType::Comma { self.advance(); }
 }
 }
 self.expect_token(TokenType::RightBracket);
 Some(Expr { kind: ExprKind::Array(elements), ty: None })
 }
 TokenType::LeftBrace => {
 let block = self.parse_block_inner()?;
 Some(Expr { kind: ExprKind::Block(block), ty: None })
 }
 _ => { self.advance(); None }
 }
 }
 
 /// Parse if expression
 fn parse_if_expr(&mut self) -> Option<Expr> {
 self.advance(); // consume 'if'
 let condition = self.parse_expr()?;
 let then_branch = self.parse_expr()?;
 let else_branch = if self.match_keyword(Keyword::Else) {
 self.parse_expr()?
 } else {
 Expr { kind: ExprKind::Literal(Literal::Unit), ty: None }
 };
 Some(Expr {
 kind: ExprKind::If { condition: Box::new(condition), then_branch: Box::new(then_branch), else_branch: Box::new(else_branch) },
 ty: None,
 })
 }
 
 /// Parse match expression
 fn parse_match_expr(&mut self) -> Option<Expr> {
 self.advance(); // consume 'match'
 let value = self.parse_expr()?;
 self.expect_token(TokenType::LeftBrace);
 let mut arms = Vec::new();
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::RightBrace { break; }
 let pattern = self.parse_pattern()?;
 let guard = if self.match_keyword(Keyword::If) { Some(self.parse_expr()?) } else { None };
 self.expect_token(TokenType::Arrow);
 let body = self.parse_expr()?;
 arms.push(MatchArm { pattern, guard, body });
 if let Some(t) = self.peek() {
 if t.token_type == TokenType::Comma { self.advance(); }
 }
 }
 self.expect_token(TokenType::RightBrace);
 Some(Expr { kind: ExprKind::Match { value: Box::new(value), arms }, ty: None })
 }
 
 /// Parse pattern
 fn parse_pattern(&mut self) -> Option<Pattern> {
 let token = self.peek()?.clone();
 match token.token_type {
 TokenType::Identifier => {
 self.advance();
 if let TokenValue::Identifier(name) = token.value {
 if name == "_" { Some(Pattern::Wildcard) } else { Some(Pattern::Identifier(name)) }
 } else { None }
 }
 TokenType::Integer => { self.advance(); Some(Pattern::Literal(Literal::Integer(0))) }
 _ => { self.advance(); Some(Pattern::Wildcard) }
 }
 }
 
 /// Parse closure expression
 fn parse_closure(&mut self) -> Option<Expr> {
 self.advance(); // consume 'fn'
 let params = self.parse_param_list()?;
 self.expect_token(TokenType::Arrow);
 let body = self.parse_expr()?;
 Some(Expr { kind: ExprKind::Closure { params, body: Box::new(body) }, ty: None })
 }
 
 /// Parse with block expression
 fn parse_with_block_expr(&mut self) -> Option<Expr> {
 self.advance(); // consume 'with'
 let resource = self.expect_identifier()?;
 let body = self.parse_block_inner()?;
 Some(Expr { kind: ExprKind::Block(body), ty: None })
 }

 /// Parse pipeline expression
 /// Pipeline expressions enable left-to-right function composition:
 /// data |> f1 |> f2 |> f3
 fn parse_pipeline(&mut self, source: Expr) -> Option<Expr> {
 let mut stages = Vec::new();

 // Parse all pipeline stages
 while let Some(token) = self.peek() {
 if token.token_type != TokenType::Pipeline {
 break;
 }

 // Consume the |> operator
 self.advance();

 // Parse the next function/stage
 let func = self.parse_expr()?;

 // Check if there are additional arguments in parentheses
 let args = if let Some(token) = self.peek() {
 if token.token_type == TokenType::LeftParen {
 self.parse_call_args()?
 } else {
 Vec::new()
 }
 } else {
 Vec::new()
 };

 // Determine the stage type
 let stage = match &func.kind {
 // If it's an identifier, treat it as a function call
 ExprKind::Identifier(name) => {
 PipelineStage::Function {
 func,
 args,
 }
 }
 // Otherwise, it's a general expression
 _ => PipelineStage::Function {
 func,
 args,
 }
 };

 stages.push(stage);
 }

 // Return the pipeline expression
 Some(Expr {
 kind: ExprKind::Pipeline(PipelineExpr {
 source,
 stages,
 }),
 ty: None,
 })
 }

 /// Parse call arguments
 fn parse_call_args(&mut self) -> Option<Vec<Expr>> {
 // Expect opening parenthesis
 if let Some(token) = self.peek() {
 if token.token_type != TokenType::LeftParen {
 return Some(Vec::new());
 }
 } else {
 return Some(Vec::new());
 }

 self.advance(); // consume '('

 let mut args = Vec::new();

 // Parse arguments
 while let Some(token) = self.peek() {
 if token.token_type == TokenType::RightParen {
 break;
 }

 let arg = self.parse_expr()?;
 args.push(arg);

 // Check for comma
 if let Some(token) = self.peek() {
 if token.token_type == TokenType::Comma {
 self.advance();
 } else if token.token_type != TokenType::RightParen {
 // Error: expected comma or closing paren
 self.error_count += 1;
 break;
 }
 }
 }

 // Expect closing parenthesis
 if let Some(token) = self.peek() {
 if token.token_type == TokenType::RightParen {
 self.advance();
 } else {
 self.error_count += 1;
 }
 }

 Some(args)
 }

 /// Parse comprehension expression
 /// Syntax: [output for var1 in iter1 for var2 in iter2 ... if guard]
 /// Examples:
 /// [x * 2 for x in list]
 /// [x * y for x in list1 for y in list2]
 /// [x for x in list if x > 0]
 fn parse_comprehension(&mut self) -> Option<Expr> {
 // Expect opening bracket
 if let Some(token) = self.peek() {
 if token.token_type != TokenType::LeftBracket {
 return None;
 }
 } else {
 return None;
 }

 self.advance(); // consume '['

 // Parse output expression
 let output = self.parse_expr()?;

 // Parse iterators (one or more)
 let mut iterators = Vec::new();

 while let Some(token) = self.peek() {
 // Check for 'for' keyword
 if token.token_type != TokenType::Keyword || token.keyword != Some(Keyword::For) {
 break;
 }

 self.advance(); // consume 'for'

 // Parse iteration variable
 let var = if let Some(token) = self.peek() {
 if token.token_type == TokenType::Identifier {
 if let TokenValue::Identifier(name) = &token.value {
 self.advance();
 name
 } else {
 self.error_count += 1;
 return None;
 }
 } else {
 self.error_count += 1;
 return None;
 }
 } else {
 self.error_count += 1;
 return None;
 };

 // Expect 'in' keyword
 if let Some(token) = self.peek() {
 if token.token_type != TokenType::Keyword || token.keyword != Some(Keyword::In) {
 self.error_count += 1;
 return None;
 }
 } else {
 self.error_count += 1;
 return None;
 }

 self.advance(); // consume 'in'

 // Parse source iterable
 let source = self.parse_expr()?;

 iterators.push(ComprehensionIter { var, source });
 }

 // Parse optional guard (if condition)
 let guard = if let Some(token) = self.peek() {
 if token.token_type == TokenType::Keyword && token.keyword == Some(Keyword::If) {
 self.advance(); // consume 'if'
 Some(self.parse_expr()?)
 } else {
 None
 }
 } else {
 None
 };

 // Expect closing bracket
 if let Some(token) = self.peek() {
 if token.token_type == TokenType::RightBracket {
 self.advance();
 } else {
 self.error_count += 1;
 }
 } else {
 self.error_count += 1;
 }

 // Return comprehension expression
 Some(Expr {
 kind: ExprKind::Comprehension(ComprehensionExpr {
 output,
 iterators,
 guard,
 is_generator: false,
 }),
 ty: None,
 })
 }
 
 /// parselanguagesentence
 fn parse_stmt(&mut self) -> Option<AstNode> {
 let token = self.peek()?;
 
 match token.token_type {
 TokenType::Keyword => {
 match token.keyword? {
 Keyword::Let | Keyword::Var | Keyword::Const => self.parse_var_decl(),
 Keyword::If => self.parse_if_stmt(),
 Keyword::While => self.parse_while_stmt(),
 Keyword::For => self.parse_for_stmt(),
 Keyword::Match => self.parse_match_stmt(),
 Keyword::Return => self.parse_return_stmt(),
 Keyword::Break => self.parse_break_stmt(),
 Keyword::Continue => self.parse_continue_stmt(),
 _ => self.parse_expr_stmt(),
 }
 }
 TokenType::LeftBrace => self.parse_block(),
 _ => self.parse_expr_stmt(),
 }
 }
 
 /// parseBlock
 fn parse_block(&mut self) -> Option<AstNode> {
 let block = self.parse_block_inner()?;
 Some(AstNode::Block(block))
 }
 
 /// Parse block inner (shared by parse_block and component body)
 fn parse_block_inner(&mut self) -> Option<Block> {
 self.expect_token(TokenType::LeftBrace);
 let mut statements = Vec::new();
 let mut final_expr = None;
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::RightBrace { break; }
 if let Some(stmt) = self.parse_stmt() {
 match &stmt {
 AstNode::ExprStmt(_) => {
 if let Some(next) = self.peek() {
 if next.token_type == TokenType::RightBrace {
 if let AstNode::ExprStmt(es) = stmt {
 final_expr = Some(Box::new(es.expr));
 break;
 }
 }
 }
 }
 _ => {}
 }
 statements.push(stmt);
 }
 }
 self.expect_token(TokenType::RightBrace);
 Some(Block { statements, final_expr })
 }
 
 /// parse if statement
 fn parse_if_stmt(&mut self) -> Option<AstNode> {
 let expr = self.parse_if_expr()?;
 Some(AstNode::IfExpr(IfExpr {
 condition: match &expr.kind { ExprKind::If { condition, .. } => condition.clone(), _ => Box::new(expr.clone()) },
 then_branch: match &expr.kind { ExprKind::If { then_branch, .. } => then_branch.clone(), _ => Box::new(expr.clone()) },
 else_branch: match &expr.kind { ExprKind::If { else_branch, .. } => else_branch.clone(), _ => Box::new(Expr { kind: ExprKind::Literal(Literal::Unit), ty: None }) },
 }))
 }
 
 /// parse while statement
 fn parse_while_stmt(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'while'
 let condition = self.parse_expr()?;
 let body = self.parse_block_inner()?;
 Some(AstNode::LoopExpr(LoopExpr::While { condition, body }))
 }
 
 /// parse for statement
 fn parse_for_stmt(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'for'
 let var = self.expect_identifier()?;
 self.expect_keyword(Keyword::In);
 let iter = self.parse_expr()?;
 let body = self.parse_block_inner()?;
 Some(AstNode::LoopExpr(LoopExpr::For { var, iter, body }))
 }
 
 /// parse match statement
 fn parse_match_stmt(&mut self) -> Option<AstNode> {
 let expr = self.parse_match_expr()?;
 Some(AstNode::MatchExpr(MatchExpr {
 value: match &expr.kind { ExprKind::Match { value, .. } => value.clone(), _ => Box::new(expr.clone()) },
 arms: match &expr.kind { ExprKind::Match { arms, .. } => arms.clone(), _ => Vec::new() },
 }))
 }
 
 /// parse return statement
 fn parse_return_stmt(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'return'
 let value = if let Some(t) = self.peek() {
 if t.token_type == TokenType::Semicolon || t.token_type == TokenType::RightBrace {
 None
 } else {
 Some(self.parse_expr()?)
 }
 } else {
 None
 };
 Some(AstNode::ReturnStmt(ReturnStmt { value }))
 }
 
 /// parse break statement
 fn parse_break_stmt(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'break'
 let value = if let Some(t) = self.peek() {
 if t.token_type != TokenType::Semicolon && t.token_type != TokenType::RightBrace {
 Some(self.parse_expr()?)
 } else {
 None
 }
 } else {
 None
 };
 Some(AstNode::BreakExpr(BreakExpr { value }))
 }
 
 /// parse continue statement
 fn parse_continue_stmt(&mut self) -> Option<AstNode> {
 self.advance(); // consume 'continue'
 Some(AstNode::ContinueExpr(ContinueExpr))
 }
 
 /// parseformreachstylelanguagesentence
 fn parse_expr_stmt(&mut self) -> Option<AstNode> {
 let expr = self.parse_expr()?;
 Some(AstNode::ExprStmt(ExprStmt { expr }))
 }
 
 /// inspectionCurrent Token
 fn peek(&self) -> Option<&Token> {
 self.tokens.get(self.position)
 }
 
 /// prefixenter
 fn advance(&mut self) -> Option<&Token> {
 if !self.is_at_end() {
 self.position += 1;
 }
 self.tokens.get(self.position - 1)
 }
 
 /// CheckiftoreachfinalTail
 fn is_at_end(&self) -> bool {
 self.position >= self.tokens.len()
 }
 
 /// GetErrorcount
 pub fn get_error_count(&self) -> u32 {
 self.error_count
 }

 /// Expect identifier token and return its name
 fn expect_identifier(&mut self) -> Option<&'static str> {
 let token = self.advance()?;
 if token.token_type == TokenType::Identifier {
 if let TokenValue::Identifier(name) = &token.value {
 Some(name)
 } else {
 self.error_count += 1;
 None
 }
 } else {
 self.error_count += 1;
 None
 }
 }

 /// Expect specific token type
 fn expect_token(&mut self, expected: TokenType) {
 if let Some(token) = self.peek() {
 if token.token_type == expected {
 self.advance();
 return;
 }
 }
 self.error_count += 1;
 }

 /// Expect specific keyword
 fn expect_keyword(&mut self, expected: Keyword) {
 if let Some(token) = self.peek() {
 if token.token_type == TokenType::Keyword && token.keyword == Some(expected) {
 self.advance();
 return;
 }
 }
 self.error_count += 1;
 }

 /// Match and consume keyword if present
 fn match_keyword(&mut self, expected: Keyword) -> bool {
 if let Some(token) = self.peek() {
 if token.token_type == TokenType::Keyword && token.keyword == Some(expected) {
 self.advance();
 return true;
 }
 }
 false
 }

 /// Match and consume token type if present
 fn match_token(&mut self, expected: TokenType) -> bool {
 if let Some(token) = self.peek() {
 if token.token_type == expected {
 self.advance();
 return true;
 }
 }
 false
 }

 /// Parse type params <T, U>
 fn parse_type_params(&mut self) -> Option<Vec<&'static str>> {
 if !self.match_token(TokenType::Less) {
 return Some(Vec::new());
 }
 let mut params = Vec::new();
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::Greater { break; }
 if let Some(name) = self.expect_identifier() {
 params.push(name);
 }
 if let Some(t) = self.peek() {
 if t.token_type == TokenType::Comma { self.advance(); }
 }
 }
 self.expect_token(TokenType::Greater);
 Some(params)
 }

 /// Parse parameter list (a: T, b: U)
 fn parse_param_list(&mut self) -> Option<Vec<Parameter>> {
 if !self.match_token(TokenType::LeftParen) {
 return Some(Vec::new());
 }
 let mut params = Vec::new();
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::RightParen { break; }
 let name = self.expect_identifier()?;
 let param_type = if self.match_token(TokenType::Colon) {
 self.parse_type()?
 } else {
 Type { name: "_", type_args: Vec::new(), is_mut_ref: false, is_ref: false, lifetime: None }
 };
 let is_mut = false;
 let default = None;
 params.push(Parameter { name, param_type, is_mut, default });
 if let Some(t) = self.peek() {
 if t.token_type == TokenType::Comma { self.advance(); }
 }
 }
 self.expect_token(TokenType::RightParen);
 Some(params)
 }

 /// Parse return type -> T
 fn parse_return_type(&mut self) -> Option<Option<Type>> {
 if self.match_token(TokenType::Arrow) {
 Some(Some(self.parse_type()?))
 } else {
 Some(None)
 }
 }

 /// Parse type
 fn parse_type(&mut self) -> Option<Type> {
 let token = self.advance()?;
 match token.token_type {
 TokenType::Identifier => {
 if let TokenValue::Identifier(name) = &token.value {
 Some(Type { name, type_args: Vec::new(), is_mut_ref: false, is_ref: false, lifetime: None })
 } else {
 None
 }
 }
 TokenType::Keyword => {
 let name = match token.keyword {
 Some(Keyword::Int) => "i32",
 Some(Keyword::Uint) => "u32",
 Some(Keyword::Float) => "f64",
 Some(Keyword::Bool) => "bool",
 Some(Keyword::Char) => "char",
 Some(Keyword::Str) => "str",
 _ => "_",
 };
 Some(Type { name, type_args: Vec::new(), is_mut_ref: false, is_ref: false, lifetime: None })
 }
 _ => {
 self.error_count += 1;
 None
 }
 }
 }

 /// Parse field list for struct
 fn parse_field_list(&mut self) -> Option<Vec<Field>> {
 if !self.match_token(TokenType::LeftBrace) {
 return Some(Vec::new());
 }
 let mut fields = Vec::new();
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::RightBrace { break; }
 let name = self.expect_identifier()?;
 self.expect_token(TokenType::Colon);
 let field_type = self.parse_type()?;
 let is_pub = false;
 let default = None;
 fields.push(Field { name, field_type, is_pub, default });
 if let Some(t) = self.peek() {
 if t.token_type == TokenType::Comma { self.advance(); }
 }
 }
 self.expect_token(TokenType::RightBrace);
 Some(fields)
 }

 /// Parse variant list for enum
 fn parse_variant_list(&mut self) -> Option<Vec<Variant>> {
 if !self.match_token(TokenType::LeftBrace) {
 return Some(Vec::new());
 }
 let mut variants = Vec::new();
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::RightBrace { break; }
 let name = self.expect_identifier()?;
 let data = if self.match_token(TokenType::LeftParen) {
 let mut types = Vec::new();
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::RightParen { break; }
 types.push(self.parse_type()?);
 if let Some(t) = self.peek() {
 if t.token_type == TokenType::Comma { self.advance(); }
 }
 }
 self.expect_token(TokenType::RightParen);
 Some(types)
 } else {
 None
 };
 variants.push(Variant { name, data });
 if let Some(t) = self.peek() {
 if t.token_type == TokenType::Comma { self.advance(); }
 }
 }
 self.expect_token(TokenType::RightBrace);
 Some(variants)
 }

 /// Parse method list for trait/impl
 fn parse_method_list(&mut self) -> Option<Vec<FunctionDef>> {
 if !self.match_token(TokenType::LeftBrace) {
 return Some(Vec::new());
 }
 let mut methods = Vec::new();
 while let Some(t) = self.peek() {
 if t.token_type == TokenType::RightBrace { break; }
 if t.token_type == TokenType::Keyword && t.keyword == Some(Keyword::Fn) {
 if let Some(AstNode::FunctionDef(f)) = self.parse_function_def() {
 methods.push(f);
 }
 }
 }
 self.expect_token(TokenType::RightBrace);
 Some(methods)
 }
}