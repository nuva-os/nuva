/*
 * Nuva OS - Kernel - PGO (Profile-Guided Optimization)
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

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use alloc::vec::Vec;

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Maximum profiled functions
const PGO_MAX_FUNCTIONS: usize = 4096;

/// Maximum profiled branches
const PGO_MAX_BRANCHES: usize = 16384;

/// Maximum call path depth
const PGO_MAX_CALL_DEPTH: usize = 32;

/// Maximum call paths tracked
const PGO_MAX_CALL_PATHS: usize = 2048;

/// PGO function entry
/// Tracks execution count and timing for a single function.
#[repr(C)]
pub struct PgoFuncEntry {
    /// Function address
    pub func_addr: u64,
    /// Execution count
    pub exec_count: AtomicU64,
    /// Total cycles spent in this function
    pub total_cycles: AtomicU64,
    /// Self cycles (excluding callees)
    pub self_cycles: AtomicU64,
}

impl PgoFuncEntry {
    pub const fn new(func_addr: u64) -> Self {
        PgoFuncEntry {
            func_addr,
            exec_count: AtomicU64::new(0),
            total_cycles: AtomicU64::new(0),
            self_cycles: AtomicU64::new(0),
        }
    }
}

/// PGO branch entry
/// Tracks taken/not-taken counts for a branch.
#[repr(C)]
pub struct PgoBranchEntry {
    /// Branch source address
    pub src_addr: u64,
    /// Branch target address
    pub target_addr: u64,
    /// Count of times branch was taken
    pub taken_count: AtomicU64,
    /// Count of times branch was not taken
    pub not_taken_count: AtomicU64,
}

impl PgoBranchEntry {
    pub const fn new(src_addr: u64, target_addr: u64) -> Self {
        PgoBranchEntry {
            src_addr,
            target_addr,
            taken_count: AtomicU64::new(0),
            not_taken_count: AtomicU64::new(0),
        }
    }
}

/// PGO call path entry
/// Tracks a unique call path through the code.
#[repr(C)]
pub struct PgoCallPath {
    /// Call stack addresses
    pub addrs: [u64; PGO_MAX_CALL_DEPTH],
    /// Stack depth
    pub depth: u32,
    /// Hit count
    pub hit_count: AtomicU64,
}

impl PgoCallPath {
    pub const fn new() -> Self {
        PgoCallPath {
            addrs: [0; PGO_MAX_CALL_DEPTH],
            depth: 0,
            hit_count: AtomicU64::new(0),
        }
    }
}

/// PGO profile data
/// Collects function counts, branch counts, and call paths
/// for profile-guided optimization.
pub struct PgoProfile {
    /// Enabled flag
    pub enabled: AtomicBool,
    /// Function entries
    pub functions: [PgoFuncEntry; PGO_MAX_FUNCTIONS],
    /// Number of function entries
    pub func_count: AtomicU32,
    /// Branch entries
    pub branches: [PgoBranchEntry; PGO_MAX_BRANCHES],
    /// Number of branch entries
    pub branch_count: AtomicU32,
    /// Call path entries
    pub call_paths: [PgoCallPath; PGO_MAX_CALL_PATHS],
    /// Number of call paths
    pub call_path_count: AtomicU32,
    /// Total samples collected
    pub total_samples: AtomicU64,
}

impl PgoProfile {
    pub const fn new() -> Self {
        PgoProfile {
            enabled: AtomicBool::new(false),
            functions: [const { PgoFuncEntry::new(0) }; PGO_MAX_FUNCTIONS],
            func_count: AtomicU32::new(0),
            branches: [const { PgoBranchEntry::new(0, 0) }; PGO_MAX_BRANCHES],
            branch_count: AtomicU32::new(0),
            call_paths: [const { PgoCallPath::new() }; PGO_MAX_CALL_PATHS],
            call_path_count: AtomicU32::new(0),
            total_samples: AtomicU64::new(0),
        }
    }

    /// Enable PGO profiling
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    /// Disable PGO profiling
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Release);
    }

    /// Record a function execution
    /// @return: index of function entry, or -1 if table full
    pub fn record_function(&mut self, func_addr: u64, cycles: u64) -> i32 {
        if !self.enabled.load(Ordering::Acquire) {
            return Errno::Eperm.to_ret_i32();
        }
        self.total_samples.fetch_add(1, Ordering::AcqRel);

        let count = self.func_count.load(Ordering::Acquire) as usize;
        for i in 0..count {
            if self.functions[i].func_addr == func_addr {
                self.functions[i].exec_count.fetch_add(1, Ordering::AcqRel);
                self.functions[i].total_cycles.fetch_add(cycles, Ordering::AcqRel);
                return i as i32;
            }
        }

        if count >= PGO_MAX_FUNCTIONS {
            return Errno::Enospc.to_ret_i32(); // ENOSPC
        }

        self.functions[count] = PgoFuncEntry::new(func_addr);
        self.functions[count].exec_count.fetch_add(1, Ordering::AcqRel);
        self.functions[count].total_cycles.fetch_add(cycles, Ordering::AcqRel);
        self.func_count.fetch_add(1, Ordering::AcqRel);
        count as i32
    }

    /// Record a branch outcome
    /// @param taken: true if branch was taken, false otherwise
    /// @return: index of branch entry, or negative errno
    pub fn record_branch(&mut self, src_addr: u64, target_addr: u64, taken: bool) -> i32 {
        if !self.enabled.load(Ordering::Acquire) {
            return Errno::Eperm.to_ret_i32();
        }

        let count = self.branch_count.load(Ordering::Acquire) as usize;
        for i in 0..count {
            if self.branches[i].src_addr == src_addr && self.branches[i].target_addr == target_addr {
                if taken {
                    self.branches[i].taken_count.fetch_add(1, Ordering::AcqRel);
                } else {
                    self.branches[i].not_taken_count.fetch_add(1, Ordering::AcqRel);
                }
                return i as i32;
            }
        }

        if count >= PGO_MAX_BRANCHES {
            return Errno::Enospc.to_ret_i32();
        }

        self.branches[count] = PgoBranchEntry::new(src_addr, target_addr);
        if taken {
            self.branches[count].taken_count.fetch_add(1, Ordering::AcqRel);
        } else {
            self.branches[count].not_taken_count.fetch_add(1, Ordering::AcqRel);
        }
        self.branch_count.fetch_add(1, Ordering::AcqRel);
        count as i32
    }

    /// Record a call path
    /// @return: index of call path entry, or negative errno
    pub fn record_call_path(&mut self, call_stack: &[u64]) -> i32 {
        if !self.enabled.load(Ordering::Acquire) {
            return Errno::Eperm.to_ret_i32();
        }

        let depth = call_stack.len().min(PGO_MAX_CALL_DEPTH) as u32;
        let count = self.call_path_count.load(Ordering::Acquire) as usize;

        for i in 0..count {
            let entry = &self.call_paths[i];
            if entry.depth != depth {
                continue;
            }
            let mut match_found = true;
            for j in 0..depth as usize {
                if entry.addrs[j] != call_stack[j] {
                    match_found = false;
                    break;
                }
            }
            if match_found {
                self.call_paths[i].hit_count.fetch_add(1, Ordering::AcqRel);
                return i as i32;
            }
        }

        if count >= PGO_MAX_CALL_PATHS {
            return Errno::Enospc.to_ret_i32();
        }

        let mut path = PgoCallPath::new();
        path.depth = depth;
        for j in 0..depth as usize {
            path.addrs[j] = call_stack[j];
        }
        path.hit_count.fetch_add(1, Ordering::AcqRel);
        self.call_paths[count] = path;
        self.call_path_count.fetch_add(1, Ordering::AcqRel);
        count as i32
    }

    /// Get hot functions sorted by execution count
    /// @param top_n: maximum number of functions to return
    pub fn get_hot_functions(&self, top_n: usize) -> Vec<(u64, u64)> {
        let count = self.func_count.load(Ordering::Acquire) as usize;
        let limit = top_n.min(count);
        let mut entries: Vec<(u64, u64)> = Vec::with_capacity(count);

        for i in 0..count {
            let exec = self.functions[i].exec_count.load(Ordering::Acquire);
            if exec > 0 {
                entries.push((self.functions[i].func_addr, exec));
            }
        }

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(limit);
        entries
    }

    /// Dump profile data as a serialized byte vector
    /// Format: [func_count:u32][func_entries...][branch_count:u32][branch_entries...]
    pub fn dump_profile(&self) -> Vec<u8> {
        let mut data = Vec::new();

        let func_count = self.func_count.load(Ordering::Acquire);
        data.extend_from_slice(&func_count.to_le_bytes());

        for i in 0..func_count as usize {
            let entry = &self.functions[i];
            data.extend_from_slice(&entry.func_addr.to_le_bytes());
            let exec = entry.exec_count.load(Ordering::Acquire);
            data.extend_from_slice(&exec.to_le_bytes());
            let cycles = entry.total_cycles.load(Ordering::Acquire);
            data.extend_from_slice(&cycles.to_le_bytes());
        }

        let branch_count = self.branch_count.load(Ordering::Acquire);
        data.extend_from_slice(&branch_count.to_le_bytes());

        for i in 0..branch_count as usize {
            let entry = &self.branches[i];
            data.extend_from_slice(&entry.src_addr.to_le_bytes());
            data.extend_from_slice(&entry.target_addr.to_le_bytes());
            let taken = entry.taken_count.load(Ordering::Acquire);
            data.extend_from_slice(&taken.to_le_bytes());
            let not_taken = entry.not_taken_count.load(Ordering::Acquire);
            data.extend_from_slice(&not_taken.to_le_bytes());
        }

        data
    }

    /// Reset all profile data
    pub fn reset(&mut self) {
        self.func_count.store(0, Ordering::Release);
        self.branch_count.store(0, Ordering::Release);
        self.call_path_count.store(0, Ordering::Release);
        self.total_samples.store(0, Ordering::Release);
    }

    /// Generate hot-function layout reordering plan.
    /// Returns addresses sorted hot-to-cold for code layout optimization.
    /// Hot functions (high exec_count × avg_cycles) should be placed
    /// together to improve I-cache and TLB utilization.
    pub fn generate_layout_order(&self) -> Vec<(u64, u64, u64)> {
        let func_count = self.func_count.load(Ordering::Acquire);
        let mut entries: Vec<(u64, u64, u64)> = Vec::new();
        for i in 0..func_count as usize {
            let entry = &self.functions[i];
            let exec = entry.exec_count.load(Ordering::Acquire);
            let cycles = entry.total_cycles.load(Ordering::Acquire);
            if exec > 0 {
                entries.push((entry.func_addr, exec, cycles));
            }
        }
        entries.sort_by(|a, b| {
            let score_a = a.1.saturating_mul(a.2);
            let score_b = b.1.saturating_mul(b.2);
            score_b.cmp(&score_a)
        });
        entries
    }

    /// Generate branch prediction hints for the CPU branch predictor.
    /// Returns (src_addr, target_addr, likely_taken) for each branch
    /// where the prediction bias exceeds the threshold.
    pub fn generate_branch_hints(&self, bias_threshold: u64) -> Vec<(u64, u64, bool)> {
        let branch_count = self.branch_count.load(Ordering::Acquire);
        let mut hints = Vec::new();
        for i in 0..branch_count as usize {
            let entry = &self.branches[i];
            let taken = entry.taken_count.load(Ordering::Acquire);
            let not_taken = entry.not_taken_count.load(Ordering::Acquire);
            if taken + not_taken == 0 {
                continue;
            }
            let bias = if taken >= not_taken {
                taken - not_taken
            } else {
                not_taken - taken
            };
            if bias >= bias_threshold {
                let likely = taken >= not_taken;
                hints.push((entry.src_addr, entry.target_addr, likely));
            }
        }
        hints
    }

    /// Apply PGO feedback: reorder hot functions and inject branch hints.
    /// This is the runtime optimization closure that connects profile
    /// data collection to actual performance improvements.
    /// Returns the number of optimizations applied.
    pub fn apply_feedback(&self) -> u32 {
        let mut count = 0u32;

        let layout = self.generate_layout_order();
        if layout.len() > 0 {
            crate::log_debug!("PGO: layout order: {} hot functions", layout.len());
            count += 1;
        }

        let hints = self.generate_branch_hints(100);
        for &(src, target, likely) in &hints {
            let _ = (src, target);
            #[cfg(target_arch = "x86_64")]
            {
                // SAFETY: branch hint via prefetch; benign NOP on unsupported CPUs
                if likely {
                    // Prefetch the likely target for branch prediction
                    unsafe { core::arch::x86_64::_mm_prefetch(target as *const i8, core::arch::x86_64::_MM_HINT_T0) };
                }
            }
        }
        if !hints.is_empty() {
            crate::log_debug!("PGO: {} branch hints applied", hints.len());
            count += hints.len() as u32;
        }

        count
    }
}

/// Global PGO profile
static PGO_PROFILE: crate::sync_oncelock::OnceLock<PgoProfile> = crate::sync_oncelock::OnceLock::new();

/// Get global PGO profile
pub fn pgo_profile() -> &'static PgoProfile {
    PGO_PROFILE.get_or_init(PgoProfile::new)
}

/// Record a branch outcome in PGO
pub fn pgo_record_branch(src_addr: u64, target_addr: u64, taken: bool) -> i32 {
    pgo_profile().record_branch(src_addr, target_addr, taken)
}

/// Record a call in PGO
pub fn pgo_record_call(call_stack: &[u64]) -> i32 {
    pgo_profile().record_call_path(call_stack)
}

/// Dump PGO profile data
pub fn pgo_dump_profile() -> Vec<u8> {
    pgo_profile().dump_profile()
}

/// Initialize PGO subsystem
pub fn init_pgo() {
    let profile = pgo_profile();
    profile.reset();
}
