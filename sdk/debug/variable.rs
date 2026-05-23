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

// ! variableOperation

/// variable
#[derive(Debug, Clone)]
pub struct Variable {
 /// variablename
 pub name: String,
 /// variabletype
 pub var_type: VariableType,
 /// variablevalue
 pub value: VariableValue,
 /// iswhethercanmodify
 pub writable: bool,
 /// childvariable(usestruct/Array)
 pub children: Vec<Variable>,
}

impl Default for Variable {
 fn default() -> Self {
 Self {
 name: String::new(),
 var_type: VariableType::Unknown,
 value: VariableValue::Void,
 writable: false,
 children: vec![],
 }
 }
}

impl Variable {
 pub fn new(name: impl Into<String>, var_type: VariableType, value: VariableValue) -> Self {
 Self {
 name: name.into(),
 var_type,
 value,
 writable: true,
 children: vec![],
 }
 }

 pub fn with_children(mut self, children: Vec<Variable>) -> Self {
 self.children = children;
 self
 }

 pub fn readonly(mut self) -> Self {
 self.writable = false;
 self
 }
}

/// variabletype
#[derive(Debug, Clone)]
pub enum VariableType {
 Unknown,
 Void,
 Bool,
 Int { bits: u32, signed: bool },
 Float { bits: u32 },
 Pointer { pointee: Box<VariableType> },
 Array { element: Box<VariableType>, size: usize },
 Struct { name: String },
 Enum { name: String },
 String,
}

/// variablevalue
#[derive(Debug, Clone)]
pub enum VariableValue {
 Void,
 Bool(bool),
 Int(i64),
 UInt(u64),
 Float(f64),
 Pointer(u64),
 String(String),
 Bytes(Vec<u8>),
 Address(u64),
}

impl VariableValue {
 pub fn to_string_repr(&self) -> String {
 match self {
 VariableValue::Void => "void".to_string(),
 VariableValue::Bool(b) => b.to_string(),
 VariableValue::Int(i) => i.to_string(),
 VariableValue::UInt(u) => u.to_string(),
 VariableValue::Float(f) => f.to_string(),
 VariableValue::Pointer(p) => format!("0x{:016x}", p),
 VariableValue::String(s) => format!("\"{}\"", s),
 VariableValue::Bytes(b) => format!("{:02x?}", b),
 VariableValue::Address(a) => format!("0x{:016x}", a),
 }
 }
}

/// variablereference
#[derive(Debug, Clone)]
pub struct VariableReference {
 /// variable ID
 pub id: u32,
 /// variablename
 pub name: String,
 /// memoryaddress
 pub address: Option<u64>,
}