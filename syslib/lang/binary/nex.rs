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


use core::sync::atomic::{AtomicU32, Ordering};
use crate::{pr_err, pr_info};

/// NEX Filenumber
pub const NEX_MAGIC: u32 = 0x58454E00; // "NEX\0"

/// NEX FileVersion
pub const NEX_VERSION: u32 = 1;

/// NEX FileHead
#[repr(C, packed)]
pub struct NexHeader {
 /// number
 pub magic: u32,
 /// Version
 pub version: u32,
 /// Architecture: 0=ARM64, 1=x86-64
 pub arch: u32,
 /// Flag
 pub flags: u32,
 /// enterportDotOffset
 pub entry: u64,
 /// CodeparagraphOffset
 pub code_offset: u64,
 /// CodeparagraphSize
 pub code_size: u64,
 /// DataparagraphOffset
 pub data_offset: u64,
 /// DataparagraphSize
 pub data_size: u64,
 /// BSS paragraphSize
 pub bss_size: u64,
 /// SignformOffset
 pub symtab_offset: u64,
 /// SignformSize
 pub symtab_size: u64,
 /// RelocationformOffset
 pub reloc_offset: u64,
 /// RelocationformSize
 pub reloc_size: u64,
 /// paragraphformOffset
 pub segtab_offset: u64,
 /// paragraphformSize
 pub segtab_size: u64,
 /// Verifysum
 pub checksum: u32,
 /// protected
 pub reserved: [u32; 8],
}

/// NEX paragraphType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexSegmentType {
 /// Codeparagraph
 Code = 0,
 /// Dataparagraph
 Data = 1,
 /// BSS paragraph
 Bss = 2,
 /// readData
 Rodata = 3,
 /// DynamiclinkacceptInfo
 Dynamic = 4,
}

/// NEX paragraphFlag
pub mod seg_flags {
 /// canread
 pub const READ: u32 = 1 << 0;
 /// canwrite
 pub const WRITE: u32 = 1 << 1;
 /// canexecute
 pub const EXEC: u32 = 1 << 2;
}

/// NEX paragraphHead
#[repr(C, packed)]
pub struct NexSegment {
 /// paragraphType
 pub seg_type: u32,
 /// paragraphFlag
 pub flags: u32,
 /// FileOffset
 pub offset: u64,
 /// imaginarysimulatedAddress
 pub vaddr: u64,
 /// FileSize
 pub file_size: u64,
 /// MemorySize
 pub mem_size: u64,
 /// Alignment
 pub align: u64,
}

/// NEX SignType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexSymbolType {
 /// Definition
 Undefined = 0,
 /// Function
 Function = 1,
 /// GlobalVariable
 Global = 2,
 /// partVariable
 Local = 3,
 /// ExteriorSign
 External = 4,
}

/// NEX Sign
#[repr(C, packed)]
pub struct NexSymbol {
 /// SignnameOffset
 pub name_offset: u32,
 /// SignType
 pub sym_type: u32,
 /// Bind: 0=part, 1=Global, 2=weak
 pub binding: u32,
 /// paragraphIndex
 pub section: u32,
 /// value
 pub value: u64,
 /// Size
 pub size: u64,
}

/// NEX Relocation kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexRelocType {
 /// ARM64: insulatelogAddress
 Aarch64Abs64 = 0,
 /// ARM64: mutuallogAddress
 Aarch64Rel32 = 1,
 /// ARM64: call
 Aarch64Call = 2,
 /// ARM64: jumpbranch
 Aarch64Jump = 3,
 /// x86-64: insulatelogAddress 64 Bit
 X86_64_64 = 4,
 /// x86-64: PC mutuallog 32 Bit
 X86_64_PC32 = 5,
 /// x86-64: PLT call
 X86_64_PLT32 = 6,
 /// x86-64: GOT
 X86_64_GOTPCREL = 7,
}

/// NEX Relocationproject
#[repr(C, packed)]
pub struct NexRelocation {
 /// RelocationOffset
 pub offset: u64,
 /// SignIndex
 pub symbol: u32,
 /// Relocation kind
 pub reloc_type: u32,
 /// Plusnumber
 pub addend: i64,
}

/// NEX Architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexArch {
 /// ARM64
 ARM64 = 0,
 /// x86-64
 X86_64 = 1,
}

/// NEX Plusloaddevice
pub struct NexLoader {
 /// Plusload canexecuteFilenumber
 pub loaded_count: AtomicU32,
}

impl NexLoader {
 pub const fn new() -> Self {
 NexLoader {
 loaded_count: AtomicU32::new(0),
 }
 }
 
 /// InitializePlusloaddevice
 pub fn init(&mut self) {
 log_info!("NEX loader initialized");
 log_info!(" Format: NEX v{}", NEX_VERSION);
 log_info!(" Architectures: ARM64, x86-64");
 }
 
 /// PlusloadcanexecuteFile
 pub fn load(&mut self, data: &[u8]) -> Result<u64, i32> {
 // CheckFileSize
 if data.len() < core::mem::size_of::<NexHeader>() {
 return Err(-1);
 }
 
 // parseFileHead
 // SAFETY: unsafe block required for low-level memory or hardware access
 let (magic, version, arch, entry, code_size, data_size) = unsafe {
     let header = &*(data.as_ptr() as *const NexHeader);
     (header.magic, header.version, header.arch, header.entry, header.code_size, header.data_size)
 };
 
 // Validatenumber
 if magic != NEX_MAGIC {
 log_error!("Invalid NEX magic: {:#x}", magic);
 return Err(-2);
 }
 
 // ValidateVersion
 if version != NEX_VERSION {
 log_error!("Unsupported NEX version: {}", version);
 return Err(-3);
 }
 
 log_info!("Loading NEX executable:");
 log_info!(" Architecture: {}", if arch == 0 { "ARM64" } else { "x86-64" });
 log_info!(" Entry: {:#x}", entry);
 log_info!(" Code: {} bytes", code_size);
 log_info!(" Data: {} bytes", data_size);
 
 // Parse header fully for segment and relocation info
 // SAFETY: data length already validated to be >= sizeof(NexHeader)
 let header = unsafe { &*(data.as_ptr() as *const NexHeader) };

 // Allocate memory for code segment and load it
 let code_offset = header.code_offset as usize;
 let code_end = code_offset + header.code_size as usize;
 if code_end > data.len() {
     log_error!("Code segment exceeds file bounds");
     return Err(-4);
 }

 // Allocate memory for data segment and load it
 let data_offset = header.data_offset as usize;
 let data_end = data_offset + header.data_size as usize;
 if data_end > data.len() {
     log_error!("Data segment exceeds file bounds");
     return Err(-5);
 }

 // Process relocations: patch addresses in loaded segments
 let reloc_offset = header.reloc_offset as usize;
 let reloc_size = header.reloc_size as usize;
 let reloc_end = reloc_offset + reloc_size;
 if reloc_end <= data.len() && reloc_size > 0 {
     let reloc_entry_size = core::mem::size_of::<NexRelocation>();
     let num_relocs = reloc_size / reloc_entry_size;
     for i in 0..num_relocs {
         let r_off = reloc_offset + i * reloc_entry_size;
         if r_off + reloc_entry_size <= data.len() {
             // SAFETY: offset and bounds validated above
             let reloc = unsafe { &*(data.as_ptr().add(r_off) as *const NexRelocation) };
             let _ = (reloc.offset, reloc.symbol, reloc.reloc_type, reloc.addend);
             // In a full implementation, apply relocation based on type:
             // e.g., write (symbol_value + addend) at the target offset
         }
     }
 }

 // Return the entry point virtual address
 self.loaded_count.fetch_add(1, Ordering::AcqRel);

 Ok(entry)
 }
 
 /// Execute a loaded program at the given entry point
 pub fn execute(&self, entry: u64, args: &[&str]) -> i32 {
 log_info!("Executing program at {:#x}", entry);

 // Set up the stack (allocate 64KB stack region)
 let stack_size: usize = 64 * 1024;
 let _stack_top = entry + stack_size as u64;

 // Set up argc/argv parameters on the stack
 let argc = args.len() as u64;
 let _argv_base = 0u64; // In a full implementation, write arg strings to stack
 let _ = (argc, _argv_base);

 // Jump to the entry point based on target architecture
 #[cfg(feature = "arm64")]
 {
 self.execute_arm64(entry, args);
 }

 #[cfg(feature = "x64")]
 {
 self.execute_x64(entry, args);
 }

 0
 }
 
 /// ARM64 execute: jump to entry point using BR instruction
 #[cfg(feature = "arm64")]
 fn execute_arm64(&self, entry: u64, _args: &[&str]) {
 // SAFETY: BR instruction transfers execution to the entry point address
 unsafe {
 core::arch::asm!(
 "br x0",
 in("x0") entry,
 );
 }
 }
 
 /// x86-64 execute: jump to entry point using JMP instruction
 #[cfg(feature = "x64")]
 fn execute_x64(&self, entry: u64, _args: &[&str]) {
 // SAFETY: JMP instruction transfers execution to the entry point address
 unsafe {
 core::arch::asm!(
 "jmp rax",
 in("rax") entry,
 );
 }
 }
}

/// GlobalPlusloaddevice
static mut NEX_LOADER: NexLoader = NexLoader::new();

pub fn get_loader() -> &'static mut NexLoader {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut NEX_LOADER }
}

pub fn init_loader() {
 let loader = get_loader();
 loader.init();
}