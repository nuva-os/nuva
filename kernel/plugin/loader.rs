/*
 * Nuva OS - Kernel - Plugin - Loader
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
/*
 * Plugin Loader - Dynamic Loading
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module handles dynamic loading of plugins from
 * ELF binaries in kernel mode. It implements a minimal
 * ELF parser for loading kernel plugins without relying
 * on user-space dlopen/LoadLibrary.
 */

use alloc::alloc::{alloc, dealloc, Layout};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use super::core::{Plugin, PluginContext, PluginError, PluginMeta, PluginState};

// ============================================================================
// ELF Constants
// ============================================================================

const EI_MAG0: u8 = 0x7F;
const EI_MAG1: u8 = b'E';
const EI_MAG2: u8 = b'L';
const EI_MAG3: u8 = b'F';

const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const ET_DYN: u16 = 3;
const EM_X86_64: u16 = 62;
const EM_AARCH64: u16 = 183;
const EM_LOONGARCH: u16 = 258;

const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;

const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;

const R_X86_64_RELATIVE: u32 = 8;
const R_AARCH64_RELATIVE: u32 = 0x403;
const R_LARCH_RELATIVE: u32 = 83;

const STB_GLOBAL: u8 = 1;
const STB_WEAK: u8 = 2;

// ============================================================================
// ELF Header Structures (64-bit)
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Sym {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: u64,
    size: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Rela {
    r_offset: u64,
    r_info: u64,
    r_addend: i64,
}

// ============================================================================
// ELF Parser
// ============================================================================

#[derive(Debug, Clone)]
pub struct ElfParseError {
    pub message: String,
}

impl ElfParseError {
    fn new(msg: &str) -> Self {
        Self {
            message: String::from(msg),
        }
    }
}

impl fmt::Display for ElfParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ELF parse error: {}", self.message)
    }
}

/// Parsed ELF image containing loaded segments and symbol information
#[derive(Debug, Clone)]
pub struct ParsedElf {
    /// Load base address (for PIE/shared objects)
    pub load_base: u64,
    /// Entry point address (adjusted for load base)
    pub entry_point: u64,
    /// Loaded segment information
    pub segments: Vec<LoadedSegment>,
    /// Exported symbol names
    pub symbols: Vec<String>,
    /// Raw ELF data reference (owned copy for safety)
    pub raw_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct LoadedSegment {
    /// Virtual address of this segment
    pub vaddr: u64,
    /// Size in memory (may be larger than file size for BSS)
    pub memsz: u64,
    /// Offset in raw_data for file content
    pub file_offset: u64,
    /// Size of file content
    pub filesz: u64,
    /// Segment flags (PF_R=4, PF_W=2, PF_X=1)
    pub flags: u32,
}

/// Minimal ELF parser for kernel plugin loading
pub struct ElfParser;

impl ElfParser {
    /// Parse an ELF binary image and return structured information
    pub fn parse(data: &[u8]) -> Result<ParsedElf, ElfParseError> {
        if data.len() < core::mem::size_of::<Elf64Ehdr>() {
            return Err(ElfParseError::new("Data too small for ELF header"));
        }

        let ehdr = Self::read_ehdr(data)?;

        Self::validate_header(&ehdr)?;

        let segments = Self::parse_load_segments(data, &ehdr)?;

        let load_base = if !segments.is_empty() {
            segments[0].vaddr
        } else {
            0
        };

        let entry_point = if ehdr.e_entry >= load_base {
            ehdr.e_entry
        } else {
            load_base + ehdr.e_entry
        };

        let symbols = Self::parse_symbols(data, &ehdr)?;

        Ok(ParsedElf {
            load_base,
            entry_point,
            segments,
            symbols,
            raw_data: Vec::from(data),
        })
    }

    fn read_ehdr(data: &[u8]) -> Result<Elf64Ehdr, ElfParseError> {
        // SAFETY: We verified data is large enough for Elf64Ehdr.
        // We read via byte copy to avoid alignment issues.
        let mut ehdr = Elf64Ehdr {
            e_ident: [0; 16],
            e_type: 0,
            e_machine: 0,
            e_version: 0,
            e_entry: 0,
            e_phoff: 0,
            e_shoff: 0,
            e_flags: 0,
            e_ehsize: 0,
            e_phentsize: 0,
            e_phnum: 0,
            e_shentsize: 0,
            e_shnum: 0,
            e_shstrndx: 0,
        };

        let src = &data[..core::mem::size_of::<Elf64Ehdr>()];
        let dst = core::slice::from_raw_parts_mut(
            &mut ehdr as *mut Elf64Ehdr as *mut u8,
            core::mem::size_of::<Elf64Ehdr>(),
        );
        dst.copy_from_slice(src);

        Ok(ehdr)
    }

    fn validate_header(ehdr: &Elf64Ehdr) -> Result<(), ElfParseError> {
        if ehdr.e_ident[0] != EI_MAG0
            || ehdr.e_ident[1] != EI_MAG1
            || ehdr.e_ident[2] != EI_MAG2
            || ehdr.e_ident[3] != EI_MAG3
        {
            return Err(ElfParseError::new("Invalid ELF magic"));
        }

        if ehdr.e_ident[4] != ELFCLASS64 {
            return Err(ElfParseError::new("Not a 64-bit ELF"));
        }

        if ehdr.e_ident[5] != ELFDATA2LSB {
            return Err(ElfParseError::new("Not little-endian"));
        }

        if ehdr.e_type != ET_DYN {
            return Err(ElfParseError::new(
                "Not a shared object (ET_DYN required for plugins)",
            ));
        }

        let valid_machine = ehdr.e_machine == EM_X86_64
            || ehdr.e_machine == EM_AARCH64
            || ehdr.e_machine == EM_LOONGARCH;
        if !valid_machine {
            return Err(ElfParseError::new("Unsupported machine type"));
        }

        Ok(())
    }

    fn parse_load_segments(
        data: &[u8],
        ehdr: &Elf64Ehdr,
    ) -> Result<Vec<LoadedSegment>, ElfParseError> {
        let mut segments = Vec::new();

        if ehdr.e_phoff == 0 || ehdr.e_phnum == 0 {
            return Ok(segments);
        }

        let phentsize = ehdr.e_phentsize as usize;
        if phentsize < core::mem::size_of::<Elf64Phdr>() {
            return Err(ElfParseError::new("Program header entry size too small"));
        }

        for i in 0..ehdr.e_phnum as usize {
            let offset = ehdr.e_phoff as usize + i * phentsize;
            let end = offset + core::mem::size_of::<Elf64Phdr>();
            if end > data.len() {
                break;
            }

            let phdr = Self::read_phdr(&data[offset..end]);

            if phdr.p_type == PT_LOAD {
                segments.push(LoadedSegment {
                    vaddr: phdr.p_vaddr,
                    memsz: phdr.p_memsz,
                    file_offset: phdr.p_offset,
                    filesz: phdr.p_filesz,
                    flags: phdr.p_flags,
                });
            }
        }

        Ok(segments)
    }

    fn read_phdr(data: &[u8]) -> Elf64Phdr {
        // SAFETY: Caller ensures data is exactly size_of::<Elf64Phdr>().
        let mut phdr = Elf64Phdr {
            p_type: 0,
            p_flags: 0,
            p_offset: 0,
            p_vaddr: 0,
            p_paddr: 0,
            p_filesz: 0,
            p_memsz: 0,
            p_align: 0,
        };
        let dst = core::slice::from_raw_parts_mut(
            &mut phdr as *mut Elf64Phdr as *mut u8,
            core::mem::size_of::<Elf64Phdr>(),
        );
        dst.copy_from_slice(&data[..core::mem::size_of::<Elf64Phdr>()]);
        phdr
    }

    fn parse_symbols(data: &[u8], ehdr: &Elf64Ehdr) -> Result<Vec<String>, ElfParseError> {
        let mut symbols = Vec::new();

        if ehdr.e_shoff == 0 || ehdr.e_shnum == 0 {
            return Ok(symbols);
        }

        let shentsize = ehdr.e_shentsize as usize;
        if shentsize < core::mem::size_of::<Elf64Shdr>() {
            return Ok(symbols);
        }

        let mut symtab_offset: u64 = 0;
        let mut symtab_size: u64 = 0;
        let mut strtab_offset: u64 = 0;
        let mut strtab_size: u64 = 0;

        for i in 0..ehdr.e_shnum as usize {
            let offset = ehdr.e_shoff as usize + i * shentsize;
            let end = offset + core::mem::size_of::<Elf64Shdr>();
            if end > data.len() {
                break;
            }

            let shdr = Self::read_shdr(&data[offset..end]);

            if shdr.sh_type == SHT_SYMTAB {
                symtab_offset = shdr.sh_offset;
                symtab_size = shdr.sh_size;
                let link = shdr.sh_link as usize;
                if link < ehdr.e_shnum as usize {
                    let str_offset = ehdr.e_shoff as usize + link * shentsize;
                    let str_end = str_offset + core::mem::size_of::<Elf64Shdr>();
                    if str_end <= data.len() {
                        let str_shdr = Self::read_shdr(&data[str_offset..str_end]);
                        strtab_offset = str_shdr.sh_offset;
                        strtab_size = str_shdr.sh_size;
                    }
                }
            }
        }

        if symtab_offset == 0 || strtab_offset == 0 {
            return Ok(symbols);
        }

        let sym_entry_size = core::mem::size_of::<Elf64Sym>();
        let num_syms = symtab_size as usize / sym_entry_size;

        for i in 0..num_syms {
            let offset = symtab_offset as usize + i * sym_entry_size;
            let end = offset + sym_entry_size;
            if end > data.len() {
                break;
            }

            let sym = Self::read_sym(&data[offset..end]);

            let binding = sym.st_info >> 4;
            if binding != STB_GLOBAL && binding != STB_WEAK {
                continue;
            }
            if sym.st_shndx == 0 {
                continue;
            }

            let name_offset = strtab_offset as usize + sym.st_name as usize;
            if name_offset >= data.len() {
                continue;
            }

            let name = Self::read_cstring(data, name_offset);
            if !name.is_empty() {
                symbols.push(name);
            }
        }

        Ok(symbols)
    }

    fn read_shdr(data: &[u8]) -> Elf64Shdr {
        let mut shdr = Elf64Shdr {
            sh_name: 0,
            sh_type: 0,
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: 0,
            sh_size: 0,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 0,
            sh_entsize: 0,
        };
        let dst = core::slice::from_raw_parts_mut(
            &mut shdr as *mut Elf64Shdr as *mut u8,
            core::mem::size_of::<Elf64Shdr>(),
        );
        dst.copy_from_slice(&data[..core::mem::size_of::<Elf64Shdr>()]);
        shdr
    }

    fn read_sym(data: &[u8]) -> Elf64Sym {
        let mut sym = Elf64Sym {
            st_name: 0,
            st_info: 0,
            st_other: 0,
            st_shndx: 0,
            st_value: 0,
            size: 0,
        };
        let dst = core::slice::from_raw_parts_mut(
            &mut sym as *mut Elf64Sym as *mut u8,
            core::mem::size_of::<Elf64Sym>(),
        );
        dst.copy_from_slice(&data[..core::mem::size_of::<Elf64Sym>()]);
        sym
    }

    fn read_cstring(data: &[u8], offset: usize) -> String {
        if offset >= data.len() {
            return String::new();
        }
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        let slice = &data[offset..end];
        match core::str::from_utf8(slice) {
            Ok(s) => String::from(s),
            Err(_) => String::new(),
        }
    }

    /// Apply RELA relocations to the loaded image.
    /// base_addr is the virtual address where the ELF was loaded.
    /// image is the mutable slice of the loaded image.
    pub fn apply_relocations(
        base_addr: u64,
        image: &mut [u8],
        ehdr: &Elf64Ehdr,
        data: &[u8],
    ) -> Result<(), ElfParseError> {
        let shentsize = ehdr.e_shentsize as usize;
        if shentsize < core::mem::size_of::<Elf64Shdr>() {
            return Ok(());
        }

        let rela_entry_size = core::mem::size_of::<Elf64Rela>();

        for i in 0..ehdr.e_shnum as usize {
            let offset = ehdr.e_shoff as usize + i * shentsize;
            let end = offset + core::mem::size_of::<Elf64Shdr>();
            if end > data.len() {
                break;
            }

            let shdr = Self::read_shdr(&data[offset..end]);

            if shdr.sh_type != SHT_RELA {
                continue;
            }

            let num_relas = shdr.sh_size as usize / rela_entry_size;

            for j in 0..num_relas {
                let rela_offset = shdr.sh_offset as usize + j * rela_entry_size;
                let rela_end = rela_offset + rela_entry_size;
                if rela_end > data.len() {
                    break;
                }

                let rela = Self::read_rela(&data[rela_offset..rela_end]);

                let r_type = rela.r_info as u32;
                let is_relative = r_type == R_X86_64_RELATIVE
                    || r_type == R_AARCH64_RELATIVE
                    || r_type == R_LARCH_RELATIVE;

                if !is_relative {
                    continue;
                }

                let target_offset = if rela.r_offset >= base_addr {
                    (rela.r_offset - base_addr) as usize
                } else {
                    rela.r_offset as usize
                };

                if target_offset + 8 > image.len() {
                    continue;
                }

                let value = (base_addr as i64 + rela.r_addend) as u64;
                let bytes = value.to_le_bytes();
                image[target_offset..target_offset + 8].copy_from_slice(&bytes);
            }
        }

        Ok(())
    }

    fn read_rela(data: &[u8]) -> Elf64Rela {
        let mut rela = Elf64Rela {
            r_offset: 0,
            r_info: 0,
            r_addend: 0,
        };
        let dst = core::slice::from_raw_parts_mut(
            &mut rela as *mut Elf64Rela as *mut u8,
            core::mem::size_of::<Elf64Rela>(),
        );
        dst.copy_from_slice(&data[..core::mem::size_of::<Elf64Rela>()]);
        rela
    }

    /// Look up a symbol by name in the parsed ELF
    pub fn find_symbol(elf: &ParsedElf, name: &str) -> Option<u64> {
        for sym_name in &elf.symbols {
            if sym_name.as_str() == name {
                return Some(elf.entry_point);
            }
        }
        None
    }
}

// ============================================================================
// Plugin Loader
// ============================================================================

/// Plugin loader
/// Handles loading plugins from ELF binaries in kernel mode.
pub struct PluginLoader {
    /// Loaded plugin images
    loaded_images: Vec<LoadedImage>,

    /// Plugin entry point name
    entry_point: String,

    /// Loader configuration
    config: LoaderConfig,
}

/// A loaded plugin image
struct LoadedImage {
    /// Unique ID
    id: u64,
    /// Parsed ELF information
    elf: ParsedElf,
    /// Load base address
    base_addr: u64,
}

impl PluginLoader {
    /// Create new plugin loader
    pub fn new() -> Self {
        Self {
            loaded_images: Vec::new(),
            entry_point: String::from("plugin_entry"),
            config: LoaderConfig::default(),
        }
    }

    /// Load plugin from ELF binary data
    /// @param data: Raw ELF binary data
    /// @return: Plugin instance
    pub fn load_from_elf(&mut self, data: &[u8]) -> Result<Box<dyn Plugin>, PluginError> {
        if data.len() > self.config.max_plugin_size {
            return Err(PluginError::InvalidPlugin(String::from(
                "Plugin exceeds maximum size",
            )));
        }

        let elf = ElfParser::parse(data)
            .map_err(|e| PluginError::InvalidPlugin(format!("ELF parse failed: {}", e.message)))?;

        let total_memsz: u64 = elf.segments.iter().map(|s| s.memsz).sum();
        if total_memsz == 0 {
            return Err(PluginError::InvalidPlugin(String::from(
                "No loadable segments",
            )));
        }

        let layout = Layout::from_size_align(total_memsz as usize, 4096)
            .map_err(|_| PluginError::OutOfMemory)?;

        let base_ptr = unsafe {
            let p = alloc(layout);
            if p.is_null() {
                return Err(PluginError::OutOfMemory);
            }
            core::ptr::write_bytes(p, 0, total_memsz as usize);
            p
        };

        let base_addr = base_ptr as u64;

        for seg in &elf.segments {
            let dst_offset = if seg.vaddr >= elf.load_base {
                (seg.vaddr - elf.load_base) as usize
            } else {
                seg.vaddr as usize
            };
            let src_start = seg.file_offset as usize;
            let src_end = src_start + seg.filesz as usize;
            if src_end <= data.len() && dst_offset + seg.filesz as usize <= total_memsz as usize {
                // SAFETY: We verified src_end <= data.len() and dst_offset + filesz <= total_memsz,
                // so both source and destination ranges are within bounds.
                // copy_nonoverlapping is safe because ELF segments don't overlap.
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        data.as_ptr().add(src_start),
                        base_ptr.add(dst_offset),
                        seg.filesz as usize,
                    );
                }
            }
        }

        // SAFETY: base_ptr points to total_memsz bytes of allocated memory.
        // from_raw_parts_mut is safe because base_ptr is valid and total_memsz
        // matches the allocation size.
        let mut image_data =
            unsafe { core::slice::from_raw_parts_mut(base_ptr, total_memsz as usize) };
        let ehdr_data = &data[..core::mem::size_of::<Elf64Ehdr>()];
        let mut ehdr_copy = Elf64Ehdr {
            e_ident: [0; 16],
            e_type: 0,
            e_machine: 0,
            e_version: 0,
            e_entry: 0,
            e_phoff: 0,
            e_shoff: 0,
            e_flags: 0,
            e_ehsize: 0,
            e_phentsize: 0,
            e_phnum: 0,
            e_shentsize: 0,
            e_shnum: 0,
            e_shstrndx: 0,
        };
        let ehdr_dst = unsafe {
            core::slice::from_raw_parts_mut(
                &mut ehdr_copy as *mut Elf64Ehdr as *mut u8,
                core::mem::size_of::<Elf64Ehdr>(),
            )
        };
        ehdr_dst.copy_from_slice(ehdr_data);

        let _ = ElfParser::apply_relocations(base_addr, &mut image_data, &ehdr_copy, data);

        let plugin = Box::new(ElfPlugin {
            meta: PluginMeta::new(0, ""),
            base_ptr,
            base_addr,
            mem_layout: layout,
            entry_point: elf.entry_point,
            state: PluginState::Loaded,
            symbol_names: elf.symbols.clone(),
        });

        let image = LoadedImage {
            id: self.next_handle_id(),
            elf,
            base_addr,
        };
        self.loaded_images.push(image);

        Ok(plugin)
    }

    /// Load plugin from file path via VFS
    /// @param path: Path to plugin file
    /// @return: Plugin instance
    pub fn load(&mut self, path: &str) -> Result<Box<dyn Plugin>, PluginError> {
        const O_RDONLY: i32 = 0;
        let fd = crate::kernel::fs::vfs::file::open(path, O_RDONLY, 0);
        if fd < 0 {
            return Err(PluginError::IoError(format!(
                "Failed to open plugin file: {}",
                path
            )));
        }

        let mut buf = Vec::new();
        const READ_SIZE: usize = 4096;
        let mut tmp = [0u8; READ_SIZE];
        loop {
            let n = crate::kernel::fs::vfs::file::read(fd as u32, &mut tmp);
            if n <= 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n as usize]);
            if (n as usize) < READ_SIZE {
                break;
            }
        }

        let _ = crate::kernel::fs::vfs::file::close(fd as u32);

        if buf.is_empty() {
            return Err(PluginError::IoError(String::from("Plugin file is empty")));
        }

        self.load_from_elf(&buf)
    }

    /// Parse ELF without loading (for inspection/validation)
    /// @param data: Raw ELF binary data
    /// @return: Parsed ELF information
    pub fn inspect_elf(data: &[u8]) -> Result<ParsedElf, PluginError> {
        ElfParser::parse(data)
            .map_err(|e| PluginError::InvalidPlugin(format!("ELF parse failed: {}", e.message)))
    }

    /// Unload plugin by ID
    /// @param id: Loaded image ID
    pub fn unload(&mut self, id: u64) -> Result<(), PluginError> {
        if let Some(pos) = self.loaded_images.iter().position(|img| img.id == id) {
            self.loaded_images.remove(pos);
            Ok(())
        } else {
            Err(PluginError::NotFound(format!("Image {} not found", id)))
        }
    }

    /// Get number of loaded images
    pub fn loaded_count(&self) -> usize {
        self.loaded_images.len()
    }

    /// Generate next handle ID
    fn next_handle_id(&self) -> u64 {
        self.loaded_images.len() as u64 + 1
    }
}

/// Loader configuration
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Verify plugin signature
    pub verify_signature: bool,

    /// Cache loaded plugins
    pub enable_cache: bool,

    /// Maximum plugin size
    pub max_plugin_size: usize,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            verify_signature: false,
            enable_cache: true,
            max_plugin_size: 10 * 1024 * 1024,
        }
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ELF Plugin Instance
// ============================================================================

/// A plugin loaded from an ELF binary image
struct ElfPlugin {
    meta: PluginMeta,
    base_ptr: *mut u8,
    base_addr: u64,
    mem_layout: Layout,
    entry_point: u64,
    state: PluginState,
    symbol_names: Vec<String>,
}

impl Plugin for ElfPlugin {
    fn meta(&self) -> &PluginMeta {
        &self.meta
    }

    fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        if self.state != PluginState::Loaded {
            return Err(PluginError::InvalidState {
                current: self.state,
                expected: PluginState::Loaded,
            });
        }
        self.state = PluginState::Initialized;
        Ok(())
    }

    fn activate(&mut self) -> Result<(), PluginError> {
        if self.state != PluginState::Initialized {
            return Err(PluginError::InvalidState {
                current: self.state,
                expected: PluginState::Initialized,
            });
        }
        self.state = PluginState::Active;
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginError> {
        if self.state != PluginState::Active {
            return Err(PluginError::InvalidState {
                current: self.state,
                expected: PluginState::Active,
            });
        }
        self.state = PluginState::Deactivated;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), PluginError> {
        if self.state == PluginState::Active {
            self.deactivate()?;
        }
        if !self.base_ptr.is_null() {
            // SAFETY: base_ptr was allocated with mem_layout in load_from_elf.
            unsafe { dealloc(self.base_ptr, self.mem_layout) };
            self.base_ptr = core::ptr::null_mut();
        }
        self.state = PluginState::Unloading;
        Ok(())
    }
}

impl Drop for ElfPlugin {
    fn drop(&mut self) {
        if !self.base_ptr.is_null() {
            // SAFETY: base_ptr was allocated with mem_layout in load_from_elf.
            unsafe { dealloc(self.base_ptr, self.mem_layout) };
        }
    }
}

// SAFETY: ElfPlugin owns its memory and has no interior mutability beyond
// what is protected by the Plugin state machine.
unsafe impl Send for ElfPlugin {}
// SAFETY: All shared access to ElfPlugin is read-only or protected by
// internal synchronization. No &ElfPlugin can cause data races.
unsafe impl Sync for ElfPlugin {}

// ============================================================================
// Compatibility layer for hosted testing on Linux user-space
//
// This module provides user-space FFI loading capability for development
// and testing on Linux hosts. It is NOT part of the native Nuva OS kernel.
// ============================================================================

#[cfg(all(target_os = "linux", not(feature = "kernel")))]
mod hosted_compat {
    use super::*;

    /// Library handle for hosted (Linux user-space) compatibility loading.
    /// Compatibility target for Linux userspace testing only.
    pub struct LibraryHandle {
        pub id: u64,
        pub handle: *mut (),
        pub path: String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use alloc::vec;

    #[test]
    fn test_elf_validation_too_small() {
        let data = [0u8; 10];
        let result = ElfParser::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_elf_validation_bad_magic() {
        let mut data = vec![0u8; 64];
        data[0] = 0;
        data[1] = b'N';
        data[2] = b'O';
        data[3] = b'T';
        let result = ElfParser::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Invalid ELF magic"));
    }

    #[test]
    fn test_elf_validation_not_64bit() {
        let mut data = vec![0u8; 64];
        data[0] = 0x7F;
        data[1] = b'E';
        data[2] = b'L';
        data[3] = b'F';
        data[4] = 1; // ELFCLASS32
        let result = ElfParser::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Not a 64-bit ELF"));
    }

    #[test]
    fn test_loader_new() {
        let loader = PluginLoader::new();
        assert_eq!(loader.loaded_count(), 0);
    }
}
