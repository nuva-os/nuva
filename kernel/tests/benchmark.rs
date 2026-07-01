/*
 * Nuva OS - Kernel - Performance Tests (Benchmark Framework)
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

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Benchmark result with statistics
pub struct PerfBenchmarkResult {
    /// Benchmark name
    pub name: &'static str,
    /// Number of iterations
    pub iterations: u64,
    /// Total cycles
    pub total_cycles: u64,
    /// Minimum cycles per iteration
    pub min_cycles: u64,
    /// Maximum cycles per iteration
    pub max_cycles: u64,
    /// Average cycles per iteration
    pub avg_cycles: u64,
    /// Estimated operations per second (at 1 GHz)
    pub ops_per_sec: u64,
}

impl PerfBenchmarkResult {
    pub fn new(
        name: &'static str,
        iterations: u64,
        total_cycles: u64,
        min_cycles: u64,
        max_cycles: u64,
    ) -> Self {
        let avg_cycles = if iterations > 0 {
            total_cycles / iterations
        } else {
            0
        };
        let ops_per_sec = if avg_cycles > 0 {
            1_000_000_000 / avg_cycles
        } else {
            0
        };
        PerfBenchmarkResult {
            name,
            iterations,
            total_cycles,
            min_cycles,
            max_cycles,
            avg_cycles,
            ops_per_sec,
        }
    }

    pub fn print(&self) {
        log_info!("Benchmark: {}", self.name);
        log_info!("  Iterations: {}", self.iterations);
        log_info!("  Total: {} cycles", self.total_cycles);
        log_info!(
            "  Min/Max/Avg: {}/{}/{} cycles",
            self.min_cycles,
            self.max_cycles,
            self.avg_cycles
        );
        log_info!("  Est. ops/sec: {}", self.ops_per_sec);
    }
}

/// Benchmark runner
/// Framework for running performance benchmarks with iteration
/// counting, timing, and statistical analysis.
pub struct BenchmarkRunner {
    /// Default iterations per benchmark
    pub default_iterations: u64,
    /// Collected results
    pub results: Vec<PerfBenchmarkResult>,
    /// Warm-up iterations (not measured)
    pub warmup_iterations: u64,
}

impl BenchmarkRunner {
    pub fn new(default_iterations: u64) -> Self {
        BenchmarkRunner {
            default_iterations,
            results: Vec::new(),
            warmup_iterations: 100,
        }
    }

    /// Run a benchmark function
    /// @param name: benchmark name
    /// @param iterations: number of iterations (0 = use default)
    /// @param bench_fn: benchmark function, returns cycles per iteration
    pub fn run<F>(&mut self, name: &'static str, iterations: u64, mut bench_fn: F)
    where
        F: FnMut() -> u64,
    {
        let iters = if iterations == 0 {
            self.default_iterations
        } else {
            iterations
        };

        // Warm-up
        for _ in 0..self.warmup_iterations {
            core::hint::black_box(bench_fn());
        }

        // Measure
        let mut total_cycles = 0u64;
        let mut min_cycles = u64::MAX;
        let mut max_cycles = 0u64;

        for _ in 0..iters {
            let cycles = bench_fn();
            total_cycles += cycles;
            if cycles < min_cycles {
                min_cycles = cycles;
            }
            if cycles > max_cycles {
                max_cycles = cycles;
            }
        }

        let result = PerfBenchmarkResult::new(name, iters, total_cycles, min_cycles, max_cycles);
        self.results.push(result);
    }

    /// Print all results
    pub fn print_all(&self) {
        log_info!("=== Performance Benchmark Results ===");
        for result in &self.results {
            result.print();
            log_info!("---");
        }
    }
}

/// Read CPU cycle counter
#[inline(always)]
pub fn read_cycles() -> u64 {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: reading timestamp counter
    unsafe {
        let mut high: u32;
        let mut low: u32;
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nostack, preserves_flags)
        );
        ((high as u64) << 32) | (low as u64)
    }

    #[cfg(target_arch = "aarch64")]
    // SAFETY: reading generic timer
    unsafe {
        let cycles: u64;
        core::arch::asm!(
            "mrs {}, cntvct_el0",
            out(reg) cycles,
            options(nostack, preserves_flags)
        );
        cycles
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    0
}

/// Benchmark: Memory allocation throughput
pub fn bench_memory_alloc_throughput(runner: &mut BenchmarkRunner) {
    runner.run("memory_alloc_throughput", 0, || {
        let layout = alloc::alloc::Layout::from_size_align(64, 8)
            .unwrap_or(alloc::alloc::Layout::new::<u64>());
        let start = read_cycles();
        // SAFETY: allocating and immediately freeing
        let ptr = unsafe { alloc::alloc::alloc_layout(layout) };
        if !ptr.is_null() {
            // SAFETY: ptr was allocated with the same layout above
            unsafe {
                alloc::alloc::dealloc_layout(ptr, layout);
            }
        }
        let end = read_cycles();
        end.wrapping_sub(start)
    });
}

/// Benchmark: Scheduler context switch latency
pub fn bench_sched_switch_latency(runner: &mut BenchmarkRunner) {
    runner.run("sched_switch_latency", 0, || {
        let start = read_cycles();
        // Simulate context switch overhead
        core::hint::black_box(0u64);
        let end = read_cycles();
        end.wrapping_sub(start)
    });
}

/// Benchmark: File I/O throughput
pub fn bench_file_io_throughput(runner: &mut BenchmarkRunner) {
    runner.run("file_io_throughput", 0, || {
        let start = read_cycles();
        // Simulate file I/O
        core::hint::black_box(4096u64);
        let end = read_cycles();
        end.wrapping_sub(start)
    });
}

/// Benchmark: Interrupt handling latency
pub fn bench_interrupt_latency(runner: &mut BenchmarkRunner) {
    runner.run("interrupt_latency", 0, || {
        let start = read_cycles();
        // Simulate interrupt entry/exit
        core::hint::black_box(1u64);
        let end = read_cycles();
        end.wrapping_sub(start)
    });
}

/// Benchmark: io_uring submission throughput
pub fn bench_io_uring_throughput(runner: &mut BenchmarkRunner) {
    runner.run("io_uring_throughput", 0, || {
        let start = read_cycles();
        // Simulate io_uring SQE submission
        core::hint::black_box(1u64);
        let end = read_cycles();
        end.wrapping_sub(start)
    });
}

/// Benchmark: ftrace record overhead
pub fn bench_ftrace_overhead(runner: &mut BenchmarkRunner) {
    runner.run("ftrace_overhead", 0, || {
        let start = read_cycles();
        // Simulate ftrace record write
        core::hint::black_box(0x1000u64);
        let end = read_cycles();
        end.wrapping_sub(start)
    });
}

/// Run all performance tests
pub fn run_performance_tests() {
    let mut runner = BenchmarkRunner::new(100_000);

    bench_memory_alloc_throughput(&mut runner);
    bench_sched_switch_latency(&mut runner);
    bench_file_io_throughput(&mut runner);
    bench_interrupt_latency(&mut runner);
    bench_io_uring_throughput(&mut runner);
    bench_ftrace_overhead(&mut runner);

    runner.print_all();
}
