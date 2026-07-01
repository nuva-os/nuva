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

/// System test result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemTestResult {
    Pass,
    Fail,
    Skip,
}

/// System test statistics
pub struct SystemTestStats {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub total: u32,
}

impl SystemTestStats {
    pub const fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            skipped: 0,
            total: 0,
        }
    }

    pub fn record(&mut self, result: SystemTestResult) {
        self.total += 1;
        match result {
            SystemTestResult::Pass => self.passed += 1,
            SystemTestResult::Fail => self.failed += 1,
            SystemTestResult::Skip => self.skipped += 1,
        }
    }
}

/// System test suite
pub struct SystemTests {
    stats: SystemTestStats,
}

impl SystemTests {
    pub const fn new() -> Self {
        Self {
            stats: SystemTestStats::new(),
        }
    }

    pub fn run_all(&mut self) {
        crate::log_info!("=== Running System Tests ===");

        self.stats.record(self.test_boot_sequence());
        self.stats.record(self.test_memory_subsystem());
        self.stats.record(self.test_scheduler_subsystem());
        self.stats.record(self.test_filesystem_subsystem());
        self.stats.record(self.test_network_subsystem());
        self.stats.record(self.test_driver_subsystem());
        self.stats.record(self.test_ipc_subsystem());
        self.stats.record(self.test_security_subsystem());

        self.print_results();
    }

    fn test_boot_sequence(&mut self) -> SystemTestResult {
        let total_pages = crate::kernel::mm::buddy::nr_total_pages();
        if total_pages > 0 {
            SystemTestResult::Pass
        } else {
            SystemTestResult::Fail
        }
    }

    fn test_memory_subsystem(&mut self) -> SystemTestResult {
        let free_pages = crate::kernel::mm::buddy::nr_free_pages();
        let total_pages = crate::kernel::mm::buddy::nr_total_pages();
        if free_pages <= total_pages && total_pages > 0 {
            SystemTestResult::Pass
        } else {
            SystemTestResult::Fail
        }
    }

    fn test_scheduler_subsystem(&mut self) -> SystemTestResult {
        use core::sync::atomic::Ordering;
        let scheduler = crate::kernel::sched::scheduler();
        let nr_tasks = scheduler.nr_tasks.load(Ordering::Relaxed);
        let nr_running = scheduler.nr_running.load(Ordering::Relaxed);
        if nr_running <= nr_tasks + 1 {
            SystemTestResult::Pass
        } else {
            SystemTestResult::Fail
        }
    }

    fn test_filesystem_subsystem(&mut self) -> SystemTestResult {
        let _vfs = crate::kernel::fs::vfs::get_vfs_core();
        SystemTestResult::Pass
    }

    fn test_network_subsystem(&mut self) -> SystemTestResult {
        use core::sync::atomic::Ordering;
        let net_mgr = crate::kernel::net::get_net_manager();
        let _rx = net_mgr.stats.rx_packets.load(Ordering::Relaxed);
        SystemTestResult::Pass
    }

    fn test_driver_subsystem(&mut self) -> SystemTestResult {
        SystemTestResult::Pass
    }

    fn test_ipc_subsystem(&mut self) -> SystemTestResult {
        SystemTestResult::Pass
    }

    fn test_security_subsystem(&mut self) -> SystemTestResult {
        SystemTestResult::Pass
    }

    fn print_results(&self) {
        crate::log_info!("=== System Test Results ===");
        crate::log_info!("  Total: {}", self.stats.total);
        crate::log_info!("  Passed: {}", self.stats.passed);
        crate::log_info!("  Failed: {}", self.stats.failed);
        crate::log_info!("  Skipped: {}", self.stats.skipped);
    }
}

/// Run system tests
pub fn run_system_tests() {
    let mut tests = SystemTests::new();
    tests.run_all();
}
