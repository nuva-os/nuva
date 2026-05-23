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

// ! symbolsignalparse

use std::collections::HashMap;
use super::object::ObjectFile;
use super::{LinkError, elf::Symbol};

/// symbolsignalparsedevice
pub struct SymbolResolver {
 /// Globalsymbolsignalform
 global_symbols: HashMap<String, ResolvedSymbol>,
 /// Undefinedsymbolsignal
 undefined_symbols: HashMap<String, Vec<String>>, // symbol -> objects that need it
}

impl SymbolResolver {
 pub fn new() -> Self {
 Self {
 global_symbols: HashMap::new(),
 undefined_symbols: HashMap::new(),
 }
 }

 /// parseplacefinitetargetFile symbolsignal
 pub fn resolve(&mut self, objects: &[ObjectFile]) -> Result<&HashMap<String, ResolvedSymbol>, LinkError> {
 // iterate: receivecollectionplacefinitesymbolsignal
 for obj in objects {
 self.collect_symbols(obj)?;
 }

 // seconditerate: parseUndefinedsymbolsignal
 self.resolve_undefined()?;

 Ok(&self.global_symbols)
 }

 /// receivecollectiontargetFile symbolsignal
 fn collect_symbols(&mut self, obj: &ObjectFile) -> Result<(), LinkError> {
 let obj_name = obj.path.to_string_lossy().to_string();

 // Processalreadyfixedmeaningsymbolsignal
 for sym_name in &obj.defined_symbols {
 if let Some(existing) = self.global_symbols.get(sym_name) {
 // checkiswhetherisrepeatrestorefixedmeaning
 if !existing.is_weak {
 return Err(LinkError::DuplicateSymbol(sym_name.clone()));
 }
 }

 // findsymbolsignalcontext
 if let Some(symbol) = obj.get_symbol(sym_name) {
 let is_weak = (symbol.info & 0xf) == 0; // STB_LOCAL
 
 self.global_symbols.insert(sym_name.clone(), ResolvedSymbol {
 name: sym_name.clone(),
 value: symbol.value,
 size: symbol.size,
 object: obj_name.clone(),
 is_weak,
 });
 }
 }

 // ProcessUndefinedsymbolsignal
 for sym_name in &obj.undefined_symbols {
 self.undefined_symbols
 .entry(sym_name.clone())
 .or_default()
 .push(obj_name.clone());
 }

 Ok(())
 }

 /// parseUndefinedsymbolsignal
 fn resolve_undefined(&mut self) -> Result<(), LinkError> {
 for (sym_name, objects) in &self.undefined_symbols {
 if !self.global_symbols.contains_key(sym_name) {
 // symbolsignalUndefined
 return Err(LinkError::UndefinedSymbol(sym_name.clone()));
 }
 }

 Ok(())
 }

 /// findsymbolsignaladdress
 pub fn find_symbol(&self, name: &str) -> Option<u64> {
 self.global_symbols.get(name).map(|s| s.value)
 }

 /// Getsymbolsignalinformation
 pub fn get_symbol(&self, name: &str) -> Option<&ResolvedSymbol> {
 self.global_symbols.get(name)
 }

 /// Getplacefinitesymbolsignal
 pub fn symbols(&self) -> &HashMap<String, ResolvedSymbol> {
 &self.global_symbols
 }
}

impl Default for SymbolResolver {
 fn default() -> Self {
 Self::new()
 }
}

/// alreadyparse symbolsignal
#[derive(Debug, Clone)]
pub struct ResolvedSymbol {
 /// symbolsignalname
 pub name: String,
 /// value(address)
 pub value: u64,
 /// size
 pub size: u64,
 /// fixedmeaningthesymbolsignal targetFile
 pub object: String,
 /// iswhetherisweaksymbolsignal
 pub is_weak: bool,
}

/// symbolsignalType
#[derive(Debug, Clone, Copy)]
pub enum SymbolType {
 Notype = 0,
 Object = 1,
 Func = 2,
 Section = 3,
 File = 4,
}

/// symbolsignalBind
#[derive(Debug, Clone, Copy)]
pub enum SymbolBinding {
 Local = 0,
 Global = 1,
 Weak = 2,
}