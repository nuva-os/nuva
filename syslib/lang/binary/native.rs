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


use alloc::vec::Vec;

use crate::syslib::lang::codegen::ir::{IrModule, IrFunction, IrInstruction, IrValue, IrType};
use crate::{pr_debug, pr_info};

/// sourcecreateCode Generationdevice
pub struct NativeCodeGen {
    /// targetArchitecture: 0=ARM64, 1=x86-64
    pub arch: u32,
    /// OutputBuffer
    pub output: Vec<u8>,
    /// CurrentOffset
    pub offset: u64,
}

impl NativeCodeGen {
    pub fn new(arch: u32) -> Self {
        NativeCodeGen {
            arch,
            output: Vec::new(),
            offset: 0,
        }
    }
    
    /// encodingtranslateModule
    pub fn compile_module(&mut self, module: &IrModule) -> Result<Vec<u8>, i32> {
        log_info!("Compiling module: {}", module.name);
        
        // generateFileHead
        self.emit_header(module)?;
        
        // encodingtranslateplacefiniteFunction
        for func in &module.functions {
            self.compile_function(func)?;
        }
        
        // generateGlobalVariable
        for global in &module.globals {
            let (id, name, ty) = global;
            self.emit_global(*id, name, ty)?;
        }
        
        Ok(self.output.clone())
    }
    
    /// Generate file header with entry point and computed code size
    fn emit_header(&mut self, module: &IrModule) -> Result<(), i32> {
        // Find the main function ID to use as the entry point
        let entry = module.functions.iter()
            .find(|f| f.name == "main")
            .map(|f| f.id as u64)
            .unwrap_or(0);

        // Compute total code size from all functions
        let code_size: u64 = module.functions.iter()
            .map(|f| {
                let instr_count: usize = f.blocks.iter()
                    .map(|b| b.instructions.len())
                    .sum();
                // Approximate: 4 bytes per instruction + prologue/epilogue
                let func_body = instr_count * 4;
                let prologue = if self.arch == 0 { 8 } else { 4 };  // ARM64 vs x86-64
                let epilogue = if self.arch == 0 { 8 } else { 3 };
                (func_body + prologue + epilogue) as u64
            })
            .sum();

        let header = NexHeader {
            magic: NEX_MAGIC,
            version: NEX_VERSION,
            arch: self.arch,
            flags: 0,
            entry,
            code_offset: 64,
            code_size,
            data_offset: 0,
            data_size: 0,
            bss_size: 0,
            symtab_offset: 0,
            symtab_size: 0,
            reloc_offset: 0,
            reloc_size: 0,
            segtab_offset: 0,
            segtab_size: 0,
            checksum: 0,
            reserved: [0; 8],
        };
        
        // WriteHeadpart
        // SAFETY: unsafe block required for low-level memory or hardware access
        let header_bytes = unsafe {
            core::slice::from_raw_parts(
                &header as *const NexHeader as *const u8,
                core::mem::size_of::<NexHeader>()
            )
        };
        
        self.output.extend_from_slice(header_bytes);
        self.offset += header_bytes.len() as u64;
        
        Ok(())
    }
    
    /// encodingtranslateFunction
    fn compile_function(&mut self, func: &IrFunction) -> Result<(), i32> {
        log_debug!("Compiling function: {}", func.name);
        
        // Generating Functionorderlanguage
        self.emit_prologue(func)?;
        
        // encodingtranslateplacefinitebasebookBlock
        for block in &func.blocks {
            for instr in &block.instructions {
                self.compile_instruction(instr)?;
            }
        }
        
        // Generating FunctionTailsound
        self.emit_epilogue(func)?;
        
        Ok(())
    }
    
    /// encodingtranslateInstruction
    fn compile_instruction(&mut self, instr: &IrInstruction) -> Result<(), i32> {
        match instr {
            IrInstruction::LoadConst { dest, value } => {
                self.emit_load_const(*dest, value)?;
            }
            IrInstruction::LoadVar { dest, var_id } => {
                self.emit_load_var(*dest, *var_id)?;
            }
            IrInstruction::StoreVar { var_id, src } => {
                self.emit_store_var(*var_id, *src)?;
            }
            IrInstruction::Binary { dest, op, left, right } => {
                self.emit_binary(*dest, op, *left, *right)?;
            }
            IrInstruction::Unary { dest, op, operand } => {
                self.emit_unary(*dest, op, *operand)?;
            }
            IrInstruction::Compare { dest, op, left, right } => {
                self.emit_compare(*dest, op, *left, *right)?;
            }
            IrInstruction::Jump { target } => {
                self.emit_jump(*target)?;
            }
            IrInstruction::JumpIf { cond, then_target, else_target } => {
                self.emit_jump_if(*cond, *then_target, *else_target)?;
            }
            IrInstruction::Call { dest, func, args } => {
                self.emit_call(*dest, *func, args)?;
            }
            IrInstruction::Return { value } => {
                self.emit_return(value)?;
            }
            IrInstruction::Alloca { dest, size, align } => {
                self.emit_alloca(*dest, *size, *align)?;
            }
            IrInstruction::Load { dest, ptr, offset } => {
                self.emit_load(*dest, *ptr, *offset)?;
            }
            IrInstruction::Store { ptr, offset, src } => {
                self.emit_store(*ptr, *offset, *src)?;
            }
            IrInstruction::GetField { dest, object, field_idx } => {
                self.emit_get_field(*dest, *object, *field_idx)?;
            }
            IrInstruction::SetField { object, field_idx, src } => {
                self.emit_set_field(*object, *field_idx, *src)?;
            }
            IrInstruction::NewArray { dest, elem_type, size } => {
                self.emit_new_array(*dest, elem_type, *size)?;
            }
            IrInstruction::ArrayAccess { dest, array, index } => {
                self.emit_array_access(*dest, *array, *index)?;
            }
            IrInstruction::Cast { dest, src, target_type } => {
                self.emit_cast(*dest, *src, target_type)?;
            }
            IrInstruction::Phi { dest, incoming } => {
                self.emit_phi(*dest, incoming)?;
            }
        }
        
        Ok(())
    }
    
    /// Generating Functionorderlanguage
    fn emit_prologue(&mut self, _func: &IrFunction) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64
            // stp x29, x30, [sp, #-16]!
            // mov x29, sp
            self.emit_bytes(&[0xFD, 0x7B, 0xBF, 0xA9])?;
            self.emit_bytes(&[0xFD, 0x03, 0x00, 0x91])?;
        } else {
            // x86-64
            // push rbp
            // mov rbp, rsp
            self.emit_bytes(&[0x55])?;
            self.emit_bytes(&[0x48, 0x89, 0xE5])?;
        }
        
        Ok(())
    }
    
    /// Generating FunctionTailsound
    fn emit_epilogue(&mut self, _func: &IrFunction) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64
            // ldp x29, x30, [sp], #16
            // ret
            self.emit_bytes(&[0xFD, 0x7B, 0xC1, 0xA8])?;
            self.emit_bytes(&[0xC0, 0x03, 0x5F, 0xD6])?;
        } else {
            // x86-64
            // pop rbp
            // ret
            self.emit_bytes(&[0x5D])?;
            self.emit_bytes(&[0xC3])?;
        }
        
        Ok(())
    }
    
    /// generatePlusloadConstant
    fn emit_load_const(&mut self, dest: u32, value: &IrValue) -> Result<(), i32> {
        match value {
            IrValue::Integer(v) => {
                if self.arch == 0 {
                    // ARM64: mov xN, #imm
                    self.emit_mov_imm64(dest, *v as u64)?;
                } else {
                    // x86-64: mov rax, imm64
                    self.emit_bytes(&[0x48, 0xB8 + (dest as u8 & 0x7)])?;
                    self.emit_u64(*v as u64)?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// generatebinary operationcalculation
    fn emit_binary(&mut self, dest: u32, op: &crate::syslib::lang::codegen::ir::IrBinaryOp, left: u32, right: u32) -> Result<(), i32> {
        use crate::syslib::lang::codegen::ir::IrBinaryOp;
        
        if self.arch == 0 {
            // ARM64
            match op {
                IrBinaryOp::Add => {
                    // add xN, xM, xK
                    let enc = 0x8B000000 | (right << 16) | (left << 5) | dest;
                    self.emit_u32(enc)?;
                }
                IrBinaryOp::Sub => {
                    // sub xN, xM, xK
                    let enc = 0xCB000000 | (right << 16) | (left << 5) | dest;
                    self.emit_u32(enc)?;
                }
                IrBinaryOp::Mul => {
                    // mul xN, xM, xK
                    let enc = 0x9B000000 | (right << 16) | (left << 5) | dest;
                    self.emit_u32(enc)?;
                }
                _ => {}
            }
        } else {
            // x86-64
            match op {
                IrBinaryOp::Add => {
                    // add rax, rbx
                    self.emit_bytes(&[0x48, 0x01, 0xD8])?;
                }
                IrBinaryOp::Sub => {
                    // sub rax, rbx
                    self.emit_bytes(&[0x48, 0x29, 0xD8])?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// generatejumpbranch
    fn emit_jump(&mut self, target: u32) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: b offset
            let offset = (target as i32) << 2;
            let enc = 0x14000000 | ((offset >> 2) as u32 & 0x3FFFFFF);
            self.emit_u32(enc)?;
        } else {
            // x86-64: jmp rel32
            self.emit_bytes(&[0xE9])?;
            self.emit_u32(target)?;
        }
        
        Ok(())
    }
    
    /// generatetuneuse
    fn emit_call(&mut self, _dest: Option<u32>, func: u32, _args: &[u32]) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: bl offset
            let offset = (func as i32) << 2;
            let enc = 0x94000000 | ((offset >> 2) as u32 & 0x3FFFFFF);
            self.emit_u32(enc)?;
        } else {
            // x86-64: call rel32
            self.emit_bytes(&[0xE8])?;
            self.emit_u32(func)?;
        }
        
        Ok(())
    }
    
    /// generateReturn
    fn emit_return(&mut self, value: &Option<u32>) -> Result<(), i32> {
        if let Some(v) = value {
            // MoveReturn ValuetoReturnRegister
            if self.arch == 0 {
                // ARM64: mov x0, xN
                let enc = 0xAA0003E0 | (*v << 16);
                self.emit_u32(enc)?;
            } else {
                // x86-64: mov rax, rN
                self.emit_bytes(&[0x48, 0x89, 0xC0])?;
            }
        }
        
        // ReturnInstructionin epilogue infixgenerate
        Ok(())
    }
    
    /// auxiliaryFunction
    fn emit_bytes(&mut self, bytes: &[u8]) -> Result<(), i32> {
        self.output.extend_from_slice(bytes);
        self.offset += bytes.len() as u64;
        Ok(())
    }
    
    fn emit_u32(&mut self, val: u32) -> Result<(), i32> {
        self.output.extend_from_slice(&val.to_le_bytes());
        self.offset += 4;
        Ok(())
    }
    
    fn emit_u64(&mut self, val: u64) -> Result<(), i32> {
        self.output.extend_from_slice(&val.to_le_bytes());
        self.offset += 8;
        Ok(())
    }
    
    fn emit_mov_imm64(&mut self, reg: u32, val: u64) -> Result<(), i32> {
        // ARM64: movz + movk Sequence
        let low = (val & 0xFFFF) as u16;
        let mid = ((val >> 16) & 0xFFFF) as u16;
        let high = ((val >> 32) & 0xFFFF) as u16;
        let top = ((val >> 48) & 0xFFFF) as u16;
        
        // movz xN, low
        self.emit_u32(0xD2800000 | (reg << 0) | ((low as u32) << 5))?;
        
        if mid != 0 {
            // movk xN, mid, lsl #16
            self.emit_u32(0xF2A00000 | (reg << 0) | ((mid as u32) << 5))?;
        }
        
        if high != 0 {
            // movk xN, high, lsl #32
            self.emit_u32(0xF2C00000 | (reg << 0) | ((high as u32) << 5))?;
        }
        
        if top != 0 {
            // movk xN, top, lsl #48
            self.emit_u32(0xF2E00000 | (reg << 0) | ((top as u32) << 5))?;
        }
        
        Ok(())
    }
    
    fn emit_load_var(&mut self, dest: u32, var_id: u32) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: ldr xN, [x29, #offset] (load from frame pointer + var offset)
            let offset = (var_id as u32) * 8;
            let enc = 0xF94003A0 | (dest & 0x1F) | ((offset / 8) << 10);
            self.emit_u32(enc)?;
        } else {
            // x86-64: mov rN, [rbp - offset]
            self.emit_bytes(&[0x48, 0x8B])?;
            let offset = (var_id as u8).wrapping_mul(8);
            self.emit_bytes(&[0x45, offset])?;
        }
        Ok(())
    }

    fn emit_store_var(&mut self, var_id: u32, src: u32) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: str xN, [x29, #offset] (store to frame pointer + var offset)
            let offset = (var_id as u32) * 8;
            let enc = 0xF90003A0 | (src & 0x1F) | ((offset / 8) << 10);
            self.emit_u32(enc)?;
        } else {
            // x86-64: mov [rbp - offset], rN
            self.emit_bytes(&[0x48, 0x89])?;
            let offset = (var_id as u8).wrapping_mul(8);
            self.emit_bytes(&[0x45, offset])?;
        }
        Ok(())
    }

    fn emit_unary(&mut self, dest: u32, op: &crate::syslib::lang::codegen::ir::IrUnaryOp, operand: u32) -> Result<(), i32> {
        use crate::syslib::lang::codegen::ir::IrUnaryOp;
        if self.arch == 0 {
            match op {
                IrUnaryOp::Neg => {
                    // ARM64: neg xN, xM
                    let enc = 0xCB000000 | (operand << 16) | (0x1F << 10) | dest;
                    self.emit_u32(enc)?;
                }
                IrUnaryOp::Not => {
                    // ARM64: cbz fallback — use EOR with 1 for boolean not
                    // mov xN, #1; eor xN, xN, xM
                    self.emit_u32(0xD2800020 | dest)?;
                    let eor = 0xCA000000 | (operand << 16) | (dest << 5) | dest;
                    self.emit_u32(eor)?;
                }
                IrUnaryOp::BitNot => {
                    // ARM64: mvn xN, xM
                    let enc = 0xAA000000 | (operand << 16) | (0x1F << 10) | dest;
                    self.emit_u32(enc)?;
                }
            }
        } else {
            match op {
                IrUnaryOp::Neg => {
                    // x86-64: neg rax
                    self.emit_bytes(&[0x48, 0xF7, 0xD8])?;
                }
                IrUnaryOp::Not => {
                    // x86-64: xor rax, 1
                    self.emit_bytes(&[0x48, 0x83, 0xF0, 0x01])?;
                }
                IrUnaryOp::BitNot => {
                    // x86-64: not rax
                    self.emit_bytes(&[0x48, 0xF7, 0xD0])?;
                }
            }
        }
        Ok(())
    }

    fn emit_compare(&mut self, dest: u32, op: &crate::syslib::lang::codegen::ir::IrCompareOp, left: u32, right: u32) -> Result<(), i32> {
        use crate::syslib::lang::codegen::ir::IrCompareOp;
        if self.arch == 0 {
            // ARM64: cmp xL, xR then cset xD, cond
            let cmp = 0xEB00001F | (right << 16) | (left << 5);
            self.emit_u32(cmp)?;
            let cond: u32 = match op {
                IrCompareOp::Equal => 0x0,
                IrCompareOp::NotEqual => 0x1,
                IrCompareOp::Less => 0xC,
                IrCompareOp::LessEqual => 0xD,
                IrCompareOp::Greater => 0xA,
                IrCompareOp::GreaterEqual => 0xB,
            };
            // cset xD, cond
            let cset = 0x1A800000 | (cond << 12) | dest;
            self.emit_u32(cset)?;
        } else {
            // x86-64: cmp rax, rbx then setcc al
            self.emit_bytes(&[0x48, 0x39, 0xD8])?;
            let setcc: u8 = match op {
                IrCompareOp::Equal => 0x94,
                IrCompareOp::NotEqual => 0x95,
                IrCompareOp::Less => 0x9C,
                IrCompareOp::LessEqual => 0x9E,
                IrCompareOp::Greater => 0x9F,
                IrCompareOp::GreaterEqual => 0x9D,
            };
            self.emit_bytes(&[0x0F, setcc, 0xC0])?;
        }
        Ok(())
    }

    fn emit_jump_if(&mut self, cond: u32, then_target: u32, else_target: u32) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: cbnz xCond, then_target; b else_target
            let then_off = ((then_target as i64) << 2) as u32;
            let cbnz = 0x35000000 | ((then_off >> 2) & 0x7FFFF) | (cond & 0x1F);
            self.emit_u32(cbnz)?;
            let else_off = ((else_target as i64) << 2) as u32;
            let b_enc = 0x14000000 | ((else_off >> 2) & 0x3FFFFFF);
            self.emit_u32(b_enc)?;
        } else {
            // x86-64: test rCond, rCond; jnz then; jmp else
            self.emit_bytes(&[0x48, 0x85, 0xC0])?;
            self.emit_bytes(&[0x0F, 0x85])?;
            self.emit_u32(then_target)?;
            self.emit_bytes(&[0xE9])?;
            self.emit_u32(else_target)?;
        }
        Ok(())
    }

    fn emit_global(&mut self, id: u32, _name: &str, _ty: &IrType) -> Result<(), i32> {
        // Emit a placeholder for global variable storage (8-byte aligned slot)
        if self.arch == 0 {
            // ARM64: emit 8-byte zero slot, record at offset for symbol resolution
            self.emit_u64(id as u64)?;
        } else {
            // x86-64: dq 0 (8-byte placeholder)
            self.emit_u64(id as u64)?;
        }
        Ok(())
    }

    /// Emit stack allocation instruction
    fn emit_alloca(&mut self, _dest: u32, size: usize, align: usize) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: sub sp, sp, #size (aligned)
            let aligned_size = ((size + align - 1) / align * align) as u32;
            if aligned_size < 0x1000 {
                let enc = 0xD10003FF | ((aligned_size / 4) << 10);
                self.emit_u32(enc)?;
            }
        } else {
            // x86-64: sub rsp, size
            self.emit_bytes(&[0x48, 0x81, 0xEC])?;
            self.emit_u32(size as u32)?;
        }
        Ok(())
    }

    /// Emit memory load instruction
    fn emit_load(&mut self, dest: u32, ptr: u32, offset: usize) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: ldr xD, [xPtr, #offset]
            let enc = 0xF9400000 | (dest & 0x1F) | ((ptr & 0x1F) << 5) | ((offset as u32 / 8) << 10);
            self.emit_u32(enc)?;
        } else {
            // x86-64: mov rD, [rPtr + offset]
            self.emit_bytes(&[0x48, 0x8B])?;
            self.emit_bytes(&[(dest as u8 & 0x7) << 3 | (ptr as u8 & 0x7)])?;
            if offset != 0 {
                self.emit_u32(offset as u32)?;
            }
        }
        Ok(())
    }

    /// Emit memory store instruction
    fn emit_store(&mut self, ptr: u32, offset: usize, src: u32) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: str xSrc, [xPtr, #offset]
            let enc = 0xF9000000 | (src & 0x1F) | ((ptr & 0x1F) << 5) | ((offset as u32 / 8) << 10);
            self.emit_u32(enc)?;
        } else {
            // x86-64: mov [rPtr + offset], rSrc
            self.emit_bytes(&[0x48, 0x89])?;
            self.emit_bytes(&[(src as u8 & 0x7) << 3 | (ptr as u8 & 0x7)])?;
            if offset != 0 {
                self.emit_u32(offset as u32)?;
            }
        }
        Ok(())
    }

    /// Emit field get instruction (load from object at field offset)
    fn emit_get_field(&mut self, dest: u32, object: u32, field_idx: u32) -> Result<(), i32> {
        self.emit_load(dest, object, (field_idx as usize) * 8)
    }

    /// Emit field set instruction (store to object at field offset)
    fn emit_set_field(&mut self, object: u32, field_idx: u32, src: u32) -> Result<(), i32> {
        self.emit_store(object, (field_idx as usize) * 8, src)
    }

    /// Emit array creation instruction
    fn emit_new_array(&mut self, _dest: u32, _elem_type: &IrType, _size: u32) -> Result<(), i32> {
        // Placeholder: runtime call for heap allocation
        if self.arch == 0 {
            // ARM64: bl __nuva_alloc_array
            self.emit_u32(0x94000001)?;
        } else {
            // x86-64: call __nuva_alloc_array
            self.emit_bytes(&[0xE8, 0x01, 0x00, 0x00, 0x00])?;
        }
        Ok(())
    }

    /// Emit array access instruction
    fn emit_array_access(&mut self, dest: u32, array: u32, index: u32) -> Result<(), i32> {
        if self.arch == 0 {
            // ARM64: calculate offset = index * 8, then ldr
            let add = 0x8B000000 | (index << 16) | (array << 5) | dest;
            self.emit_u32(add)?;
        } else {
            // x86-64: lea rD, [rArray + rIndex*8]
            self.emit_bytes(&[0x48, 0x8D, 0x04, 0xD8])?;
            let _ = dest;
        }
        Ok(())
    }

    /// Emit type cast instruction
    fn emit_cast(&mut self, _dest: u32, _src: u32, _target_type: &IrType) -> Result<(), i32> {
        // Casts are often no-op at machine level (same register, different interpretation)
        Ok(())
    }

    /// Emit phi node (resolved during register allocation, emit as move)
    fn emit_phi(&mut self, dest: u32, incoming: &alloc::vec::Vec<(u32, u32)>) -> Result<(), i32> {
        // Phi nodes are resolved by SSA construction; emit a move from first incoming value
        if let Some(&(_block, src)) = incoming.first() {
            if self.arch == 0 {
                // ARM64: mov xD, xSrc
                let enc = 0xAA0003E0 | (src << 16) | dest;
                self.emit_u32(enc)?;
            } else {
                // x86-64: mov rD, rSrc
                self.emit_bytes(&[0x48, 0x89, 0xC0])?;
                let _ = (dest, src);
            }
        }
        Ok(())
    }
}

use crate::syslib::lang::binary::nex::{NexHeader, NEX_MAGIC, NEX_VERSION};

/// encodingtranslateassourcecreateCode
pub fn compile_native(module: &IrModule, arch: u32) -> Result<Vec<u8>, i32> {
    let mut codegen = NativeCodeGen::new(arch);
    codegen.compile_module(module)
}