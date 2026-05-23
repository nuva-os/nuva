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

// ! dependencyfixedmeaning

use super::meta::{Dependency, Version, VersionReq, DependencySource};

/// dependencydiagram
#[derive(Debug, Default)]
pub struct DependencyGraph {
 /// Node
 nodes: Vec<DepNode>,
 /// edge
 edges: Vec<DepEdge>,
}

impl DependencyGraph {
 pub fn new() -> Self {
 Self::default()
 }

 pub fn add_node(&mut self, node: DepNode) -> usize {
 let id = self.nodes.len();
 self.nodes.push(node);
 id
 }

 pub fn add_edge(&mut self, from: usize, to: usize) {
 self.edges.push(DepEdge { from, to });
 }

 pub fn nodes(&self) -> &[DepNode] {
 &self.nodes
 }

 pub fn edges(&self) -> &[DepEdge] {
 &self.edges
 }

 /// topologysort
 pub fn topological_sort(&self) -> Vec<usize> {
 let n = self.nodes.len();
 let mut in_degree = vec![0; n];
 let mut result = Vec::with_capacity(n);
 
 for edge in &self.edges {
 in_degree[edge.to] += 1;
 }
 
 let mut queue: Vec<usize> = in_degree.iter()
 .enumerate()
 .filter(|(_, &d)| d == 0)
 .map(|(i, _)| i)
 .collect();
 
 while let Some(node) = queue.pop() {
 result.push(node);
 
 for edge in self.edges.iter().filter(|e| e.from == node) {
 in_degree[edge.to] -= 1;
 if in_degree[edge.to] == 0 {
 queue.push(edge.to);
 }
 }
 }
 
 result
 }

 /// detectloopdependency
 pub fn detect_cycles(&self) -> Vec<Vec<usize>> {
 let n = self.nodes.len();
 let mut visited = vec![false; n];
 let mut rec_stack = vec![false; n];
 let mut cycles = Vec::new();
 
 for i in 0..n {
 if !visited[i] {
 let mut path = Vec::new();
 self.find_cycles_dfs(i, &mut visited, &mut rec_stack, &mut path, &mut cycles);
 }
 }
 
 cycles
 }

 fn find_cycles_dfs(
 &self,
 node: usize,
 visited: &mut Vec<bool>,
 rec_stack: &mut Vec<bool>,
 path: &mut Vec<usize>,
 cycles: &mut Vec<Vec<usize>>,
 ) {
 visited[node] = true;
 rec_stack[node] = true;
 path.push(node);
 
 for edge in self.edges.iter().filter(|e| e.from == node) {
 if !visited[edge.to] {
 self.find_cycles_dfs(edge.to, visited, rec_stack, path, cycles);
 } else if rec_stack[edge.to] {
 // findtoloop
 let cycle_start = path.iter().position(|&n| n == edge.to).unwrap();
 cycles.push(path[cycle_start..].to_vec());
 }
 }
 
 path.pop();
 rec_stack[node] = false;
 }
}

/// dependencyNode
#[derive(Debug, Clone)]
pub struct DepNode {
 pub name: String,
 pub version: Version,
 pub depth: usize,
}

/// dependencyedge
#[derive(Debug, Clone)]
pub struct DepEdge {
 pub from: usize,
 pub to: usize,
}

/// itydiagram
#[derive(Debug, Default)]
pub struct FeatureGraph {
 /// enable ity
 enabled_features: Vec<String>,
 /// itydependency
 feature_deps: std::collections::HashMap<String, Vec<String>>,
}

impl FeatureGraph {
 pub fn new() -> Self {
 Self::default()
 }

 pub fn enable_feature(&mut self, feature: &str) {
 if !self.enabled_features.contains(&feature.to_string()) {
 self.enabled_features.push(feature.to_string());
 }
 }

 pub fn add_feature_dep(&mut self, feature: &str, dep: &str) {
 self.feature_deps
 .entry(feature.to_string())
 .or_default()
 .push(dep.to_string());
 }

 pub fn resolve_features(&self) -> Vec<String> {
 let mut result = Vec::new();
 let mut visited = std::collections::HashSet::new();
 
 for feature in &self.enabled_features {
 self.resolve_feature_recursive(feature, &mut result, &mut visited);
 }
 
 result
 }

 fn resolve_feature_recursive(
 &self,
 feature: &str,
 result: &mut Vec<String>,
 visited: &mut std::collections::HashSet<String>,
 ) {
 if visited.contains(feature) {
 return;
 }
 
 visited.insert(feature.to_string());
 result.push(feature.to_string());
 
 if let Some(deps) = self.feature_deps.get(feature) {
 for dep in deps {
 self.resolve_feature_recursive(dep, result, visited);
 }
 }
 }
}