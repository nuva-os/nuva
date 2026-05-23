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

/// SignType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
 /// Variable
 Variable,
 /// Function
 Function,
 /// Struct
 Struct,
 /// Enum
 Enum,
 /// mass
 Trait,
 /// Module
 Module,
 /// TypeParameter
 TypeParam,
 /// Field
 Field,
 /// Method
 Method,
}

/// Sign
#[derive(Debug, Clone)]
pub struct Symbol {
 /// Sign ID
 pub id: u32,
 /// Signname
 pub name: &'static str,
 /// SignType
 pub kind: SymbolKind,
 /// TypeInfo
 pub type_info: Option<TypeInfo>,
 /// makeuseFieldSheaflevel
 pub scope_level: u32,
 /// ifcanchange
 pub is_mut: bool,
 /// ifpublicopen
 pub is_pub: bool,
}

/// TypeInfo
#[derive(Debug, Clone)]
pub struct TypeInfo {
 /// Typename
 pub name: &'static str,
 /// TypeSize
 pub size: usize,
 /// TypeAlignment
 pub align: usize,
 /// ifasbaseType
 pub is_primitive: bool,
}

/// makeuseField
pub struct Scope {
 /// makeuseFieldSheaflevel
 pub level: u32,
 /// ParentmakeuseField
 pub parent: Option<u32>,
 /// Signform
 pub symbols: [Option<Symbol>; 64],
 /// Signcount
 pub num_symbols: u32,
}

impl Scope {
 pub const fn new(level: u32, parent: Option<u32>) -> Self {
 Scope {
 level,
 parent,
 symbols: [None; 64],
 num_symbols: 0,
 }
 }
 
 /// addSign
 pub fn add_symbol(&mut self, symbol: Symbol) -> bool {
 for slot in self.symbols.iter_mut() {
 if slot.is_none() {
 *slot = Some(symbol);
 self.num_symbols += 1;
 return true;
 }
 }
 false
 }
 
 /// FindSign
 pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
 for slot in self.symbols.iter() {
 if let Some(ref symbol) = slot {
 if symbol.name == name {
 return Some(symbol);
 }
 }
 }
 None
 }
}

/// Signform
pub struct SymbolTable {
 /// makeuseFieldArray
 scopes: [Option<Scope>; 32],
 /// makeuseFieldcount
 num_scopes: u32,
 /// CurrentmakeuseField
 current_scope: AtomicU32,
 /// NextSign ID
 next_symbol_id: AtomicU32,
}

impl SymbolTable {
 pub const fn new() -> Self {
 SymbolTable {
 scopes: [None; 32],
 num_scopes: 0,
 current_scope: AtomicU32::new(0),
 next_symbol_id: AtomicU32::new(1),
 }
 }
 
 /// Initialize
 pub fn init(&mut self) {
 // CreateGlobalmakeuseField
 self.push_scope(None);
 
 log_info!("Symbol table initialized");
 }
 
 /// EnternewmakeuseField
 pub fn push_scope(&mut self, parent: Option<u32>) -> u32 {
 let level = self.num_scopes;
 
 for (i, slot) in self.scopes.iter_mut().enumerate() {
 if slot.is_none() {
 *slot = Some(Scope::new(level, parent));
 self.num_scopes += 1;
 self.current_scope.store(i as u32, Ordering::Release);
 return i as u32;
 }
 }
 
 0
 }
 
 /// ExitmakeuseField
 pub fn pop_scope(&mut self) {
 let current = self.current_scope.load(Ordering::Acquire);
 
 if let Some(ref scope) = self.scopes.get(current as usize)? {
 if let Some(parent) = scope.parent {
 self.current_scope.store(parent, Ordering::Release);
 }
 }
 }
 
 /// addSign
 pub fn add_symbol(&mut self, name: &'static str, kind: SymbolKind, type_info: Option<TypeInfo>, is_mut: bool, is_pub: bool) -> Option<u32> {
 let current = self.current_scope.load(Ordering::Acquire);
 let symbol_id = self.next_symbol_id.fetch_add(1, Ordering::AcqRel);
 
 if let Some(ref mut scope) = self.scopes.get_mut(current as usize)? {
 let symbol = Symbol {
 id: symbol_id,
 name,
 kind,
 type_info,
 scope_level: scope.level,
 is_mut,
 is_pub,
 };
 
 if scope.add_symbol(symbol) {
 return Some(symbol_id);
 }
 }
 
 None
 }
 
 /// FindSign
 pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
 let current = self.current_scope.load(Ordering::Acquire);
 self.find_symbol_in_scope(name, current)
 }
 
 /// inexpfixedmakeuseFieldFindSign
 fn find_symbol_in_scope(&self, name: &str, scope_id: u32) -> Option<&Symbol> {
 if let Some(ref scope) = self.scopes.get(scope_id as usize)? {
 // inCurrentmakeuseFieldFind
 if let Some(symbol) = scope.find_symbol(name) {
 return Some(symbol);
 }
 
 // inParentmakeuseFieldFind
 if let Some(parent) = scope.parent {
 return self.find_symbol_in_scope(name, parent);
 }
 }
 
 None
 }
 
 /// GetCurrentmakeuseFieldSheaflevel
 pub fn get_current_level(&self) -> u32 {
 let current = self.current_scope.load(Ordering::Acquire);
 if let Some(ref scope) = self.scopes.get(current as usize)? {
 return scope.level;
 }
 0
 }
}

/// GlobalSignform
static mut SYMBOL_TABLE: SymbolTable = SymbolTable::new();

pub fn get_symbol_table() -> &'static mut SymbolTable {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut SYMBOL_TABLE }
}

pub fn init_symbol_table() {
 let table = get_symbol_table();
 table.init();
}