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

// ! Nuva languagelanguage AST Definition

use crate::Nuva_compiler::lexer::TokenKind;

/// AST Node ID
pub type NodeId = u32;

/// sourcecodePosition
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceLocation {
 pub line: u32,
 pub column: u32,
 pub offset: u32,
}

/// Typeformreachstyle
#[derive(Debug, Clone)]
pub enum TypeExpr {
 /// simpleformTypename
 Simple {
 name: [u8; 64],
 name_len: u8,
 },
 /// GenericType
 Generic {
 base: [u8; 64],
 base_len: u8,
 args: [TypeExpr; 4],
 num_args: u8,
 },
 /// optionalType
 Optional {
 inner: Box<TypeExpr>,
 },
 /// ArrayType
 Array {
 element: Box<TypeExpr>,
 size: Option<u32>,
 },
 /// GroupType
 Tuple {
 elements: [TypeExpr; 8],
 num_elements: u8,
 },
 /// FunctionType
 Function {
 params: [TypeExpr; 8],
 num_params: u8,
 return_type: Box<TypeExpr>,
 is_async: bool,
 throws: bool,
 },
}

/// formreachstyle
#[derive(Debug, Clone)]
pub enum Expr {
 /// IntegerLiteral
 IntegerLiteral {
 value: u64,
 loc: SourceLocation,
 },
 /// DotLiteral
 FloatLiteral {
 value: f64,
 loc: SourceLocation,
 },
 /// StringLiteral
 StringLiteral {
 value: [u8; 256],
 value_len: u8,
 loc: SourceLocation,
 },
 /// booleanLiteral
 BoolLiteral {
 value: bool,
 loc: SourceLocation,
 },
 /// nil Literal
 NilLiteral {
 loc: SourceLocation,
 },
 /// Identifier
 Identifier {
 name: [u8; 64],
 name_len: u8,
 loc: SourceLocation,
 },
 /// binary operationcalculation
 Binary {
 op: BinaryOp,
 left: Box<Expr>,
 right: Box<Expr>,
 loc: SourceLocation,
 },
 /// aoperationcalculation
 Unary {
 op: UnaryOp,
 operand: Box<Expr>,
 loc: SourceLocation,
 },
 /// Function Calling
 Call {
 callee: Box<Expr>,
 args: [Expr; 8],
 num_args: u8,
 loc: SourceLocation,
 },
 /// Memberaccess
 Member {
 object: Box<Expr>,
 member: [u8; 64],
 member_len: u8,
 loc: SourceLocation,
 },
 /// downloadstandardaccess
 Subscript {
 object: Box<Expr>,
 index: Box<Expr>,
 loc: SourceLocation,
 },
 /// ArrayLiteral
 ArrayLiteral {
 elements: [Expr; 16],
 num_elements: u8,
 loc: SourceLocation,
 },
 /// WordLiteral
 DictionaryLiteral {
 keys: [Expr; 16],
 values: [Expr; 16],
 num_pairs: u8,
 loc: SourceLocation,
 },
 /// closedPackage
 Closure {
 params: [Parameter; 8],
 num_params: u8,
 return_type: Option<TypeExpr>,
 body: Box<Stmt>,
 is_async: bool,
 loc: SourceLocation,
 },
 /// stripcase
 Ternary {
 condition: Box<Expr>,
 then_expr: Box<Expr>,
 else_expr: Box<Expr>,
 loc: SourceLocation,
 },
 /// optionallink
 OptionalChain {
 expr: Box<Expr>,
 loc: SourceLocation,
 },
 /// ForcesolvePackage
 ForceUnwrap {
 expr: Box<Expr>,
 loc: SourceLocation,
 },
 /// Typeconvert
 Cast {
 expr: Box<Expr>,
 target_type: TypeExpr,
 loc: SourceLocation,
 },
 /// await formreachstyle
 Await {
 expr: Box<Expr>,
 loc: SourceLocation,
 },
}

/// binaryOperator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
 Add,
 Sub,
 Mul,
 Div,
 Mod,
 And,
 Or,
 Eq,
 Ne,
 Lt,
 Le,
 Gt,
 Ge,
 BitAnd,
 BitOr,
 BitXor,
 Shl,
 Shr,
 NilCoalesce,
 Range,
 RangeInclusive,
 Assign,
 AddAssign,
 SubAssign,
 MulAssign,
 DivAssign,
 ModAssign,
}

impl BinaryOp {
 pub fn from_token(kind: TokenKind) -> Option<Self> {
 match kind {
 TokenKind::Plus => Some(Self::Add),
 TokenKind::Minus => Some(Self::Sub),
 TokenKind::Star => Some(Self::Mul),
 TokenKind::Slash => Some(Self::Div),
 TokenKind::Percent => Some(Self::Mod),
 TokenKind::AmpAmp => Some(Self::And),
 TokenKind::PipePipe => Some(Self::Or),
 TokenKind::EqEq => Some(Self::Eq),
 TokenKind::BangEq => Some(Self::Ne),
 TokenKind::Lt => Some(Self::Lt),
 TokenKind::LtEq => Some(Self::Le),
 TokenKind::Gt => Some(Self::Gt),
 TokenKind::GtEq => Some(Self::Ge),
 TokenKind::Amp => Some(Self::BitAnd),
 TokenKind::Pipe => Some(Self::BitOr),
 TokenKind::Caret => Some(Self::BitXor),
 TokenKind::LtLt => Some(Self::Shl),
 TokenKind::GtGt => Some(Self::Shr),
 TokenKind::NilCoalesce => Some(Self::NilCoalesce),
 TokenKind::Range => Some(Self::Range),
 TokenKind::RangeInclusive => Some(Self::RangeInclusive),
 TokenKind::Eq => Some(Self::Assign),
 TokenKind::PlusEq => Some(Self::AddAssign),
 TokenKind::MinusEq => Some(Self::SubAssign),
 TokenKind::StarEq => Some(Self::MulAssign),
 TokenKind::SlashEq => Some(Self::DivAssign),
 TokenKind::PercentEq => Some(Self::ModAssign),
 _ => None,
 }
 }
}

/// aOperator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
 Neg,
 Not,
 BitNot,
 Deref,
 Ref,
}

impl UnaryOp {
 pub fn from_token(kind: TokenKind) -> Option<Self> {
 match kind {
 TokenKind::Minus => Some(Self::Neg),
 TokenKind::Exclaim => Some(Self::Not),
 TokenKind::Tilde => Some(Self::BitNot),
 _ => None,
 }
 }
}

/// languagesentence
#[derive(Debug, Clone)]
pub enum Stmt {
 /// formreachstylelanguagesentence
 Expr {
 expr: Expr,
 loc: SourceLocation,
 },
 /// VariableDeclaration
 VarDecl {
 name: [u8; 64],
 name_len: u8,
 type_annotation: Option<TypeExpr>,
 init: Option<Expr>,
 is_mutable: bool,
 loc: SourceLocation,
 },
 /// if languagesentence
 If {
 condition: Expr,
 then_branch: Box<Stmt>,
 else_branch: Option<Box<Stmt>>,
 loc: SourceLocation,
 },
 /// while Ring
 While {
 condition: Expr,
 body: Box<Stmt>,
 loc: SourceLocation,
 },
 /// for-in Ring
 ForIn {
 var_name: [u8; 64],
 var_name_len: u8,
 iterable: Expr,
 body: Box<Stmt>,
 loc: SourceLocation,
 },
 /// switch languagesentence
 Switch {
 subject: Expr,
 cases: [SwitchCase; 16],
 num_cases: u8,
 default_body: Option<Box<Stmt>>,
 loc: SourceLocation,
 },
 /// return languagesentence
 Return {
 value: Option<Expr>,
 loc: SourceLocation,
 },
 /// throw languagesentence
 Throw {
 error: Expr,
 loc: SourceLocation,
 },
 /// break languagesentence
 Break {
 loc: SourceLocation,
 },
 /// continue languagesentence
 Continue {
 loc: SourceLocation,
 },
 /// Blocklanguagesentence
 Block {
 stmts: [Stmt; 64],
 num_stmts: u8,
 loc: SourceLocation,
 },
 /// defer languagesentence
 Defer {
 body: Box<Stmt>,
 loc: SourceLocation,
 },
}

/// switch case
#[derive(Debug, Clone)]
pub struct SwitchCase {
 pub pattern: Pattern,
 pub guard: Option<Expr>,
 pub body: Stmt,
}

/// Mode
#[derive(Debug, Clone)]
pub enum Pattern {
 /// matchsymbol _
 Wildcard,
 /// valueBind
 Binding {
 name: [u8; 64],
 name_len: u8,
 type_annotation: Option<TypeExpr>,
 },
 /// GroupMode
 Tuple {
 elements: [Pattern; 8],
 num_elements: u8,
 },
 /// Enum case
 EnumCase {
 type_name: [u8; 64],
 type_name_len: u8,
 case_name: [u8; 64],
 case_name_len: u8,
 associated: Option<Box<Pattern>>,
 },
 /// valueMatch
 Value {
 value: Expr,
 },
}

/// Parameter
#[derive(Debug, Clone)]
pub struct Parameter {
 pub external_name: [u8; 64],
 pub external_name_len: u8,
 pub internal_name: [u8; 64],
 pub internal_name_len: u8,
 pub type_annotation: TypeExpr,
 pub default_value: Option<Expr>,
 pub is_inout: bool,
}

/// Declaration
#[derive(Debug, Clone)]
pub enum Decl {
 /// FunctionDeclaration
 Func {
 name: [u8; 64],
 name_len: u8,
 params: [Parameter; 8],
 num_params: u8,
 return_type: Option<TypeExpr>,
 body: Option<Stmt>,
 is_async: bool,
 throws: bool,
 is_static: bool,
 access: AccessLevel,
 loc: SourceLocation,
 },
 /// ClassDeclaration
 Class {
 name: [u8; 64],
 name_len: u8,
 super_class: Option<TypeExpr>,
 protocols: [TypeExpr; 8],
 num_protocols: u8,
 members: [Decl; 64],
 num_members: u8,
 access: AccessLevel,
 loc: SourceLocation,
 },
 /// StructDeclaration
 Struct {
 name: [u8; 64],
 name_len: u8,
 protocols: [TypeExpr; 8],
 num_protocols: u8,
 members: [Decl; 64],
 num_members: u8,
 access: AccessLevel,
 loc: SourceLocation,
 },
 /// EnumDeclaration
 Enum {
 name: [u8; 64],
 name_len: u8,
 raw_type: Option<TypeExpr>,
 cases: [EnumCase; 32],
 num_cases: u8,
 members: [Decl; 32],
 num_members: u8,
 access: AccessLevel,
 loc: SourceLocation,
 },
 /// ProtocolDeclaration
 Protocol {
 name: [u8; 64],
 name_len: u8,
 parent_protocols: [TypeExpr; 8],
 num_parents: u8,
 requirements: [Decl; 32],
 num_requirements: u8,
 access: AccessLevel,
 loc: SourceLocation,
 },
 /// ScalingDeclaration
 Extension {
 extended_type: TypeExpr,
 protocols: [TypeExpr; 8],
 num_protocols: u8,
 members: [Decl; 32],
 num_members: u8,
 access: AccessLevel,
 loc: SourceLocation,
 },
 /// conductenterDeclaration
 Import {
 module: [u8; 128],
 module_len: u8,
 loc: SourceLocation,
 },
}

/// Enum case
#[derive(Debug, Clone)]
pub struct EnumCase {
 pub name: [u8; 64],
 pub name_len: u8,
 pub associated_values: [TypeExpr; 4],
 num_associated: u8,
 pub raw_value: Option<Expr>,
}

/// accessLevel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessLevel {
 Private,
 Internal,
 Public,
}

/// Module
#[derive(Debug, Clone)]
pub struct Module {
 pub name: [u8; 64],
 pub name_len: u8,
 pub declarations: [Decl; 256],
 pub num_declarations: u16,
}

impl Module {
 pub fn new(name: &[u8]) -> Self {
 let mut name_buf = [0u8; 64];
 let len = name.len().min(63);
 name_buf[..len].copy_from_slice(&name[..len]);
 
 Self {
 name: name_buf,
 name_len: len as u8,
 declarations: [Decl; 256],
 num_declarations: 0,
 }
 }

 pub fn add_decl(&mut self, decl: Decl) {
 let idx = self.num_declarations as usize;
 if idx < 256 {
 self.declarations[idx] = decl;
 self.num_declarations += 1;
 }
 }
}