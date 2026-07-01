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

// ! LinkerModule
/*!*/
// ! Support ELF gridstyle targetFilesumcanexecuteFileProcess

pub mod elf;
pub mod object;
pub mod symbol;
pub mod relocation;
pub mod script;

use std::path::PathBuf;
use std::collections::HashMap;
use alloc::vec;
use alloc::vec::Vec;

/// Linker
pub struct Linker {
    /// linkacceptConfiguration
    config: LinkerConfig,
    /// symbolsignalparsedevice
    symbol_resolver: symbol::SymbolResolver,
    /// repeatfixedpositiondevice
    relocator: relocation::Relocator,
}

impl Linker {
    pub fn new(config: LinkerConfig) -> Self {
        Self {
            config,
            symbol_resolver: symbol::SymbolResolver::new(),
            relocator: relocation::Relocator::new(),
        }
    }

    /// linkaccepttargetFile
    pub fn link(&mut self, objects: &[PathBuf], output: &PathBuf) -> Result<LinkResult, LinkError> {
        // 1. parseplacefinitetargetFile
        let parsed_objects: Vec<object::ObjectFile> = objects.iter()
            .map(|p| object::ObjectFile::parse(p))
            .collect::<Result<Vec<_>, _>>()?;

        // 2. combineparallelsymbolsignalform
        let symbols = self.symbol_resolver.resolve(&parsed_objects)?;

        // 3. ComputeSectionlayout
        let layout = self.compute_layout(&parsed_objects)?;

        // 4. shoulduserepeatfixedposition
        let relocated = self.relocator.relocate(&parsed_objects, &symbols, &layout)?;

        // 5. Generate executable file
        let executable = self.generate_executable(&relocated, &layout)?;

        // 6. writeoutputFile
        self.write_output(&executable, output)?;

        Ok(LinkResult {
            output: output.clone(),
            symbols: symbols.len(),
            sections: layout.sections.len(),
        })
    }

    /// ComputeSectionlayout
    fn compute_layout(&self, objects: &[object::ObjectFile]) -> Result<SectionLayout, LinkError> {
        let mut layout = SectionLayout::default();

        // makeuselinkacceptScript(iffinite)
        if let Some(ref script_path) = self.config.script {
            let linker_script = script::LinkerScript::parse(script_path)?;
            layout.apply_script(&linker_script);
        } else {
            // defaultlayout
            layout.set_default();
        }

        // ComputeSectionsizesumaddress
        for obj in objects {
            for section in &obj.sections {
                layout.add_section(section.clone());
            }
        }

        Ok(layout)
    }

    /// Generate executable file
    fn generate_executable(&self, objects: &[object::ObjectFile], layout: &SectionLayout) -> Result<elf::ElfFile, LinkError> {
        let mut elf = elf::ElfFile::new_executable();

        // Settingsenterportpoint
        if let Some(entry) = self.config.entry.clone() {
            elf.set_entry(entry);
        } else {
            // find _start or main symbolsignal
            if let Some(addr) = self.symbol_resolver.find_symbol("_start") {
                elf.set_entry(addr);
            } else if let Some(addr) = self.symbol_resolver.find_symbol("main") {
                elf.set_entry(addr);
            }
        }

        // addPlusSection
        for section in &layout.sections {
            elf.add_section(section.clone());
        }

        Ok(elf)
    }

    /// writeoutputFile
    fn write_output(&self, elf: &elf::ElfFile, output: &PathBuf) -> Result<(), LinkError> {
        let data = elf.serialize()?;
        std::fs::write(output, data)
            .map_err(|e| LinkError::IoError(e.to_string()))?;
        Ok(())
    }
}

/// LinkerConfiguration
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    /// outputgridstyle
    pub output_format: OutputFormat,
    /// enterportpoint
    pub entry: Option<u64>,
    /// linkacceptScript
    pub script: Option<PathBuf>,
    /// LibrarysearchPath
    pub library_paths: Vec<PathBuf>,
    /// needwantlinkaccept Library
    pub libraries: Vec<String>,
    /// iswhetherStaticlinkaccept
    pub static_link: bool,
    /// iswhethergenerateDebugginginformation
    pub debug: bool,
    /// iswhethergoDividesymbolsignalform
    pub strip: bool,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Elf64,
            entry: None,
            script: None,
            library_paths: vec![],
            libraries: vec![],
            static_link: false,
            debug: false,
            strip: false,
        }
    }
}

/// outputgridstyle
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Elf32,
    Elf64,
    Binary,
}

/// Sectionlayout
#[derive(Debug, Default)]
pub struct SectionLayout {
    pub sections: Vec<elf::Section>,
    pub section_map: HashMap<String, usize>,
}

impl SectionLayout {
    pub fn set_default(&mut self) {
        // defaultSectionlayout
        self.sections = vec![
            elf::Section::new(".text", elf::SectionType::Code, 0x400000),
            elf::Section::new(".rodata", elf::SectionType::ReadOnlyData, 0),
            elf::Section::new(".data", elf::SectionType::Data, 0),
            elf::Section::new(".bss", elf::SectionType::Bss, 0),
        ];
    }

    pub fn apply_script(&mut self, script: &script::LinkerScript) {
        // shoulduselinkacceptScript layout
        for cmd in &script.commands {
            match cmd {
                script::Command::Sections(sections) => {
                    self.sections = sections.clone();
                }
                _ => {}
            }
        }
    }

    pub fn add_section(&mut self, section: elf::Section) {
        let name = section.name.clone();
        let idx = self.sections.len();
        self.sections.push(section);
        self.section_map.insert(name, idx);
    }
}

/// Link result
#[derive(Debug)]
pub struct LinkResult {
    pub output: PathBuf,
    pub symbols: usize,
    pub sections: usize,
}

/// Link error
#[derive(Debug)]
pub enum LinkError {
    IoError(String),
    ParseError(String),
    SymbolError(String),
    RelocationError(String),
    UndefinedSymbol(String),
    DuplicateSymbol(String),
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::IoError(msg) => write!(f, "IO error: {}", msg),
            LinkError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LinkError::SymbolError(msg) => write!(f, "Symbol error: {}", msg),
            LinkError::RelocationError(msg) => write!(f, "Relocation error: {}", msg),
            LinkError::UndefinedSymbol(name) => write!(f, "Undefined symbol: {}", name),
            LinkError::DuplicateSymbol(name) => write!(f, "Duplicate symbol: {}", name),
        }
    }
}

impl std::error::Error for LinkError {}