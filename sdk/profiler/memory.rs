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

// ! memoryperformanceanalyzedevice

use std::collections::HashMap;
use crate::error::SdkError;
use alloc::vec;
use alloc::vec::Vec;

/// memoryanalyzedevice
pub struct MemProfiler {
 /// iswhetherpositiveinanalyze
 profiling: bool,
 /// targetprocess
 pid: Option<u32>,
 /// allocatelog
 allocations: Vec<Allocation>,
}

impl MemProfiler {
 pub fn new() -> Self {
 Self {
 profiling: false,
 pid: None,
 allocations: vec![],
 }
 }

 /// startanalyze
 pub fn start(&mut self, pid: Option<u32>) -> Result<(), SdkError> {
 self.profiling = true;
 self.pid = pid;
 self.allocations.clear();
 Ok(())
 }

 /// stopanalyze
 pub fn stop(&mut self) -> Result<MemProfile, SdkError> {
 self.profiling = false;
 
 // detectmemoryleakomission
 let leaks = self.detect_leaks();
 
 // statisticsallocateinformation
 let stats = self.compute_stats();
 
 Ok(MemProfile {
 allocations: std::mem::take(&mut self.allocations),
 leaks,
 stats,
 })
 }

 /// logallocate
 pub fn record_alloc(&mut self, alloc: Allocation) {
 if self.profiling {
 self.allocations.push(alloc);
 }
 }

 /// detectmemoryleakomission
 fn detect_leaks(&self) -> Vec<Leak> {
 let mut active_allocs: HashMap<u64, &Allocation> = HashMap::new();
 let mut leaks = vec![];
 
 for alloc in &self.allocations {
 match alloc.kind {
 AllocKind::Alloc => {
 active_allocs.insert(alloc.address, alloc);
 }
 AllocKind::Free => {
 active_allocs.remove(&alloc.address);
 }
 AllocKind::Realloc => {
 // Simplifiedprocess
 }
 }
 }
 
 for (_, alloc) in active_allocs {
 leaks.push(Leak {
 address: alloc.address,
 size: alloc.size,
 stack: alloc.stack.clone(),
 });
 }
 
 leaks
 }

 /// calculatestatisticsinformation
 fn compute_stats(&self) -> MemStats {
 let mut stats = MemStats::default();
 
 for alloc in &self.allocations {
 if alloc.kind == AllocKind::Alloc {
 stats.total_allocated += alloc.size;
 stats.alloc_count += 1;
 stats.peak_usage = stats.peak_usage.max(stats.current_usage + alloc.size);
 stats.current_usage += alloc.size;
 } else if alloc.kind == AllocKind::Free {
 stats.total_freed += alloc.size;
 stats.free_count += 1;
 stats.current_usage = stats.current_usage.saturating_sub(alloc.size);
 }
 }
 
 stats
 }
}

impl Default for MemProfiler {
 fn default() -> Self {
 Self::new()
 }
}

/// memoryanalyzeresult
#[derive(Debug)]
pub struct MemProfile {
 /// allocatelog
 pub allocations: Vec<Allocation>,
 /// memoryleakomission
 pub leaks: Vec<Leak>,
 /// statisticsinformation
 pub stats: MemStats,
}

/// allocatelog
#[derive(Debug, Clone)]
pub struct Allocation {
 /// address
 pub address: u64,
 /// size
 pub size: usize,
 /// allocateclasstype
 pub kind: AllocKind,
 /// time
 pub timestamp_us: u64,
 /// callstack
 pub stack: Vec<String>,
}

/// allocateclasstype
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocKind {
 Alloc,
 Free,
 Realloc,
}

/// memoryleakomission
#[derive(Debug, Clone)]
pub struct Leak {
 /// address
 pub address: u64,
 /// size
 pub size: usize,
 /// callstack
 pub stack: Vec<String>,
}

/// memorystatistics
#[derive(Debug, Default)]
pub struct MemStats {
 /// totalallocatequantification
 pub total_allocated: usize,
 /// totalreleasequantification
 pub total_freed: usize,
 /// currentmakeusequantification
 pub current_usage: usize,
 /// peakvaluemakeusequantification
 pub peak_usage: usize,
 /// allocatetimenumber
 pub alloc_count: usize,
 /// releasetimenumber
 pub free_count: usize,
}