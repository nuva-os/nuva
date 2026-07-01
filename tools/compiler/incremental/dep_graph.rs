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

// ! dependencydiagram
/*!*/
// ! TracingFilebetween dependencyclosesystem

use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use alloc::vec;
use alloc::format;
use alloc::vec::Vec;

/// dependencydiagram
pub struct DependencyGraph {
 /// File -> dependency File
 dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
 /// File -> bydependency File(inversedirectiondependency)
 dependents: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl DependencyGraph {
 pub fn new() -> Self {
 Self {
 dependencies: HashMap::new(),
 dependents: HashMap::new(),
 }
 }

 /// addPlusdependencyclosesystem
 pub fn add_dependency(&mut self, from: &PathBuf, to: &PathBuf) {
 // addPluspositivedirectiondependency
 self.dependencies
 .entry(from.clone())
 .or_default()
 .insert(to.clone());

 // addPlusinversedirectiondependency
 self.dependents
 .entry(to.clone())
 .or_default()
 .insert(from.clone());
 }

 /// GetFile placefinitedependency
 pub fn get_dependencies(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
 self.dependencies.get(file)
 }

 /// GetdependencytheFile placefiniteFile
 pub fn get_dependents(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
 self.dependents.get(file)
 }

 /// Gettransmitdependency(placefinitebetweenacceptdependency)
 pub fn get_transitive_dependencies(&self, file: &PathBuf) -> HashSet<PathBuf> {
 let mut result = HashSet::new();
 let mut queue = vec![file.clone()];
 let mut visited = HashSet::new();

 while let Some(current) = queue.pop() {
 if visited.contains(&current) {
 continue;
 }
 visited.insert(current.clone());

 if let Some(deps) = self.dependencies.get(&current) {
 for dep in deps {
 result.insert(dep.clone());
 if !visited.contains(dep) {
 queue.push(dep.clone());
 }
 }
 }
 }

 result
 }

 /// Gettransmitdependencyer(placefinitebetweenacceptdependencytheFile File)
 pub fn get_transitive_dependents(&self, file: &PathBuf) -> HashSet<PathBuf> {
 let mut result = HashSet::new();
 let mut queue = vec![file.clone()];
 let mut visited = HashSet::new();

 while let Some(current) = queue.pop() {
 if visited.contains(&current) {
 continue;
 }
 visited.insert(current.clone());

 if let Some(deps) = self.dependents.get(&current) {
 for dep in deps {
 result.insert(dep.clone());
 if !visited.contains(dep) {
 queue.push(dep.clone());
 }
 }
 }
 }

 result
 }

 /// detectloopdependency
 pub fn detect_cycles(&self) -> Vec<Vec<PathBuf>> {
 let mut cycles = vec![];
 let mut visited = HashSet::new();
 let mut rec_stack = HashSet::new();

 for node in self.dependencies.keys() {
 let mut path = vec![];
 self.find_cycles_dfs(node, &mut visited, &mut rec_stack, &mut path, &mut cycles);
 }

 cycles
 }

 fn find_cycles_dfs(
 &self,
 node: &PathBuf,
 visited: &mut HashSet<PathBuf>,
 rec_stack: &mut HashSet<PathBuf>,
 path: &mut Vec<PathBuf>,
 cycles: &mut Vec<Vec<PathBuf>>,
 ) {
 if rec_stack.contains(node) {
 // findtoloop
 if let Some(start) = path.iter().position(|p| p == node) {
 cycles.push(path[start..].to_vec());
 }
 return;
 }

 if visited.contains(node) {
 return;
 }

 visited.insert(node.clone());
 rec_stack.insert(node.clone());
 path.push(node.clone());

 if let Some(deps) = self.dependencies.get(node) {
 for dep in deps {
 self.find_cycles_dfs(dep, visited, rec_stack, path, cycles);
 }
 }

 path.pop();
 rec_stack.remove(node);
 }

 /// topologysort
 pub fn topological_sort(&self) -> Vec<PathBuf> {
 let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();
 
 // Initializeentermeasurement
 for node in self.dependencies.keys() {
 in_degree.entry(node.clone()).or_insert(0);
 }
 
 for deps in self.dependencies.values() {
 for dep in deps {
 *in_degree.entry(dep.clone()).or_insert(0) += 1;
 }
 }

 // findexitentermeasurementas 0 Node
 let mut queue: Vec<PathBuf> = in_degree.iter()
 .filter(|(_, &d)| d == 0)
 .map(|(p, _)| p.clone())
 .collect();

 let mut result = vec![];

 while let Some(node) = queue.pop() {
 result.push(node.clone());

 if let Some(deps) = self.dependencies.get(&node) {
 for dep in deps {
 if let Some(d) = in_degree.get_mut(dep) {
 *d -= 1;
 if *d == 0 {
 queue.push(dep.clone());
 }
 }
 }
 }
 }

 result
 }

 /// clearemptydependencydiagram
 pub fn clear(&mut self) {
 self.dependencies.clear();
 self.dependents.clear();
 }

 /// GetNode count
 pub fn node_count(&self) -> usize {
 self.dependencies.len()
 }

 /// Getedgecount
 pub fn edge_count(&self) -> usize {
 self.dependencies.values().map(|d| d.len()).sum()
 }
}

impl Default for DependencyGraph {
 fn default() -> Self {
 Self::new()
 }
}

/// dependencyAnalysisdevice
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
 pub fn new() -> Self {
 Self
 }

 /// AnalysissourceFile dependency
 pub fn analyze(&self, source: &PathBuf) -> Result<Vec<PathBuf>, AnalyzeError> {
 let content = std::fs::read_to_string(source)
 .map_err(|e| AnalyzeError::IoError(e.to_string()))?;

 let mut deps = vec![];

 // simpleform dependencytake
 // TODO: Parse correctly using lexical analyzer
 for line in content.lines() {
 let line = line.trim();
 
 // Process #include or use languagesentence
 if line.starts_with("#include") {
 if let Some(path) = self.extract_include_path(line) {
 deps.push(source.parent().unwrap().join(path));
 }
 } else if line.starts_with("use ") || line.starts_with("mod ") {
 if let Some(path) = self.extract_module_path(line) {
 deps.push(source.parent().unwrap().join(path));
 }
 }
 }

 Ok(deps)
 }

 fn extract_include_path(&self, line: &str) -> Option<String> {
 // #include "path" or #include <path>
 let start = line.find('"').or_else(|| line.find('<'))?;
 let end = line[start + 1..].find('"').or_else(|| line[start + 1..].find('>'))?;
 Some(line[start + 1..start + 1 + end].to_string())
 }

 fn extract_module_path(&self, line: &str) -> Option<String> {
 // use path::module; or mod module;
 let parts: Vec<&str> = line.split_whitespace().collect();
 if parts.len() >= 2 {
 let path = parts[1].trim_end_matches(';');
 Some(format!("{}.nuva", path.replace("::", "/")))
 } else {
 None
 }
 }
}

impl Default for DependencyAnalyzer {
 fn default() -> Self {
 Self::new()
 }
}

/// AnalysisError
#[derive(Debug)]
pub enum AnalyzeError {
 IoError(String),
 ParseError(String),
}

impl std::fmt::Display for AnalyzeError {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 match self {
 AnalyzeError::IoError(msg) => write!(f, "IO error: {}", msg),
 AnalyzeError::ParseError(msg) => write!(f, "Parse error: {}", msg),
 }
 }
}

impl std::error::Error for AnalyzeError {}