/*
 * Nuva OS
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

// ! Optimization pass Implementation

use super::{IR, OptError};

/// optimize Pass trait
pub trait OptimizationPass {
 /// Pass name
 fn name(&self) -> &str;
 
 /// executeoptimize
 fn run(&self, ir: &mut IR) -> Result<bool, OptError>;
}

/// get Pass
pub fn get_pass(name: &str) -> Option<Box<dyn OptimizationPass>> {
 match name {
 "mem2reg" => Some(Box::new(Mem2RegPass)),
 "dce" => Some(Box::new(DeadCodeEliminationPass)),
 "simplify_cfg" => Some(Box::new(SimplifyCFGPass)),
 "inline" => Some(Box::new(InlinePass)),
 "const_fold" => Some(Box::new(ConstantFoldingPass)),
 "gvn" => Some(Box::new(GlobalValueNumberingPass)),
 "loop_opt" => Some(Box::new(LoopOptimizationPass)),
 "vectorize" => Some(Box::new(VectorizationPass)),
 "strip_dead" => Some(Box::new(StripDeadPass)),
 _ => None,
 }
}

/// Mem2Reg Pass(Memorytoregisterupgrade)
pub struct Mem2RegPass;

impl OptimizationPass for Mem2RegPass {
 fn name(&self) -> &str {
 "mem2reg"
 }

 fn run(&self, ir: &mut IR) -> Result<bool, OptError> {
 // TODO: Implement memory-to-register promotion
 // willStackupload simpleformVariableupgradetoregister
 Ok(false)
 }
}

/// deadCodeDivide Pass
pub struct DeadCodeEliminationPass;

impl OptimizationPass for DeadCodeEliminationPass {
 fn name(&self) -> &str {
 "dce"
 }

 fn run(&self, ir: &mut IR) -> Result<bool, OptError> {
 let mut changed = false;
 
 // Dividemakeuse Function
 let used_functions = self.find_used_functions(ir);
 let original_count = ir.functions.len();
 
 ir.functions.retain(|f| used_functions.contains(&f.name));
 
 if ir.functions.len() != original_count {
 changed = true;
 }
 
 // Dividemakeuse GlobalVariable
 let used_globals = self.find_used_globals(ir);
 let original_count = ir.globals.len();
 
 ir.globals.retain(|g| used_globals.contains(&g.name));
 
 if ir.globals.len() != original_count {
 changed = true;
 }
 
 Ok(changed)
 }
}

impl DeadCodeEliminationPass {
 fn find_used_functions(&self, ir: &IR) -> std::collections::HashSet<String> {
 let mut used = std::collections::HashSet::new();
 
 // enterportpointtotalisbymakeuse
 used.insert("main".to_string());
 
 // AnalysisFunctiontuneuse
 for func in &ir.functions {
 if used.contains(&func.name) {
 for inst in &func.body {
 if inst.opcode == "call" {
 if let Some(operand) = inst.operands.first() {
 used.insert(operand.value.clone());
 }
 }
 }
 }
 }
 
 used
 }

 fn find_used_globals(&self, ir: &IR) -> std::collections::HashSet<String> {
 let mut used = std::collections::HashSet::new();
 
 for func in &ir.functions {
 for inst in &func.body {
 if inst.opcode == "load" || inst.opcode == "store" {
 if let Some(operand) = inst.operands.first() {
 used.insert(operand.value.clone());
 }
 }
 }
 }
 
 used
 }
}

/// CFG simplification pass
pub struct SimplifyCFGPass;

impl OptimizationPass for SimplifyCFGPass {
 fn name(&self) -> &str {
 "simplify_cfg"
 }

 fn run(&self, ir: &mut IR) -> Result<bool, OptError> {
 let mut changed = false;
 
 for func in &mut ir.functions {
 changed |= self.simplify_cfg_in_function(func);
 }
 
 if changed {
 log_debug!("CFG simplification pass made changes");
 }
 
 Ok(changed)
 }
}

impl SimplifyCFGPass {
 fn simplify_cfg_in_function(&self, func: &mut super::Function) -> bool {
 let mut changed = false;
 
 // Remove unreachable code after return/exit
 changed |= self.remove_unreachable_code(func);
 
 // Simplify conditional jumps
 changed |= self.simplify_conditional_jumps(func);
 
 // Remove redundant jumps
 changed |= self.remove_redundant_jumps(func);
 
 changed
 }

 fn remove_unreachable_code(&self, func: &mut super::Function) -> bool {
 let mut changed = false;
 let mut new_instructions = Vec::new();
 let mut reached_return = false;
 
 for inst in &func.body {
 if reached_return {
 // Skip code after return/exit
 log_debug!("Removing unreachable code after return");
 changed = true;
 continue;
 }
 
 new_instructions.push(inst.clone());
 
 if inst.opcode == "ret" || inst.opcode == "exit" {
 reached_return = true;
 }
 }
 
 func.body = new_instructions;
 changed
 }

 fn simplify_conditional_jumps(&self, func: &mut super::Function) -> bool {
 let mut changed = false;
 let mut new_instructions = Vec::new();
 let mut i = 0;
 
 while i < func.body.len() {
 let inst = &func.body[i];
 
 // Simplify: if (true) goto L1; goto L2; -> goto L1;
 if inst.opcode == "br_if" {
 if let Some(condition) = inst.operands.first() {
 if self.is_constant_true(condition) {
 // Replace conditional jump with unconditional
 if let Some(target) = inst.operands.get(1) {
 new_instructions.push(super::Instruction {
 opcode: "br".to_string(),
 operands: vec![target.clone()],
 });
 log_debug!("Simplified br_if with constant true to br");
 changed = true;
 i += 1;
 continue;
 }
 } else if self.is_constant_false(condition) {
 // Remove conditional jump when condition is false
 log_debug!("Removed br_if with constant false");
 changed = true;
 i += 1;
 continue;
 }
 }
 }
 
 // Simplify: if (cond) goto L; goto L; -> goto L;
 if i + 1 < func.body.len() {
 let next_inst = &func.body[i + 1];
 if inst.opcode == "br_if" && next_inst.opcode == "br" {
 if inst.operands.len() >= 2 && next_inst.operands.len() >= 1 {
 if inst.operands[1].value == next_inst.operands[0].value {
 // Both jumps go to same target, remove condition
 new_instructions.push(super::Instruction {
 opcode: "br".to_string(),
 operands: vec![next_inst.operands[0].clone()],
 });
 log_debug!("Merged duplicate jumps to same target");
 changed = true;
 i += 2;
 continue;
 }
 }
 }
 }
 
 new_instructions.push(inst.clone());
 i += 1;
 }
 
 func.body = new_instructions;
 changed
 }

 fn remove_redundant_jumps(&self, func: &mut super::Function) -> bool {
 let mut changed = false;
 let mut new_instructions = Vec::new();
 let mut i = 0;
 
 while i < func.body.len() {
 let inst = &func.body[i];
 
 // Remove: goto L; L: ... (jump to next instruction)
 if inst.opcode == "br" && i + 1 < func.body.len() {
 let next_inst = &func.body[i + 1];
 if next_inst.opcode == "label" {
 if inst.operands.len() >= 1 && next_inst.operands.len() >= 1 {
 if inst.operands[0].value == next_inst.operands[0].value {
 log_debug!("Removed redundant jump to next label");
 changed = true;
 i += 1;
 continue;
 }
 }
 }
 }
 
 new_instructions.push(inst.clone());
 i += 1;
 }
 
 func.body = new_instructions;
 changed
 }

 fn is_constant_true(&self, operand: &super::Operand) -> bool {
 if operand.kind == super::OperandKind::Immediate {
 let value = operand.value.trim().to_lowercase();
 value == "1" || value == "true"
 } else {
 false
 }
 }

 fn is_constant_false(&self, operand: &super::Operand) -> bool {
 if operand.kind == super::OperandKind::Immediate {
 let value = operand.value.trim().to_lowercase();
 value == "0" || value == "false"
 } else {
 false
 }
 }
}

/// Function inlining pass
pub struct InlinePass;

impl OptimizationPass for InlinePass {
 fn name(&self) -> &str {
 "inline"
 }

 fn run(&self, ir: &mut IR) -> Result<bool, OptError> {
 let mut changed = false;
 
 // Build function map for quick lookup
 let function_map: std::collections::HashMap<String, super::Function> = 
 ir.functions.iter()
 .filter(|f| f.name != "main") // Don't inline main
 .map(|f| (f.name.clone(), f.clone()))
 .collect();
 
 // Inline small functions and functions marked inline
 for func in &mut ir.functions {
 if func.name == "main" {
 changed |= self.inline_calls_in_function(func, &function_map);
 }
 }
 
 if changed {
 log_debug!("Function inlining pass made changes");
 }
 
 Ok(changed)
 }
}

impl InlinePass {
 fn inline_calls_in_function(
 &self,
 func: &mut super::Function,
 function_map: &std::collections::HashMap<String, super::Function>
 ) -> bool {
 let mut changed = false;
 let mut new_instructions = Vec::new();
 
 for inst in &func.body {
 if inst.opcode == "call" {
 if let Some(callee_name) = inst.operands.first() {
 if let Some(callee) = function_map.get(&callee_name.value) {
 if self.should_inline(callee) {
 log_debug!("Inlining call to {}", callee.name);
 self.inline_function_call(func, inst, callee, &mut new_instructions);
 changed = true;
 continue;
 }
 }
 }
 }
 new_instructions.push(inst.clone());
 }
 
 func.body = new_instructions;
 changed
 }

 fn should_inline(&self, func: &super::Function) -> bool {
 // Inline if marked as inline or if it's small
 if func.is_inline {
 return true;
 }
 
 // Count instructions (heuristic: inline if < 10 instructions)
 let instruction_count = func.body.len();
 instruction_count < 10
 }

 fn inline_function_call(
 &self,
 caller: &mut super::Function,
 call_inst: &super::Instruction,
 callee: &super::Function,
 new_instructions: &mut Vec<super::Instruction>
 ) {
 // Create temporary registers for parameters
 let mut param_mapping = std::collections::HashMap::new();
 
 for (i, param) in callee.params.iter().enumerate() {
 if let Some(arg) = call_inst.operands.get(i + 1) {
 let temp_reg = format!("{}_tmp_{}", param.name, param_mapping.len());
 param_mapping.insert(param.name.clone(), temp_reg.clone());
 
 // Move argument to temporary register
 new_instructions.push(super::Instruction {
 opcode: "mov".to_string(),
 operands: vec![
 super::Operand {
 kind: super::OperandKind::Register,
 value: temp_reg.clone(),
 },
 arg.clone(),
 ],
 });
 }
 }
 
 // Clone and adapt callee instructions
 for inst in &callee.body {
 let mut adapted_inst = inst.clone();
 
 // Replace parameter references with temporary registers
 for operand in &mut adapted_inst.operands {
 if let Some(temp_reg) = param_mapping.get(&operand.value) {
 operand.value = temp_reg.clone();
 }
 }
 
 new_instructions.push(adapted_inst);
 }
 
 // Handle return value
 if let Some(result_operand) = call_inst.operands.first() {
 // Find return instruction in callee and move to caller's result
 for inst in &callee.body {
 if inst.opcode == "ret" {
 if let Some(return_value) = inst.operands.first() {
 new_instructions.push(super::Instruction {
 opcode: "mov".to_string(),
 operands: vec![
 result_operand.clone(),
 return_value.clone(),
 ],
 });
 }
 break;
 }
 }
 }
 }
}

/// Constant folding pass
pub struct ConstantFoldingPass;

impl OptimizationPass for ConstantFoldingPass {
 fn name(&self) -> &str {
 "const_fold"
 }

 fn run(&self, ir: &mut IR) -> Result<bool, OptError> {
 let mut changed = false;
 
 for func in &mut ir.functions {
 changed |= self.fold_constants_in_function(func);
 }
 
 if changed {
 log_debug!("Constant folding pass made changes");
 }
 
 Ok(changed)
 }
}

impl ConstantFoldingPass {
 fn fold_constants_in_function(&self, func: &mut super::Function) -> bool {
 let mut changed = false;
 let mut new_instructions = Vec::new();
 
 for inst in &func.body {
 if self.is_constant_operation(inst) {
 if let Some(result) = self.evaluate_constant(inst) {
 // Replace constant operation with result
 new_instructions.push(super::Instruction {
 opcode: "mov".to_string(),
 operands: vec![
 super::Operand {
 kind: super::OperandKind::Register,
 value: inst.operands[0].value.clone(),
 },
 super::Operand {
 kind: super::OperandKind::Immediate,
 value: result,
 },
 ],
 });
 changed = true;
 log_debug!("Folded constant expression: {} -> {}", 
 self.format_instruction(inst), result);
 continue;
 }
 }
 new_instructions.push(inst.clone());
 }
 
 func.body = new_instructions;
 changed
 }

 fn is_constant_operation(&self, inst: &super::Instruction) -> bool {
 match inst.opcode.as_str() {
 "add" | "sub" | "mul" | "div" | "mod" => {
 inst.operands.len() >= 3 && 
 self.is_constant(&inst.operands[1]) && 
 self.is_constant(&inst.operands[2])
 }
 _ => false,
 }
 }

 fn is_constant(&self, operand: &super::Operand) -> bool {
 matches!(operand.kind, super::OperandKind::Immediate)
 }

 fn evaluate_constant(&self, inst: &super::Instruction) -> Option<String> {
 let op1 = self.parse_integer(&inst.operands[1].value)?;
 let op2 = self.parse_integer(&inst.operands[2].value)?;
 
 let result = match inst.opcode.as_str() {
 "add" => op1 + op2,
 "sub" => op1 - op2,
 "mul" => op1 * op2,
 "div" => op1 / op2,
 "mod" => op1 % op2,
 _ => return None,
 };
 
 Some(result.to_string())
 }

 fn parse_integer(&self, s: &str) -> Option<i64> {
 s.trim().parse().ok()
 }

 fn format_instruction(&self, inst: &super::Instruction) -> String {
 let operands: Vec<String> = inst.operands.iter()
 .map(|op| op.value.clone())
 .collect();
 format!("{} {}", inst.opcode, operands.join(", "))
 }
}

/// Globalvalueencodingsignal Pass
pub struct GlobalValueNumberingPass;

impl OptimizationPass for GlobalValueNumberingPass {
 fn name(&self) -> &str {
 "gvn"
 }

 fn run(&self, _ir: &mut IR) -> Result<bool, OptError> {
 // TODO: Implement GVN
 // - DivideremainderCompute
 // - publicsharedchildformreachstyleDivide
 Ok(false)
 }
}

/// loopOptimization Pass
pub struct LoopOptimizationPass;

impl OptimizationPass for LoopOptimizationPass {
 fn name(&self) -> &str {
 "loop_opt"
 }

 fn run(&self, _ir: &mut IR) -> Result<bool, OptError> {
 // TODO: Implement loop optimization
 // - loopnotVariableoutside
 // - loopextendopen
 // - strongmeasurementweak
 Ok(false)
 }
}

/// directionquantize Pass
pub struct VectorizationPass;

impl OptimizationPass for VectorizationPass {
 fn name(&self) -> &str {
 "vectorize"
 }

 fn run(&self, _ir: &mut IR) -> Result<bool, OptError> {
 // TODO: Implement vectorization
 // - SIMD directionquantize
 // - loopdirectionquantize
 Ok(false)
 }
}

/// Dividedeadsymbolsignal Pass
pub struct StripDeadPass;

impl OptimizationPass for StripDeadPass {
 fn name(&self) -> &str {
 "strip_dead"
 }

 fn run(&self, ir: &mut IR) -> Result<bool, OptError> {
 // Divideplacefinitemakeuse symbolsignal
 let dce = DeadCodeEliminationPass;
 dce.run(ir)
 }
}