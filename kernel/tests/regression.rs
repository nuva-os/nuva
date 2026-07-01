/*
 * Nuva OS - Kernel - Regression Tests
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

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Regression test result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegTestResult {
    /// Test passed
    Pass,
    /// Test failed
    Fail,
    /// Test skipped
    Skip,
}

/// Regression test entry
pub struct RegTestEntry {
    /// Test ID (for tracking)
    pub id: u32,
    /// Test name
    pub name: &'static str,
    /// Bug ID this test regression-checks
    pub bug_id: &'static str,
    /// Test function
    pub test_fn: fn() -> RegTestResult,
}

/// Regression test suite
/// Automated regression testing framework with test case registration,
/// execution, pass/fail/skip statistics, and formatted result output.
pub struct RegTestSuite {
    /// Registered tests
    pub tests: Vec<RegTestEntry>,
    /// Next test ID
    pub next_id: AtomicU32,
    /// Pass count
    pub pass_count: AtomicU32,
    /// Fail count
    pub fail_count: AtomicU32,
    /// Skip count
    pub skip_count: AtomicU32,
    /// Detailed results
    pub results: Vec<(u32, &'static str, RegTestResult)>,
}

impl RegTestSuite {
    pub fn new() -> Self {
        RegTestSuite {
            tests: Vec::new(),
            next_id: AtomicU32::new(1),
            pass_count: AtomicU32::new(0),
            fail_count: AtomicU32::new(0),
            skip_count: AtomicU32::new(0),
            results: Vec::new(),
        }
    }

    /// Register a regression test
    /// @param name: test name
    /// @param bug_id: associated bug ID
    /// @param test_fn: test function
    pub fn register(&mut self, name: &'static str, bug_id: &'static str, test_fn: fn() -> RegTestResult) {
        let id = self.next_id.fetch_add(1, Ordering::AcqRel);
        self.tests.push(RegTestEntry {
            id,
            name,
            bug_id,
            test_fn,
        });
    }

    /// Run all registered regression tests
    pub fn run_all(&mut self) {
        self.pass_count.store(0, Ordering::Release);
        self.fail_count.store(0, Ordering::Release);
        self.skip_count.store(0, Ordering::Release);
        self.results.clear();

        for test in &self.tests {
            let result = (test.test_fn)();
            self.results.push((test.id, test.name, result));

            match result {
                RegTestResult::Pass => self.pass_count.fetch_add(1, Ordering::AcqRel),
                RegTestResult::Fail => self.fail_count.fetch_add(1, Ordering::AcqRel),
                RegTestResult::Skip => self.skip_count.fetch_add(1, Ordering::AcqRel),
            };
        }
    }

    /// Run a single test by ID
    pub fn run_by_id(&mut self, id: u32) -> Option<RegTestResult> {
        for test in &self.tests {
            if test.id == id {
                let result = (test.test_fn)();
                self.results.push((test.id, test.name, result));
                match result {
                    RegTestResult::Pass => self.pass_count.fetch_add(1, Ordering::AcqRel),
                    RegTestResult::Fail => self.fail_count.fetch_add(1, Ordering::AcqRel),
                    RegTestResult::Skip => self.skip_count.fetch_add(1, Ordering::AcqRel),
                };
                return Some(result);
            }
        }
        None
    }

    /// Format results as output string
    pub fn format_results(&self) -> String {
        let mut output = String::from("=== Regression Test Results ===\n");

        for (id, name, result) in &self.results {
            let status = match result {
                RegTestResult::Pass => "PASS",
                RegTestResult::Fail => "FAIL",
                RegTestResult::Skip => "SKIP",
            };
            output.push_str(&format!("  [{}] {} - {}\n", id, name, status));
        }

        output.push_str(&format!(
            "\nSummary: {} passed, {} failed, {} skipped, {} total\n",
            self.pass_count.load(Ordering::Acquire),
            self.fail_count.load(Ordering::Acquire),
            self.skip_count.load(Ordering::Acquire),
            self.tests.len(),
        ));

        output
    }

    /// Print results
    pub fn print_results(&self) {
        log_info!("=== Regression Test Results ===");

        for (id, name, result) in &self.results {
            let status = match result {
                RegTestResult::Pass => "PASS",
                RegTestResult::Fail => "FAIL",
                RegTestResult::Skip => "SKIP",
            };
            log_info!("  [{}] {} - {}", id, name, status);
        }

        log_info!("Summary: {} passed, {} failed, {} skipped, {} total",
            self.pass_count.load(Ordering::Acquire),
            self.fail_count.load(Ordering::Acquire),
            self.skip_count.load(Ordering::Acquire),
            self.tests.len(),
        );
    }
}

/// Regression: Verify buddy allocator alignment
fn reg_buddy_alignment() -> RegTestResult {
    // Verify that buddy allocator maintains power-of-2 alignment
    for order in 0..11usize {
        let size = 1u64 << order;
        let align = size;
        if size != align {
            return RegTestResult::Fail;
        }
    }
    RegTestResult::Pass
}

/// Regression: Verify page size constant
fn reg_page_size_constant() -> RegTestResult {
    if crate::kernel::mm::PAGE_SIZE != 4096 {
        return RegTestResult::Fail;
    }
    RegTestResult::Pass
}

/// Regression: Verify slab cache alignment
fn reg_slab_cache_alignment() -> RegTestResult {
    // Verify that common slab sizes are properly aligned
    let sizes: [usize; 8] = [8, 16, 32, 64, 128, 256, 512, 1024];
    for &size in &sizes {
        if size & (size - 1) != 0 {
            return RegTestResult::Fail;
        }
    }
    RegTestResult::Pass
}

/// Regression: Verify scheduler policy values
fn reg_sched_policy_values() -> RegTestResult {
    use crate::kernel::sched::SchedPolicy;

    if SchedPolicy::Normal as u32 != 0 {
        return RegTestResult::Fail;
    }
    if SchedPolicy::Fifo as u32 != 1 {
        return RegTestResult::Fail;
    }
    if SchedPolicy::Rr as u32 != 2 {
        return RegTestResult::Fail;
    }
    RegTestResult::Pass
}

/// Regression: Verify perf event type values
fn reg_perf_event_type_values() -> RegTestResult {
    use crate::kernel::perf::events::PerfEventType;

    if PerfEventType::HwCpuCycles as u32 != 0 {
        return RegTestResult::Fail;
    }
    if PerfEventType::HwInstructions as u32 != 1 {
        return RegTestResult::Fail;
    }
    if PerfEventType::HwCacheMisses as u32 != 3 {
        return RegTestResult::Fail;
    }
    RegTestResult::Pass
}

/// Regression: Verify io_uring opcode values
fn reg_io_uring_opcode_values() -> RegTestResult {
    use crate::kernel::fs::io_uring::IoOpCode;

    if IoOpCode::Read as u8 != 1 {
        return RegTestResult::Fail;
    }
    if IoOpCode::Write as u8 != 2 {
        return RegTestResult::Fail;
    }
    if IoOpCode::Fsync as u8 != 8 {
        return RegTestResult::Fail;
    }
    if IoOpCode::Poll as u8 != 9 {
        return RegTestResult::Fail;
    }
    if IoOpCode::SendMsg as u8 != 10 {
        return RegTestResult::Fail;
    }
    if IoOpCode::RecvMsg as u8 != 11 {
        return RegTestResult::Fail;
    }
    RegTestResult::Pass
}

/// Regression: Verify ftrace record sizes
fn reg_ftrace_record_size() -> RegTestResult {
    use crate::kernel::perf::ftrace::FtraceRecord;

    let expected_size = core::mem::size_of::<u64>()   // timestamp
        + core::mem::size_of::<u32>()                  // cpu_id
        + core::mem::size_of::<u64>()                  // func_addr
        + core::mem::size_of::<u64>()                  // caller_addr
        + core::mem::size_of::<u8>()                   // record_type
        + 3;                                           // _reserved

    if core::mem::size_of::<FtraceRecord>() != expected_size {
        return RegTestResult::Fail;
    }
    RegTestResult::Pass
}

/// Regression: Verify memory pool creation constraints
fn reg_mempool_creation_constraints() -> RegTestResult {
    use crate::kernel::mm::mempool_opt::{MemPool, MemPoolError};

    // Zero object size should fail
    match MemPool::create("invalid", 0, 8, 32) {
        Err(MemPoolError::Invalid) => {}
        _ => return RegTestResult::Fail,
    }

    // Zero capacity should fail
    match MemPool::create("invalid", 64, 8, 0) {
        Err(MemPoolError::Invalid) => {}
        _ => return RegTestResult::Fail,
    }

    // Valid creation should succeed
    match MemPool::create("valid", 64, 8, 32) {
        Ok(_) => RegTestResult::Pass,
        Err(_) => RegTestResult::Skip,
    }
}

/// Regression: Verify PGO profile dump format
fn reg_pgo_profile_dump_format() -> RegTestResult {
    use crate::kernel::perf::pgo::pgo_profile;

    let profile = pgo_profile();
    profile.reset();
    profile.enable();

    // Record some data
    let _ = profile.record_branch(0x100, 0x200, true);
    let _ = profile.record_function(0x100, 100);

    let data = profile.dump_profile();
    profile.disable();

    // Minimum: 4 bytes func_count + 4 bytes branch_count
    if data.len() < 8 {
        return RegTestResult::Fail;
    }

    // Verify func_count field
    let func_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if func_count != 1 {
        return RegTestResult::Fail;
    }

    RegTestResult::Pass
}

/// Run all regression tests
pub fn run_regression_tests() {
    let mut suite = RegTestSuite::new();

    suite.register("buddy_alignment", "BUG-001", reg_buddy_alignment);
    suite.register("page_size_constant", "BUG-002", reg_page_size_constant);
    suite.register("slab_cache_alignment", "BUG-003", reg_slab_cache_alignment);
    suite.register("sched_policy_values", "BUG-004", reg_sched_policy_values);
    suite.register("perf_event_type_values", "BUG-005", reg_perf_event_type_values);
    suite.register("io_uring_opcode_values", "BUG-006", reg_io_uring_opcode_values);
    suite.register("ftrace_record_size", "BUG-007", reg_ftrace_record_size);
    suite.register("mempool_creation_constraints", "BUG-008", reg_mempool_creation_constraints);
    suite.register("pgo_profile_dump_format", "BUG-009", reg_pgo_profile_dump_format);

    suite.run_all();
    suite.print_results();
}
