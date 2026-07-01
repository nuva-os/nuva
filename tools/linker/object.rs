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

// ! targetFileparse

use std::path::PathBuf;
use super::elf::{ElfFile, Section, Symbol};
use alloc::vec;
use alloc::vec::Vec;

/// targetFile
#[derive(Debug, Clone)]
pub struct ObjectFile {
 /// FilePath
 pub path: PathBuf,
 /// ELF File
 pub elf: ElfFile,
 /// SectionList
 pub sections: Vec<Section>,
 /// symbolsignalList
 pub symbols: Vec<Symbol>,
 /// Undefinedsymbolsignal
 pub undefined_symbols: Vec<String>,
 /// alreadyfixedmeaningsymbolsignal
 pub defined_symbols: Vec<String>,
}

impl ObjectFile {
 /// parsetargetFile
 pub fn parse(path: &PathBuf) -> Result<Self, ObjectError> {
 let elf = ElfFile::parse(path)?;
 
 // takesymbolsignal
 let mut defined_symbols = vec![];
 let mut undefined_symbols = vec![];
 
 for symbol in &elf.symbols {
 if symbol.shndx == 0 {
 undefined_symbols.push(symbol.name.clone());
 } else {
 defined_symbols.push(symbol.name.clone());
 }
 }
 
 Ok(Self {
 path: path.clone(),
 elf,
 sections: elf.sections.clone(),
 symbols: elf.symbols.clone(),
 undefined_symbols,
 defined_symbols,
 })
 }

 /// GetSection
 pub fn get_section(&self, name: &str) -> Option<&Section> {
 self.sections.iter().find(|s| s.name == name)
 }

 /// Getsymbolsignal
 pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
 self.symbols.iter().find(|s| s.name == name)
 }

 /// checkiswhetherfixedmeaning symbolsignal
 pub fn defines(&self, name: &str) -> bool {
 self.defined_symbols.contains(&name.to_string())
 }

 /// checkiswhetherneedwantsymbolsignal
 pub fn needs(&self, name: &str) -> bool {
 self.undefined_symbols.contains(&name.to_string())
 }
}

/// targetFileError
#[derive(Debug)]
pub enum ObjectError {
 IoError(String),
 ParseError(String),
}

impl std::fmt::Display for ObjectError {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 match self {
 ObjectError::IoError(msg) => write!(f, "IO error: {}", msg),
 ObjectError::ParseError(msg) => write!(f, "Parse error: {}", msg),
 }
 }
}

impl std::error::Error for ObjectError {}