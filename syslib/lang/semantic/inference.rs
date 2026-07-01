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

use crate::nuva_lang::parser::ast::*;
use crate::nuva_lang::semantic::types::{Type, TypeKind, TypeEnv};

/// Type Inference
/// Implements Hindley-Milner type inference (Algorithm W) for
/// automatic type deduction in declarative expressions.

/// Type Variable
/// Represents an unknown type that needs to be inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar {
    /// Unique identifier
    pub id: u32,
}

/// Type Scheme
/// Represents polymorphic types: ∀α₁...αₙ. τ
#[derive(Debug, Clone)]
pub struct TypeScheme {
    /// Quantified type variables
    pub vars: Vec<TypeVar>,
    /// The type
    pub ty: Type,
}

/// Type Constraint
/// Represents an equality constraint between two types.
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    /// Left-hand side type
    pub left: Type,
    /// Right-hand side type
    pub right: Type,
}

/// Substitution
/// Maps type variables to types.
#[derive(Debug, Clone)]
pub struct Substitution {
    /// Mapping from type variables to types
    pub mapping: Vec<(TypeVar, Type)>,
}

impl Substitution {
    /// Create empty substitution
    pub fn new() -> Self {
        Substitution { mapping: Vec::new() }
    }

    /// Apply substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        match &ty.kind {
            TypeKind::TypeParam(id) => {
                // Look up type variable in substitution mapping
                for (var, replacement) in &self.mapping {
                    if var.id == *id as u32 {
                        return self.apply(replacement);
                    }
                }
                ty.clone()
            }
            TypeKind::Function { params, ret } => Type::new(
                TypeKind::Function {
                    params: params.iter().map(|p| self.apply(p)).collect(),
                    ret: Box::new(self.apply(ret)),
                },
                ty.line,
                ty.column,
            ),
            TypeKind::Array { elem_type, size } => Type::new(
                TypeKind::Array {
                    elem_type: Box::new(self.apply(elem_type)),
                    size: *size,
                },
                ty.line,
                ty.column,
            ),
            TypeKind::Optional(inner) => Type::new(
                TypeKind::Optional(Box::new(self.apply(inner))),
                ty.line,
                ty.column,
            ),
            TypeKind::Tuple(elems) => Type::new(
                TypeKind::Tuple(elems.iter().map(|e| self.apply(e)).collect()),
                ty.line,
                ty.column,
            ),
            _ => ty.clone(),
        }
    }

    /// Compose two substitutions
    /// Result S satisfies: S(t) = self(other(t))
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = Substitution::new();

        // For each mapping in self, apply other to the replacement type first
        for (var, ty) in &self.mapping {
            let composed_ty = other.apply(ty);
            result.mapping.push((*var, composed_ty));
        }

        // Add mappings from other that are not already in self
        for (var, ty) in &other.mapping {
            let already_in_self = self.mapping.iter().any(|(v, _)| v.id == var.id);
            if !already_in_self {
                result.mapping.push((*var, ty.clone()));
            }
        }

        result
    }
}

/// Type Inference Engine
/// Implements Algorithm W for Hindley-Milner type inference.
pub struct TypeInference {
    /// Next type variable ID
    next_var_id: u32,
    /// Generated constraints
    constraints: Vec<TypeConstraint>,
    /// Current substitution
    substitution: Substitution,
    /// Type environment for variable bindings
    type_env: TypeEnv,
}

impl TypeInference {
    /// Create new type inference engine
    pub fn new() -> Self {
        TypeInference {
            next_var_id: 0,
            constraints: Vec::new(),
            substitution: Substitution::new(),
            type_env: TypeEnv::new(),
        }
    }

    /// Generate fresh type variable
    pub fn fresh_var(&mut self) -> TypeVar {
        let var = TypeVar { id: self.next_var_id };
        self.next_var_id += 1;
        var
    }

    /// Generate fresh type
    pub fn fresh_type(&mut self) -> Type {
        Type::new(TypeKind::TypeParam(self.next_var_id as usize), 0, 0)
    }

    /// Algorithm W: Infer type for an expression
    /// Returns the inferred type and a substitution.
    pub fn infer(&mut self, expr: &Expr) -> Result<Type, InferenceError> {
        match &expr.kind {
            // Literals have known types
            ExprKind::Literal(lit) => Ok(self.infer_literal(lit)),

            // Identifiers: look up in type environment
            ExprKind::Identifier(name) => {
                // Look up identifier in type environment
                if let Some(ty) = self.type_env.find(*name) {
                    Ok(ty.clone())
                } else {
                    // Unknown identifier: create fresh type variable for later inference
                    Ok(self.fresh_type())
                }
            }

            // Binary operations
            ExprKind::Binary { left, op, right } => {
                self.infer_binary(left, op, right)
            }

            // Unary operations
            ExprKind::Unary { op, operand } => {
                self.infer_unary(op, operand)
            }

            // Function application
            ExprKind::Call { callee, args } => {
                self.infer_call(callee, args)
            }

            // Pipeline expressions
            ExprKind::Pipeline(pipeline) => {
                self.infer_pipeline(pipeline)
            }

            // Comprehension expressions
            ExprKind::Comprehension(comp) => {
                self.infer_comprehension(comp)
            }

            // If expressions
            ExprKind::If { condition, then_branch, else_branch } => {
                self.infer_if(condition, then_branch, else_branch)
            }

            // Match expressions
            ExprKind::Match { value, arms } => {
                self.infer_match(value, arms)
            }

            // Lambda expressions
            ExprKind::Closure { params, body } => {
                self.infer_closure(params, body)
            }

            // Default: return fresh type variable
            _ => Ok(self.fresh_type()),
        }
    }

    /// Infer type for literal
    fn infer_literal(&mut self, lit: &Literal) -> Type {
        match lit {
            Literal::Integer(_) => Type::int(),
            Literal::Unsigned(_) => Type::uint(),
            Literal::Float(_) => Type::float(),
            Literal::String(_) => Type::string(),
            Literal::Char(_) => Type::char(),
            Literal::Bool(_) => Type::bool(),
            Literal::Unit => Type::unit(),
            Literal::None => Type::new(TypeKind::Optional(Box::new(Type::unknown())), 0, 0),
        }
    }

    /// Infer type for binary operation
    fn infer_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<Type, InferenceError> {
        let left_type = self.infer(left)?;
        let right_type = self.infer(right)?;

        match op {
            // Arithmetic operations: both operands must be numeric
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // Add constraint: left_type == right_type
                self.add_constraint(left_type.clone(), right_type);
                Ok(left_type)
            }

            // Comparison operations: result is bool
            BinaryOp::Equal | BinaryOp::NotEqual |
            BinaryOp::Less | BinaryOp::LessEqual |
            BinaryOp::Greater | BinaryOp::GreaterEqual => {
                // Add constraint: left_type == right_type
                self.add_constraint(left_type, right_type);
                Ok(Type::bool())
            }

            // Logical operations: both operands must be bool
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                // Add constraints: left_type == bool, right_type == bool
                self.add_constraint(left_type, Type::bool());
                self.add_constraint(right_type, Type::bool());
                Ok(Type::bool())
            }

            // Bitwise operations: both operands must be integers
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor |
            BinaryOp::LeftShift | BinaryOp::RightShift => {
                // Add constraint: left_type == right_type
                self.add_constraint(left_type.clone(), right_type);
                Ok(left_type)
            }

            // Pipeline: special handling
            BinaryOp::Pipeline => {
                // Pipeline is handled separately in infer_pipeline
                Ok(self.fresh_type())
            }

            // Compose: function composition
            BinaryOp::Compose => {
                Ok(self.fresh_type())
            }

            // Power: numeric result
            BinaryOp::Pow => {
                self.add_constraint(left_type.clone(), right_type);
                Ok(left_type)
            }
        }
    }

    /// Infer type for unary operation
    fn infer_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<Type, InferenceError> {
        let operand_type = self.infer(operand)?;

        match op {
            UnaryOp::Neg => {
                // Operand must be numeric
                Ok(operand_type)
            }
            UnaryOp::Not => {
                // Operand must be bool
                self.add_constraint(operand_type, Type::bool());
                Ok(Type::bool())
            }
            UnaryOp::BitNot => {
                // Operand must be integer
                Ok(operand_type)
            }
            UnaryOp::Dereference => {
                // Handle reference types: if operand is Reference<T>, result is T
                if let TypeKind::Reference { inner, .. } = operand_type.kind {
                    Ok(*inner)
                } else {
                    // Not a reference type: add constraint and return inner type
                    Ok(operand_type)
                }
            }
            UnaryOp::Try => {
                // Handle optional types: if operand is Optional<T>, result is T
                if let TypeKind::Optional(inner) = operand_type.kind {
                    Ok(*inner)
                } else if let TypeKind::Result { ok_type, .. } = operand_type.kind {
                    Ok(*ok_type)
                } else {
                    // Not optional/result: return as-is (may be a type error caught later)
                    Ok(operand_type)
                }
            }
            UnaryOp::Lazy => {
                // Handle lazy types: Lazy<T> has the same type T, just deferred evaluation
                // The type is the same as the operand, evaluation is simply delayed
                Ok(operand_type)
            }
        }
    }

    /// Infer type for function call
    fn infer_call(&mut self, callee: &Expr, args: &Vec<Expr>) -> Result<Type, InferenceError> {
        let callee_type = self.infer(callee)?;
        let arg_types: Vec<Type> = args.iter().map(|arg| self.infer(arg)).collect::<Result<Vec<_>, _>>()?;

        // Create fresh type variable for result
        let result_type = self.fresh_type();

        // Add constraint: callee_type == (arg_types) -> result_type
        let func_type = Type::new(
            TypeKind::Function {
                params: arg_types,
                ret: Box::new(result_type.clone()),
            },
            0,
            0,
        );
        self.add_constraint(callee_type, func_type);

        Ok(result_type)
    }

    /// Infer type for pipeline expression
    fn infer_pipeline(&mut self, pipeline: &PipelineExpr) -> Result<Type, InferenceError> {
        let mut current_type = self.infer(&pipeline.source)?;

        for stage in &pipeline.stages {
            current_type = self.infer_pipeline_stage(stage, current_type)?;
        }

        Ok(current_type)
    }

    /// Infer type for pipeline stage
    fn infer_pipeline_stage(&mut self, stage: &PipelineStage, input_type: Type) -> Result<Type, InferenceError> {
        match stage {
            PipelineStage::Function { func, args } => {
                let func_type = self.infer(func)?;
                let arg_types: Vec<Type> = args.iter().map(|arg| self.infer(arg)).collect::<Result<Vec<_>, _>>()?;

                // Create fresh type variable for result
                let result_type = self.fresh_type();

                // Build function type: (input_type, arg_types) -> result_type
                let mut params = vec![input_type];
                params.extend(arg_types);

                let expected_func_type = Type::new(
                    TypeKind::Function {
                        params,
                        ret: Box::new(result_type.clone()),
                    },
                    0,
                    0,
                );

                // Add constraint: func_type == expected_func_type
                self.add_constraint(func_type, expected_func_type);

                Ok(result_type)
            }
            _ => Ok(input_type), // For other stages, preserve input type
        }
    }

    /// Infer type for comprehension expression
    fn infer_comprehension(&mut self, comp: &ComprehensionExpr) -> Result<Type, InferenceError> {
        // Check all iterators
        for iter in &comp.iterators {
            let source_type = self.infer(&iter.source)?;
            // Verify source_type is iterable (Array, Slice, or has Iterator trait)
            match &source_type.kind {
                TypeKind::Array { .. } | TypeKind::Slice(_) => {
                    // Valid iterable type
                }
                _ => {
                    // Add constraint requiring source to be iterable
                    // For now, we accept any type and let unification catch errors
                    let elem_type = self.fresh_type();
                    self.add_constraint(
                        source_type,
                        Type::new(TypeKind::Array { elem_type: Box::new(elem_type), size: 0 }, 0, 0),
                    );
                }
            }
        }

        // Check guard if present
        if let Some(ref guard) = comp.guard {
            let guard_type = self.infer(guard)?;
            self.add_constraint(guard_type, Type::bool());
        }

        // Infer output type
        let output_type = self.infer(&comp.output)?;

        // Result is an array of output_type
        Ok(Type::new(
            TypeKind::Array {
                elem_type: Box::new(output_type),
                size: 0,
            },
            0,
            0,
        ))
    }

    /// Infer type for if expression
    fn infer_if(&mut self, condition: &Expr, then_branch: &Expr, else_branch: &Expr) -> Result<Type, InferenceError> {
        let cond_type = self.infer(condition)?;
        let then_type = self.infer(then_branch)?;
        let else_type = self.infer(else_branch)?;

        // Condition must be bool
        self.add_constraint(cond_type, Type::bool());

        // Both branches must have the same type
        self.add_constraint(then_type.clone(), else_type);

        Ok(then_type)
    }

    /// Infer type for match expression
    fn infer_match(&mut self, value: &Expr, arms: &Vec<MatchArm>) -> Result<Type, InferenceError> {
        let value_type = self.infer(value)?;

        // All arms must have the same type
        let mut result_type = None;

        for arm in arms {
            // Check guard if present
            if let Some(ref guard) = arm.guard {
                let guard_type = self.infer(guard)?;
                self.add_constraint(guard_type, Type::bool());
            }

            // Infer arm body type
            let arm_type = self.infer(&arm.body)?;

            // All arms must have the same type
            if let Some(ref expected_type) = result_type {
                self.add_constraint(arm_type.clone(), expected_type.clone());
            } else {
                result_type = Some(arm_type);
            }
        }

        Ok(result_type.unwrap_or_else(|| self.fresh_type()))
    }

    /// Infer type for closure
    fn infer_closure(&mut self, params: &Vec<Parameter>, body: &Expr) -> Result<Type, InferenceError> {
        // Create fresh type variables for parameters
        let param_types: Vec<Type> = params.iter().map(|_| self.fresh_type()).collect();

        // Add parameter types to type environment
        for (param, param_type) in params.iter().zip(param_types.iter()) {
            self.type_env.add(param.name, param_type.clone());
        }

        // Infer body type
        let body_type = self.infer(body)?;

        // Result is a function type
        Ok(Type::new(
            TypeKind::Function {
                params: param_types,
                ret: Box::new(body_type),
            },
            0,
            0,
        ))
    }

    /// Add type constraint
    fn add_constraint(&mut self, left: Type, right: Type) {
        self.constraints.push(TypeConstraint { left, right });
    }

    /// Solve constraints using unification
    pub fn solve_constraints(&mut self) -> Result<Substitution, InferenceError> {
        let mut subst = Substitution::new();

        for constraint in &self.constraints {
            self.unify(&constraint.left, &constraint.right, &mut subst)?;
        }

        self.substitution = subst.clone();
        Ok(subst)
    }

    /// Unification algorithm
    fn unify(&self, t1: &Type, t2: &Type, subst: &mut Substitution) -> Result<(), InferenceError> {
        let t1 = self.apply_subst(t1, subst);
        let t2 = self.apply_subst(t2, subst);

        match (&t1.kind, &t2.kind) {
            // Same type variable
            (TypeKind::TypeParam(id1), TypeKind::TypeParam(id2)) if id1 == id2 => Ok(()),

            // Type variable on left: bind it
            (TypeKind::TypeParam(id), _) => {
                // Occurs check
                if self.occurs(*id, &t2) {
                    return Err(InferenceError::OccursCheck {
                        var: TypeVar { id: *id as u32 },
                        ty: t2.clone(),
                    });
                }
                subst.mapping.push((TypeVar { id: *id as u32 }, t2.clone()));
                Ok(())
            }

            // Type variable on right: bind it
            (_, TypeKind::TypeParam(id)) => {
                // Occurs check
                if self.occurs(*id, &t1) {
                    return Err(InferenceError::OccursCheck {
                        var: TypeVar { id: *id as u32 },
                        ty: t1.clone(),
                    });
                }
                subst.mapping.push((TypeVar { id: *id as u32 }, t1.clone()));
                Ok(())
            }

            // Function types: unify parameters and return types
            (
                TypeKind::Function { params: p1, ret: r1 },
                TypeKind::Function { params: p2, ret: r2 },
            ) => {
                if p1.len() != p2.len() {
                    return Err(InferenceError::Unification {
                        left: t1.clone(),
                        right: t2.clone(),
                    });
                }
                for (param1, param2) in p1.iter().zip(p2.iter()) {
                    self.unify(param1, param2, subst)?;
                }
                self.unify(r1, r2, subst)
            }

            // Array types: unify element types
            (TypeKind::Array { elem_type: e1, .. }, TypeKind::Array { elem_type: e2, .. }) => {
                self.unify(e1, e2, subst)
            }

            // Optional types: unify inner types
            (TypeKind::Optional(o1), TypeKind::Optional(o2)) => self.unify(o1, o2, subst),

            // Tuple types: unify all elements
            (TypeKind::Tuple(elems1), TypeKind::Tuple(elems2)) => {
                if elems1.len() != elems2.len() {
                    return Err(InferenceError::Unification {
                        left: t1.clone(),
                        right: t2.clone(),
                    });
                }
                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    self.unify(e1, e2, subst)?;
                }
                Ok(())
            }

            // Same base types
            (TypeKind::Int, TypeKind::Int) => Ok(()),
            (TypeKind::UInt, TypeKind::UInt) => Ok(()),
            (TypeKind::Float, TypeKind::Float) => Ok(()),
            (TypeKind::Bool, TypeKind::Bool) => Ok(()),
            (TypeKind::Char, TypeKind::Char) => Ok(()),
            (TypeKind::String, TypeKind::String) => Ok(()),
            (TypeKind::Unit, TypeKind::Unit) => Ok(()),
            (TypeKind::Unknown, _) | (_, TypeKind::Unknown) => Ok(()),

            // Type mismatch
            _ => Err(InferenceError::Unification {
                left: t1.clone(),
                right: t2.clone(),
            }),
        }
    }

    /// Apply substitution to a type
    fn apply_subst(&self, ty: &Type, subst: &Substitution) -> Type {
        match &ty.kind {
            TypeKind::TypeParam(id) => {
                // Look up in substitution
                for (var, replacement) in &subst.mapping {
                    if var.id == *id as u32 {
                        return self.apply_subst(replacement, subst);
                    }
                }
                ty.clone()
            }
            TypeKind::Function { params, ret } => Type::new(
                TypeKind::Function {
                    params: params.iter().map(|p| self.apply_subst(p, subst)).collect(),
                    ret: Box::new(self.apply_subst(ret, subst)),
                },
                ty.line,
                ty.column,
            ),
            TypeKind::Array { elem_type, size } => Type::new(
                TypeKind::Array {
                    elem_type: Box::new(self.apply_subst(elem_type, subst)),
                    size: *size,
                },
                ty.line,
                ty.column,
            ),
            TypeKind::Optional(inner) => Type::new(
                TypeKind::Optional(Box::new(self.apply_subst(inner, subst))),
                ty.line,
                ty.column,
            ),
            TypeKind::Tuple(elems) => Type::new(
                TypeKind::Tuple(elems.iter().map(|e| self.apply_subst(e, subst)).collect()),
                ty.line,
                ty.column,
            ),
            _ => ty.clone(),
        }
    }

    /// Occurs check: does type variable occur in type?
    fn occurs(&self, var_id: usize, ty: &Type) -> bool {
        match &ty.kind {
            TypeKind::TypeParam(id) => *id == var_id,
            TypeKind::Function { params, ret } => {
                params.iter().any(|p| self.occurs(var_id, p)) || self.occurs(var_id, ret)
            }
            TypeKind::Array { elem_type, .. } => self.occurs(var_id, elem_type),
            TypeKind::Optional(inner) => self.occurs(var_id, inner),
            TypeKind::Tuple(elems) => elems.iter().any(|e| self.occurs(var_id, e)),
            _ => false,
        }
    }
}

/// Inference Error
#[derive(Debug, Clone)]
pub enum InferenceError {
    /// Unification failure
    Unification { left: Type, right: Type },
    /// Occurs check failure (infinite type)
    OccursCheck { var: TypeVar, ty: Type },
    /// Undefined variable
    UndefinedVariable(&'static str),
    /// Type mismatch
    TypeMismatch { expected: Type, got: Type },
}

#[cfg(test)]
mod tests {
    use super::*;
use alloc::vec;
use alloc::vec::Vec;

    #[test]
    fn test_fresh_var() {
        let mut inference = TypeInference::new();
        let var1 = inference.fresh_var();
        let var2 = inference.fresh_var();
        assert_ne!(var1.id, var2.id);
    }

    #[test]
    fn test_infer_literal() {
        let mut inference = TypeInference::new();
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            ty: None,
        };
        let result = inference.infer(&expr);
        assert!(result.is_ok());
    }
}
