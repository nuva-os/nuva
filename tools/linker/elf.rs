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

// ! ELF FilegridstyleProcess

use std::path::PathBuf;

/// ELF File
#[derive(Debug, Clone)]
pub struct ElfFile {
 /// Fileheader
 pub header: ElfHeader,
 /// Sectionheaderform
 pub sections: Vec<Section>,
 /// processorderheaderform
 pub segments: Vec<Segment>,
 /// symbolsignalform
 pub symbols: Vec<Symbol>,
 /// Stringform
 pub strtab: Vec<u8>,
}

impl ElfFile {
 /// createnew canexecuteFile
 pub fn new_executable() -> Self {
 Self {
 header: ElfHeader {
 class: ElfClass::Class64,
 data: ElfData::LittleEndian,
 version: 1,
 osabi: 0,
 abiversion: 0,
 type_: ElfType::Executable,
 machine: ElfMachine::None,
 version2: 1,
 entry: 0,
 phoff: 0,
 shoff: 0,
 flags: 0,
 ehsize: 64,
 phentsize: 56,
 phnum: 0,
 shentsize: 64,
 shnum: 0,
 shstrndx: 0,
 },
 sections: vec![],
 segments: vec![],
 symbols: vec![],
 strtab: vec![],
 }
 }

 /// Settingsenterportpoint
 pub fn set_entry(&mut self, entry: u64) {
 self.header.entry = entry;
 }

 /// addPlusSection
 pub fn add_section(&mut self, section: Section) {
 self.sections.push(section);
 self.header.shnum = self.sections.len() as u16;
 }

 /// addPlusparagraph
 pub fn add_segment(&mut self, segment: Segment) {
 self.segments.push(segment);
 self.header.phnum = self.segments.len() as u16;
 }

 /// SerializationascharacterSection
 pub fn serialize(&self) -> Result<Vec<u8>, ElfError> {
 let mut output = Vec::new();

 // write ELF number
 output.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46]); // \x7fELF

 // writeFileheader
 output.push(self.header.class as u8);
 output.push(self.header.data as u8);
 output.push(self.header.version);
 output.push(self.header.osabi);
 output.push(self.header.abiversion);
 output.extend_from_slice(&[0; 7]); // padding

 // type
 output.extend_from_slice(&(self.header.type_ as u16).to_le_bytes());
 // machinedevice
 output.extend_from_slice(&(self.header.machine as u16).to_le_bytes());
 // version
 output.extend_from_slice(&self.header.version2.to_le_bytes());
 // enterportpoint
 output.extend_from_slice(&self.header.entry.to_le_bytes());
 // processorderheaderoffset
 output.extend_from_slice(&self.header.phoff.to_le_bytes());
 // Sectionheaderoffset
 output.extend_from_slice(&self.header.shoff.to_le_bytes());
 // flag
 output.extend_from_slice(&self.header.flags.to_le_bytes());
 // ELF headersize
 output.extend_from_slice(&self.header.ehsize.to_le_bytes());
 // processorderheaderprojectsize
 output.extend_from_slice(&self.header.phentsize.to_le_bytes());
 // processorderheadercount
 output.extend_from_slice(&self.header.phnum.to_le_bytes());
 // Sectionheaderprojectsize
 output.extend_from_slice(&self.header.shentsize.to_le_bytes());
 // Sectionheadercount
 output.extend_from_slice(&self.header.shnum.to_le_bytes());
 // SectionnameStringformindex
 output.extend_from_slice(&self.header.shstrndx.to_le_bytes());

 // TODO: Write section and segment data

 Ok(output)
 }

 /// secondaryFileparse
 pub fn parse(path: &PathBuf) -> Result<Self, ElfError> {
 let data = std::fs::read(path)
 .map_err(|e| ElfError::IoError(e.to_string()))?;

 Self::parse_bytes(&data)
 }

 /// secondarycharacterSectionparse
 pub fn parse_bytes(data: &[u8]) -> Result<Self, ElfError> {
 if data.len() < 16 {
 return Err(ElfError::InvalidFormat("File too small".to_string()));
 }

 // checknumber
 if &data[0..4] != &[0x7f, 0x45, 0x4c, 0x46] {
 return Err(ElfError::InvalidFormat("Not an ELF file".to_string()));
 }

 let class = match data[4] {
 1 => ElfClass::Class32,
 2 => ElfClass::Class64,
 _ => return Err(ElfError::InvalidFormat("Invalid ELF class".to_string())),
 };

 let data_endian = match data[5] {
 1 => ElfData::LittleEndian,
 2 => ElfData::BigEndian,
 _ => return Err(ElfError::InvalidFormat("Invalid ELF data encoding".to_string())),
 };

 // TODO: Parse complete ELF file

 Ok(Self {
 header: ElfHeader {
 class,
 data: data_endian,
 version: data[6],
 osabi: data[7],
 abiversion: data[8],
 type_: ElfType::None,
 machine: ElfMachine::None,
 version2: 1,
 entry: 0,
 phoff: 0,
 shoff: 0,
 flags: 0,
 ehsize: 64,
 phentsize: 56,
 phnum: 0,
 shentsize: 64,
 shnum: 0,
 shstrndx: 0,
 },
 sections: vec![],
 segments: vec![],
 symbols: vec![],
 strtab: vec![],
 })
 }
}

/// ELF Fileheader
#[derive(Debug, Clone)]
pub struct ElfHeader {
 pub class: ElfClass,
 pub data: ElfData,
 pub version: u8,
 pub osabi: u8,
 pub abiversion: u8,
 pub type_: ElfType,
 pub machine: ElfMachine,
 pub version2: u32,
 pub entry: u64,
 pub phoff: u64,
 pub shoff: u64,
 pub flags: u32,
 pub ehsize: u16,
 pub phentsize: u16,
 pub phnum: u16,
 pub shentsize: u16,
 pub shnum: u16,
 pub shstrndx: u16,
}

/// ELF category
#[derive(Debug, Clone, Copy)]
pub enum ElfClass {
 Class32 = 1,
 Class64 = 2,
}

/// ELF dataEncoding
#[derive(Debug, Clone, Copy)]
pub enum ElfData {
 LittleEndian = 1,
 BigEndian = 2,
}

/// ELF type
#[derive(Debug, Clone, Copy)]
pub enum ElfType {
 None = 0,
 Relocatable = 1,
 Executable = 2,
 Shared = 3,
 Core = 4,
}

/// ELF machinedeviceType
#[derive(Debug, Clone, Copy)]
pub enum ElfMachine {
 None = 0,
 Arm = 40,
 X86_64 = 62,
 AArch64 = 183,
}

/// Section
#[derive(Debug, Clone)]
pub struct Section {
 pub name: String,
 pub type_: SectionType,
 pub flags: u64,
 pub addr: u64,
 pub offset: u64,
 pub size: u64,
 pub link: u32,
 pub info: u32,
 pub addralign: u64,
 pub entsize: u64,
 pub data: Vec<u8>,
}

impl Section {
 pub fn new(name: &str, type_: SectionType, addr: u64) -> Self {
 Self {
 name: name.to_string(),
 type_,
 flags: 0,
 addr,
 offset: 0,
 size: 0,
 link: 0,
 info: 0,
 addralign: 1,
 entsize: 0,
 data: vec![],
 }
 }
}

/// Section kind
#[derive(Debug, Clone, Copy)]
pub enum SectionType {
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
 DynSym = 11,
 Code,
 ReadOnlyData,
 Data,
 Bss,
}

/// paragraph
#[derive(Debug, Clone)]
pub struct Segment {
 pub type_: SegmentType,
 pub flags: u64,
 pub offset: u64,
 pub vaddr: u64,
 pub paddr: u64,
 pub filesz: u64,
 pub memsz: u64,
 pub align: u64,
}

/// paragraphType
#[derive(Debug, Clone, Copy)]
pub enum SegmentType {
 Null = 0,
 Load = 1,
 Dynamic = 2,
 Interp = 3,
 Note = 4,
 Phdr = 6,
}

/// symbolsignal
#[derive(Debug, Clone)]
pub struct Symbol {
 pub name: String,
 pub info: u8,
 pub other: u8,
 pub shndx: u16,
 pub value: u64,
 pub size: u64,
}

/// ELF error
#[derive(Debug)]
pub enum ElfError {
 IoError(String),
 InvalidFormat(String),
 ParseError(String),
}

impl std::fmt::Display for ElfError {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 match self {
 ElfError::IoError(msg) => write!(f, "IO error: {}", msg),
 ElfError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
 ElfError::ParseError(msg) => write!(f, "Parse error: {}", msg),
 }
 }
}

impl std::error::Error for ElfError {}