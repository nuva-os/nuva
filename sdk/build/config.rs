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

// ! buildconfigure

use std::path::PathBuf;
use std::collections::HashMap;
use crate::error::SdkError;
use super::target::{Target, TargetKind};
use alloc::vec;
use alloc::vec::Vec;

/// buildconfigure
#[derive(Debug, Clone)]
pub struct BuildConfig {
 /// projectname
 pub name: String,
 /// projectversion
 pub version: String,
 /// targetlist
 pub targets: Vec<Target>,
 /// ity
 pub features: HashMap<String, Vec<String>>,
 /// defaultity
 pub default_features: Vec<String>,
 /// optimizationetclevel
 pub opt_level: OptLevel,
 /// debuginformation
 pub debug_info: bool,
 /// LTO
 pub lto: bool,
 /// targetArchitecture
 pub target_arch: Option<String>,
 /// targetTuple
 pub target_triple: Option<String>,
 /// outputdirectory
 pub out_dir: PathBuf,
}

impl BuildConfig {
 /// secondary Nuva.toml load
 pub fn from_file(path: &PathBuf) -> Result<Self, SdkError> {
 let content = std::fs::read_to_string(path)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 let config: BuildConfigToml = toml::from_str(&content)
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 Ok(config.into_build_config(path.parent().unwrap_or(&PathBuf::from("."))))
 }

 /// gettarget
 pub fn get_target(&self, name: &str) -> Option<&Target> {
 self.targets.iter().find(|t| t.name == name)
 }

 /// getdefaulttarget
 pub fn default_target(&self) -> Option<&Target> {
 // advantagefirstreturnlibrarytarget, thenthenisentercontroltarget
 self.targets.iter()
 .find(|t| t.kind == TargetKind::Lib)
 .or_else(|| self.targets.iter().find(|t| t.kind == TargetKind::Bin))
 }
}

impl Default for BuildConfig {
 fn default() -> Self {
 Self {
 name: "unnamed".to_string(),
 version: "0.1.0".to_string(),
 targets: vec![],
 features: HashMap::new(),
 default_features: vec![],
 opt_level: OptLevel::Debug,
 debug_info: true,
 lto: false,
 target_arch: None,
 target_triple: None,
 out_dir: PathBuf::from("target"),
 }
 }
}

/// optimizationetclevel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
 /// infiniteoptimization
 Debug,
 /// basebookoptimization
 Opt1,
 /// standardcriterionoptimization
 Opt2,
 /// enteroptimization
 Opt3,
 /// optimizationsize
 OptSize,
 /// optimizationsize(enter)
 OptSizeZ,
}

impl OptLevel {
 pub fn from_str(s: &str) -> Self {
 match s {
 "0" | "debug" => OptLevel::Debug,
 "1" => OptLevel::Opt1,
 "2" => OptLevel::Opt2,
 "3" => OptLevel::Opt3,
 "s" => OptLevel::OptSize,
 "z" => OptLevel::OptSizeZ,
 _ => OptLevel::Debug,
 }
 }

 pub fn to_flag(&self) -> &'static str {
 match self {
 OptLevel::Debug => "-O0",
 OptLevel::Opt1 => "-O1",
 OptLevel::Opt2 => "-O2",
 OptLevel::Opt3 => "-O3",
 OptLevel::OptSize => "-Os",
 OptLevel::OptSizeZ => "-Oz",
 }
 }
}

// TOML configurestruct
#[derive(Debug, serde::Deserialize)]
struct BuildConfigToml {
 package: PackageSection,
 features: Option<HashMap<String, Vec<String>>>,
 lib: Option<TargetSection>,
 bin: Option<Vec<TargetSection>>,
 test: Option<Vec<TargetSection>>,
 bench: Option<Vec<TargetSection>>,
 example: Option<Vec<TargetSection>>,
}

#[derive(Debug, serde::Deserialize)]
struct PackageSection {
 name: String,
 version: String,
}

#[derive(Debug, serde::Deserialize)]
struct TargetSection {
 name: Option<String>,
 path: Option<String>,
}

impl BuildConfigToml {
 fn into_build_config(self, root: &Path) -> BuildConfig {
 let mut targets = vec![];
 
 // librarytarget
 if let Some(lib) = self.lib {
 targets.push(Target {
 name: lib.name.unwrap_or_else(|| self.package.name.clone()),
 kind: TargetKind::Lib,
 path: lib.path.map(|p| root.join(p)).unwrap_or_else(|| root.join("src/lib.nuva")),
 dependencies: vec![],
 });
 }
 
 // entercontroltarget
 if let Some(bins) = self.bin {
 for bin in bins {
 targets.push(Target {
 name: bin.name.unwrap_or_else(|| self.package.name.clone()),
 kind: TargetKind::Bin,
 path: bin.path.map(|p| root.join(p)).unwrap_or_else(|| root.join("src/main.nuva")),
 dependencies: vec![],
 });
 }
 }
 
 // testtarget
 if let Some(tests) = self.test {
 for test in tests {
 targets.push(Target {
 name: test.name.unwrap_or_else(|| "test".to_string()),
 kind: TargetKind::Test,
 path: test.path.map(|p| root.join(p)).unwrap_or_else(|| root.join("tests")),
 dependencies: vec![],
 });
 }
 }
 
 BuildConfig {
 name: self.package.name,
 version: self.package.version,
 targets,
 features: self.features.unwrap_or_default(),
 default_features: vec![],
 opt_level: OptLevel::Debug,
 debug_info: true,
 lto: false,
 target_arch: None,
 target_triple: None,
 out_dir: root.join("target"),
 }
 }
}