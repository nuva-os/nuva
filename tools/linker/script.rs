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

// ! linkacceptScriptparse

use std::path::PathBuf;
use super::elf::Section;

/// linkacceptScript
#[derive(Debug, Clone)]
pub struct LinkerScript {
 /// List
 pub commands: Vec<Command>,
 /// enterportpoint
 pub entry: Option<String>,
 /// Memoryzonedomain
 pub memory_regions: Vec<MemoryRegion>,
}

impl LinkerScript {
 /// parselinkacceptScript
 pub fn parse(path: &PathBuf) -> Result<Self, ScriptError> {
 let content = std::fs::read_to_string(path)
 .map_err(|e| ScriptError::IoError(e.to_string()))?;

 Self::parse_str(&content)
 }

 /// secondaryStringparse
 pub fn parse_str(content: &str) -> Result<Self, ScriptError> {
 let mut script = Self {
 commands: vec![],
 entry: None,
 memory_regions: vec![],
 };

 // simpleform parsedevice
 for line in content.lines() {
 let line = line.trim();
 
 // jumpovercommentsumemptyrow
 if line.is_empty() || line.starts_with("/*") || line.starts_with("//") {
 continue;
 }

 // parse ENTRY 
 if line.starts_with("ENTRY(") {
 if let Some(entry) = Self::parse_entry(line) {
 script.entry = Some(entry);
 }
 }

 // parse SECTIONS 
 if line.starts_with("SECTIONS") {
 // TODO: Parse SECTIONS block
 }

 // parse MEMORY 
 if line.starts_with("MEMORY") {
 // TODO: Parse MEMORY block
 }
 }

 Ok(script)
 }

 /// parse ENTRY
 fn parse_entry(line: &str) -> Option<String> {
 let start = line.find('(')?;
 let end = line.find(')')?;
 Some(line[start + 1..end].to_string())
 }
}

/// linkacceptScript
#[derive(Debug, Clone)]
pub enum Command {
 /// enterportpoint
 Entry(String),
 /// Sectionfixedmeaning
 Sections(Vec<Section>),
 /// Memoryzonedomain
 Memory(Vec<MemoryRegion>),
 /// symbolsignalAssignment
 SymbolAssign {
 name: String,
 value: Expr,
 },
 /// packetFile
 Include(String),
}

/// Memoryzonedomain
#[derive(Debug, Clone)]
pub struct MemoryRegion {
 /// zonedomainname
 pub name: String,
 /// startbeginaddress
 pub origin: u64,
 /// strengthmeasurement
 pub length: u64,
 /// Property
 pub attributes: String,
}

/// formreachstyle
#[derive(Debug, Clone)]
pub enum Expr {
 Number(u64),
 Symbol(String),
 Add(Box<Expr>, Box<Expr>),
 Sub(Box<Expr>, Box<Expr>),
 Mul(Box<Expr>, Box<Expr>),
 Div(Box<Expr>, Box<Expr>),
 Neg(Box<Expr>),
}

impl Expr {
 /// Computeformreachstylevalue
 pub fn evaluate(&self, symbols: &std::collections::HashMap<String, u64>) -> Result<u64, ScriptError> {
 match self {
 Expr::Number(n) => Ok(*n),
 Expr::Symbol(name) => {
 symbols.get(name)
 .copied()
 .ok_or_else(|| ScriptError::UndefinedSymbol(name.clone()))
 }
 Expr::Add(a, b) => {
 Ok(a.evaluate(symbols)? + b.evaluate(symbols)?)
 }
 Expr::Sub(a, b) => {
 Ok(a.evaluate(symbols)? - b.evaluate(symbols)?)
 }
 Expr::Mul(a, b) => {
 Ok(a.evaluate(symbols)? * b.evaluate(symbols)?)
 }
 Expr::Div(a, b) => {
 let b_val = b.evaluate(symbols)?;
 if b_val == 0 {
 return Err(ScriptError::DivisionByZero);
 }
 Ok(a.evaluate(symbols)? / b_val)
 }
 Expr::Neg(a) => {
 Ok(!a.evaluate(symbols)? + 1) // entercontrolpatchcode
 }
 }
 }
}

/// linkacceptScriptError
#[derive(Debug)]
pub enum ScriptError {
 IoError(String),
 ParseError(String),
 UndefinedSymbol(String),
 DivisionByZero,
}

impl std::fmt::Display for ScriptError {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 match self {
 ScriptError::IoError(msg) => write!(f, "IO error: {}", msg),
 ScriptError::ParseError(msg) => write!(f, "Parse error: {}", msg),
 ScriptError::UndefinedSymbol(name) => write!(f, "Undefined symbol: {}", name),
 ScriptError::DivisionByZero => write!(f, "Division by zero"),
 }
 }
}

impl std::error::Error for ScriptError {}

/// defaultlinkacceptScript
pub fn default_script() -> LinkerScript {
 LinkerScript {
 commands: vec![
 Command::Entry("_start".to_string()),
 Command::Sections(vec![
 Section::new(".text", super::elf::SectionType::Code, 0x400000),
 Section::new(".rodata", super::elf::SectionType::ReadOnlyData, 0),
 Section::new(".data", super::elf::SectionType::Data, 0),
 Section::new(".bss", super::elf::SectionType::Bss, 0),
 ]),
 ],
 entry: Some("_start".to_string()),
 memory_regions: vec![
 MemoryRegion {
 name: "RAM".to_string(),
 origin: 0,
 length: 0x100000000, // 4GB
 attributes: "rwx".to_string(),
 },
 ],
 }
}