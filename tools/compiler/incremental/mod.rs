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

// ! increasequantificationencodingtranslateModule
/*!*/
// ! SupportonlyrepeatnewencodingtranslateModify partsplit, highencodingtranslateeffectrate

pub mod cache;
pub mod dep_graph;

use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Compilation error types
#[derive(Debug)]
pub enum CompileError {
 /// Compilation failed
 CompilationFailed(String),
 /// Compilation error with message
 CompilationError(String),
 /// Dependency error
 DependencyError(String),
 /// Unsupported file type
 UnsupportedFileType(String),
 /// IO error
 IoError(String),
 /// Cache error
 CacheError(String),
}

impl fmt::Display for CompileError {
 fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 match self {
 CompileError::CompilationFailed(msg) => write!(f, "Compilation failed: {}", msg),
 CompileError::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
 CompileError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
 CompileError::UnsupportedFileType(msg) => write!(f, "Unsupported file type: {}", msg),
 CompileError::IoError(msg) => write!(f, "IO error: {}", msg),
 CompileError::CacheError(msg) => write!(f, "Cache error: {}", msg),
 }
 }
}

impl std::error::Error for CompileError {}

/// increasequantificationencodingtranslatedevice
pub struct IncrementalCompiler {
 /// encodingtranslateCache
 cache: cache::CompilationCache,
 /// dependencydiagram
 dep_graph: dep_graph::DependencyGraph,
 /// alreadyencodingtranslate form
 compiled_units: HashMap<PathBuf, CompilationUnit>,
 /// Modifydetectdevice
 change_detector: ChangeDetector,
}

impl IncrementalCompiler {
 /// createnew increasequantificationencodingtranslatedevice
 pub fn new(cache_dir: PathBuf) -> Self {
 Self {
 cache: cache::CompilationCache::new(cache_dir),
 dep_graph: dep_graph::DependencyGraph::new(),
 compiled_units: HashMap::new(),
 change_detector: ChangeDetector::new(),
 }
 }

 /// Compile source files with incremental compilation
 pub fn compile(&mut self, source: &PathBuf) -> Result<CompilationResult, CompileError> {
 log_info!("Incremental compilation: {}", source.display());
 
 // 1. Check if up to date
 if self.is_up_to_date(source) {
 log_debug!("Using cached compilation for: {}", source.display());
 return Ok(CompilationResult::Cached {
 path: source.clone(),
 });
 }

 // 2. Analyze dependencies
 log_debug!("Analyzing dependencies for: {}", source.display());
 let analyzer = dep_graph::DependencyAnalyzer::new();
 let dependencies = analyzer.analyze(source)
 .map_err(|e| CompileError::DependencyError(format!("Failed to analyze dependencies: {}", e)))?;
 
 // Update dependency graph
 for dep in &dependencies {
 self.dep_graph.add_dependency(source, dep);
 }

 // 3. Get affected compilation units
 let affected = self.get_affected_units(source);
 log_debug!("Found {} affected units", affected.len());

 // 4. Recompile affected units
 let mut results = vec![];
 for unit in &affected {
 log_debug!("Compiling affected unit: {}", unit.display());
 let result = self.compile_unit(unit)?;
 results.push(result);
 }

 // 5. Update cache and dependency graph
 self.update_cache(source)?;

 log_info!("Incremental compilation completed: {} units recompiled", results.len());
 
 Ok(CompilationResult::Compiled {
 outputs: results,
 })
 }

 /// checkFileiswhetherismostnew
 fn is_up_to_date(&self, source: &PathBuf) -> bool {
 // checkCacheinfixiswhetherfinitetheFile
 if !self.cache.has(source) {
 return false;
 }

 // checkFileiswhetherbyModify
 if self.change_detector.is_modified(source) {
 return false;
 }

 // checkdependencyiswhetherbyModify
 if let Some(deps) = self.dep_graph.get_dependencies(source) {
 for dep in deps {
 if self.change_detector.is_modified(dep) {
 return false;
 }
 }
 }

 true
 }

 /// Get encodingtranslateform
 fn get_affected_units(&self, source: &PathBuf) -> Vec<PathBuf> {
 let mut affected = HashSet::new();
 affected.insert(source.clone());

 // GetplacefinitedependencytheFile form
 if let Some(dependents) = self.dep_graph.get_dependents(source) {
 for dep in dependents {
 affected.insert(dep.clone());
 }
 }

 affected.into_iter().collect()
 }

 /// Compile a single compilation unit
 fn compile_unit(&mut self, unit: &PathBuf) -> Result<PathBuf, CompileError> {
 let output = unit.with_extension("o");
 
 // Determine file type and invoke appropriate compiler
 let ext = unit.extension().and_then(|e| e.to_str()).unwrap_or("");
 
 log_debug!("Compiling unit: {} -> {}", unit.display(), output.display());
 
 match ext {
 "nuva" => self.compile_nuva_file(unit, &output)?,
 "rs" => self.compile_rust_file(unit, &output)?,
 "c" => self.compile_c_file(unit, &output)?,
 "cpp" | "cc" | "cxx" => self.compile_cpp_file(unit, &output)?,
 _ => {
 return Err(CompileError::UnsupportedFileType(format!(
 "Unsupported file type: {}", ext
 )));
 }
 }
 
 // Store compilation unit
 self.compiled_units.insert(unit.clone(), CompilationUnit {
 source: unit.clone(),
 output: output.clone(),
 timestamp: std::time::SystemTime::now(),
 });
 
 Ok(output)
 }

 /// Compile Nuva source file
 fn compile_nuva_file(&self, input: &PathBuf, output: &PathBuf) -> Result<(), CompileError> {
 log_debug!("Compiling Nuva file: {}", input.display());
 
 let mut cmd = std::process::Command::new("nuvac");
 cmd.arg("-c"); // Compile only
 cmd.arg("-o").arg(output);
 cmd.arg(input);
 
 let result = cmd.output()
 .map_err(|e| CompileError::CompilationFailed(format!("Failed to execute compiler: {}", e)))?;
 
 if !result.status.success() {
 let stderr = String::from_utf8_lossy(&result.stderr);
 return Err(CompileError::CompilationError(format!(
 "Compilation failed for {}: {}",
 input.display(),
 stderr
 )));
 }
 
 Ok(())
 }

 /// Compile Rust source file
 fn compile_rust_file(&self, input: &PathBuf, output: &PathBuf) -> Result<(), CompileError> {
 log_debug!("Compiling Rust file: {}", input.display());
 
 let mut cmd = std::process::Command::new("rustc");
 cmd.arg("--emit=obj");
 cmd.arg("-o").arg(output);
 cmd.arg(input);
 
 let result = cmd.output()
 .map_err(|e| CompileError::CompilationFailed(format!("Failed to execute rustc: {}", e)))?;
 
 if !result.status.success() {
 let stderr = String::from_utf8_lossy(&result.stderr);
 return Err(CompileError::CompilationError(format!(
 "Compilation failed for {}: {}",
 input.display(),
 stderr
 )));
 }
 
 Ok(())
 }

 /// Compile C source file
 fn compile_c_file(&self, input: &PathBuf, output: &PathBuf) -> Result<(), CompileError> {
 log_debug!("Compiling C file: {}", input.display());
 
 let mut cmd = std::process::Command::new("gcc");
 cmd.arg("-c");
 cmd.arg("-o").arg(output);
 cmd.arg(input);
 
 let result = cmd.output()
 .map_err(|e| CompileError::CompilationFailed(format!("Failed to execute gcc: {}", e)))?;
 
 if !result.status.success() {
 let stderr = String::from_utf8_lossy(&result.stderr);
 return Err(CompileError::CompilationError(format!(
 "Compilation failed for {}: {}",
 input.display(),
 stderr
 )));
 }
 
 Ok(())
 }

 /// Compile C++ source file
 fn compile_cpp_file(&self, input: &PathBuf, output: &PathBuf) -> Result<(), CompileError> {
 log_debug!("Compiling C++ file: {}", input.display());
 
 let mut cmd = std::process::Command::new("g++");
 cmd.arg("-c");
 cmd.arg("-o").arg(output);
 cmd.arg(input);
 
 let result = cmd.output()
 .map_err(|e| CompileError::CompilationFailed(format!("Failed to execute g++: {}", e)))?;
 
 if !result.status.success() {
 let stderr = String::from_utf8_lossy(&result.stderr);
 return Err(CompileError::CompilationError(format!(
 "Compilation failed for {}: {}",
 input.display(),
 stderr
 )));
 }
 
 Ok(())
 }

 /// UpdateCache
 fn update_cache(&mut self, source: &PathBuf) -> Result<(), CompileError> {
 self.cache.update(source)?;
 self.change_detector.mark_compiled(source);
 Ok(())
 }

 /// clearadministrationCache
 pub fn clean(&mut self) -> Result<(), CompileError> {
 self.cache.clear()?;
 self.compiled_units.clear();
 self.dep_graph.clear();
 Ok(())
 }

 /// GetCacheStatistics
 pub fn cache_stats(&self) -> CacheStats {
 CacheStats {
 entries: self.compiled_units.len(),
 hit_rate: self.cache.hit_rate(),
 }
 }
}

/// Compilation result
#[derive(Debug)]
pub enum CompilationResult {
 /// secondaryCachePlusload
 Cached {
 path: PathBuf,
 },
 /// repeatnewencodingtranslate
 Compiled {
 outputs: Vec<PathBuf>,
 },
}

/// encodingtranslateform
#[derive(Debug, Clone)]
pub struct CompilationUnit {
 /// sourceFile
 pub source: PathBuf,
 /// outputFile
 pub output: PathBuf,
 /// encodingtranslatetimebetween
 pub timestamp: std::time::SystemTime,
}

/// Modifydetectdevice
pub struct ChangeDetector {
 /// FileHash
 file_hashes: HashMap<PathBuf, String>,
 /// FileModification time
 file_mtimes: HashMap<PathBuf, std::time::SystemTime>,
}

impl ChangeDetector {
 pub fn new() -> Self {
 Self {
 file_hashes: HashMap::new(),
 file_mtimes: HashMap::new(),
 }
 }

 /// checkFileiswhetherbyModify
 pub fn is_modified(&self, path: &PathBuf) -> bool {
 // checkModification time
 if let Ok(metadata) = std::fs::metadata(path) {
 if let Ok(modified) = metadata.modified() {
 if let Some(&last_mtime) = self.file_mtimes.get(path) {
 return modified > last_mtime;
 }
 }
 }

 // if notfiniteRecord, asisnewFile
 !self.file_hashes.contains_key(path)
 }

 /// standardFilealreadyencodingtranslate
 pub fn mark_compiled(&mut self, path: &PathBuf) {
 if let Ok(metadata) = std::fs::metadata(path) {
 if let Ok(modified) = metadata.modified() {
 self.file_mtimes.insert(path.clone(), modified);
 }
 }

 // ComputeparallelexistHash
 if let Ok(hash) = self.compute_hash(path) {
 self.file_hashes.insert(path.clone(), hash);
 }
 }

 /// ComputeFileHash
 fn compute_hash(&self, path: &PathBuf) -> Result<String, std::io::Error> {
 use std::io::Read;
 
 let mut file = std::fs::File::open(path)?;
 let mut hasher = blake3::Hasher::new();
 let mut buffer = [0u8; 8192];
 
 loop {
 let bytes_read = file.read(&mut buffer)?;
 if bytes_read == 0 {
 break;
 }
 hasher.update(&buffer[..bytes_read]);
 }
 
 Ok(hasher.finalize().to_hex().to_string())
 }
}

impl Default for ChangeDetector {
 fn default() -> Self {
 Self::new()
 }
}

/// CacheStatistics
pub struct CacheStats {
 pub entries: usize,
 pub hit_rate: f64,
}