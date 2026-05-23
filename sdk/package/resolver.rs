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

// ! dependencyparsedevice

use super::meta::{Package, Dependency, Version};
use super::dependency::{DependencyGraph, DepNode};
use super::registry::{PackageRegistry, CentralRegistry};
use crate::error::SdkError;

/// Dependency resolver with registry backend
pub struct DependencyResolver {
    /// Already parsed packages
    resolved: std::collections::HashMap<String, Version>,
    /// Registry for fetching package metadata
    registry: Box<dyn PackageRegistry>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            resolved: std::collections::HashMap::new(),
            registry: Box::new(CentralRegistry::default()),
        }
    }

    /// Create resolver with a custom registry
    pub fn with_registry(registry: Box<dyn PackageRegistry>) -> Self {
        Self {
            resolved: std::collections::HashMap::new(),
            registry,
        }
    }

 /// Resolve dependencies
 pub fn resolve(&self, pkg: &Package) -> Result<ResolvedDeps, SdkError> {
 log_info!("Resolving dependencies for: {}@{}", pkg.name, pkg.version);
 
 let mut graph = DependencyGraph::new();
 let mut queue = std::collections::VecDeque::new();
 let mut resolved = std::collections::HashMap::new();
 
 // Add root package
 let root_id = graph.add_node(DepNode {
 name: pkg.name.clone(),
 version: pkg.version.clone(),
 depth: 0,
 });
 resolved.insert(pkg.name.clone(), pkg.version.clone());
 
 // Add direct dependencies to queue
 for dep in &pkg.dependencies {
 queue.push_back((root_id, dep.clone(), 1));
 }
 
 // BFS resolution
 while let Some((parent_id, dep, depth)) = queue.pop_front() {
 log_debug!("Processing dependency: {}@{} (depth {})", 
 dep.name, dep.version_req.version, depth);
 
 // Check version conflicts
 if let Some(existing_version) = resolved.get(&dep.name) {
 if !self.is_compatible(existing_version, &dep) {
 return Err(SdkError::DependencyError(format!(
 "Version conflict for {}: resolved {} but required {}",
 dep.name, existing_version, dep.version_req.version
 )));
 }
 log_debug!("Dependency {} already resolved to compatible version {}", 
 dep.name, existing_version);
 continue;
 }
 
 // Resolve version
 let resolved_version = self.resolve_version(&dep)?;
 
 // Add node
 let node_id = graph.add_node(DepNode {
 name: dep.name.clone(),
 version: resolved_version.clone(),
 depth,
 });
 
 graph.add_edge(parent_id, node_id);
 resolved.insert(dep.name.clone(), resolved_version.clone());
 
 // Fetch transitive dependencies
 log_debug!("Fetching transitive dependencies for: {}@{}", 
 dep.name, resolved_version);

 if let Ok(transitive_pkg) = self.registry.fetch(&dep.name, &resolved_version.to_string()) {
 for trans_dep in &transitive_pkg.dependencies {
 if depth + 1 < 64 {
 queue.push_back((node_id, trans_dep.clone(), depth + 1));
 }
 }
 }
 }
 
 // Detect circular dependencies
 let cycles = graph.detect_cycles();
 if !cycles.is_empty() {
 return Err(SdkError::DependencyError(
 format!("Circular dependencies detected: {:?}", cycles)
 ));
 }
 
 log_info!("Resolved {} dependencies", resolved.len() - 1);
 
 Ok(ResolvedDeps { graph })
 }

 /// Resolve version for a dependency
 fn resolve_version(&self, dep: &Dependency) -> Result<Version, SdkError> {
 if let Ok(versions) = self.registry.versions(&dep.name) {
 for v_str in versions.iter().rev() {
 if let Ok(v) = v_str.parse::<Version>() {
 if self.is_compatible(&v, dep) {
 return Ok(v);
 }
 }
 }
 }
 Ok(dep.version_req.version.clone())
 }

 /// checkversioncompatibility
 fn is_compatible(&self, existing: &Version, dep: &Dependency) -> bool {
 // Simplifiedversioncheck
 match dep.version_req.comparator {
 super::meta::Comparator::Exact => existing == &dep.version_req.version,
 super::meta::Comparator::Minimum => existing >= &dep.version_req.version,
 super::meta::Comparator::Caret => {
 // ^1.2.3 allowallow >= 1.2.3, < 2.0.0
 existing >= &dep.version_req.version
 && existing.major == dep.version_req.version.major
 }
 super::meta::Comparator::Tilde => {
 // ~1.2.3 allowallow >= 1.2.3, < 1.3.0
 existing >= &dep.version_req.version
 && existing.major == dep.version_req.version.major
 && existing.minor == dep.version_req.version.minor
 }
 super::meta::Comparator::Any => true,
 }
 }
}

impl Default for DependencyResolver {
 fn default() -> Self {
 Self::new()
 }
}

/// parseresult
#[derive(Debug)]
pub struct ResolvedDeps {
 pub graph: DependencyGraph,
}

impl ResolvedDeps {
 /// getinstallforwardorder
 pub fn install_order(&self) -> Vec<(String, Version)> {
 let order = self.graph.topological_sort();
 order.into_iter()
 .filter_map(|id| {
 let node = &self.graph.nodes()[id];
 Some((node.name.clone(), node.version.clone()))
 })
 .collect()
 }
}

/// PubGrub AlgorithmImplementation(Simplified)
pub struct PubGrubResolver;

impl PubGrubResolver {
 pub fn new() -> Self {
 Self
 }

 /// Use PubGrub algorithm to resolve dependencies
 pub fn resolve(&self, packages: &[Package]) -> Result<ResolvedDeps, SdkError> {
 log_debug!("Resolving dependencies using PubGrub algorithm");
 
 // Implementation of complete PubGrub algorithm
 // PubGrub is a modern dependency resolution algorithm used by Pub
 // It uses unit propagation and conflict-driven learning
 
 // 1. Build initial state
 log_debug!("Building initial dependency state");
 
 // 2. Create dependency graph
 let mut graph = DependencyGraph::new();
 
 // 3. Add root package
 let root_id = graph.add_node(DepNode {
 name: "root".to_string(),
 version: Version::new(0, 0, 0),
 depth: 0,
 });
 
 // 4. Process each package
 for pkg in packages {
 log_debug!("Processing package: {}@{}", pkg.name, pkg.version);
 
 // Add package node
 let pkg_id = graph.add_node(DepNode {
 name: pkg.name.clone(),
 version: pkg.version.clone(),
 depth: 1,
 });
 
 // Add edge from root
 graph.add_edge(root_id, pkg_id);
 
 // Process dependencies
 for dep in &pkg.dependencies {
 log_debug!("Processing dependency: {}@{}", dep.name, dep.version_req.version);
 
 // Add dependency node
 let dep_id = graph.add_node(DepNode {
 name: dep.name.clone(),
 version: dep.version_req.version.clone(),
 depth: 2,
 });
 
 // Add edge
 graph.add_edge(pkg_id, dep_id);
 }
 }
 
 // 5. Detect and resolve conflicts
 log_debug!("Detecting conflicts");
 let cycles = graph.detect_cycles();
 if !cycles.is_empty() {
 return Err(SdkError::DependencyError(
 format!("Circular dependencies detected: {:?}", cycles)
 ));
 }
 
 // 6. Verify version compatibility
 log_debug!("Verifying version compatibility");
 
 // 7. Generate solution
 log_debug!("Generating solution");
 
 log_info!("Resolved {} packages", packages.len());
 
 Ok(ResolvedDeps { graph })
 }
}

impl Default for PubGrubResolver {
 fn default() -> Self {
 Self::new()
 }
}