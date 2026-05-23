/*
 * Nuva OS - System Library - Lang
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

/// AST Node Types
/// Nuva is a declarative programming language. The AST reflects
/// expression-oriented design where most constructs produce values.
#[derive(Debug, Clone)]
pub enum AstNode {
    /// Program root
    Program(Program),
    /// Function definition
    FunctionDef(FunctionDef),
    /// Struct definition
    StructDef(StructDef),
    /// Enum definition
    EnumDef(EnumDef),
    /// Trait definition
    TraitDef(TraitDef),
    /// Implementation block
    ImplBlock(ImplBlock),
    /// Variable declaration
    VarDecl(VarDecl),
    /// Expression statement
    ExprStmt(ExprStmt),
    /// Block expression
    Block(Block),
    /// If expression (not statement)
    IfExpr(IfExpr),
    /// Match expression
    MatchExpr(MatchExpr),
    /// Loop expression
    LoopExpr(LoopExpr),
    /// Return statement
    ReturnStmt(ReturnStmt),
    /// Break expression
    BreakExpr(BreakExpr),
    /// Continue expression
    ContinueExpr(ContinueExpr),
    /// Expression
    Expr(Expr),
    /// Pipeline expression (declarative)
    Pipeline(PipelineExpr),
    /// Comprehension expression (declarative)
    Comprehension(ComprehensionExpr),
    /// Component definition (declarative UI)
    ComponentDef(ComponentDef),
    /// Signal declaration (reactive data source)
    SignalDecl(SignalDecl),
    /// Effect declaration (reactive side effect)
    EffectDecl(EffectDecl),
    /// Resource declaration (declarative resource management)
    ResourceDecl(ResourceDecl),
    /// With block (resource scope)
    WithBlock(WithBlock),
}

/// Program
/// Top-level container for all declarations
#[derive(Debug, Clone)]
pub struct Program {
    /// Top-level declarations
    pub declarations: Vec<AstNode>,
}

/// Function Definition
/// Supports declarative annotations:
/// - pure: No side effects
/// - inline: Inline expansion
/// - async: Asynchronous execution
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// Function name
    pub name: &'static str,
    /// Generic type parameters
    pub type_params: Vec<&'static str>,
    /// Parameter list
    pub params: Vec<Parameter>,
    /// Return type
    pub return_type: Option<Type>,
    /// Function body (expression)
    pub body: Expr,
    /// Is async function
    pub is_async: bool,
    /// Is public
    pub is_pub: bool,
    /// Is pure function (no side effects) - declarative annotation
    pub is_pure: bool,
    /// Should be inlined
    pub is_inline: bool,
    /// Lifetime parameters
    pub lifetimes: Vec<&'static str>,
}

/// Parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: &'static str,
    /// Parameter type
    pub param_type: Type,
    /// Is mutable
    pub is_mut: bool,
    /// Default value (for optional parameters)
    pub default: Option<Expr>,
}

/// Type representation
#[derive(Debug, Clone)]
pub struct Type {
    /// Type name
    pub name: &'static str,
    /// Generic type arguments
    pub type_args: Vec<Type>,
    /// Is mutable reference
    pub is_mut_ref: bool,
    /// Is reference
    pub is_ref: bool,
    /// Lifetime annotation
    pub lifetime: Option<&'static str>,
}

/// Struct Definition
/// Immutable by default, supporting declarative data modeling
#[derive(Debug, Clone)]
pub struct StructDef {
    /// Struct name
    pub name: &'static str,
    /// Generic type parameters
    pub type_params: Vec<&'static str>,
    /// Field list
    pub fields: Vec<Field>,
    /// Is public
    pub is_pub: bool,
    /// Derive traits (declarative)
    pub derive: Vec<&'static str>,
}

/// Field
#[derive(Debug, Clone)]
pub struct Field {
    /// Field name
    pub name: &'static str,
    /// Field type
    pub field_type: Type,
    /// Is public
    pub is_pub: bool,
    /// Default value
    pub default: Option<Expr>,
}

/// Enum Definition
/// Algebraic data type for declarative pattern matching
#[derive(Debug, Clone)]
pub struct EnumDef {
    /// Enum name
    pub name: &'static str,
    /// Generic type parameters
    pub type_params: Vec<&'static str>,
    /// Variant list
    pub variants: Vec<Variant>,
    /// Is public
    pub is_pub: bool,
    /// Derive traits
    pub derive: Vec<&'static str>,
}

/// Variant
#[derive(Debug, Clone)]
pub struct Variant {
    /// Variant name
    pub name: &'static str,
    /// Associated data
    pub data: Option<Vec<Type>>,
}

/// Trait Definition
#[derive(Debug, Clone)]
pub struct TraitDef {
    /// Trait name
    pub name: &'static str,
    /// Generic type parameters
    pub type_params: Vec<&'static str>,
    /// Method list
    pub methods: Vec<FunctionDef>,
    /// Is public
    pub is_pub: bool,
    /// Associated types
    pub assoc_types: Vec<(&'static str, Option<Type>)>,
}

/// Implementation Block
#[derive(Debug, Clone)]
pub struct ImplBlock {
    /// Target type
    pub target_type: Type,
    /// Implemented trait
    pub trait_type: Option<Type>,
    /// Method list
    pub methods: Vec<FunctionDef>,
    /// Generic type parameters
    pub type_params: Vec<&'static str>,
}

/// Variable Declaration
/// Immutable by default (declarative principle)
#[derive(Debug, Clone)]
pub struct VarDecl {
    /// Variable name
    pub name: &'static str,
    /// Variable type
    pub var_type: Option<Type>,
    /// Initial value (required for immutability)
    pub init: Expr,
    /// Is mutable (explicit opt-out of immutability)
    pub is_mut: bool,
    /// Is constant (compile-time evaluated)
    pub is_const: bool,
}

/// Expression Statement
#[derive(Debug, Clone)]
pub struct ExprStmt {
    /// Expression
    pub expr: Expr,
}

/// Block Expression
/// Evaluates to the value of the last expression
#[derive(Debug, Clone)]
pub struct Block {
    /// Statement list
    pub statements: Vec<AstNode>,
    /// Final expression (block value)
    pub final_expr: Option<Box<Expr>>,
}

/// If Expression
/// Expression-oriented: always produces a value
#[derive(Debug, Clone)]
pub struct IfExpr {
    /// Condition
    pub condition: Expr,
    /// Then branch
    pub then_branch: Expr,
    /// Else branch (required for expression)
    pub else_branch: Expr,
}

/// Match Expression
/// Declarative pattern matching, always produces a value
#[derive(Debug, Clone)]
pub struct MatchExpr {
    /// Matched expression
    pub value: Expr,
    /// Match arms
    pub arms: Vec<MatchArm>,
}

/// Match Arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    /// Pattern
    pub pattern: Pattern,
    /// Guard condition
    pub guard: Option<Expr>,
    /// Arm body (expression)
    pub body: Expr,
}

/// Pattern
/// Declarative data extraction
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard pattern
    Wildcard,
    /// Literal pattern
    Literal(Literal),
    /// Identifier pattern (binding)
    Identifier(&'static str),
    /// Variant pattern
    Variant { name: &'static str, fields: Vec<Pattern> },
    /// Struct destructuring pattern
    Struct { name: &'static str, fields: Vec<(&'static str, Pattern)> },
    /// Range pattern
    Range { start: Literal, end: Literal, inclusive: bool },
    /// Tuple pattern
    Tuple(Vec<Pattern>),
    /// Or pattern
    Or(Vec<Pattern>),
}

/// Loop Expression
/// Supports declarative iteration
#[derive(Debug, Clone)]
pub enum LoopExpr {
    /// Infinite loop
    Infinite { body: Block },
    /// While loop (condition)
    While { condition: Expr, body: Block },
    /// For loop (iterator-based)
    For { var: &'static str, iter: Expr, body: Block },
    /// Range-based for loop
    ForRange { var: &'static str, start: Expr, end: Expr, inclusive: bool, body: Block },
}

/// Return Statement
#[derive(Debug, Clone)]
pub struct ReturnStmt {
    /// Return value
    pub value: Option<Expr>,
}

/// Break Expression
#[derive(Debug, Clone)]
pub struct BreakExpr {
    /// Break value (for expression-based loops)
    pub value: Option<Expr>,
}

/// Continue Expression
#[derive(Debug, Clone)]
pub struct ContinueExpr;

/// Expression
/// Core unit of declarative programming
#[derive(Debug, Clone)]
pub struct Expr {
    /// Expression kind
    pub kind: ExprKind,
    /// Type annotation (after type checking)
    pub ty: Option<Type>,
}

/// Expression Kind
/// All constructs are expressions in declarative Nuva
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Literal
    Literal(Literal),
    /// Identifier
    Identifier(&'static str),
    /// Binary operation
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    /// Unary operation
    Unary { op: UnaryOp, operand: Box<Expr> },
    /// Function call
    Call { callee: Box<Expr>, args: Vec<Expr> },
    /// Method call
    MethodCall { object: Box<Expr>, method: &'static str, args: Vec<Expr> },
    /// Field access
    FieldAccess { object: Box<Expr>, field: &'static str },
    /// Index access
    Index { object: Box<Expr>, index: Box<Expr> },
    /// Array literal
    Array(Vec<Expr>),
    /// Struct literal
    StructLiteral { name: &'static str, fields: Vec<(&'static str, Expr)>, base: Option<Box<Expr>> },
    /// Closure (lambda)
    Closure { params: Vec<Parameter>, body: Box<Expr> },
    /// If expression
    If { condition: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr> },
    /// Match expression
    Match { value: Box<Expr>, arms: Vec<MatchArm> },
    /// Block expression
    Block(Block),
    /// Reference
    Reference { expr: Box<Expr>, is_mut: bool },
    /// Dereference
    Dereference(Box<Expr>),
    /// Await
    Await(Box<Expr>),
    /// Range expression
    Range { start: Box<Expr>, end: Box<Expr>, inclusive: bool },
    /// Tuple expression
    Tuple(Vec<Expr>),
    /// Spread operator
    Spread(Box<Expr>),
    /// Try expression (?)
    Try(Box<Expr>),
    /// Lazy expression (delayed evaluation)
    Lazy(Box<Expr>),
    /// Pipeline expression
    Pipeline(PipelineExpr),
    /// Comprehension expression
    Comprehension(ComprehensionExpr),
}

/// Pipeline Expression
/// Declarative data transformation: data |> f1 |> f2 |> f3
#[derive(Debug, Clone)]
pub struct PipelineExpr {
    /// Initial value
    pub source: Expr,
    /// Pipeline stages
    pub stages: Vec<PipelineStage>,
}

/// Pipeline Stage
#[derive(Debug, Clone)]
pub enum PipelineStage {
    /// Method call
    Method { name: &'static str, args: Vec<Expr> },
    /// Function call
    Function { func: Expr, args: Vec<Expr> },
    /// Field access
    Field(&'static str),
    /// Filter operation
    Filter(Expr),
    /// Map operation
    Map(Expr),
    /// FlatMap operation
    FlatMap(Expr),
    /// Reduce operation
    Reduce { init: Expr, func: Expr },
    /// Tap (side effect, returns original)
    Tap(Expr),
}

/// Comprehension Expression
/// Declarative collection building: [x * 2 for x in list if x > 0]
/// Supports nested comprehensions: [x * y for x in list1 for y in list2]
#[derive(Debug, Clone)]
pub struct ComprehensionExpr {
    /// Output expression
    pub output: Expr,
    /// Iterators (supports multiple for nested comprehensions)
    pub iterators: Vec<ComprehensionIter>,
    /// Filter condition (optional)
    pub guard: Option<Expr>,
    /// Is a generator (lazy)
    pub is_generator: bool,
}

/// Comprehension Iterator
/// Represents a single iterator in a comprehension: for var in source
#[derive(Debug, Clone)]
pub struct ComprehensionIter {
    /// Iteration variable
    pub var: &'static str,
    /// Source iterable
    pub source: Expr,
}

/// Literal
#[derive(Debug, Clone)]
pub enum Literal {
    /// Integer
    Integer(i64),
    /// Unsigned integer
    Unsigned(u64),
    /// Float
    Float(f64),
    /// String
    String(&'static str),
    /// Character
    Char(char),
    /// Boolean
    Bool(bool),
    /// Unit
    Unit,
    /// None
    None,
}

/// Binary Operator
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Pow,
    // Comparison
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    // Logical
    And, Or, Xor,
    // Bitwise
    BitAnd, BitOr, BitXor, LeftShift, RightShift,
    // Pipeline (declarative)
    Pipeline,
    // Composition (declarative)
    Compose,
}

/// Unary Operator
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg, Not, BitNot, Dereference,
    // Try operator
    Try,
    // Lazy evaluation
    Lazy,
}

/// Component Definition (declarative UI)
/// Declarative UI component with reactive signals and effects
#[derive(Debug, Clone)]
pub struct ComponentDef {
    /// Component name
    pub name: &'static str,
    /// Component type parameters
    pub type_params: Vec<&'static str>,
    /// Component parameters (props)
    pub params: Vec<Parameter>,
    /// Signal declarations inside component
    pub signals: Vec<SignalDecl>,
    /// Effect declarations inside component
    pub effects: Vec<EffectDecl>,
    /// Component body (UI element tree)
    pub body: Block,
    /// Is public
    pub is_pub: bool,
}

/// Signal Declaration (reactive data source)
/// Reactive state that propagates changes to dependent effects and UI
#[derive(Debug, Clone)]
pub struct SignalDecl {
    /// Signal name
    pub name: &'static str,
    /// Signal type
    pub signal_type: Type,
    /// Initial value expression
    pub initial: Expr,
}

/// Effect Declaration (reactive side effect)
/// Runs when any referenced signal changes
#[derive(Debug, Clone)]
pub struct EffectDecl {
    /// Effect body (executed on signal change)
    pub body: Expr,
    /// Dependencies (signal names, filled during semantic analysis)
    pub dependencies: Vec<&'static str>,
}

/// Resource Declaration (declarative resource management)
/// RAII-style resource with automatic cleanup
#[derive(Debug, Clone)]
pub struct ResourceDecl {
    /// Resource name
    pub name: &'static str,
    /// Resource type
    pub resource_type: Type,
    /// Acquire expression
    pub acquire: Expr,
    /// Release expression (cleanup)
    pub release: Expr,
}

/// With Block (resource scope)
/// Ensures resource cleanup at scope exit
#[derive(Debug, Clone)]
pub struct WithBlock {
    /// Resource name
    pub resource: &'static str,
    /// Block body
    pub body: Block,
}
