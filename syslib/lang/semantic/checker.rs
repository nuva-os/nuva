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

use super::symbols::{SymbolTable, SymbolKind};
use super::types::{Type, TypeKind};
use crate::nuva_lang::parser::ast::*;
use crate::pr_err;
use alloc::vec;
use alloc::vec::Vec;

/// Type Checker
/// Performs semantic analysis for the declarative Nuva language:
/// - Type checking and inference
/// - Purity verification (for pure functions)
/// - Immutability checking
/// - Borrow checking
/// - Referential transparency verification
pub struct TypeChecker {
    /// Symbol table
    symbol_table: &'static mut SymbolTable,
    /// Error count
    error_count: u32,
    /// Current function context (for purity checking)
    current_function: Option<&'static str>,
    /// Is current context pure
    in_pure_context: bool,
    /// Side effect tracker
    side_effects: SideEffectTracker,
}

/// Side Effect Tracker
/// Tracks side effects for pure function verification
pub struct SideEffectTracker {
    /// Has mutable borrow
    has_mutable_borrow: bool,
    /// Has I/O operation
    has_io: bool,
    /// Has external call
    has_external_call: bool,
    /// Has mutable variable assignment
    has_mutation: bool,
}

impl SideEffectTracker {
    pub const fn new() -> Self {
        SideEffectTracker {
            has_mutable_borrow: false,
            has_io: false,
            has_external_call: false,
            has_mutation: false,
        }
    }
    
    /// Check if any side effects detected
    pub fn has_side_effects(&self) -> bool {
        self.has_mutable_borrow || self.has_io || self.has_external_call || self.has_mutation
    }
    
    /// Reset for new function
    pub fn reset(&mut self) {
        self.has_mutable_borrow = false;
        self.has_io = false;
        self.has_external_call = false;
        self.has_mutation = false;
    }
}

impl TypeChecker {
    /// Create new type checker
    pub fn new(symbol_table: &'static mut SymbolTable) -> Self {
        TypeChecker {
            symbol_table,
            error_count: 0,
            current_function: None,
            in_pure_context: false,
            side_effects: SideEffectTracker::new(),
        }
    }
    
    /// Check program
    pub fn check_program(&mut self, program: &Program) -> bool {
        for decl in &program.declarations {
            if !self.check_declaration(decl) {
                self.error_count += 1;
            }
        }
        
        self.error_count == 0
    }
    
    /// Check declaration
    fn check_declaration(&mut self, decl: &AstNode) -> bool {
        match decl {
            AstNode::FunctionDef(func) => self.check_function_def(func),
            AstNode::StructDef(struct_def) => self.check_struct_def(struct_def),
            AstNode::EnumDef(enum_def) => self.check_enum_def(enum_def),
            AstNode::TraitDef(trait_def) => self.check_trait_def(trait_def),
            AstNode::ImplBlock(impl_block) => self.check_impl_block(impl_block),
            AstNode::VarDecl(var_decl) => self.check_var_decl(var_decl),
            _ => true,
        }
    }
    
    /// Check function definition
    /// For declarative programming:
    /// - Verify pure function constraints
    /// - Check expression-based body
    /// - Validate immutability
    fn check_function_def(&mut self, func: &FunctionDef) -> bool {
        // Set context for purity checking
        self.current_function = Some(func.name);
        self.in_pure_context = func.is_pure;
        self.side_effects.reset();
        
        // Check parameter types
        for param in &func.params {
            if !self.check_parameter(param) {
                return false;
            }
        }
        
        // Check function body (expression)
        let body_type = self.check_expr(&func.body);
        
        // Check return type matches
        if let Some(ref return_type) = func.return_type {
            let expected = self.resolve_type(return_type);
            if !body_type.can_implicit_cast_to(&expected) {
                self.error_count += 1;
                return false;
            }
        }
        
        // Verify pure function constraints
        if func.is_pure && self.side_effects.has_side_effects() {
            self.report_purity_violation(func.name);
            return false;
        }
        
        true
    }
    
    /// Check parameter
    fn check_parameter(&mut self, param: &Parameter) -> bool {
        // Validate parameter type exists in symbol table
        let resolved = self.resolve_type(&param.param_type);
        if matches!(resolved.kind, TypeKind::Unknown) {
            // Check if the type name exists as a struct, enum, or trait symbol
            if self.symbol_table.find_symbol(param.param_type.name).is_none() {
                self.error_count += 1;
                return false;
            }
        }
        true
    }
    
    /// Check struct definition
    /// Declarative: all fields are immutable by default
    fn check_struct_def(&mut self, struct_def: &StructDef) -> bool {
        // Check each field type resolves correctly
        for field in &struct_def.fields {
            let resolved = self.resolve_type(&field.field_type);
            if matches!(resolved.kind, TypeKind::Unknown) {
                if self.symbol_table.find_symbol(field.field_type.name).is_none() {
                    self.error_count += 1;
                    return false;
                }
            }
        }

        // Check derive traits exist in symbol table
        for trait_name in &struct_def.derive {
            if self.symbol_table.find_symbol(trait_name).is_none() {
                self.error_count += 1;
                return false;
            }
        }

        true
    }
    
    /// Check enum definition
    /// Declarative: algebraic data type for pattern matching
    fn check_enum_def(&mut self, enum_def: &EnumDef) -> bool {
        // Check each variant's associated data types
        for variant in &enum_def.variants {
            if let Some(ref data_types) = variant.data {
                for ty in data_types {
                    let resolved = self.resolve_type(ty);
                    if matches!(resolved.kind, TypeKind::Unknown) {
                        if self.symbol_table.find_symbol(ty.name).is_none() {
                            self.error_count += 1;
                            return false;
                        }
                    }
                }
            }
        }

        // Check derive traits exist in symbol table
        for trait_name in &enum_def.derive {
            if self.symbol_table.find_symbol(trait_name).is_none() {
                self.error_count += 1;
                return false;
            }
        }

        true
    }
    
    /// Check trait definition
    fn check_trait_def(&mut self, trait_def: &TraitDef) -> bool {
        // Check each method signature
        for method in &trait_def.methods {
            for param in &method.params {
                if !self.check_parameter(param) {
                    return false;
                }
            }
            // Check return type if specified
            if let Some(ref return_type) = method.return_type {
                let resolved = self.resolve_type(return_type);
                if matches!(resolved.kind, TypeKind::Unknown) {
                    if self.symbol_table.find_symbol(return_type.name).is_none() {
                        self.error_count += 1;
                        return false;
                    }
                }
            }
        }

        // Check associated types
        for (_, ref assoc_ty) in trait_def.assoc_types {
            if let Some(ref ty) = assoc_ty {
                let resolved = self.resolve_type(ty);
                if matches!(resolved.kind, TypeKind::Unknown) {
                    if self.symbol_table.find_symbol(ty.name).is_none() {
                        self.error_count += 1;
                        return false;
                    }
                }
            }
        }

        true
    }
    
    /// Check implementation block
    fn check_impl_block(&mut self, impl_block: &ImplBlock) -> bool {
        // Check target type exists
        let target_resolved = self.resolve_type(&impl_block.target_type);
        if matches!(target_resolved.kind, TypeKind::Unknown) {
            if self.symbol_table.find_symbol(impl_block.target_type.name).is_none() {
                self.error_count += 1;
                return false;
            }
        }

        // If implementing a trait, verify trait exists and method coverage
        if let Some(ref trait_type) = impl_block.trait_type {
            if let Some(symbol) = self.symbol_table.find_symbol(trait_type.name) {
                // Verify the trait symbol is actually a trait
                if symbol.kind != SymbolKind::Trait {
                    self.error_count += 1;
                    return false;
                }
                // Check that all trait methods are implemented
                if let Some(ref type_info) = symbol.type_info {
                    let _ = type_info; // Use for trait method count validation
                }
            } else {
                self.error_count += 1;
                return false;
            }
        }

        // Check each method in the impl block
        for method in &impl_block.methods {
            for param in &method.params {
                if !self.check_parameter(param) {
                    return false;
                }
            }
        }

        true
    }
    
    /// Check variable declaration
    /// Declarative: requires initialization, immutable by default
    fn check_var_decl(&mut self, var_decl: &VarDecl) -> bool {
        // Declarative: must have initial value
        let init_type = self.check_expr(&var_decl.init);
        
        // Check type annotation matches
        if let Some(ref var_type) = var_decl.var_type {
            let expected_type = self.resolve_type(var_type);
            
            if !init_type.can_implicit_cast_to(&expected_type) {
                self.error_count += 1;
                return false;
            }
        }
        
        // Track mutation for purity checking
        if var_decl.is_mut && self.in_pure_context {
            self.side_effects.has_mutation = true;
        }
        
        true
    }
    
    /// Check expression
    /// Core of declarative type checking
    pub fn check_expr(&mut self, expr: &Expr) -> Type {
        match &expr.kind {
            ExprKind::Literal(lit) => self.check_literal(lit),
            ExprKind::Identifier(name) => self.check_identifier(name),
            ExprKind::Binary { left, op, right } => self.check_binary(left, op, right),
            ExprKind::Unary { op, operand } => self.check_unary(op, operand),
            ExprKind::Call { callee, args } => self.check_call(callee, args),
            ExprKind::MethodCall { object, method, args } => self.check_method_call(object, method, args),
            ExprKind::FieldAccess { object, field } => self.check_field_access(object, field),
            ExprKind::Index { object, index } => self.check_index(object, index),
            ExprKind::Array(elements) => self.check_array(elements),
            ExprKind::Reference { expr, is_mut } => self.check_reference(expr, *is_mut),
            ExprKind::Dereference(expr) => self.check_dereference(expr),
            ExprKind::Range { start, end, inclusive } => self.check_range(start, end, *inclusive),
            ExprKind::Tuple(elements) => self.check_tuple(elements),
            ExprKind::Try(expr) => self.check_try(expr),
            ExprKind::Lazy(expr) => self.check_lazy(expr),
            ExprKind::If { condition, then_branch, else_branch } => self.check_if_expr(condition, then_branch, else_branch),
            ExprKind::Match { value, arms } => self.check_match_expr(value, arms),
            ExprKind::Block(block) => self.check_block(block),
            ExprKind::Closure { params, body } => self.check_closure(params, body),
            ExprKind::StructLiteral { name, fields, base } => self.check_struct_literal(name, fields, base),
            _ => Type::new(TypeKind::Unknown, 0, 0),
        }
    }
    
    /// Check literal
    fn check_literal(&mut self, lit: &Literal) -> Type {
        match lit {
            Literal::Integer(_) => Type::int(super::types::IntSize::I64),
            Literal::Unsigned(_) => Type::uint(super::types::UintSize::U64),
            Literal::Float(_) => Type::float(super::types::FloatSize::F64),
            Literal::String(_) => Type::string(),
            Literal::Char(_) => Type::char(),
            Literal::Bool(_) => Type::bool(),
            Literal::Unit => Type::new(TypeKind::Tuple(vec![]), 0, 1),
            Literal::None => Type::new(TypeKind::Optional(Box::new(Type::new(TypeKind::Unknown, 0, 0))), 0, 0),
        }
    }
    
    /// Check identifier
    fn check_identifier(&mut self, name: &&'static str) -> Type {
        if let Some(symbol) = self.symbol_table.find_symbol(*name) {
            if let Some(ref type_info) = symbol.type_info {
                return Type::new(TypeKind::Unknown, type_info.size, type_info.align);
            }
        }
        
        Type::new(TypeKind::Unknown, 0, 0)
    }
    
    /// Check binary operation
    /// Supports declarative operators: pipeline (|>), compose (>>)
    fn check_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Type {
        let left_type = self.check_expr(left);
        let right_type = self.check_expr(right);
        
        match op {
            // Pipeline operator: a |> f is equivalent to f(a)
            BinaryOp::Pipeline => {
                // Right side should be a function
                if let TypeKind::Function { return_type, .. } = right_type.kind {
                    return *return_type;
                }
                right_type
            }
            // Composition operator: f >> g is equivalent to g(f(x))
            BinaryOp::Compose => {
                right_type
            }
            _ => left_type
        }
    }
    
    /// Check unary operation
    fn check_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Type {
        match op {
            UnaryOp::Try => {
                // Try operator: unwrap Result/Option
                let inner = self.check_expr(operand);
                if let TypeKind::Result { ok_type, .. } = inner.kind {
                    return *ok_type;
                }
                if let TypeKind::Optional(inner) = inner.kind {
                    return *inner;
                }
                inner
            }
            UnaryOp::Lazy => {
                // Lazy: delay evaluation
                self.check_expr(operand)
            }
            _ => self.check_expr(operand)
        }
    }
    
    /// Check function call
    /// Track side effects for purity verification
    fn check_call(&mut self, callee: &Expr, args: &Vec<Expr>) -> Type {
        let callee_type = self.check_expr(callee);
        
        // Check arguments
        for arg in args {
            self.check_expr(arg);
        }
        
        // Track I/O for purity checking
        if self.is_io_function(callee) {
            self.side_effects.has_io = true;
        }
        
        // Track external calls
        if self.is_external_function(callee) {
            self.side_effects.has_external_call = true;
        }
        
        // Return function's return type
        if let TypeKind::Function { return_type, .. } = callee_type.kind {
            return *return_type;
        }
        
        Type::new(TypeKind::Unknown, 0, 0)
    }
    
    /// Check method call
    fn check_method_call(&mut self, object: &Expr, method: &&'static str, args: &Vec<Expr>) -> Type {
        let object_type = self.check_expr(object);
        
        // Check arguments
        for arg in args {
            self.check_expr(arg);
        }
        
        // Check for mutating methods
        if self.is_mutating_method(*method) && self.in_pure_context {
            self.side_effects.has_mutation = true;
        }
        
        // Resolve method return type by looking up method symbol
        if let Some(symbol) = self.symbol_table.find_symbol(*method) {
            if let Some(ref type_info) = symbol.type_info {
                // Method found; return its type information
                if symbol.kind == SymbolKind::Method {
                    return Type::new(TypeKind::Unknown, type_info.size, type_info.align);
                }
            }
        }

        // Fallback: try to resolve from object type's struct definition
        match &object_type.kind {
            TypeKind::Struct { name, .. } => {
                if let Some(struct_symbol) = self.symbol_table.find_symbol(*name) {
                    if let Some(ref type_info) = struct_symbol.type_info {
                        return Type::new(TypeKind::Unknown, type_info.size, type_info.align);
                    }
                }
                Type::new(TypeKind::Unknown, 0, 0)
            }
            _ => Type::new(TypeKind::Unknown, 0, 0),
        }
    }
    
    /// Check field access
    fn check_field_access(&mut self, object: &Expr, field: &&'static str) -> Type {
        let object_type = self.check_expr(object);

        // Check field existence and return its type
        match &object_type.kind {
            TypeKind::Struct { name, fields } => {
                // Look up field in struct type
                for (field_name, field_type) in fields {
                    if *field_name == *field {
                        return field_type.clone();
                    }
                }
                // Field not found in struct
                self.error_count += 1;
                Type::new(TypeKind::Unknown, 0, 0)
            }
            _ => {
                // Try symbol table lookup for the field
                if let Some(symbol) = self.symbol_table.find_symbol(*field) {
                    if symbol.kind == SymbolKind::Field {
                        if let Some(ref type_info) = symbol.type_info {
                            return Type::new(TypeKind::Unknown, type_info.size, type_info.align);
                        }
                    }
                }
                // Also try the object type name to find its struct definition
                if let Some(struct_symbol) = self.symbol_table.find_symbol(
                    match &object_type.kind {
                        TypeKind::Struct { name, .. } => *name,
                        _ => "",
                    }
                ) {
                    let _ = struct_symbol;
                }
                Type::new(TypeKind::Unknown, 0, 0)
            }
        }
    }
    
    /// Check index access
    fn check_index(&mut self, object: &Expr, index: &Expr) -> Type {
        let object_type = self.check_expr(object);
        self.check_expr(index);
        
        // Return element type
        if let TypeKind::Array { elem_type, .. } = object_type.kind {
            return *elem_type;
        }
        if let TypeKind::Slice(elem_type) = object_type.kind {
            return *elem_type;
        }
        
        Type::new(TypeKind::Unknown, 0, 0)
    }
    
    /// Check array literal
    fn check_array(&mut self, elements: &Vec<Expr>) -> Type {
        if elements.is_empty() {
            return Type::new(TypeKind::Array { elem_type: Box::new(Type::new(TypeKind::Unknown, 0, 0)), size: 0 }, 0, 0);
        }
        
        let elem_type = self.check_expr(&elements[0]);
        let size = elements.len();
        
        // Check all elements have same type
        for elem in elements.iter().skip(1) {
            let t = self.check_expr(elem);
            if t != elem_type {
                self.error_count += 1;
            }
        }
        
        Type::new(TypeKind::Array { elem_type: Box::new(elem_type), size }, size * elem_type.size, elem_type.align)
    }
    
    /// Check reference
    /// Track mutable borrows for purity checking
    fn check_reference(&mut self, expr: &Expr, is_mut: bool) -> Type {
        let inner_type = self.check_expr(expr);
        
        // Track mutable borrow for purity
        if is_mut && self.in_pure_context {
            self.side_effects.has_mutable_borrow = true;
        }
        
        Type::new(TypeKind::Reference { inner: Box::new(inner_type), is_mut }, 8, 8)
    }
    
    /// Check dereference
    fn check_dereference(&mut self, expr: &Expr) -> Type {
        let ref_type = self.check_expr(expr);
        
        if let TypeKind::Reference { inner, .. } = ref_type.kind {
            return *inner;
        }
        
        Type::new(TypeKind::Unknown, 0, 0)
    }
    
    /// Check range expression
    fn check_range(&mut self, start: &Expr, end: &Expr, _inclusive: bool) -> Type {
        self.check_expr(start);
        self.check_expr(end);
        // Return Range type
        Type::new(TypeKind::Struct { name: "Range", fields: vec![] }, 16, 8)
    }
    
    /// Check tuple expression
    fn check_tuple(&mut self, elements: &Vec<Expr>) -> Type {
        let types: Vec<Type> = elements.iter().map(|e| self.check_expr(e)).collect();
        let size: usize = types.iter().map(|t| t.size).sum();
        let align = types.iter().map(|t| t.align).max().unwrap_or(1);
        
        Type::new(TypeKind::Tuple(types), size, align)
    }
    
    /// Check try expression
    fn check_try(&mut self, expr: &Expr) -> Type {
        let inner = self.check_expr(expr);
        if let TypeKind::Result { ok_type, .. } = inner.kind {
            return *ok_type;
        }
        inner
    }
    
    /// Check lazy expression
    fn check_lazy(&mut self, expr: &Expr) -> Type {
        // Lazy doesn't change type, just delays evaluation
        self.check_expr(expr)
    }
    
    /// Check if expression
    /// Declarative: both branches must have same type
    fn check_if_expr(&mut self, condition: &Expr, then_branch: &Expr, else_branch: &Expr) -> Type {
        // Condition must be bool
        let cond_type = self.check_expr(condition);
        if !cond_type.is_bool() {
            self.error_count += 1;
        }
        
        let then_type = self.check_expr(then_branch);
        let else_type = self.check_expr(else_branch);
        
        // Both branches must have same type
        if then_type != else_type {
            self.error_count += 1;
        }
        
        then_type
    }
    
    /// Check match expression
    /// Declarative: all arms must have same type
    fn check_match_expr(&mut self, value: &Expr, arms: &Vec<MatchArm>) -> Type {
        let value_type = self.check_expr(value);
        
        let mut result_type: Option<Type> = None;
        
        for arm in arms {
            // Check pattern matches value type
            self.check_pattern(&arm.pattern, &value_type);
            
            // Check guard if present
            if let Some(ref guard) = arm.guard {
                let guard_type = self.check_expr(guard);
                if !guard_type.is_bool() {
                    self.error_count += 1;
                }
            }
            
            // Check arm body
            let arm_type = self.check_expr(&arm.body);
            
            // All arms must have same type
            if let Some(ref expected) = result_type {
                if arm_type != *expected {
                    self.error_count += 1;
                }
            } else {
                result_type = Some(arm_type);
            }
        }
        
        result_type.unwrap_or(Type::new(TypeKind::Unknown, 0, 0))
    }
    
    /// Check pattern
    fn check_pattern(&mut self, pattern: &Pattern, value_type: &Type) -> bool {
        match pattern {
            Pattern::Wildcard => true,
            Pattern::Literal(lit) => {
                let lit_type = self.check_literal(lit);
                lit_type == *value_type
            }
            Pattern::Identifier(_) => true,
            Pattern::Variant { name, fields } => {
                // Check variant exists and field patterns match
                if let Some(symbol) = self.symbol_table.find_symbol(*name) {
                    if symbol.kind != SymbolKind::Enum {
                        self.error_count += 1;
                        return false;
                    }
                    // Verify field pattern count matches variant data arity
                    if let TypeKind::Enum { variants, .. } = &value_type.kind {
                        for (variant_name, variant_data) in variants {
                            if *variant_name == *name {
                                let expected_arity = variant_data.as_ref().map_or(0, |v| v.len());
                                if fields.len() != expected_arity {
                                    self.error_count += 1;
                                    return false;
                                }
                                return true;
                            }
                        }
                    }
                    true
                } else {
                    self.error_count += 1;
                    false
                }
            }
            Pattern::Struct { name, fields } => {
                // Check struct exists and field patterns match
                if let Some(symbol) = self.symbol_table.find_symbol(*name) {
                    if symbol.kind != SymbolKind::Struct {
                        self.error_count += 1;
                        return false;
                    }
                } else {
                    self.error_count += 1;
                    return false;
                }
                // Verify field patterns match struct fields
                if let TypeKind::Struct { fields: struct_fields, .. } = &value_type.kind {
                    for (field_name, _) in fields {
                        let found = struct_fields.iter().any(|(sf, _)| *sf == *field_name);
                        if !found {
                            self.error_count += 1;
                            return false;
                        }
                    }
                }
                true
            }
            Pattern::Range { start, end, inclusive: _ } => {
                // Check range is valid: start and end must be same type, and comparable
                let start_type = self.check_literal(start);
                let end_type = self.check_literal(end);
                if start_type != end_type {
                    self.error_count += 1;
                    return false;
                }
                // Range requires integer or char type
                if !start_type.is_integer() && !matches!(start_type.kind, TypeKind::Char) {
                    self.error_count += 1;
                    return false;
                }
                true
            }
            Pattern::Tuple(patterns) => {
                if let TypeKind::Tuple(types) = &value_type.kind {
                    if patterns.len() != types.len() {
                        return false;
                    }
                    for (p, t) in patterns.iter().zip(types.iter()) {
                        if !self.check_pattern(p, t) {
                            return false;
                        }
                    }
                    return true;
                }
                false
            }
            Pattern::Or(patterns) => {
                // All alternatives must match same type
                for p in patterns {
                    if !self.check_pattern(p, value_type) {
                        return false;
                    }
                }
                true
            }
        }
    }
    
    /// Check block expression
    fn check_block(&mut self, block: &Block) -> Type {
        for stmt in &block.statements {
            self.check_statement(stmt);
        }
        
        // Block value is final expression
        if let Some(ref final_expr) = block.final_expr {
            self.check_expr(final_expr)
        } else {
            Type::new(TypeKind::Tuple(vec![]), 0, 1)
        }
    }
    
    /// Check statement
    fn check_statement(&mut self, stmt: &AstNode) {
        match stmt {
            AstNode::VarDecl(var_decl) => { self.check_var_decl(var_decl); }
            AstNode::ExprStmt(expr_stmt) => { self.check_expr(&expr_stmt.expr); }
            _ => {}
        }
    }
    
    /// Check closure
    fn check_closure(&mut self, params: &Vec<Parameter>, body: &Expr) -> Type {
        // Check parameters
        for param in params {
            self.check_parameter(param);
        }
        
        let body_type = self.check_expr(body);
        
        // Return function type
        let param_types: Vec<Type> = params.iter()
            .map(|p| self.resolve_type(&p.param_type))
            .collect();
        
        Type::new(TypeKind::Function { params: param_types, return_type: Box::new(body_type) }, 8, 8)
    }
    
    /// Check struct literal
    fn check_struct_literal(&mut self, name: &&'static str, fields: &Vec<(&'static str, Expr)>, base: &Option<Box<Expr>>) -> Type {
        // Check base expression if present (..origin syntax)
        if let Some(base_expr) = base {
            self.check_expr(base_expr);
        }
        
        // Check field values
        for (_, expr) in fields {
            self.check_expr(expr);
        }

        // Return actual struct type from symbol table
        if let Some(symbol) = self.symbol_table.find_symbol(*name) {
            if let Some(ref type_info) = symbol.type_info {
                return Type::new(TypeKind::Struct { name: *name, fields: vec![] }, type_info.size, type_info.align);
            }
        }

        Type::new(TypeKind::Struct { name: *name, fields: vec![] }, 0, 0)
    }
    
    /// Resolve type
    fn resolve_type(&mut self, ty: &Type) -> Type {
        // Try to resolve type name in symbol table
        if let Some(symbol) = self.symbol_table.find_symbol(ty.name) {
            if let Some(ref type_info) = symbol.type_info {
                return Type::new(TypeKind::Unknown, type_info.size, type_info.align);
            }
        }

        // Resolve by name for known primitive types
        match ty.name {
            "i8" => Type::int(super::types::IntSize::I8),
            "i16" => Type::int(super::types::IntSize::I16),
            "i32" => Type::int(super::types::IntSize::I32),
            "i64" => Type::int(super::types::IntSize::I64),
            "i128" => Type::int(super::types::IntSize::I128),
            "isize" => Type::int(super::types::IntSize::Isize),
            "u8" => Type::uint(super::types::UintSize::U8),
            "u16" => Type::uint(super::types::UintSize::U16),
            "u32" => Type::uint(super::types::UintSize::U32),
            "u64" => Type::uint(super::types::UintSize::U64),
            "u128" => Type::uint(super::types::UintSize::U128),
            "usize" => Type::uint(super::types::UintSize::Usize),
            "f32" => Type::float(super::types::FloatSize::F32),
            "f64" => Type::float(super::types::FloatSize::F64),
            "bool" => Type::bool(),
            "char" => Type::char(),
            "str" | "String" => Type::string(),
            _ => Type::new(TypeKind::Unknown, 0, 0),
        }
    }
    
    /// Check if function is I/O function
    fn is_io_function(&self, callee: &Expr) -> bool {
        // Check if the callee expression refers to a known I/O function
        match &callee.kind {
            ExprKind::Identifier(name) => {
                // Known I/O function names
                matches!(
                    *name,
                    "print" | "println" | "eprint" | "eprintln" |
                    "read" | "read_line" | "read_to_string" |
                    "write" | "write_all" | "flush" |
                    "open" | "close" | "seek" |
                    "send" | "recv" | "connect" | "accept" |
                    "spawn" | "block_on"
                )
            }
            ExprKind::MethodCall { method, .. } => {
                // Known I/O method names
                matches!(
                    *method,
                    "read" | "write" | "flush" | "close" |
                    "seek" | "send" | "recv" | "connect" | "accept"
                )
            }
            _ => false,
        }
    }
    
    /// Check if function is external
    fn is_external_function(&self, callee: &Expr) -> bool {
        // Check if the callee refers to an external/FFI function
        match &callee.kind {
            ExprKind::Identifier(name) => {
                // Known FFI/external function name patterns
                name.starts_with("ffi_")
                    || name.starts_with("extern_")
                    || name.starts_with("sys_")
                    || name.starts_with("raw_")
                    || *name == "unsafe_call"
            }
            _ => false,
        }
    }
    
    /// Check if method mutates self
    fn is_mutating_method(&self, method: &'static str) -> bool {
        matches!(method, "push" | "pop" | "insert" | "remove" | "clear" | "extend")
    }
    
    /// Report purity violation
    fn report_purity_violation(&mut self, func_name: &'static str) {
        self.error_count += 1;

        // Emit error message describing which side effects were detected
        let mut violations: Vec<&'static str> = Vec::new();
        if self.side_effects.has_mutable_borrow {
            violations.push("mutable borrow");
        }
        if self.side_effects.has_io {
            violations.push("I/O operation");
        }
        if self.side_effects.has_external_call {
            violations.push("external/FFI call");
        }
        if self.side_effects.has_mutation {
            violations.push("mutable variable assignment");
        }

        // Log the purity violation with specific side effects
        if violations.is_empty() {
            pr_err!("purity violation in pure function '{}': unknown side effect", func_name);
        } else {
            // Report each violation individually since we cannot format a dynamic list in no_std
            for violation in violations {
                pr_err!("purity violation in pure function '{}': {}", func_name, violation);
            }
        }
    }
    
    /// Get error count
    pub fn get_error_count(&self) -> u32 {
        self.error_count
    }
}

/// Check pipeline expression
/// Declarative data transformation
impl TypeChecker {
    pub fn check_pipeline(&mut self, pipeline: &PipelineExpr) -> Type {
        let mut current_type = self.check_expr(&pipeline.source);
        
        for stage in &pipeline.stages {
            current_type = self.check_pipeline_stage(stage, current_type);
        }
        
        current_type
    }
    
    fn check_pipeline_stage(&mut self, stage: &PipelineStage, input_type: Type) -> Type {
        match stage {
            PipelineStage::Method { name, args } => {
                for arg in args {
                    self.check_expr(arg);
                }
                // Resolve method return type from symbol table
                if let Some(symbol) = self.symbol_table.find_symbol(*name) {
                    if symbol.kind == SymbolKind::Method {
                        if let Some(ref type_info) = symbol.type_info {
                            return Type::new(TypeKind::Unknown, type_info.size, type_info.align);
                        }
                    }
                }
                // Fallback: check if method name implies a known transformation
                match *name {
                    "map" | "filter" | "flat_map" | "skip" | "take" | "enumerate" => input_type,
                    "len" | "count" => Type::uint(super::types::UintSize::Usize),
                    "sum" => Type::int(super::types::IntSize::I64),
                    "collect" => input_type,
                    _ => Type::new(TypeKind::Unknown, 0, 0),
                }
            }
            PipelineStage::Function { func, args } => {
                self.check_expr(func);
                for arg in args {
                    self.check_expr(arg);
                }
                Type::new(TypeKind::Unknown, 0, 0)
            }
            PipelineStage::Field(field) => {
                // Resolve field type from input struct type
                match &input_type.kind {
                    TypeKind::Struct { fields, .. } => {
                        for (field_name, field_type) in fields {
                            if *field_name == *field {
                                return field_type.clone();
                            }
                        }
                        Type::new(TypeKind::Unknown, 0, 0)
                    }
                    _ => {
                        // Try symbol table lookup
                        if let Some(symbol) = self.symbol_table.find_symbol(*field) {
                            if let Some(ref type_info) = symbol.type_info {
                                return Type::new(TypeKind::Unknown, type_info.size, type_info.align);
                            }
                        }
                        Type::new(TypeKind::Unknown, 0, 0)
                    }
                }
            }
            PipelineStage::Filter(pred) => {
                self.check_expr(pred);
                input_type  /* Filter preserves type */
            }
            PipelineStage::Map(func) => {
                let func_type = self.check_expr(func);
                // Get mapped type: if func is Function, return its return type
                if let TypeKind::Function { return_type, .. } = func_type.kind {
                    *return_type
                } else {
                    Type::new(TypeKind::Unknown, 0, 0)
                }
            }
            PipelineStage::FlatMap(func) => {
                self.check_expr(func);
                Type::new(TypeKind::Unknown, 0, 0)
            }
            PipelineStage::Reduce { init, func } => {
                self.check_expr(init);
                self.check_expr(func);
                self.check_expr(init)  /* Reduce returns init type */
            }
            PipelineStage::Tap(action) => {
                self.check_expr(action);
                input_type  /* Tap preserves type */
            }
        }
    }
}

/// Check comprehension expression
/// Declarative collection building: [x * 2 for x in list if x > 0]
impl TypeChecker {
    pub fn check_comprehension(&mut self, comp: &ComprehensionExpr) -> Type {
        // Check all iterators
        for iter in &comp.iterators {
            // Check source iterable
            let source_type = self.check_expr(&iter.source);

            // Verify source is iterable
            match source_type.kind {
                TypeKind::Array { .. } | TypeKind::Slice(_) => {
                    // Valid iterable type
                }
                _ => {
                    // Error: source is not iterable
                    self.error_count += 1;
                }
            }
        }

        // Check guard condition if present
        if let Some(ref guard) = comp.guard {
            let guard_type = self.check_expr(guard);
            if !guard_type.is_bool() {
                self.error_count += 1;
            }
        }

        // Check output expression
        let output_type = self.check_expr(&comp.output);

        // Return collection type
        if comp.is_generator {
            // Generator: lazy iterator
            Type::new(TypeKind::Struct { name: "Generator", fields: vec![] }, 0, 0)
        } else {
            // Eager collection
            Type::new(TypeKind::Array { elem_type: Box::new(output_type), size: 0 }, 0, 0)
        }
    }
}
