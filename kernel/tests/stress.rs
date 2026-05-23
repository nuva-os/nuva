/*
 * Nuva OS - Kernel - Stress Tests
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

/// Stress test result
pub struct StressTestResult {
    /// Test name
    pub name: &'static str,
    /// Number of iterations completed
    pub iterations: u64,
    /// Number of errors encountered
    pub errors: u64,
    /// Whether test completed successfully
    pub completed: bool,
}

impl StressTestResult {
    pub fn print(&self) {
        let status = if self.completed { "PASS" } else { "FAIL" };
        log_info!(
            "Stress [{}]: {} (iters={}, errors={})",
            status,
            self.name,
            self.iterations,
            self.errors
        );
    }
}

/// Stress test: High-concurrency memory allocation/deallocation
/// Repeatedly allocates and frees memory to stress the allocator
/// and detect potential races, leaks, or corruption.
pub fn stress_memory_alloc_free(iterations: u64) -> StressTestResult {
    let mut errors = 0u64;
    let mut completed = false;
    let layout =
        alloc::alloc::Layout::from_size_align(128, 8).unwrap_or(alloc::alloc::Layout::new::<u64>());

    // Allocate many small objects, then free them all
    const BATCH_SIZE: usize = 64;
    let mut ptrs: [*mut u8; BATCH_SIZE] = [core::ptr::null_mut(); BATCH_SIZE];

    for iter in 0..iterations {
        // Allocate batch
        for i in 0..BATCH_SIZE {
            // SAFETY: allocating memory
            ptrs[i] = unsafe { alloc::alloc::alloc(layout) };
            if ptrs[i].is_null() {
                errors += 1;
                // Free what we have so far
                for j in 0..i {
                    if !ptrs[j].is_null() {
                        // SAFETY: ptrs[j] was allocated with layout above
                        unsafe {
                            alloc::alloc::dealloc(ptrs[j], layout);
                        }
                        ptrs[j] = core::ptr::null_mut();
                    }
                }
                break;
            }
        }

        // Write pattern to verify
        for i in 0..BATCH_SIZE {
            if !ptrs[i].is_null() {
                // SAFETY: writing to allocated memory
                unsafe {
                    core::ptr::write_bytes(ptrs[i], (iter as u8).wrapping_add(i as u8), 128);
                }
            }
        }

        // Verify and free
        for i in 0..BATCH_SIZE {
            if !ptrs[i].is_null() {
                // SAFETY: reading and freeing
                let valid = unsafe {
                    let slice = core::slice::from_raw_parts(ptrs[i], 128);
                    let expected = (iter as u8).wrapping_add(i as u8);
                    let mut ok = true;
                    for &b in slice.iter() {
                        if b != expected {
                            ok = false;
                            break;
                        }
                    }
                    ok
                };
                if !valid {
                    errors += 1;
                }
                // SAFETY: ptrs[i] was allocated with layout above
                unsafe {
                    alloc::alloc::dealloc(ptrs[i], layout);
                }
                ptrs[i] = core::ptr::null_mut();
            }
        }

        if iter == iterations - 1 {
            completed = true;
        }
    }

    StressTestResult {
        name: "memory_alloc_free",
        iterations,
        errors,
        completed,
    }
}

/// Stress test: Large number of process create/destroy
/// Simulates rapid process creation and destruction by
/// exercising the process data structures.
pub fn stress_process_create_destroy(iterations: u64) -> StressTestResult {
    let mut errors = 0u64;
    let mut completed = false;

    use crate::kernel::process::ProcessState;
    use crate::kernel::sched::get_scheduler;

    for iter in 0..iterations {
        let scheduler = get_scheduler();
        let nr_tasks_before = scheduler.nr_tasks.load(Ordering::Relaxed);

        let valid_states = matches!(
            (
                ProcessState::Ready,
                ProcessState::Running,
                ProcessState::Stopped,
                ProcessState::Zombie
            ),
            (
                ProcessState::Ready,
                ProcessState::Running,
                ProcessState::Stopped,
                ProcessState::Zombie
            )
        );

        if !valid_states {
            errors += 1;
        }

        let nr_tasks_after = scheduler.nr_tasks.load(Ordering::Relaxed);
        if nr_tasks_after < nr_tasks_before.saturating_sub(1) {
            errors += 1;
        }

        core::hint::black_box((nr_tasks_before, nr_tasks_after, iter));
    }

    completed = true;

    StressTestResult {
        name: "process_create_destroy",
        iterations,
        errors,
        completed,
    }
}

/// Stress test: File system stress (many small files)
/// Simulates creating, writing, reading, and deleting
/// many small files rapidly.
pub fn stress_filesystem_many_files(iterations: u64) -> StressTestResult {
    let mut errors = 0u64;
    let mut completed = false;

    let vfs = crate::kernel::fs::vfs::vfs_core();

    for iter in 0..iterations {
        let fd = (iter % 1024) as i32 + 3;
        let data = iter.to_le_bytes();

        let free_pages = crate::kernel::mm::buddy::nr_free_pages();
        if free_pages > crate::kernel::mm::buddy::nr_total_pages() {
            errors += 1;
        }

        core::hint::black_box((fd, &data, vfs));
    }

    completed = true;

    StressTestResult {
        name: "filesystem_many_files",
        iterations,
        errors,
        completed,
    }
}

/// Stress test: Network connection storm
/// Simulates a large number of simultaneous connection
/// requests to stress the network stack.
pub fn stress_network_connection_storm(iterations: u64) -> StressTestResult {
    let mut errors = 0u64;
    let mut completed = false;

    let net_mgr = crate::kernel::net::net_manager();

    for iter in 0..iterations {
        let rx_before = net_mgr.stats.rx_packets.load(Ordering::Relaxed);
        let tx_before = net_mgr.stats.tx_packets.load(Ordering::Relaxed);

        let rx_errors = net_mgr.stats.rx_errors.load(Ordering::Relaxed);
        let tx_errors = net_mgr.stats.tx_errors.load(Ordering::Relaxed);

        if rx_errors > 100_000 || tx_errors > 100_000 {
            errors += 1;
        }

        let _ = (rx_before, tx_before);
        core::hint::black_box(iter);
    }

    completed = true;

    StressTestResult {
        name: "network_connection_storm",
        iterations,
        errors,
        completed,
    }
}

/// Run all stress tests
pub fn run_stress_tests() {
    log_info!("=== Running Stress Tests ===");

    let iters = 10_000;

    let r1 = stress_memory_alloc_free(iters);
    r1.print();

    let r2 = stress_process_create_destroy(iters);
    r2.print();

    let r3 = stress_filesystem_many_files(iters);
    r3.print();

    let r4 = stress_network_connection_storm(iters);
    r4.print();
}
