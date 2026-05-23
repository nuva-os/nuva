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


use super::ast::{BinaryOp, UnaryOp};

/// OperatorPriority
#[derive(Debug, Clone, Copy)]
pub struct Precedence {
 /// Priorityvalue
 pub value: u8,
 /// ifrightcombine
 pub right_associative: bool,
}

impl Precedence {
 /// CreatenewPriority
 pub const fn new(value: u8, right_associative: bool) -> Self {
 Precedence { value, right_associative }
 }
}

/// GetbinaryOperatorPriority
pub fn get_binary_precedence(op: BinaryOp) -> Precedence {
 match op {
 // Assignment (lowest priority)
 BinaryOp::Pipeline => Precedence::new(2, false),

 // Logical OR
 BinaryOp::Or => Precedence::new(3, false),

 // Logical AND
 BinaryOp::And => Precedence::new(4, false),

 // Bitor
 BinaryOp::BitOr => Precedence::new(5, false),

 // Bitwise XOR
 BinaryOp::BitXor => Precedence::new(6, false),

 // Bitwith
 BinaryOp::BitAnd => Precedence::new(7, false),

 // mutualetcity
 BinaryOp::Equal | BinaryOp::NotEqual => Precedence::new(8, false),

 // Compare
 BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
 Precedence::new(9, false)
 }

 // Bit
 BinaryOp::LeftShift | BinaryOp::RightShift => Precedence::new(10, false),

 // PlusMinus (left-associative)
 BinaryOp::Add | BinaryOp::Sub => Precedence::new(11, false),

 // MultiplyDividemodel (left-associative)
 BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => Precedence::new(12, false),

 // Composition (function composition)
 BinaryOp::Compose => Precedence::new(13, false),

 // Power (highest arithmetic, right-associative)
 BinaryOp::Pow => Precedence::new(14, true),

 // Xor (logical)
 BinaryOp::Xor => Precedence::new(4, false),
 }
}

/// GetaOperatorPriority
pub fn get_unary_precedence(_op: UnaryOp) -> Precedence {
 Precedence::new(15, false)
}

/// PriorityConstant
pub mod precedence {
 /// mostlowPriority
 pub const LOWEST: u8 = 0;
 /// Assignment
 pub const ASSIGNMENT: u8 = 1;
 /// Pipeline
 pub const PIPELINE: u8 = 2;
 /// Logical OR
 pub const OR: u8 = 3;
 /// Logical AND
 pub const AND: u8 = 4;
 /// Bitor
 pub const BIT_OR: u8 = 5;
 /// Bitwise XOR
 pub const BIT_XOR: u8 = 6;
 /// Bitwith
 pub const BIT_AND: u8 = 7;
 /// mutualetcity
 pub const EQUALITY: u8 = 8;
 /// Compare
 pub const COMPARISON: u8 = 9;
 /// Bit
 pub const SHIFT: u8 = 10;
 /// PlusMinus
 pub const TERM: u8 = 11;
 /// MultiplyDividemodel
 pub const FACTOR: u8 = 12;
 /// Composition
 pub const COMPOSE: u8 = 13;
 /// Power
 pub const POWER: u8 = 14;
 /// a
 pub const UNARY: u8 = 15;
 /// call
 pub const CALL: u8 = 16;
 /// mosthighPriority
 pub const HIGHEST: u8 = 17;
}

/// combineity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
 /// leftcombine
 Left,
 /// rightcombine
 Right,
 /// infinitecombineity
 None,
}

/// GetbinaryOperatorcombineity
pub fn get_binary_associativity(op: BinaryOp) -> Associativity {
 match op {
 BinaryOp::Pow => Associativity::Right,
 _ => Associativity::Left,
 }
}