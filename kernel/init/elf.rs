use crate::{pr_info};
/*
 * Nuva OS - Kernel - ELF Loader
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * ELF executable and shared object loader.
 */

use core::sync::atomic::{AtomicU32, Ordering};

/// ELF Magic
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// ELF Class
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfClass {
    None = 0,
    /// 32-bit
    Class32 = 1,
    /// 64-bit
    Class64 = 2,
}

/// ELF Data Encoding
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfData {
    None = 0,
    /// Little endian
    Lsb = 1,
    /// Big endian
    Msb = 2,
}

/// ELF Type
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfType {
    None = 0,
    /// Relocatable
    Rel = 1,
    /// Executable
    Exec = 2,
    /// Shared object
    Dyn = 3,
    /// Core dump
    Core = 4,
}

/// ELF Machine
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfMachine {
    None = 0,
    /// x86
    I386 = 3,
    /// ARM
    Arm = 40,
    /// x86-64
    X86_64 = 62,
    /// AArch64
    AArch64 = 183,
}

/// ELF Header (64-bit)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64Ehdr {
    /// Magic number
    pub e_ident: [u8; 16],
    /// Object file type
    pub e_type: ElfType,
    /// Architecture
    pub e_machine: ElfMachine,
    /// Object file version
    pub e_version: u32,
    /// Entry point address
    pub e_entry: u64,
    /// Program header table file offset
    pub e_phoff: u64,
    /// Section header table file offset
    pub e_shoff: u64,
    /// Processor-specific flags
    pub e_flags: u32,
    /// ELF header size
    pub e_ehsize: u16,
    /// Program header table entry size
    pub e_phentsize: u16,
    /// Program header table entry count
    pub e_phnum: u16,
    /// Section header table entry size
    pub e_shentsize: u16,
    /// Section header table entry count
    pub e_shnum: u16,
    /// Section header string table index
    pub e_shstrndx: u16,
}

/// Program Header Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfPhType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interp = 3,
    Note = 4,
    Shlib = 5,
    Phdr = 6,
    Tls = 7,
    GnuEhFrame = 0x6474E550,
    GnuStack = 0x6474E551,
    GnuRelro = 0x6474E552,
}

/// Program Header Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ElfPhFlags: u32 {
        const Executable = 1;
        const Writable = 2;
        const Readable = 4;
    }
}

impl Clone for ElfPhFlags {
    fn clone(&self) -> Self { *self }
}
impl Copy for ElfPhFlags {}

/// Program Header (64-bit)
#[repr(C)]
pub struct Elf64Phdr {
    /// Segment type
    pub p_type: ElfPhType,
    /// Segment flags
    pub p_flags: ElfPhFlags,
    /// Segment file offset
    pub p_offset: u64,
    /// Segment virtual address
    pub p_vaddr: u64,
    /// Segment physical address
    pub p_paddr: u64,
    /// Segment size in file
    pub p_filesz: u64,
    /// Segment size in memory
    pub p_memsz: u64,
    /// Segment alignment
    pub p_align: u64,
}

/// Section Header Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfShType {
    Null = 0,
    ProgBits = 1,
    SymTab = 2,
    StrTab = 3,
    Rela = 4,
    Hash = 5,
    Dynamic = 6,
    Note = 7,
    NoBits = 8,
    Rel = 9,
    ShLib = 10,
    DynSym = 11,
    InitArray = 14,
    FiniArray = 15,
    PreInitArray = 16,
    Group = 17,
    Tls = 18,
}

/// Section Header Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ElfShFlags: u64 {
        const Write = 1;
        const Alloc = 2;
        const ExecInstr = 4;
        const MaskProc = 0xF0000000;
        const MaskOs = 0x0FF00000;
    }
}

/// Section Header (64-bit)
#[repr(C)]
pub struct Elf64Shdr {
    /// Section name
    pub sh_name: u32,
    /// Section type
    pub sh_type: ElfShType,
    /// Section flags
    pub sh_flags: ElfShFlags,
    /// Section virtual address
    pub sh_addr: u64,
    /// Section file offset
    pub sh_offset: u64,
    /// Section size
    pub sh_size: u64,
    /// Section link
    pub sh_link: u32,
    /// Section info
    pub sh_info: u32,
    /// Section alignment
    pub sh_addralign: u64,
    /// Section entry size
    pub sh_entsize: u64,
}

/// Symbol Binding
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfSymBind {
    Local = 0,
    Global = 1,
    Weak = 2,
}

/// Symbol Type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfSymType {
    NoType = 0,
    Object = 1,
    Func = 2,
    Section = 3,
    File = 4,
    Common = 5,
    Tls = 6,
}

/// Symbol (64-bit)
#[repr(C)]
pub struct Elf64Sym {
    /// Symbol name
    pub st_name: u32,
    /// Symbol info (binding and type)
    pub st_info: u8,
    /// Other
    pub st_other: u8,
    /// Section index
    pub st_shndx: u16,
    /// Symbol value
    pub st_value: u64,
    /// Symbol size
    pub size: u64,
}

/// Relocation (64-bit)
#[repr(C)]
pub struct Elf64Rel {
    /// Address where to apply relocation
    pub r_offset: u64,
    /// Relocation type and symbol index
    pub r_info: u64,
}

/// Relocation with addend (64-bit)
#[repr(C)]
pub struct Elf64Rela {
    /// Address where to apply relocation
    pub r_offset: u64,
    /// Relocation type and symbol index
    pub r_info: u64,
    /// Addend
    pub r_addend: i64,
}

/// Dynamic Entry (64-bit)
#[repr(C)]
pub struct Elf64Dyn {
    pub d_tag: i64,
    pub d_val: u64,
}

/// ELF Loader
pub struct ElfLoader {
    /// Loaded segments
    pub segments: [Option<LoadedSegment>; 16],
    /// Segment count
    pub segment_count: AtomicU32,
    /// Entry point
    pub entry: u64,
    /// Base address
    pub base: u64,
    /// End address
    pub end: u64,
    /// Interpreter path
    pub interp: [u8; 256],
    /// Has interpreter
    pub has_interp: bool,
}

/// Loaded Segment
#[derive(Clone)]
pub struct LoadedSegment {
    /// Virtual address
    pub vaddr: u64,
    /// Size
    pub size: u64,
    /// Flags
    pub flags: ElfPhFlags,
    /// Data pointer
    pub data: *mut u8,
}

impl ElfLoader {
    pub const fn new() -> Self {
        ElfLoader {
            segments: [const { None }; 16],
            segment_count: AtomicU32::new(0),
            entry: 0,
            base: u64::MAX,
            end: 0,
            interp: [0; 256],
            has_interp: false,
        }
    }
    
    /// Load ELF from memory
    pub fn load(&mut self, data: &[u8]) -> Result<u64, i32> {
        // Check minimum size
        if data.len() < core::mem::size_of::<Elf64Ehdr>() {
            return Err(-22); // EINVAL
        }
        
        // Parse header
        let header = self.parse_header(data)?;
        
        // Validate header
        self.validate_header(&header)?;
        
        // Load program segments
        self.load_segments(data, &header)?;
        
        // Set entry point
        self.entry = header.e_entry as u64;
        
        Ok(self.entry)
    }
    
    /// Parse ELF header
    fn parse_header(&self, data: &[u8]) -> Result<Elf64Ehdr, i32> {
        // Check magic
        if data[0..4] != ELF_MAGIC {
            return Err(-22);
        }
        
        // Parse header
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let header = &*(data.as_ptr() as *const Elf64Ehdr);
            Ok(*header)
        }
    }
    
    /// Validate ELF header
    fn validate_header(&self, header: &Elf64Ehdr) -> Result<(), i32> {
        // Check class (must be 64-bit)
        if header.e_ident[4] != ElfClass::Class64 as u8 {
            return Err(-22);
        }
        
        // Check data encoding (must be little endian)
        if header.e_ident[5] != ElfData::Lsb as u8 {
            return Err(-22);
        }
        
        // Check version
        if header.e_version != 1 {
            return Err(-22);
        }
        
        // Check machine
        match header.e_machine {
            ElfMachine::X86_64 | ElfMachine::AArch64 => {}
            _ => return Err(-22),
        }
        
        // Check type
        match header.e_type {
            ElfType::Exec | ElfType::Dyn => {}
            _ => return Err(-22),
        }
        
        Ok(())
    }
    
    /// Load program segments
    fn load_segments(&mut self, data: &[u8], header: &Elf64Ehdr) -> Result<(), i32> {
        let phoff = header.e_phoff as usize;
        let phentsize = header.e_phentsize as usize;
        let phnum = header.e_phnum as usize;
        
        for i in 0..phnum {
            let offset = phoff + i * phentsize;
            
            if offset + phentsize > data.len() {
                continue;
            }
            
            // SAFETY: unsafe block required for low-level memory or hardware access
            let phdr = unsafe {
                &*(data.as_ptr().add(offset) as *const Elf64Phdr)
            };
            
            self.load_segment(data, phdr)?;
        }
        
        Ok(())
    }
    
    /// Load single segment
    fn load_segment(&mut self, data: &[u8], phdr: &Elf64Phdr) -> Result<(), i32> {
        match phdr.p_type {
            ElfPhType::Load => {
                self.load_load_segment(data, phdr)?;
            }
            ElfPhType::Interp => {
                self.load_interp_segment(data, phdr)?;
            }
            ElfPhType::Tls => {
                // TODO: Handle TLS
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Load PT_LOAD segment
    fn load_load_segment(&mut self, data: &[u8], phdr: &Elf64Phdr) -> Result<(), i32> {
        let vaddr = phdr.p_vaddr;
        let memsz = phdr.p_memsz;
        let filesz = phdr.p_filesz;
        let offset = phdr.p_offset as usize;
        
        // Allocate memory for segment
        // TODO: Use proper memory allocation
        let mem = self.allocate_memory(vaddr, memsz)?;
        
        // Zero memory
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_bytes(mem, 0, memsz as usize);
        }
        
        // Copy file data
        if filesz > 0 && offset + filesz as usize <= data.len() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(offset),
                    mem,
                    filesz as usize,
                );
            }
        }
        
        // Update base and end
        if vaddr < self.base {
            self.base = vaddr;
        }
        if vaddr + memsz > self.end {
            self.end = vaddr + memsz;
        }
        
        // Store segment info
        let idx = self.segment_count.load(Ordering::Acquire) as usize;
        if idx < 16 {
            self.segments[idx] = Some(LoadedSegment {
                vaddr,
                size: memsz,
                flags: phdr.p_flags,
                data: mem,
            });
            self.segment_count.fetch_add(1, Ordering::AcqRel);
        }
        
        Ok(())
    }
    
    /// Load PT_INTERP segment
    fn load_interp_segment(&mut self, data: &[u8], phdr: &Elf64Phdr) -> Result<(), i32> {
        let offset = phdr.p_offset as usize;
        let filesz = phdr.p_filesz as usize;
        
        if offset + filesz > data.len() {
            return Err(-22);
        }
        
        // Copy interpreter path
        let len = filesz.min(255);
        self.interp[..len].copy_from_slice(&data[offset..offset + len]);
        self.interp[len] = 0;
        self.has_interp = true;
        
        Ok(())
    }
    
    /// Allocate memory
    fn allocate_memory(&self, vaddr: u64, size: u64) -> Result<*mut u8, i32> {
        // TODO: Use proper memory allocation through MMU
        // For now, return a placeholder
        let _ = (vaddr, size);
        Err(-12) // ENOMEM
    }
    
    /// Get entry point
    pub fn entry_point(&self) -> u64 {
        self.entry
    }
    
    /// Get memory range
    pub fn memory_range(&self) -> (u64, u64) {
        (self.base, self.end)
    }
    
    /// Get interpreter
    pub fn interpreter(&self) -> Option<&[u8]> {
        if !self.has_interp {
            return None;
        }
        
        let len = self.interp.iter().position(|&c| c == 0).unwrap_or(255);
        Some(&self.interp[..len])
    }
}

/// ELF Symbol Table
pub struct ElfSymbolTable {
    /// Symbols
    pub symbols: *mut Elf64Sym,
    /// Symbol count
    pub count: u32,
    /// String table
    pub strtab: *const u8,
    /// String table size
    pub strtab_size: u32,
}

impl ElfSymbolTable {
    pub fn new() -> Self {
        ElfSymbolTable {
            symbols: core::ptr::null_mut(),
            count: 0,
            strtab: core::ptr::null(),
            strtab_size: 0,
        }
    }
    
    /// Find symbol by name
    pub fn find(&self, name: &[u8]) -> Option<&Elf64Sym> {
        if self.symbols.is_null() || self.strtab.is_null() {
            return None;
        }
        
        for i in 0..self.count {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let sym = &*self.symbols.add(i as usize);
                let sym_name = self.get_name(sym.st_name);
                
                if sym_name == name {
                    return Some(sym);
                }
            }
        }
        
        None
    }
    
    /// Get symbol name
    fn get_name(&self, offset: u32) -> &[u8] {
        if self.strtab.is_null() || offset >= self.strtab_size {
            return b"";
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let start = self.strtab.add(offset as usize);
            let len = (0..self.strtab_size as usize - offset as usize)
                .position(|i| *start.add(i) == 0)
                .unwrap_or(0);
            core::slice::from_raw_parts(start, len)
        }
    }
}

/// Global ELF Loader
static ELF_LOADER: core::sync::OnceLock<ElfLoader> = core::sync::OnceLock::new();

/// Get ELF loader
pub fn elf_loader() -> &'static ElfLoader {
    ELF_LOADER.get_or_init(ElfLoader::new)
}

/// Initialize ELF loader
pub fn init_elf() {
    log_info!("ELF loader initialized");
}

// Convenience functions

/// Load ELF from memory
pub fn load_elf(data: &[u8]) -> Result<u64, i32> {
    get_elf_loader().load(data)
}

/// Check if data is valid ELF
pub fn is_valid_elf(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    data[0..4] == ELF_MAGIC
}

/// Get ELF class
pub fn get_elf_class(data: &[u8]) -> Option<ElfClass> {
    if data.len() < 5 {
        return None;
    }
    match data[4] {
        1 => Some(ElfClass::Class32),
        2 => Some(ElfClass::Class64),
        _ => None,
    }
}

/// Get ELF machine
pub fn get_elf_machine(data: &[u8]) -> Option<ElfMachine> {
    if data.len() < core::mem::size_of::<Elf64Ehdr>() {
        return None;
    }
    
    // SAFETY: unsafe block required for low-level memory or hardware access
    let header = unsafe { &*(data.as_ptr() as *const Elf64Ehdr) };
    Some(header.e_machine)
}
