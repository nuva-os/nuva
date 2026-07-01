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

// ! CPU performanceanalyzedevice

use std::collections::HashMap;
use crate::error::SdkError;
use alloc::vec;
use alloc::vec::Vec;

/// CPU analyzedevice
pub struct CpuProfiler {
 /// iswhetherpositiveinanalyze
 profiling: bool,
 /// targetprocess
 pid: Option<u32>,
 /// patterndata
 samples: Vec<Sample>,
}

impl CpuProfiler {
 pub fn new() -> Self {
 Self {
 profiling: false,
 pid: None,
 samples: vec![],
 }
 }

 /// startanalyze
 pub fn start(&mut self, pid: Option<u32>) -> Result<(), SdkError> {
 self.profiling = true;
 self.pid = pid;
 self.samples.clear();
 Ok(())
 }

 /// stopanalyze
 pub fn stop(&mut self) -> Result<CpuProfile, SdkError> {
 self.profiling = false;
 
 // buildcalldiagram
 let call_graph = self.build_call_graph();
 
 // statisticsfunctioninformation
 let functions = self.compute_function_stats();
 
 Ok(CpuProfile {
 samples: std::mem::take(&mut self.samples),
 call_graph,
 functions,
 })
 }

 /// addpattern
 pub fn add_sample(&mut self, sample: Sample) {
 if self.profiling {
 self.samples.push(sample);
 }
 }

 /// buildcalldiagram
 fn build_call_graph(&self) -> CallGraph {
 let mut graph = CallGraph::new();
 
 for sample in &self.samples {
 let mut parent = None;
 for frame in &sample.stack {
 let node_id = graph.add_or_get_node(&frame.function, frame.address);
 if let Some(p) = parent {
 graph.add_edge(p, node_id);
 }
 parent = Some(node_id);
 }
 }
 
 graph
 }

 /// calculatefunctionstatistics
 fn compute_function_stats(&self) -> HashMap<String, FunctionStats> {
 let mut stats = HashMap::new();
 
 for sample in &self.samples {
 if let Some(frame) = sample.stack.first() {
 let entry = stats.entry(frame.function.clone()).or_default();
 entry.sample_count += 1;
 entry.total_time_us += sample.timestamp_us;
 }
 }
 
 stats
 }
}

impl Default for CpuProfiler {
 fn default() -> Self {
 Self::new()
 }
}

/// CPU analyzeresult
#[derive(Debug)]
pub struct CpuProfile {
 /// patterndata
 pub samples: Vec<Sample>,
 /// calldiagram
 pub call_graph: CallGraph,
 /// functionstatistics
 pub functions: HashMap<String, FunctionStats>,
}

/// patterndata
#[derive(Debug, Clone)]
pub struct Sample {
 /// time(microsecond)
 pub timestamp_us: u64,
 /// callstack
 pub stack: Vec<StackFrame>,
 /// thread ID
 pub thread_id: u64,
}

/// stackframe
#[derive(Debug, Clone)]
pub struct StackFrame {
 /// functionname
 pub function: String,
 /// address
 pub address: u64,
 /// sourcefile
 pub file: Option<String>,
 /// Line number
 pub line: Option<u32>,
}

/// calldiagram
#[derive(Debug, Default)]
pub struct CallGraph {
 /// Node
 nodes: Vec<CallNode>,
 /// edge
 edges: Vec<(usize, usize)>,
 /// Nodeindex
 node_index: HashMap<String, usize>,
}

impl CallGraph {
 pub fn new() -> Self {
 Self::default()
 }

 pub fn add_or_get_node(&mut self, name: &str, address: u64) -> usize {
 if let Some(&id) = self.node_index.get(name) {
 return id;
 }
 
 let id = self.nodes.len();
 self.nodes.push(CallNode {
 name: name.to_string(),
 address,
 weight: 0,
 });
 self.node_index.insert(name.to_string(), id);
 id
 }

 pub fn add_edge(&mut self, from: usize, to: usize) {
 self.edges.push((from, to));
 }

 pub fn nodes(&self) -> &[CallNode] {
 &self.nodes
 }

 pub fn edges(&self) -> &[(usize, usize)] {
 &self.edges
 }
}

/// calldiagramNode
#[derive(Debug, Clone)]
pub struct CallNode {
 pub name: String,
 pub address: u64,
 pub weight: usize,
}

/// functionstatistics
#[derive(Debug, Default)]
pub struct FunctionStats {
 /// patterntimenumber
 pub sample_count: usize,
 /// totaltime(microsecond)
 pub total_time_us: u64,
}