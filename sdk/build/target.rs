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

// ! buildtarget

use std::path::PathBuf;

/// buildtarget
#[derive(Debug, Clone)]
pub struct Target {
 /// targetname
 pub name: String,
 /// targetclasstype
 pub kind: TargetKind,
 /// sourcefilePath
 pub path: PathBuf,
 /// dependency
 pub dependencies: Vec<String>,
}

impl Target {
 pub fn new(name: impl Into<String>, kind: TargetKind, path: impl Into<PathBuf>) -> Self {
 Self {
 name: name.into(),
 kind,
 path: path.into(),
 dependencies: vec![],
 }
 }

 pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
 self.dependencies = deps;
 self
 }

 /// getoutputfilename
 pub fn output_name(&self) -> String {
 match self.kind {
 TargetKind::Lib => format!("lib{}.a", self.name),
 TargetKind::Bin => self.name.clone(),
 TargetKind::Test => format!("{}-test", self.name),
 TargetKind::Bench => format!("{}-bench", self.name),
 TargetKind::Example => format!("{}-example", self.name),
 }
 }

 /// iswhetherneedwantlink
 pub fn needs_link(&self) -> bool {
 matches!(self.kind, TargetKind::Bin | TargetKind::Test | TargetKind::Bench | TargetKind::Example)
 }
}

/// targetclasstype
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
 /// library
 Lib,
 /// entercontrol
 Bin,
 /// test
 Test,
 /// basecriteriontest
 Bench,
 /// example
 Example,
}

/// targetconfigure
#[derive(Debug, Clone)]
pub struct TargetConfig {
 /// targetname
 pub name: String,
 /// targetTuple
 pub triple: String,
 /// Architecture
 pub arch: String,
 /// Platform
 pub platform: String,
 /// compiledeviceflag
 pub cflags: Vec<String>,
 /// linkdeviceflag
 pub ldflags: Vec<String>,
}

impl TargetConfig {
 /// ARM64 target
 pub fn arm64() -> Self {
 Self {
 name: "arm64".to_string(),
 triple: "aarch64-nuva".to_string(),
 arch: "aarch64".to_string(),
 platform: "nuva".to_string(),
 cflags: vec![
 "-target".to_string(),
 "aarch64-nuva".to_string(),
 ],
 ldflags: vec![],
 }
 }

 /// x64 target
 pub fn x64() -> Self {
 Self {
 name: "x64".to_string(),
 triple: "x86_64-nuva".to_string(),
 arch: "x86_64".to_string(),
 platform: "nuva".to_string(),
 cflags: vec![
 "-target".to_string(),
 "x86_64-nuva".to_string(),
 ],
 ldflags: vec![],
 }
 }
}