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

// ! callstackOperation

/// stackframe
#[derive(Debug, Clone)]
pub struct StackFrame {
 /// frame ID
 pub id: u32,
 /// functionname
 pub function: String,
 /// sourcefile
 pub file: Option<String>,
 /// Line number
 pub line: Option<u32>,
 /// Column number
 pub column: Option<u32>,
 /// instructionaddress
 pub address: u64,
 /// stackpointer
 pub stack_pointer: u64,
 /// partvariable
 pub locals: Vec<StackVariable>,
}

impl StackFrame {
 pub fn new(id: u32, function: impl Into<String>, address: u64) -> Self {
 Self {
 id,
 function: function.into(),
 file: None,
 line: None,
 column: None,
 address,
 stack_pointer: 0,
 locals: vec![],
 }
 }

 pub fn with_source(mut self, file: impl Into<String>, line: u32, column: u32) -> Self {
 self.file = Some(file.into());
 self.line = Some(line);
 self.column = Some(column);
 self
 }

 pub fn with_locals(mut self, locals: Vec<StackVariable>) -> Self {
 self.locals = locals;
 self
 }
}

/// stackvariable
#[derive(Debug, Clone)]
pub struct StackVariable {
 /// variablename
 pub name: String,
 /// type
 pub var_type: String,
 /// value
 pub value: String,
 /// memoryaddress
 pub address: Option<u64>,
}

impl StackVariable {
 pub fn new(name: impl Into<String>, var_type: impl Into<String>, value: impl Into<String>) -> Self {
 Self {
 name: name.into(),
 var_type: var_type.into(),
 value: value.into(),
 address: None,
 }
 }
}

/// callstack
#[derive(Debug, Default)]
pub struct CallStack {
 /// stackframelist
 frames: Vec<StackFrame>,
}

impl CallStack {
 pub fn new() -> Self {
 Self::default()
 }

 pub fn push(&mut self, frame: StackFrame) {
 self.frames.push(frame);
 }

 pub fn pop(&mut self) -> Option<StackFrame> {
 self.frames.pop()
 }

 pub fn top(&self) -> Option<&StackFrame> {
 self.frames.last()
 }

 pub fn get(&self, id: u32) -> Option<&StackFrame> {
 self.frames.iter().find(|f| f.id == id)
 }

 pub fn frames(&self) -> &[StackFrame] {
 &self.frames
 }

 pub fn depth(&self) -> usize {
 self.frames.len()
 }
}