/* * Nuva OS - Tools - Toolchain Linker
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

// ! Nuva linker

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Linker output type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LinkOutputType {
    Executable = 0,
    SharedLibrary = 1,
    StaticLibrary = 2,
    ObjectFile = 3,
    Kernel = 4,
}

/// Link options
#[derive(Debug, Clone)]
pub struct LinkOptions {
    pub output_type: LinkOutputType,
    pub output_name: [u8; 256],
    pub output_name_len: u8,
    pub entry_point: [u8; 64],
    pub entry_point_len: u8,
    pub library_paths: [[u8; 256]; 32],
    pub num_library_paths: u8,
    pub libraries: [[u8; 64]; 64],
    pub num_libraries: u8,
    pub gc_sections: bool,
    pub strip_debug: bool,
    pub static_linking: bool,
    pub base_address: u64,
}

impl Default for LinkOptions {
    fn default() -> Self {
        Self {
            output_type: LinkOutputType::Executable,
            output_name: [0; 256],
            output_name_len: 0,
            entry_point: *b"_start\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            entry_point_len: 6,
            library_paths: [[0; 256]; 32],
            num_library_paths: 0,
            libraries: [[0; 64]; 64],
            num_libraries: 0,
            gc_sections: true,
            strip_debug: false,
            static_linking: false,
            base_address: 0x400000,
        }
    }
}

/// Symbol
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: [u8; 64],
    pub name_len: u8,
    pub value: u64,
    pub size: u64,
    pub kind: SymbolKind,
    pub binding: SymbolBinding,
    pub section_index: u16,
}

/// Symbol kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SymbolKind {
    Unknown = 0,
    Function = 1,
    Object = 2,
    Section = 3,
    File = 4,
    Common = 5,
    TLS = 6,
}

/// Symbol binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SymbolBinding {
    Local = 0,
    Global = 1,
    Weak = 2,
}

/// Relocation entry
#[derive(Debug, Clone)]
pub struct Relocation {
    pub offset: u64,
    pub symbol_index: u32,
    pub kind: RelocationKind,
    pub addend: i64,
}

/// Relocation kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum RelocationKind {
    // ARM64
    AArch64Abs64 = 257,
    AArch64Call26 = 283,
    AArch64Jump26 = 282,
    AArch64AdrPrelLo21 = 274,
    AArch64AdrPrelHi21 = 273,

    // x86_64
    X86_64_64 = 1,
    X86_64PC32 = 2,
    X86_64PLT32 = 4,
    X86_64GOTPCRel = 9,
}

/// Section
#[derive(Debug)]
pub struct Section {
    pub name: [u8; 32],
    pub name_len: u8,
    pub kind: SectionKind,
    pub address: u64,
    pub size: u64,
    pub data: [u8; 65536],
    pub data_len: u32,
    pub flags: u32,
}

/// Section kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SectionKind {
    Unknown = 0,
    Text = 1,
    Data = 2,
    BSS = 3,
    ReadOnlyData = 4,
    TLS = 5,
    TLSData = 6,
    TLSBSS = 7,
    SymTab = 8,
    StrTab = 9,
    Rel = 10,
    Rela = 11,
    Debug = 12,
}

/// Object file
#[derive(Debug)]
pub struct ObjectFile {
    pub name: [u8; 256],
    pub name_len: u8,
    pub sections: [Section; 64],
    pub num_sections: u8,
    pub symbols: [Symbol; 1024],
    pub num_symbols: AtomicU32,
    pub relocations: [Relocation; 4096],
    pub num_relocations: AtomicU32,
}

impl ObjectFile {
    pub fn new(name: &[u8]) -> Self {
        let mut name_buf = [0u8; 256];
        let len = name.len().min(255);
        name_buf[..len].copy_from_slice(&name[..len]);

        Self {
            name: name_buf,
            name_len: len as u8,
            sections: [Section {
                name: [0; 32],
                name_len: 0,
                kind: SectionKind::Unknown,
                address: 0,
                size: 0,
                data: [0; 65536],
                data_len: 0,
                flags: 0,
            }; 64],
            num_sections: 0,
            symbols: [Symbol {
                name: [0; 64],
                name_len: 0,
                value: 0,
                size: 0,
                kind: SymbolKind::Unknown,
                binding: SymbolBinding::Local,
                section_index: 0,
            }; 1024],
            num_symbols: AtomicU32::new(0),
            relocations: [Relocation {
                offset: 0,
                symbol_index: 0,
                kind: RelocationKind::AArch64Abs64,
                addend: 0,
            }; 4096],
            num_relocations: AtomicU32::new(0),
        }
    }

    pub fn add_section(&mut self, section: Section) -> u8 {
        if self.num_sections < 64 {
            self.sections[self.num_sections as usize] = section;
            self.num_sections += 1;
            self.num_sections - 1
        } else {
            0
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) -> u32 {
        let idx = self.num_symbols.load(Ordering::Relaxed);
        if idx < 1024 {
            self.symbols[idx as usize] = symbol;
            self.num_symbols.fetch_add(1, Ordering::Release);
        }
        idx
    }

    pub fn add_relocation(&mut self, reloc: Relocation) {
        let idx = self.num_relocations.load(Ordering::Relaxed);
        if idx < 4096 {
            self.relocations[idx as usize] = reloc;
            self.num_relocations.fetch_add(1, Ordering::Release);
        }
    }
}

/// Link result
#[derive(Debug)]
pub struct LinkResult {
    pub success: bool,
    pub output_path: [u8; 256],
    pub output_path_len: u8,
    pub errors: [LinkError; 32],
    pub num_errors: u8,
    pub warnings: [LinkWarning; 32],
    pub num_warnings: u8,
}

impl LinkResult {
    pub fn new() -> Self {
        Self {
            success: true,
            output_path: [0; 256],
            output_path_len: 0,
            errors: [LinkError {
                message: [0; 256],
                message_len: 0,
            }; 32],
            num_errors: 0,
            warnings: [LinkWarning {
                message: [0; 256],
                message_len: 0,
            }; 32],
            num_warnings: 0,
        }
    }
}

/// Link error
#[derive(Debug, Clone)]
pub struct LinkError {
    pub message: [u8; 256],
    pub message_len: u8,
}

/// Link warning
#[derive(Debug, Clone)]
pub struct LinkWarning {
    pub message: [u8; 256],
    pub message_len: u8,
}

/// Linker
pub struct Linker {
    options: LinkOptions,
    objects: [ObjectFile; 64],
    num_objects: u8,
    global_symbols: [Symbol; 4096],
    num_global_symbols: AtomicU32,
}

impl Linker {
    pub fn new(options: LinkOptions) -> Self {
        Self {
            options,
            objects: [ObjectFile::new(b""); 64],
            num_objects: 0,
            global_symbols: [Symbol {
                name: [0; 64],
                name_len: 0,
                value: 0,
                size: 0,
                kind: SymbolKind::Unknown,
                binding: SymbolBinding::Local,
                section_index: 0,
            }; 4096],
            num_global_symbols: AtomicU32::new(0),
        }
    }

    pub fn add_object(&mut self, obj: ObjectFile) {
        if self.num_objects < 64 {
            self.objects[self.num_objects as usize] = obj;
            self.num_objects += 1;
        }
    }

    pub fn link(&mut self) -> LinkResult {
        let mut result = LinkResult::new();

        // 1. Symbol resolution
        self.resolve_symbols(&mut result);
        if !result.success {
            return result;
        }

        // 2. Section merging
        self.merge_sections();

        // 3. Address assignment
        self.assign_addresses();

        // 4. Apply relocations
        self.apply_relocations();

        // 5. Generate output
        self.generate_output(&mut result);

        result
    }

    fn resolve_symbols(&mut self, result: &mut LinkResult) {
        for i in 0..self.num_objects as usize {
            let obj = &self.objects[i];
            let num_syms = obj.num_symbols.load(Ordering::Relaxed);

            for j in 0..num_syms as usize {
                let sym = &obj.symbols[j];

                if sym.binding == SymbolBinding::Global {
                    // Check if already defined
                    let mut found = false;
                    let num_global = self.num_global_symbols.load(Ordering::Relaxed);

                    for k in 0..num_global as usize {
                        if self.global_symbols[k].name[..self.global_symbols[k].name_len as usize]
                            == sym.name[..sym.name_len as usize]
                        {
                            // Symbol redefinition
                            if self.global_symbols[k].binding == SymbolBinding::Global {
                                let mut msg = [0u8; 256];
                                let err_msg = b"duplicate symbol: ";
                                msg[..err_msg.len()].copy_from_slice(err_msg);
                                msg[err_msg.len()..err_msg.len() + sym.name_len as usize]
                                    .copy_from_slice(&sym.name[..sym.name_len as usize]);

                                result.errors[result.num_errors as usize] = LinkError {
                                    message: msg,
                                    message_len: (err_msg.len() + sym.name_len as usize) as u8,
                                };
                                result.num_errors += 1;
                                result.success = false;
                            }
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        let idx = self.num_global_symbols.load(Ordering::Relaxed);
                        if idx < 4096 {
                            self.global_symbols[idx as usize] = sym.clone();
                            self.num_global_symbols.fetch_add(1, Ordering::Release);
                        }
                    }
                }
            }
        }
    }

    fn merge_sections(&mut self) {
        // Merge sections of the same type
    }

    fn assign_addresses(&mut self) {
        // Assign virtual addresses
        let mut addr = self.options.base_address;

        for i in 0..self.num_objects as usize {
            for j in 0..self.objects[i].num_sections as usize {
                let section = &mut self.objects[i].sections[j];
                section.address = addr;
                addr += section.size;
                addr = (addr + 0xfff) & !0xfff; // Page alignment
            }
        }
    }

    fn apply_relocations(&mut self) {
        // Apply relocations
        for i in 0..self.num_objects as usize {
            let obj = &mut self.objects[i];
            let num_relocs = obj.num_relocations.load(Ordering::Relaxed);

            for j in 0..num_relocs as usize {
                let reloc = &obj.relocations[j];
                let _ = reloc; // Simplified
            }
        }
    }

    fn generate_output(&self, result: &mut LinkResult) {
        // Generate executable file
        let name = &self.options.output_name[..self.options.output_name_len as usize];
        result.output_path[..name.len()].copy_from_slice(name);
        result.output_path_len = self.options.output_name_len;
    }
}
