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

use alloc::boxed::Box;
use alloc::vec::Vec;

/// IR Instruction
#[derive(Debug, Clone)]
pub enum IrInstruction {
 /// PlusloadConstant
 LoadConst { dest: u32, value: IrValue },
 /// PlusloadVariable
 LoadVar { dest: u32, var_id: u32 },
 /// existVariable
 StoreVar { var_id: u32, src: u32 },
 /// binary operationcalculation
 Binary { dest: u32, op: IrBinaryOp, left: u32, right: u32 },
 /// aoperationcalculation
 Unary { dest: u32, op: IrUnaryOp, operand: u32 },
 /// Compareoperationcalculation
 Compare { dest: u32, op: IrCompareOp, left: u32, right: u32 },
 /// jumpbranch
 Jump { target: u32 },
 /// stripcasejumpbranch
 JumpIf { cond: u32, then_target: u32, else_target: u32 },
 /// callFunction
 Call { dest: Option<u32>, func: u32, args: Vec<u32> },
 /// return
 Return { value: Option<u32> },
 /// AllocateStackemptybetween
 Alloca { dest: u32, size: usize, align: usize },
 /// PlusloadMemory
 Load { dest: u32, ptr: u32, offset: usize },
 /// existMemory
 Store { ptr: u32, offset: usize, src: u32 },
 /// GetField
 GetField { dest: u32, object: u32, field_idx: u32 },
 /// SetField
 SetField { object: u32, field_idx: u32, src: u32 },
 /// CreateArray
 NewArray { dest: u32, elem_type: IrType, size: u32 },
 /// Arrayaccess
 ArrayAccess { dest: u32, array: u32, index: u32 },
 /// Typeconvert
 Cast { dest: u32, src: u32, target_type: IrType },
 /// Phi Node
 Phi { dest: u32, incoming: Vec<(u32, u32)> },
}

/// IR value
#[derive(Debug, Clone)]
pub enum IrValue {
 /// Integer
 Integer(i64),
 /// Float
 Float(f64),
 /// boolean
 Bool(bool),
 /// String
 String(&'static str),
 /// Character
 Char(char),
 /// None
 None,
}

/// IR Type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
 /// Integer
 Int(u8),
 /// noneSignInteger
 Uint(u8),
 /// Float
 Float(u8),
 /// boolean
 Bool,
 /// pointer
 Pointer(Box<IrType>),
 /// Array
 Array { elem: Box<IrType>, size: usize },
 /// Struct
 Struct { name: &'static str, size: usize },
 /// Function
 Function { params: Vec<IrType>, ret: Box<IrType> },
 /// Void
 Void,
}

/// IR binary operationcalculation
#[derive(Debug, Clone, Copy)]
pub enum IrBinaryOp {
 Add, Sub, Mul, Div, Mod,
 BitAnd, BitOr, BitXor, LeftShift, RightShift,
}

/// IR aoperationcalculation
#[derive(Debug, Clone, Copy)]
pub enum IrUnaryOp {
 Neg, Not, BitNot,
}

/// IR Compareoperationcalculation
#[derive(Debug, Clone, Copy)]
pub enum IrCompareOp {
 Equal, NotEqual,
 Less, LessEqual, Greater, GreaterEqual,
}

/// IR basebookBlock
pub struct IrBasicBlock {
 /// Block ID
 pub id: u32,
 /// InstructionList
 pub instructions: Vec<IrInstruction>,
 /// prefixBlock
 pub predecessors: Vec<u32>,
 /// thenBlock
 pub successors: Vec<u32>,
}

impl IrBasicBlock {
 pub fn new(id: u32) -> Self {
 IrBasicBlock {
 id,
 instructions: Vec::new(),
 predecessors: Vec::new(),
 successors: Vec::new(),
 }
 }
 
 /// addInstruction
 pub fn add_instruction(&mut self, instr: IrInstruction) {
 self.instructions.push(instr);
 }
}

/// IR Function
pub struct IrFunction {
 /// Function ID
 pub id: u32,
 /// Functionname
 pub name: &'static str,
 /// ParameterList
 pub params: Vec<(u32, IrType)>,
 /// returnType
 pub return_type: IrType,
 /// basebookBlockList
 pub blocks: Vec<IrBasicBlock>,
 /// partVariablecount
 pub num_locals: u32,
}

impl IrFunction {
 pub fn new(id: u32, name: &'static str) -> Self {
 IrFunction {
 id,
 name,
 params: Vec::new(),
 return_type: IrType::Void,
 blocks: Vec::new(),
 num_locals: 0,
 }
 }
 
 /// addPlusbasebookBlock
 pub fn add_block(&mut self, block: IrBasicBlock) {
 self.blocks.push(block);
 }
 
 /// AllocatepartVariable
 pub fn alloc_local(&mut self) -> u32 {
 let id = self.num_locals;
 self.num_locals += 1;
 id
 }
}

/// IR Module
pub struct IrModule {
 /// Modulename
 pub name: &'static str,
 /// FunctionList
 pub functions: Vec<IrFunction>,
 /// GlobalVariableList
 pub globals: Vec<(u32, &'static str, IrType)>,
}

impl IrModule {
 pub fn new(name: &'static str) -> Self {
 IrModule {
 name,
 functions: Vec::new(),
 globals: Vec::new(),
 }
 }
 
 /// addFunction
 pub fn add_function(&mut self, func: IrFunction) {
 self.functions.push(func);
 }
 
 /// addPlusGlobalVariable
 pub fn add_global(&mut self, name: &'static str, ty: IrType) -> u32 {
 let id = self.globals.len() as u32;
 self.globals.push((id, name, ty));
 id
 }
}