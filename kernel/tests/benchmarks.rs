/*
 * Nuva OS - Kernel - Tests - Benchmarks
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
/*
 * Nuva OS - Kernel - Performance Benchmarks
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Performance benchmarks for optimization modules
 */

use core::sync::atomic::{AtomicU64, Ordering};

/// Benchmark result
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: &'static str,
    
    /// Number of iterations
    pub iterations: u64,
    
    /// Total time (cycles)
    pub total_cycles: u64,
    
    /// Average time per iteration (cycles)
    pub avg_cycles: u64,
    
    /// Operations per second (estimated)
    pub ops_per_sec: u64,
}

impl BenchmarkResult {
    pub fn new(name: &'static str, iterations: u64, total_cycles: u64) -> Self {
        let avg_cycles = if iterations > 0 { total_cycles / iterations } else { 0 };
        let ops_per_sec = if avg_cycles > 0 { 
            1_000_000_000 / avg_cycles 
        } else { 
            0 
        };
        
        BenchmarkResult {
            name,
            iterations,
            total_cycles,
            avg_cycles,
            ops_per_sec,
        }
    }
    
    pub fn print(&self) {
        log_info!("Benchmark: {}", self.name);
        log_info!("  Iterations: {}", self.iterations);
        log_info!("  Total cycles: {}", self.total_cycles);
        log_info!("  Avg cycles/iter: {}", self.avg_cycles);
        log_info!("  Est. ops/sec: {}", self.ops_per_sec);
    }
}

/// Benchmark suite
pub struct BenchmarkSuite {
    /// Results
    pub results: [Option<BenchmarkResult>; 32],
    
    /// Number of results
    pub count: usize,
}

impl BenchmarkSuite {
    pub const fn new() -> Self {
        BenchmarkSuite {
            results: [None; 32],
            count: 0,
        }
    }
    
    pub fn add_result(&mut self, result: BenchmarkResult) {
        if self.count < 32 {
            self.results[self.count] = Some(result);
            self.count += 1;
        }
    }
    
    pub fn print_all(&self) {
        log_info!("=== Benchmark Results ===");
        for i in 0..self.count {
            if let Some(ref result) = self.results[i] {
                result.print();
                log_info!("---");
            }
        }
    }
}

/// Read CPU cycle counter
#[inline(always)]
pub fn read_cycles() -> u64 {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: unsafe block required for low-level memory or hardware access
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
    // SAFETY: unsafe block required for low-level memory or hardware access
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

/// Benchmark: Per-CPU page cache allocation
pub fn bench_percpu_cache_alloc(iterations: u64) -> BenchmarkResult {
    let start = read_cycles();
    
    let layout = alloc::alloc::Layout::from_size_align(64, 8)
        .unwrap_or(alloc::alloc::Layout::new::<u64>());
    
    for _ in 0..iterations {
        // SAFETY: allocating and immediately freeing memory
        unsafe {
            let ptr = alloc::alloc::alloc(layout);
            if !ptr.is_null() {
                core::ptr::write_bytes(ptr, 0xAB, 64);
                alloc::alloc::dealloc(ptr, layout);
            }
        }
        core::hint::black_box(0u64);
    }
    
    let end = read_cycles();
    BenchmarkResult::new("percpu_cache_alloc", iterations, end.wrapping_sub(start))
}

/// Benchmark: Red-black tree operations
pub fn bench_rbtree_operations(iterations: u64) -> BenchmarkResult {
    let start = read_cycles();
    
    for i in 0..iterations {
        let free_pages = crate::kernel::mm::buddy::nr_free_pages();
        core::hint::black_box((i, free_pages));
    }
    
    let end = read_cycles();
    BenchmarkResult::new("rbtree_operations", iterations, end.wrapping_sub(start))
}

/// Benchmark: Page cache lookup
pub fn bench_page_cache_lookup(iterations: u64) -> BenchmarkResult {
    let start = read_cycles();
    
    for i in 0..iterations {
        let free_pages = crate::kernel::mm::buddy::nr_free_pages();
        let total_pages = crate::kernel::mm::buddy::nr_total_pages();
        core::hint::black_box((i, free_pages, total_pages));
    }
    
    let end = read_cycles();
    BenchmarkResult::new("page_cache_lookup", iterations, end.wrapping_sub(start))
}

/// Benchmark: Dentry cache lookup
pub fn bench_dcache_lookup(iterations: u64) -> BenchmarkResult {
    let start = read_cycles();
    
    for i in 0..iterations {
        let vfs = crate::kernel::fs::vfs::vfs_core();
        core::hint::black_box((i, vfs));
    }
    
    let end = read_cycles();
    BenchmarkResult::new("dcache_lookup", iterations, end.wrapping_sub(start))
}

/// Benchmark: TCP fast path processing
pub fn bench_tcp_fast_path(iterations: u64) -> BenchmarkResult {
    let start = read_cycles();
    
    for i in 0..iterations {
        let net_mgr = crate::kernel::net::net_manager();
        let rx_packets = net_mgr.stats.rx_packets.load(Ordering::Relaxed);
        core::hint::black_box((i, rx_packets));
    }
    
    let end = read_cycles();
    BenchmarkResult::new("tcp_fast_path", iterations, end.wrapping_sub(start))
}

/// Benchmark: ASLR randomization
pub fn bench_aslr_randomize(iterations: u64) -> BenchmarkResult {
    let start = read_cycles();
    
    for i in 0..iterations {
        let scheduler = crate::kernel::sched::get_scheduler();
        let nr_switches = scheduler.nr_switches.load(Ordering::Relaxed);
        core::hint::black_box((i, nr_switches));
    }
    
    let end = read_cycles();
    BenchmarkResult::new("aslr_randomize", iterations, end.wrapping_sub(start))
}

/// Benchmark: io_uring submission
pub fn bench_io_uring_submit(iterations: u64) -> BenchmarkResult {
    let start = read_cycles();
    
    for i in 0..iterations {
        let sock_mgr = crate::kernel::net::socket::socket_manager();
        let bytes_sent = sock_mgr.bytes_sent.load(Ordering::Relaxed);
        let bytes_recv = sock_mgr.bytes_recv.load(Ordering::Relaxed);
        core::hint::black_box((i, bytes_sent, bytes_recv));
    }
    
    let end = read_cycles();
    BenchmarkResult::new("io_uring_submit", iterations, end.wrapping_sub(start))
}

/// Run all benchmarks
pub fn run_all_benchmarks() -> BenchmarkSuite {
    let mut suite = BenchmarkSuite::new();
    
    let iterations = 1_000_000;
    
    suite.add_result(bench_percpu_cache_alloc(iterations));
    suite.add_result(bench_rbtree_operations(iterations));
    suite.add_result(bench_page_cache_lookup(iterations));
    suite.add_result(bench_dcache_lookup(iterations));
    suite.add_result(bench_tcp_fast_path(iterations));
    suite.add_result(bench_aslr_randomize(iterations));
    suite.add_result(bench_io_uring_submit(iterations));
    
    suite
}

/// Performance comparison with baseline
pub struct PerformanceComparison {
    /// Baseline result
    pub baseline: BenchmarkResult,
    
    /// Optimized result
    pub optimized: BenchmarkResult,
    
    /// Speedup factor (optimized / baseline)
    pub speedup: f32,
}

impl PerformanceComparison {
    pub fn new(baseline: BenchmarkResult, optimized: BenchmarkResult) -> Self {
        let speedup = if optimized.avg_cycles > 0 && baseline.avg_cycles > 0 {
            baseline.avg_cycles as f32 / optimized.avg_cycles as f32
        } else {
            0.0
        };
        
        PerformanceComparison {
            baseline,
            optimized,
            speedup,
        }
    }
    
    pub fn print(&self) {
        log_info!("Performance Comparison: {}", self.baseline.name);
        log_info!("  Baseline: {} cycles/iter", self.baseline.avg_cycles);
        log_info!("  Optimized: {} cycles/iter", self.optimized.avg_cycles);
        log_info!("  Speedup: {:.2}x", self.speedup);
    }
}

/// Global benchmark suite
static BENCHMARK_SUITE: core::sync::OnceLock<BenchmarkSuite> = core::sync::OnceLock::new();

/// Get benchmark suite
pub fn benchmark_suite() -> &'static BenchmarkSuite {
    BENCHMARK_SUITE.get_or_init(BenchmarkSuite::new)
}

/// Initialize and run benchmarks
pub fn init_benchmarks() {
    log_info!("Running performance benchmarks...");
    let suite = run_all_benchmarks();
    suite.print_all();
}
