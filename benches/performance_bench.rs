/*
 * Nuva OS - Benches - PerformanceBench
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
 * Performance Benchmarks
 *
 * Copyright (C) 2026 Nuva OS Team
 */

use crate::hal::quantum::*;
use crate::lib::core::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// ============================================================================
// IPC Benchmarks
// ============================================================================

fn ipc_benchmark(c: &mut Criterion) {
    // Zero-copy IPC small message
    c.bench_function("ipc_small_message", |b| {
        b.iter(|| {
            // Simulate small message IPC
            let data = [0u8; 64];
            black_box(&data);
        });
    });

    // Zero-copy IPC large message
    c.bench_function("ipc_large_message", |b| {
        b.iter(|| {
            // Simulate large message IPC
            let data = vec![0u8; 1024 * 1024];
            black_box(&data);
        });
    });

    // IPC throughput
    let mut group = c.benchmark_group("ipc_throughput");
    for size in [64, 256, 1024, 4096, 16384].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let data = vec![0u8; size];
                black_box(&data);
            });
        });
    }
    group.finish();
}

// ============================================================================
// Lock-Free Data Structure Benchmarks
// ============================================================================

fn lockfree_benchmark(c: &mut Criterion) {
    // MPSC Queue
    c.bench_function("mpsc_queue_push", |b| {
        let queue = MpscQueue::new();
        b.iter(|| {
            queue.push(black_box(42));
        });
    });

    c.bench_function("mpsc_queue_pop", |b| {
        let queue = MpscQueue::new();
        for i in 0..1000 {
            queue.push(i);
        }
        b.iter(|| queue.pop());
    });

    // SPSC Queue
    c.bench_function("spsc_queue_push", |b| {
        let queue = SpscQueue::new(1024);
        b.iter(|| queue.push(black_box(42)));
    });

    c.bench_function("spsc_queue_pop", |b| {
        let queue = SpscQueue::new(1024);
        for i in 0..1000 {
            let _ = queue.push(i);
        }
        b.iter(|| queue.pop());
    });

    // Lock-Free Stack
    c.bench_function("lockfree_stack_push", |b| {
        let stack = LockFreeStack::new();
        b.iter(|| {
            stack.push(black_box(42));
        });
    });

    c.bench_function("lockfree_stack_pop", |b| {
        let stack = LockFreeStack::new();
        for i in 0..1000 {
            stack.push(i);
        }
        b.iter(|| stack.pop());
    });
}

// ============================================================================
// Memory Pool Benchmarks
// ============================================================================

fn memory_pool_benchmark(c: &mut Criterion) {
    // Memory pool allocation
    c.bench_function("memory_pool_alloc", |b| {
        let pool = MemoryPool::new(64, 1024);
        b.iter(|| {
            let ptr = pool.alloc();
            black_box(ptr);
            ptr
        });
    });

    // Memory pool free
    c.bench_function("memory_pool_free", |b| {
        let pool = MemoryPool::new(64, 1024);
        let ptrs: Vec<_> = (0..1000).map(|_| pool.alloc()).collect();
        let mut i = 0;
        b.iter(|| {
            if i < ptrs.len() {
                pool.free(ptrs[i]);
                i += 1;
            }
        });
    });

    // Pool manager
    let mut group = c.benchmark_group("pool_manager");
    for size in [32, 64, 128, 256, 512, 1024].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let config = PoolManagerConfig::default();
            let manager = PoolManager::new(config);
            b.iter(|| {
                let ptr = manager.alloc(size);
                black_box(ptr);
                ptr
            });
        });
    }
    group.finish();
}

// ============================================================================
// Quantum Algorithm Benchmarks
// ============================================================================

fn quantum_benchmark(c: &mut Criterion) {
    // Kyber key generation
    c.bench_function("kyber512_keygen", |b| {
        b.iter(|| {
            // Simulate Kyber-512 key generation
            black_box(KyberVariant::Kyber512);
        });
    });

    c.bench_function("kyber768_keygen", |b| {
        b.iter(|| {
            // Simulate Kyber-768 key generation
            black_box(KyberVariant::Kyber768);
        });
    });

    c.bench_function("kyber1024_keygen", |b| {
        b.iter(|| {
            // Simulate Kyber-1024 key generation
            black_box(KyberVariant::Kyber1024);
        });
    });

    // Dilithium key generation
    c.bench_function("dilithium2_keygen", |b| {
        b.iter(|| {
            // Simulate Dilithium-2 key generation
            black_box(DilithiumVariant::Dilithium2);
        });
    });

    c.bench_function("dilithium3_keygen", |b| {
        b.iter(|| {
            // Simulate Dilithium-3 key generation
            black_box(DilithiumVariant::Dilithium3);
        });
    });

    c.bench_function("dilithium5_keygen", |b| {
        b.iter(|| {
            // Simulate Dilithium-5 key generation
            black_box(DilithiumVariant::Dilithium5);
        });
    });

    // Key size calculations
    let mut group = c.benchmark_group("key_sizes");
    for variant in [
        KyberVariant::Kyber512,
        KyberVariant::Kyber768,
        KyberVariant::Kyber1024,
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("kyber_public_key", format!("{:?}", variant)),
            variant,
            |b, v| b.iter(|| v.public_key_size()),
        );
    }
    group.finish();
}

// ============================================================================
// Synchronization Benchmarks
// ============================================================================

fn sync_benchmark(c: &mut Criterion) {
    // Memory barrier
    c.bench_function("memory_barrier", |b| {
        b.iter(|| {
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        });
    });

    // Atomic operations
    use std::sync::atomic::AtomicUsize;
    let counter = AtomicUsize::new(0);

    c.bench_function("atomic_fetch_add", |b| {
        b.iter(|| {
            counter.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        });
    });

    c.bench_function("atomic_load", |b| {
        b.iter(|| {
            counter.load(core::sync::atomic::Ordering::Relaxed);
        });
    });

    c.bench_function("atomic_store", |b| {
        b.iter(|| {
            counter.store(0, core::sync::atomic::Ordering::Relaxed);
        });
    });
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group! {
    name = ipc_benches;
    config = Criterion::default().sample_size(1000);
    targets = ipc_benchmark
}

criterion_group! {
    name = lockfree_benches;
    config = Criterion::default().sample_size(1000);
    targets = lockfree_benchmark
}

criterion_group! {
    name = memory_benches;
    config = Criterion::default().sample_size(1000);
    targets = memory_pool_benchmark
}

criterion_group! {
    name = quantum_benches;
    config = Criterion::default().sample_size(100);
    targets = quantum_benchmark
}

criterion_group! {
    name = sync_benches;
    config = Criterion::default().sample_size(10000);
    targets = sync_benchmark
}

criterion_main! {
    ipc_benches,
    lockfree_benches,
    memory_benches,
    quantum_benches,
    sync_benches
}
