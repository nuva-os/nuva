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


use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use super::ir::*;
use alloc::vec::Vec;

/// OptimizationLevel
#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
 /// noneOptimization
 None = 0,
 /// baseOptimization
 Basic = 1,
 /// standardcriterionOptimization
 Standard = 2,
 /// enterOptimization
 Aggressive = 3,
}

/// IR Optimizer
pub struct IrOptimizer {
 /// OptimizationLevel
 level: OptLevel,
}

impl IrOptimizer {
 /// CreatenewOptimizer
 pub fn new(level: OptLevel) -> Self {
 IrOptimizer { level }
 }
 
 /// OptimizationModule
 pub fn optimize(&self, module: &mut IrModule) {
 if self.level == OptLevel::None {
 return;
 }
 
 // OptimizationallFunction
 for i in 0..module.functions.len() {
 self.optimize_function(&mut module.functions[i], &module.functions);
 }
 }
 
 /// OptimizationFunction
 fn optimize_function(&self, func: &mut IrFunction, all_functions: &[IrFunction]) {
 // baseOptimization
 self.constant_folding(func);
 self.dead_code_elimination(func);
 
 // standardcriterionOptimization
 if self.level >= OptLevel::Standard {
 self.common_subexpression_elimination(func);
 self.copy_propagation(func);
 }
 
 // enterOptimization
 if self.level >= OptLevel::Aggressive {
 self.loop_optimization(func);
 self.inline_expansion(func, all_functions);
 }
 }
 
 /// ConstantCollapse
 fn constant_folding(&self, func: &mut IrFunction) {
 for block in func.blocks.iter_mut() {
 let mut new_instructions = Vec::new();
 
 for instr in block.instructions.iter() {
 match instr {
 IrInstruction::Binary { dest, op, left, right } => {
     let left_const = Self::find_const_value(&new_instructions, *left);
     let right_const = Self::find_const_value(&new_instructions, *right);
     if let (Some(lv), Some(rv)) = (left_const, right_const) {
         if let Some(result) = Self::fold_binary(*op, &lv, &rv) {
             new_instructions.push(IrInstruction::LoadConst {
                 dest: *dest,
                 value: result,
             });
             continue;
         }
     }
     new_instructions.push(instr.clone());
 }
 IrInstruction::Unary { dest, op, operand } => {
     let operand_const = Self::find_const_value(&new_instructions, *operand);
     if let Some(v) = operand_const {
         if let Some(result) = Self::fold_unary(*op, &v) {
             new_instructions.push(IrInstruction::LoadConst {
                 dest: *dest,
                 value: result,
             });
             continue;
         }
     }
     new_instructions.push(instr.clone());
 }
 _ => new_instructions.push(instr.clone()),
 }
 }
 
 block.instructions = new_instructions;
 }
 }

 /// Find constant value for a given value ID from prior instructions
 fn find_const_value(instructions: &[IrInstruction], value_id: u32) -> Option<IrValue> {
 for instr in instructions.iter().rev() {
 match instr {
 IrInstruction::LoadConst { dest, value } if *dest == value_id => {
     return Some(value.clone());
 }
 _ => {}
 }
 }
 None
 }

 /// Fold binary operation on two constant values
 fn fold_binary(op: IrBinaryOp, left: &IrValue, right: &IrValue) -> Option<IrValue> {
 match (left, right) {
 (IrValue::Integer(l), IrValue::Integer(r)) => {
     let result = match op {
         IrBinaryOp::Add => l.checked_add(*r)?,
         IrBinaryOp::Sub => l.checked_sub(*r)?,
         IrBinaryOp::Mul => l.checked_mul(*r)?,
         IrBinaryOp::Div => if *r != 0 { l.checked_div(*r)? } else { return None },
         IrBinaryOp::Mod => if *r != 0 { l.checked_rem(*r)? } else { return None },
         IrBinaryOp::BitAnd => l & r,
         IrBinaryOp::BitOr => l | r,
         IrBinaryOp::BitXor => l ^ r,
         IrBinaryOp::LeftShift => {
             if *r >= 0 && (*r as u32) < 64 { l << *r as u32 } else { return None }
         }
         IrBinaryOp::RightShift => {
             if *r >= 0 && (*r as u32) < 64 { l >> *r as u32 } else { return None }
         }
     };
     Some(IrValue::Integer(result))
 }
 (IrValue::Float(l), IrValue::Float(r)) => {
     let result = match op {
         IrBinaryOp::Add => l + r,
         IrBinaryOp::Sub => l - r,
         IrBinaryOp::Mul => l * r,
         IrBinaryOp::Div => if *r != 0.0 { l / r } else { return None },
         _ => return None,
     };
     Some(IrValue::Float(result))
 }
 (IrValue::Bool(l), IrValue::Bool(r)) => {
     let result = match op {
         IrBinaryOp::BitAnd => *l && *r,
         IrBinaryOp::BitOr => *l || *r,
         IrBinaryOp::BitXor => *l ^ *r,
         _ => return None,
     };
     Some(IrValue::Bool(result))
 }
 _ => None,
 }
 }

 /// Fold unary operation on a constant value
 fn fold_unary(op: IrUnaryOp, operand: &IrValue) -> Option<IrValue> {
 match (op, operand) {
 (IrUnaryOp::Neg, IrValue::Integer(n)) => Some(IrValue::Integer(n.checked_neg()?)),
 (IrUnaryOp::Neg, IrValue::Float(f)) => Some(IrValue::Float(-f)),
 (IrUnaryOp::Not, IrValue::Bool(b)) => Some(IrValue::Bool(!b)),
 (IrUnaryOp::BitNot, IrValue::Integer(n)) => Some(IrValue::Integer(!n)),
 _ => None,
 }
 }
 
 /// deadCodeDivide
 fn dead_code_elimination(&self, func: &mut IrFunction) {
 // 1. Collect all used value IDs
 let mut used: BTreeSet<u32> = BTreeSet::new();

 // Mark values used by side-effecting instructions and terminators
 for block in &func.blocks {
 for instr in &block.instructions {
 match instr {
 IrInstruction::StoreVar { var_id, src } => {
     used.insert(*var_id);
     used.insert(*src);
 }
 IrInstruction::Store { ptr, offset: _, src } => {
     used.insert(*ptr);
     used.insert(*src);
 }
 IrInstruction::SetField { object, field_idx: _, src } => {
     used.insert(*object);
     used.insert(*src);
 }
 IrInstruction::Return { value } => {
     if let Some(v) = value { used.insert(*v); }
 }
 IrInstruction::Jump { .. } => {}
 IrInstruction::JumpIf { cond, then_target: _, else_target: _ } => {
     used.insert(*cond);
 }
 IrInstruction::Call { dest: _, func, args } => {
     // Calls may have side effects, always keep them
     used.insert(*func);
     for arg in args { used.insert(*arg); }
 }
 _ => {}
 }
 }
 }

 // 2. Iteratively propagate used values through definitions
 let mut changed = true;
 while changed {
 changed = false;
 for block in &func.blocks {
 for instr in &block.instructions {
 match instr {
 IrInstruction::Binary { dest, op: _, left, right } => {
     if used.contains(dest) {
         if used.insert(*left) { changed = true; }
         if used.insert(*right) { changed = true; }
     }
 }
 IrInstruction::Unary { dest, op: _, operand } => {
     if used.contains(dest) {
         if used.insert(*operand) { changed = true; }
     }
 }
 IrInstruction::Compare { dest, op: _, left, right } => {
     if used.contains(dest) {
         if used.insert(*left) { changed = true; }
         if used.insert(*right) { changed = true; }
     }
 }
 IrInstruction::Load { dest, ptr, offset: _ } => {
     if used.contains(dest) {
         if used.insert(*ptr) { changed = true; }
     }
 }
 IrInstruction::GetField { dest, object, field_idx: _ } => {
     if used.contains(dest) {
         if used.insert(*object) { changed = true; }
     }
 }
 IrInstruction::ArrayAccess { dest, array, index } => {
     if used.contains(dest) {
         if used.insert(*array) { changed = true; }
         if used.insert(*index) { changed = true; }
     }
 }
 IrInstruction::Cast { dest, src, target_type: _ } => {
     if used.contains(dest) {
         if used.insert(*src) { changed = true; }
     }
 }
 IrInstruction::Phi { dest, incoming } => {
     if used.contains(dest) {
         for (val, _block) in incoming {
             if used.insert(*val) { changed = true; }
         }
     }
 }
 _ => {}
 }
 }
 }
 }

 // 3. Remove instructions that define unused values
 for block in func.blocks.iter_mut() {
 block.instructions.retain(|instr| {
 match instr {
 IrInstruction::LoadConst { dest, .. } => used.contains(dest),
 IrInstruction::LoadVar { dest, .. } => used.contains(dest),
 IrInstruction::Binary { dest, .. } => used.contains(dest),
 IrInstruction::Unary { dest, .. } => used.contains(dest),
 IrInstruction::Compare { dest, .. } => used.contains(dest),
 IrInstruction::Alloca { dest, .. } => used.contains(dest),
 IrInstruction::Load { dest, .. } => used.contains(dest),
 IrInstruction::GetField { dest, .. } => used.contains(dest),
 IrInstruction::NewArray { dest, .. } => used.contains(dest),
 IrInstruction::ArrayAccess { dest, .. } => used.contains(dest),
 IrInstruction::Cast { dest, .. } => used.contains(dest),
 IrInstruction::Phi { dest, .. } => used.contains(dest),
 // Keep all side-effecting and control-flow instructions
 _ => true,
 }
 });
 }
 }
 
 /// publicsharedChildformreachstyleDivide
 fn common_subexpression_elimination(&self, func: &mut IrFunction) {
 for block in func.blocks.iter_mut() {
 // Map from expression key to the first value ID that computed it
 let mut expr_map: BTreeMap<u64, u32> = BTreeMap::new();
 let mut new_instructions = Vec::new();

 for instr in block.instructions.iter() {
 match instr {
 IrInstruction::Binary { dest, op, left, right } => {
     let key = Self::binary_key(*op, *left, *right);
     if let Some(&prev_dest) = expr_map.get(&key) {
         new_instructions.push(IrInstruction::LoadVar {
             dest: *dest,
             var_id: prev_dest,
         });
     } else {
         expr_map.insert(key, *dest);
         new_instructions.push(instr.clone());
     }
 }
 IrInstruction::Compare { dest, op, left, right } => {
     let key = Self::compare_key(*op, *left, *right);
     if let Some(&prev_dest) = expr_map.get(&key) {
         new_instructions.push(IrInstruction::LoadVar {
             dest: *dest,
             var_id: prev_dest,
         });
     } else {
         expr_map.insert(key, *dest);
         new_instructions.push(instr.clone());
     }
 }
 _ => new_instructions.push(instr.clone()),
 }
 }

 block.instructions = new_instructions;
 }
 }

 /// Encode binary expression as u64 key for CSE lookup
 fn binary_key(op: IrBinaryOp, left: u32, right: u32) -> u64 {
 let op_disc = match op {
 IrBinaryOp::Add => 0, IrBinaryOp::Sub => 1, IrBinaryOp::Mul => 2,
 IrBinaryOp::Div => 3, IrBinaryOp::Mod => 4, IrBinaryOp::BitAnd => 5,
 IrBinaryOp::BitOr => 6, IrBinaryOp::BitXor => 7, IrBinaryOp::LeftShift => 8,
 IrBinaryOp::RightShift => 9,
 } as u64;
 (op_disc << 48) | ((left as u64) << 16) | (right as u64 & 0xFFFF)
 }

 /// Encode compare expression as u64 key for CSE lookup
 fn compare_key(op: IrCompareOp, left: u32, right: u32) -> u64 {
 let op_disc = match op {
 IrCompareOp::Equal => 10, IrCompareOp::NotEqual => 11,
 IrCompareOp::Less => 12, IrCompareOp::LessEqual => 13,
 IrCompareOp::Greater => 14, IrCompareOp::GreaterEqual => 15,
 } as u64;
 (op_disc << 48) | ((left as u64) << 16) | (right as u64 & 0xFFFF)
 }
 
 /// Copytransmitbroadcast
 fn copy_propagation(&self, func: &mut IrFunction) {
 for block in func.blocks.iter_mut() {
 // Build copy chain: map from value ID to its ultimate source
 let mut copy_map: BTreeMap<u32, u32> = BTreeMap::new();

 // Identify copy instructions (LoadVar where src is just a pass-through)
 for instr in &block.instructions {
 match instr {
 IrInstruction::LoadVar { dest, var_id } => {
     // Record the copy: dest copies from var_id
     copy_map.insert(*dest, *var_id);
 }
 _ => {}
 }
 }

 // Resolve transitive copies: if a copies b and b copies c, a -> c
 let mut changed = true;
 while changed {
 changed = false;
 let updates: Vec<(u32, u32)> = copy_map.iter()
     .filter_map(|(&k, &v)| {
         if let Some(&final_src) = copy_map.get(&v) {
             if final_src != v { Some((k, final_src)) } else { None }
         } else {
             None
         }
     })
     .collect();
 for (k, v) in updates {
     if copy_map.insert(k, v).is_some() {
         changed = true;
     }
 }
 }

 // Replace uses of copied values with their ultimate sources
 if !copy_map.is_empty() {
 for instr in block.instructions.iter_mut() {
 match instr {
 IrInstruction::Binary { left, right, .. } => {
     if let Some(&src) = copy_map.get(left) { *left = src; }
     if let Some(&src) = copy_map.get(right) { *right = src; }
 }
 IrInstruction::Unary { operand, .. } => {
     if let Some(&src) = copy_map.get(operand) { *operand = src; }
 }
 IrInstruction::Compare { left, right, .. } => {
     if let Some(&src) = copy_map.get(left) { *left = src; }
     if let Some(&src) = copy_map.get(right) { *right = src; }
 }
 IrInstruction::Call { func, args, .. } => {
     if let Some(&src) = copy_map.get(func) { *func = src; }
     for arg in args.iter_mut() {
         if let Some(&src) = copy_map.get(arg) { *arg = src; }
     }
 }
 IrInstruction::JumpIf { cond, .. } => {
     if let Some(&src) = copy_map.get(cond) { *cond = src; }
 }
 IrInstruction::Return { value } => {
     if let Some(ref mut v) = value {
         if let Some(&src) = copy_map.get(v) { *v = src; }
     }
 }
 _ => {}
 }
 }
 }
 }
 }
 
 /// RingOptimization
 fn loop_optimization(&self, func: &mut IrFunction) {
 // 1. Loop-invariant code motion (LICM): move loop-invariant computations
 //    out of loop bodies to loop preheaders
 // 2. Loop unrolling for small trip counts

 // Identify loop headers: blocks that are targets of back-edge jumps
 let mut loop_headers: BTreeSet<u32> = BTreeSet::new();
 for block in &func.blocks {
 for instr in &block.instructions {
 match instr {
 IrInstruction::Jump { target } => {
     if *target <= block.id {
         loop_headers.insert(*target);
     }
 }
 IrInstruction::JumpIf { then_target, else_target, .. } => {
     if *then_target <= block.id { loop_headers.insert(*then_target); }
     if *else_target <= block.id { loop_headers.insert(*else_target); }
 }
 _ => {}
 }
 }
 }

 // LICM: for each loop body block, find instructions whose operands
 // are all defined outside the loop, and move them before the loop header
 for &header_id in &loop_headers {
 let mut invariant_instrs: Vec<IrInstruction> = Vec::new();
 let mut defs_in_loop: BTreeSet<u32> = BTreeSet::new();

 // Collect all values defined inside the loop
 for block in &func.blocks {
 if block.id >= header_id {
 for instr in &block.instructions {
 match instr {
 IrInstruction::LoadConst { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::LoadVar { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::Binary { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::Unary { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::Compare { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::Alloca { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::Load { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::GetField { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::NewArray { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::ArrayAccess { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::Cast { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::Phi { dest, .. } => { defs_in_loop.insert(*dest); }
 IrInstruction::Call { dest, .. } => {
     if let Some(d) = dest { defs_in_loop.insert(d); }
 }
 _ => {}
 }
 }
 }
 }

 // Find loop-invariant instructions in the loop body
 for block in func.blocks.iter_mut() {
 if block.id > header_id {
 let mut new_instrs = Vec::new();
 for instr in block.instructions.drain(..) {
 let is_invariant = match &instr {
 IrInstruction::Binary { left, right, .. } => {
     !defs_in_loop.contains(left) && !defs_in_loop.contains(right)
 }
 IrInstruction::Unary { operand, .. } => {
     !defs_in_loop.contains(operand)
 }
 IrInstruction::Compare { left, right, .. } => {
     !defs_in_loop.contains(left) && !defs_in_loop.contains(right)
 }
 _ => false,
 };
 if is_invariant {
     invariant_instrs.push(instr);
 } else {
     new_instrs.push(instr);
 }
 }
 block.instructions = new_instrs;
 }
 }

 // Hoist invariant instructions before the loop header
 if !invariant_instrs.is_empty() {
 if let Some(header_block) = func.blocks.get_mut(header_id as usize) {
 let mut hoisted = invariant_instrs;
 hoisted.append(&mut header_block.instructions);
 header_block.instructions = hoisted;
 }
 }
 }

 // 2. Simple loop unrolling: for single-block loops with small instruction
 //    count, duplicate the body to reduce loop overhead
 for &header_id in &loop_headers {
 if let Some(header_block) = func.blocks.get_mut(header_id as usize) {
 // Only unroll if the loop body has a small number of instructions (<=4)
 // and ends with a Jump back to itself
 let instr_count = header_block.instructions.len();
 if instr_count > 1 && instr_count <= 5 {
     if let Some(IrInstruction::Jump { target }) = header_block.instructions.last() {
         if *target == header_id {
             // Unroll once: duplicate all instructions except the final jump
             let body: Vec<IrInstruction> = header_block.instructions
                 .iter()
                 .take(instr_count - 1)
                 .cloned()
                 .collect();
             let mut unrolled = body.clone();
             unrolled.append(&mut header_block.instructions);
             header_block.instructions = unrolled;
         }
     }
 }
 }
 }
 }
 
 /// insideExpand
 fn inline_expansion(&self, func: &mut IrFunction, all_functions: &[IrFunction]) {
 // Heuristic: inline calls to functions with small bodies (<=8 instructions)
 const MAX_INLINE_SIZE: usize = 8;

 for block in func.blocks.iter_mut() {
 let mut new_instructions = Vec::new();

 for instr in block.instructions.iter() {
 match instr {
 IrInstruction::Call { dest, func: callee_id, args } => {
     // Check if callee is a small function we can inline
     if let Some(callee) = all_functions.get(*callee_id as usize) {
         if callee.blocks.len() == 1 {
             let body_len = callee.blocks[0].instructions.len();
             if body_len <= MAX_INLINE_SIZE
                 && !Self::has_recursive_call(&callee.blocks[0], callee.id)
             {
                 // Inline: map callee params to args, then emit body
                 let mut param_map: BTreeMap<u32, u32> = BTreeMap::new();
                 for (i, (param_id, _ty)) in callee.params.iter().enumerate() {
                     if i < args.len() {
                         param_map.insert(*param_id, args[i]);
                     }
                 }
                 for body_instr in &callee.blocks[0].instructions {
                     match body_instr {
                         IrInstruction::Return { value } => {
                             if let (Some(ret_val), Some(d)) = (value, dest) {
                                 let mapped = param_map.get(ret_val).copied().unwrap_or(*ret_val);
                                 new_instructions.push(IrInstruction::LoadVar {
                                     dest: d,
                                     var_id: mapped,
                                 });
                             }
                         }
                         _ => {
                             let mut inlined = body_instr.clone();
                             Self::remap_operands(&mut inlined, &param_map);
                             new_instructions.push(inlined);
                         }
                     }
                 }
                 continue;
             }
         }
     }
     new_instructions.push(instr.clone());
 }
 _ => new_instructions.push(instr.clone()),
 }
 }

 block.instructions = new_instructions;
 }
 }

 /// Check if a basic block contains a recursive call to the given function ID
 fn has_recursive_call(block: &IrBasicBlock, func_id: u32) -> bool {
 for instr in &block.instructions {
 match instr {
 IrInstruction::Call { func, .. } if *func == func_id => return true,
 _ => {}
 }
 }
 false
 }

 /// Remap operand value IDs according to a parameter mapping
 fn remap_operands(instr: &mut IrInstruction, map: &BTreeMap<u32, u32>) {
 fn remap(id: u32, map: &BTreeMap<u32, u32>) -> u32 {
 map.get(&id).copied().unwrap_or(id)
 }
 match instr {
 IrInstruction::LoadVar { var_id, .. } => { *var_id = remap(*var_id, map); }
 IrInstruction::StoreVar { var_id, src } => {
     *var_id = remap(*var_id, map);
     *src = remap(*src, map);
 }
 IrInstruction::Binary { left, right, .. } => {
     *left = remap(*left, map);
     *right = remap(*right, map);
 }
 IrInstruction::Unary { operand, .. } => { *operand = remap(*operand, map); }
 IrInstruction::Compare { left, right, .. } => {
     *left = remap(*left, map);
     *right = remap(*right, map);
 }
 IrInstruction::Call { func, args, .. } => {
     *func = remap(*func, map);
     for arg in args.iter_mut() { *arg = remap(*arg, map); }
 }
 IrInstruction::Load { ptr, .. } => { *ptr = remap(*ptr, map); }
 IrInstruction::Store { ptr, src, .. } => {
     *ptr = remap(*ptr, map);
     *src = remap(*src, map);
 }
 IrInstruction::GetField { object, .. } => { *object = remap(*object, map); }
 IrInstruction::SetField { object, src, .. } => {
     *object = remap(*object, map);
     *src = remap(*src, map);
 }
 IrInstruction::ArrayAccess { array, index, .. } => {
     *array = remap(*array, map);
     *index = remap(*index, map);
 }
 IrInstruction::Cast { src, .. } => { *src = remap(*src, map); }
 IrInstruction::Phi { incoming, .. } => {
     for (val, _block) in incoming.iter_mut() { *val = remap(*val, map); }
 }
 _ => {}
 }
 }
}

/// Optimizationresult
pub struct OptimizationResult {
 /// ifSuccess
 pub success: bool,
 /// OptimizationprefixInstructionnumber
 pub original_instructions: u32,
 /// OptimizationthenInstructionnumber
 pub optimized_instructions: u32,
 /// Minusfew Instructionnumber
 pub removed_instructions: u32,
}

impl OptimizationResult {
 /// ComputeOptimizationrate
 pub fn optimization_rate(&self) -> f32 {
 if self.original_instructions == 0 {
 return 0.0;
 }
 (self.removed_instructions as f32) / (self.original_instructions as f32) * 100.0
 }
}