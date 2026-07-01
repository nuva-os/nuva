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

// ! buildschedulingdevice

use std::collections::{HashMap, VecDeque};
use crate::error::SdkError;
use super::config::BuildConfig;
use super::target::Target;
use super::executor::BuildExecutor;
use super::{BuildResult, TargetKind};
use alloc::vec;
use alloc::vec::Vec;

/// buildschedulingdevice
pub struct BuildScheduler {
 /// Paralleltasknumber
 parallel_jobs: usize,
 /// executedevice
 executor: BuildExecutor,
}

impl BuildScheduler {
 pub fn new(config: &BuildConfig) -> Self {
 Self {
 parallel_jobs: 4, // TODO: secondaryconfigureread
 executor: BuildExecutor::new(config),
 }
 }

 /// builddependencydiagram
 pub fn build_dependency_graph(&self, target: &Target) -> Result<DependencyGraph, SdkError> {
 let mut graph = DependencyGraph::new();
 
 // addtargetNode
 let target_id = graph.add_node(BuildNode {
 name: target.name.clone(),
 kind: target.kind,
 path: target.path.clone(),
 state: BuildState::Pending,
 });
 
 // TODO: scansourcefiledependency
 self.scan_dependencies(&mut graph, target, target_id)?;
 
 Ok(graph)
 }

 /// scandependency
 fn scan_dependencies(
 &self,
 graph: &mut DependencyGraph,
 target: &Target,
 parent_id: usize,
 ) -> Result<(), SdkError> {
 // scansourcefile
 if target.path.is_dir() {
 for entry in walkdir::WalkDir::new(&target.path)
 .into_iter()
 .filter_map(|e| e.ok())
 {
 if entry.file_type().is_file() {
 let path = entry.path();
 if let Some(ext) = path.extension() {
 if ext == "nuva" || ext == "rs" {
 let node_id = graph.add_node(BuildNode {
 name: path.to_string_lossy().to_string(),
 kind: TargetKind::Lib,
 path: path.to_path_buf(),
 state: BuildState::Pending,
 });
 graph.add_edge(parent_id, node_id);
 }
 }
 }
 }
 } else if target.path.is_file() {
 let node_id = graph.add_node(BuildNode {
 name: target.path.to_string_lossy().to_string(),
 kind: target.kind,
 path: target.path.clone(),
 state: BuildState::Pending,
 });
 graph.add_edge(parent_id, node_id);
 }
 
 Ok(())
 }

 /// executebuild
 pub fn execute(&mut self, graph: &DependencyGraph) -> Result<BuildResult, SdkError> {
 // topologysort
 let order = graph.topological_sort();
 
 // executebuildtask
 let mut outputs = vec![];
 let start = std::time::Instant::now();
 
 for node_id in order {
 let node = graph.get_node(node_id);
 
 match self.executor.execute(node)? {
 BuildResult::Success { outputs: o, .. } => {
 outputs.extend(o);
 }
 BuildResult::Failed { errors } => {
 return Ok(BuildResult::Failed { errors });
 }
 BuildResult::Cached => {}
 }
 }
 
 Ok(BuildResult::Success {
 outputs,
 compile_time_ms: start.elapsed().as_millis() as u64,
 })
 }
}

/// dependencydiagram
#[derive(Debug, Default)]
pub struct DependencyGraph {
 nodes: Vec<BuildNode>,
 edges: Vec<(usize, usize)>,
}

impl DependencyGraph {
 pub fn new() -> Self {
 Self::default()
 }

 pub fn add_node(&mut self, node: BuildNode) -> usize {
 let id = self.nodes.len();
 self.nodes.push(node);
 id
 }

 pub fn add_edge(&mut self, from: usize, to: usize) {
 self.edges.push((from, to));
 }

 pub fn get_node(&self, id: usize) -> &BuildNode {
 &self.nodes[id]
 }

 pub fn topological_sort(&self) -> Vec<usize> {
 let n = self.nodes.len();
 let mut in_degree = vec![0; n];
 let mut result = Vec::with_capacity(n);
 
 for (_, to) in &self.edges {
 in_degree[*to] += 1;
 }
 
 let mut queue: VecDeque<usize> = in_degree.iter()
 .enumerate()
 .filter(|(_, &d)| d == 0)
 .map(|(i, _)| i)
 .collect();
 
 while let Some(node) = queue.pop_front() {
 result.push(node);
 
 for (from, to) in self.edges.iter().filter(|(f, _)| *f == node) {
 in_degree[*to] -= 1;
 if in_degree[*to] == 0 {
 queue.push_back(*to);
 }
 }
 }
 
 result
 }
}

/// buildNode
#[derive(Debug, Clone)]
pub struct BuildNode {
 pub name: String,
 pub kind: TargetKind,
 pub path: std::path::PathBuf,
 pub state: BuildState,
}

/// buildstate
#[derive(Debug, Clone, Copy)]
pub enum BuildState {
 Pending,
 Building,
 Completed,
 Failed,
}