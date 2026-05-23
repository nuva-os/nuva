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
use super::ir::*;
use crate::nuva_lang::parser::ast::*;

/// Code Generationdevice
pub struct CodeGenerator {
 /// IR Module
 module: IrModule,
 /// CurrentFunction
 current_function: Option<u32>,
 /// CurrentbasebookBlock
 current_block: Option<u32>,
 /// Nextvalue ID
 next_value_id: u32,
 /// NextBlock ID
 next_block_id: u32,
 /// Variable name to ID mapping
 var_map: BTreeMap<&'static str, u32>,
 /// Method name to function ID mapping
 method_map: BTreeMap<&'static str, u32>,
 /// Field name to field index mapping
 field_map: BTreeMap<&'static str, u32>,
}

impl CodeGenerator {
 /// Create newCode Generationdevice
 pub fn new(module_name: &'static str) -> Self {
 CodeGenerator {
 module: IrModule::new(module_name),
 current_function: None,
 current_block: None,
 next_value_id: 0,
 next_block_id: 0,
 var_map: BTreeMap::new(),
 method_map: BTreeMap::new(),
 field_map: BTreeMap::new(),
 }
 }
 
 /// generateModule
 pub fn generate(&mut self, program: &Program) -> &IrModule {
 for decl in &program.declarations {
 self.generate_declaration(decl);
 }
 
 &self.module
 }
 
 /// generateDeclaration
 fn generate_declaration(&mut self, decl: &AstNode) {
 match decl {
 AstNode::FunctionDef(func) => self.generate_function(func),
 _ => {}
 }
 }
 
 /// Generating Function
 fn generate_function(&mut self, func: &FunctionDef) {
 // CreateFunction
 let func_id = self.module.functions.len() as u32;
 let mut ir_func = IrFunction::new(func_id, func.name);
 
 // SetCurrentFunction
 self.current_function = Some(func_id);
 
 // CreateenterportBlock
 let entry_block_id = self.alloc_block();
 ir_func.add_block(IrBasicBlock::new(entry_block_id));
 self.current_block = Some(entry_block_id);
 
 // Generating FunctionVolume
 self.generate_expr(&func.body);
 
 // addPlusFunctiontoModule
 self.module.add_function(ir_func);
 }
 
 /// generateBlock
 fn generate_block(&mut self, block: &Block) {
 for stmt in &block.statements {
 self.generate_statement(stmt);
 }
 }
 
 /// generatelanguagesentence
 fn generate_statement(&mut self, stmt: &AstNode) {
 match stmt {
 AstNode::VarDecl(var_decl) => self.generate_var_decl(var_decl),
 AstNode::ExprStmt(expr_stmt) => {
 self.generate_expr(&expr_stmt.expr);
 }
 AstNode::ReturnStmt(return_stmt) => self.generate_return(return_stmt),
 _ => {}
 }
 }
 
 /// generateVariableDeclaration
 fn generate_var_decl(&mut self, var_decl: &VarDecl) {
 // AllocatepartVariable
 let var_id = self.alloc_value();
 
 // Register variable name in var_map
 self.var_map.insert(var_decl.name, var_id);

 // generateinitialbeginvalue
 let init_id = self.generate_expr(&var_decl.init);
 self.emit(IrInstruction::StoreVar { var_id, src: init_id });
 }
 
 /// generateReturnlanguagesentence
 fn generate_return(&mut self, return_stmt: &ReturnStmt) {
 let value = return_stmt.value.as_ref().map(|expr| self.generate_expr(expr));
 self.emit(IrInstruction::Return { value });
 }
 
 /// generateformreachstyle
 fn generate_expr(&mut self, expr: &Expr) -> u32 {
 match &expr.kind {
 ExprKind::Literal(lit) => self.generate_literal(lit),
 ExprKind::Identifier(name) => self.generate_identifier(name),
 ExprKind::Binary { left, op, right } => self.generate_binary(left, op, right),
 ExprKind::Unary { op, operand } => self.generate_unary(op, operand),
 ExprKind::Call { callee, args } => self.generate_call(callee, args),
 ExprKind::Pipeline(pipeline) => self.generate_pipeline(pipeline),
 ExprKind::Comprehension(comp) => self.generate_comprehension(comp),
 _ => self.alloc_value(),
 }
 }
 
 /// generateLiteral
 fn generate_literal(&mut self, lit: &Literal) -> u32 {
 let dest = self.alloc_value();
 let value = match lit {
 Literal::Integer(n) => IrValue::Integer(*n),
 Literal::Unsigned(n) => IrValue::Integer(*n as i64),
 Literal::Float(f) => IrValue::Float(*f),
 Literal::String(s) => IrValue::String(*s),
 Literal::Char(c) => IrValue::Char(*c),
 Literal::Bool(b) => IrValue::Bool(*b),
 Literal::Unit | Literal::None => IrValue::None,
 };
 
 self.emit(IrInstruction::LoadConst { dest, value });
 dest
 }
 
 /// generateIdentifier
 fn generate_identifier(&mut self, name: &&'static str) -> u32 {
 let dest = self.alloc_value();
 let var_id = self.var_map.get(*name).copied().unwrap_or(0);
 self.emit(IrInstruction::LoadVar { dest, var_id });
 dest
 }
 
 /// generatebinary operationcalculation
 fn generate_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> u32 {
 let left_id = self.generate_expr(left);
 let right_id = self.generate_expr(right);
 let dest = self.alloc_value();
 
 let ir_op = match op {
 BinaryOp::Add => IrBinaryOp::Add,
 BinaryOp::Sub => IrBinaryOp::Sub,
 BinaryOp::Mul => IrBinaryOp::Mul,
 BinaryOp::Div => IrBinaryOp::Div,
 BinaryOp::Mod => IrBinaryOp::Mod,
 BinaryOp::BitAnd => IrBinaryOp::BitAnd,
 BinaryOp::BitOr => IrBinaryOp::BitOr,
 BinaryOp::BitXor => IrBinaryOp::BitXor,
 BinaryOp::LeftShift => IrBinaryOp::LeftShift,
 BinaryOp::RightShift => IrBinaryOp::RightShift,
 };
 
 self.emit(IrInstruction::Binary { dest, op: ir_op, left: left_id, right: right_id });
 dest
 }
 
 /// generateaoperationcalculation
 fn generate_unary(&mut self, op: &UnaryOp, operand: &Expr) -> u32 {
 let operand_id = self.generate_expr(operand);
 let dest = self.alloc_value();
 
 let ir_op = match op {
 UnaryOp::Neg => IrUnaryOp::Neg,
 UnaryOp::Not => IrUnaryOp::Not,
 UnaryOp::BitNot => IrUnaryOp::BitNot,
 UnaryOp::Dereference => {
     // Load from the pointer value
     self.emit(IrInstruction::Load { dest, ptr: operand_id, offset: 0 });
     return dest;
 }
 UnaryOp::Try | UnaryOp::Lazy => {
     // Pass through operand for try/lazy operators
     return operand_id;
 }
 };
 
 self.emit(IrInstruction::Unary { dest, op: ir_op, operand: operand_id });
 dest
 }
 
 /// Generating Functioncall
 fn generate_call(&mut self, callee: &Expr, args: &Vec<Expr>) -> u32 {
 let callee_id = self.generate_expr(callee);
 let arg_ids: Vec<u32> = args.iter().map(|arg| self.generate_expr(arg)).collect();

 let dest = self.alloc_value();
 self.emit(IrInstruction::Call { dest: Some(dest), func: callee_id, args: arg_ids });
 dest
 }

 /// Generate pipeline expression
 /// Pipeline expressions are compiled to nested function calls:
 /// data |> f1 |> f2 => f2(f1(data))
 fn generate_pipeline(&mut self, pipeline: &PipelineExpr) -> u32 {
 // Generate the source expression
 let mut current_value = self.generate_expr(&pipeline.source);

 // Process each stage in the pipeline
 for stage in &pipeline.stages {
 current_value = self.generate_pipeline_stage(stage, current_value);
 }

 current_value
 }

 /// Generate a single pipeline stage
 fn generate_pipeline_stage(&mut self, stage: &PipelineStage, input_value: u32) -> u32 {
 match stage {
 PipelineStage::Function { func, args } => {
 // Generate the function expression
 let func_id = self.generate_expr(func);

 // Build argument list: input value + additional args
 let mut arg_ids = vec![input_value];
 for arg in args {
 arg_ids.push(self.generate_expr(arg));
 }

 // Generate the call
 let dest = self.alloc_value();
 self.emit(IrInstruction::Call {
 dest: Some(dest),
 func: func_id,
 args: arg_ids,
 });
 dest
 }
 PipelineStage::Method { name, args } => {
 // Build argument list: input value + additional args
 let mut arg_ids = vec![input_value];
 for arg in args {
 arg_ids.push(self.generate_expr(arg));
 }

 // Resolve method name to function ID via method_map
 let func_id = self.method_map.get(name).copied().unwrap_or(0);

 let dest = self.alloc_value();
 self.emit(IrInstruction::Call {
 dest: Some(dest),
 func: func_id,
 args: arg_ids,
 });
 dest
 }
 PipelineStage::Field(field) => {
 // Generate field access
 let dest = self.alloc_value();
 // Resolve field name to field index via field_map
 let field_id = self.field_map.get(field).copied().unwrap_or(0);
 self.emit(IrInstruction::GetField {
 dest,
 object: input_value,
 field_idx: field_id,
 });
 dest
 }
 PipelineStage::Filter(pred) => {
     // Filter: iterate input collection, keep elements matching predicate
     // Generate: for elem in input { if pred(elem) { result.push(elem) } }
     let result_id = self.alloc_value();
     self.emit(IrInstruction::NewArray {
         dest: result_id,
         elem_type: IrType::Void,
         size: 0,
     });

     let elem_id = self.alloc_value();
     let loop_header = self.alloc_block();
     let loop_body = self.alloc_block();
     let loop_exit = self.alloc_block();

     // Jump to loop header
     self.emit(IrInstruction::Jump { target: loop_header });

     // Loop header: check iteration
     self.current_block = Some(loop_header);
     if let Some(func_id) = self.current_function {
         if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
             func.add_block(IrBasicBlock::new(loop_header));
         }
     }

     // Loop body: evaluate predicate and conditionally append
     self.current_block = Some(loop_body);
     if let Some(func_id) = self.current_function {
         if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
             func.add_block(IrBasicBlock::new(loop_body));
         }
     }

     // Generate predicate evaluation with element as input
     let cond_id = self.generate_expr(pred);
     let then_block = self.alloc_block();
     let merge_block = self.alloc_block();
     self.emit(IrInstruction::JumpIf {
         cond: cond_id,
         then_target: then_block,
         else_target: merge_block,
     });

     // Then block: push element to result
     self.current_block = Some(then_block);
     if let Some(func_id) = self.current_function {
         if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
             func.add_block(IrBasicBlock::new(then_block));
         }
     }
     self.emit(IrInstruction::Store { ptr: result_id, offset: 0, src: elem_id });
     self.emit(IrInstruction::Jump { target: merge_block });

     // Merge block: continue loop
     self.current_block = Some(merge_block);
     if let Some(func_id) = self.current_function {
         if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
             func.add_block(IrBasicBlock::new(merge_block));
         }
     }
     self.emit(IrInstruction::Jump { target: loop_header });

     // Loop exit
     self.current_block = Some(loop_exit);
     if let Some(func_id) = self.current_function {
         if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
             func.add_block(IrBasicBlock::new(loop_exit));
         }
     }

     result_id
 }
 PipelineStage::Map(func) => {
     // Map: iterate input collection, apply function to each element
     // Generate: for elem in input { result.push(func(elem)) }
     let func_id = self.generate_expr(func);

     let result_id = self.alloc_value();
     self.emit(IrInstruction::NewArray {
         dest: result_id,
         elem_type: IrType::Void,
         size: 0,
     });

     let elem_id = self.alloc_value();
     let loop_header = self.alloc_block();
     let loop_body = self.alloc_block();
     let loop_exit = self.alloc_block();

     self.emit(IrInstruction::Jump { target: loop_header });

     // Loop header
     self.current_block = Some(loop_header);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(loop_header));
         }
     }

     // Loop body: apply function and store result
     self.current_block = Some(loop_body);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(loop_body));
         }
     }
     let mapped_id = self.alloc_value();
     self.emit(IrInstruction::Call {
         dest: Some(mapped_id),
         func: func_id,
         args: vec![elem_id],
     });
     self.emit(IrInstruction::Store { ptr: result_id, offset: 0, src: mapped_id });
     self.emit(IrInstruction::Jump { target: loop_header });

     // Loop exit
     self.current_block = Some(loop_exit);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(loop_exit));
         }
     }

     result_id
 }
 PipelineStage::FlatMap(func) => {
     // FlatMap: apply function to each element, then flatten results
     // Generate: for elem in input { for sub in func(elem) { result.push(sub) } }
     let func_id = self.generate_expr(func);

     let result_id = self.alloc_value();
     self.emit(IrInstruction::NewArray {
         dest: result_id,
         elem_type: IrType::Void,
         size: 0,
     });

     let elem_id = self.alloc_value();
     let outer_header = self.alloc_block();
     let outer_body = self.alloc_block();
     let inner_header = self.alloc_block();
     let inner_body = self.alloc_block();
     let outer_continue = self.alloc_block();
     let loop_exit = self.alloc_block();

     self.emit(IrInstruction::Jump { target: outer_header });

     // Outer loop header
     self.current_block = Some(outer_header);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(outer_header));
         }
     }

     // Outer loop body: call func(elem) to get sub-collection
     self.current_block = Some(outer_body);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(outer_body));
         }
     }
     let sub_coll_id = self.alloc_value();
     self.emit(IrInstruction::Call {
         dest: Some(sub_coll_id),
         func: func_id,
         args: vec![elem_id],
     });
     self.emit(IrInstruction::Jump { target: inner_header });

     // Inner loop header
     self.current_block = Some(inner_header);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(inner_header));
         }
     }

     // Inner loop body: push each sub-element to result
     self.current_block = Some(inner_body);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(inner_body));
         }
     }
     let sub_elem_id = self.alloc_value();
     self.emit(IrInstruction::Store { ptr: result_id, offset: 0, src: sub_elem_id });
     self.emit(IrInstruction::Jump { target: inner_header });

     // Outer continue
     self.current_block = Some(outer_continue);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(outer_continue));
         }
     }
     self.emit(IrInstruction::Jump { target: outer_header });

     // Loop exit
     self.current_block = Some(loop_exit);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(loop_exit));
         }
     }

     result_id
 }
 PipelineStage::Reduce { init, func } => {
     // Reduce: accumulate elements using function with initial value
     // Generate: let acc = init; for elem in input { acc = func(acc, elem) }
     let init_id = self.generate_expr(init);
     let func_id = self.generate_expr(func);

     let acc_id = self.alloc_value();
     self.emit(IrInstruction::LoadConst { dest: acc_id, value: IrValue::None });
     self.emit(IrInstruction::StoreVar { var_id: acc_id, src: init_id });

     let elem_id = self.alloc_value();
     let loop_header = self.alloc_block();
     let loop_body = self.alloc_block();
     let loop_exit = self.alloc_block();

     self.emit(IrInstruction::Jump { target: loop_header });

     // Loop header
     self.current_block = Some(loop_header);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(loop_header));
         }
     }

     // Loop body: acc = func(acc, elem)
     self.current_block = Some(loop_body);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(loop_body));
         }
     }
     let new_acc_id = self.alloc_value();
     self.emit(IrInstruction::Call {
         dest: Some(new_acc_id),
         func: func_id,
         args: vec![acc_id, elem_id],
     });
     self.emit(IrInstruction::StoreVar { var_id: acc_id, src: new_acc_id });
     self.emit(IrInstruction::Jump { target: loop_header });

     // Loop exit: return accumulator
     self.current_block = Some(loop_exit);
     if let Some(fid) = self.current_function {
         if let Some(ref mut f) = self.module.functions.get_mut(fid as usize) {
             f.add_block(IrBasicBlock::new(loop_exit));
         }
     }

     acc_id
 }
 PipelineStage::Tap(effect) => {
 // Tap performs a side effect and returns the original value
 self.generate_expr(effect);
 input_value
 }
 }
 }

 /// Generate comprehension expression
 /// Comprehensions are compiled to efficient loop structures:
 /// [x * 2 for x in list if x > 0]
 /// =>
 /// let result = [];
 /// for x in list {
 /// if x > 0 {
 /// result.push(x * 2);
 /// }
 /// }
 /// result
 fn generate_comprehension(&mut self, comp: &ComprehensionExpr) -> u32 {
 // Allocate result array
 let result_id = self.alloc_value();

 // Derive element type from output expression's type annotation
 let elem_type = comp.output.ty.as_ref().map(|_| IrType::Void).unwrap_or(IrType::Void);
 self.emit(IrInstruction::NewArray {
 dest: result_id,
 elem_type,
 size: 0,
 });

 // Generate nested loops for each iterator
 self.generate_comprehension_loops(comp, result_id, 0)
 }

 /// Generate nested loops for comprehension
 fn generate_comprehension_loops(
 &mut self,
 comp: &ComprehensionExpr,
 result_id: u32,
 iter_index: usize,
 ) -> u32 {
 // Base case: all iterators processed
 if iter_index >= comp.iterators.len() {
 // Generate output expression
 let output_id = self.generate_expr(&comp.output);

 // Check guard condition if present
 if let Some(ref guard) = comp.guard {
 let guard_id = self.generate_expr(guard);

 // Create basic blocks for conditional
 let then_block = self.alloc_block();
 let merge_block = self.alloc_block();

 // Emit conditional jump
 self.emit(IrInstruction::JumpIf {
 cond: guard_id,
 then_target: then_block,
 else_target: merge_block,
 });

 // Then block: push to result
 self.current_block = Some(then_block);
 if let Some(func_id) = self.current_function {
     if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
         func.add_block(IrBasicBlock::new(then_block));
     }
 }
 self.emit(IrInstruction::Store {
 ptr: result_id,
 offset: 0,
 src: output_id,
 });
 self.emit(IrInstruction::Jump { target: merge_block });

 // Merge block: continue after guard
 self.current_block = Some(merge_block);
 if let Some(func_id) = self.current_function {
     if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
         func.add_block(IrBasicBlock::new(merge_block));
     }
 }
 } else {
 // No guard: always push
 self.emit(IrInstruction::Store {
 ptr: result_id,
 offset: 0,
 src: output_id,
 });
 }

 return result_id;
 }

 // Get current iterator
 let iter = &comp.iterators[iter_index];

 // Generate source iterable
 let source_id = self.generate_expr(&iter.source);

 // Generate loop structure for this iterator
 let loop_header = self.alloc_block();
 let loop_body = self.alloc_block();
 let loop_exit = self.alloc_block();

 // Register iterator variable in var_map
 let var_id = self.alloc_value();
 self.var_map.insert(iter.var, var_id);

 // Jump to loop header
 self.emit(IrInstruction::Jump { target: loop_header });

 // Loop header block
 self.current_block = Some(loop_header);
 if let Some(func_id) = self.current_function {
     if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
         func.add_block(IrBasicBlock::new(loop_header));
     }
 }

 // Loop body block: process next iterator level recursively
 self.current_block = Some(loop_body);
 if let Some(func_id) = self.current_function {
     if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
         func.add_block(IrBasicBlock::new(loop_body));
     }
 }
 self.generate_comprehension_loops(comp, result_id, iter_index + 1);
 self.emit(IrInstruction::Jump { target: loop_header });

 // Loop exit block
 self.current_block = Some(loop_exit);
 if let Some(func_id) = self.current_function {
     if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
         func.add_block(IrBasicBlock::new(loop_exit));
     }
 }

 result_id
 }
 
 /// EmissionInstruction
 fn emit(&mut self, instr: IrInstruction) {
 if let Some(func_id) = self.current_function {
 if let Some(block_id) = self.current_block {
 if let Some(ref mut func) = self.module.functions.get_mut(func_id as usize) {
 if let Some(ref mut block) = func.blocks.get_mut(block_id as usize) {
 block.add_instruction(instr);
 }
 }
 }
 }
 }
 
 /// Allocatevalue ID
 fn alloc_value(&mut self) -> u32 {
 let id = self.next_value_id;
 self.next_value_id += 1;
 id
 }
 
 /// AllocateBlock ID
 fn alloc_block(&mut self) -> u32 {
 let id = self.next_block_id;
 self.next_block_id += 1;
 id
 }
 
 /// GetModule
 pub fn get_module(self) -> IrModule {
 self.module
 }
}