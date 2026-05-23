# Core Kernel Services

## Overview

The `kernel/core/` module provides fundamental kernel services used throughout the kernel, including CPU management, caching, memory pools, workqueues, time keeping, signal handling, POSIX compatibility, random number generation, defense mechanisms, kernel threads, performance tuning, and wait queues.

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point |
| `cpu.rs` | CPU management and Per-CPU data structures |
| `cache.rs` | Kernel cache subsystem |
| `mempool.rs` | Memory pool allocator |
| `workqueue.rs` | Work queue for deferred work |
| `wait.rs` | Wait queue implementation |
| `time.rs` | Time keeping and timers |
| `signal.rs` | Signal handling |
| `posix.rs` | POSIX compatibility layer |
| `random.rs` | Random number generation |
| `defense.rs` | Kernel defense mechanisms |
| `kernel_thread.rs` | Kernel thread management |
| `perf_tune.rs` | Performance tuning subsystem |

## Key Features

- **Per-CPU variables**: Cache-line-aligned per-CPU data for lock-free access across CPUs
- **Memory pools**: Efficient kernel memory allocation with pre-sized pool allocators (`mempool`)
- **Kernel threads**: Lightweight kernel thread creation and lifecycle management
- **Workqueues**: Deferred work execution in process context with priority scheduling
- **Wait queues**: Blocking wait with wakeup notification for synchronization between kernel tasks
- **Time keeping**: High-resolution timers and system time management
- **Signal handling**: POSIX signal delivery and handling infrastructure
- **Kernel defense**: Stack canary validation, ASLR support, and kernel hardening mechanisms

## Initialization Order

Core services initialize across multiple boot phases:

| Phase | Component | Init Function |
|-------|-----------|---------------|
| 1 — Bootstrap | cpu | `cpu::init_cpu()` |
| 2 — Memory & IRQ | mempool, random, time | `mempool::init_mempool()`, `random::init_random()`, `time::init_time()` |
| 4 — Infrastructure | workqueue | `workqueue::init_workqueue()` |
| 5 — Core Kernel | signal | `signal::init_signal()` |
| 6 — Resilience | defense, cache, perf_tune | `defense::init_defense()`, `cache::init_cache()`, `perf_tune::init_perf_tune()` |

## Dependencies

- **Internal dependencies**: HAL (L0) for CPU topology, `kernel/sync` for RCU and synchronization primitives
- **Depended by**: All other kernel subsystems — core provides the fundamental services (CPU, mempool, time, workqueue) that every kernel component builds upon

## Public Interface

- `cpu` module: Per-CPU data structures, CPU topology detection, CPU state management (`init_cpu()`, `current_cpu_id()`)
- `cache` module: Kernel cache subsystem for hot data (`init_cache()`, `cache_get()`, `cache_put()`)
- `mempool` module: Memory pool allocator with multiple pool sizes (`init_mempool()`, `pool_alloc()`, `pool_free()`)
- `workqueue` module: Work queue for deferred execution (`init_workqueue()`, `schedule_work()`, `flush_work()`)
- `wait` module: Wait queue for blocking synchronization (`wait_event()`, `wake_up()`)
- `time` module: Time keeping with high-resolution timers (`init_time()`, `get_time()`, `set_timer()`)
- `signal` module: Signal delivery framework (`init_signal()`, `send_signal()`, `handle_signal()`)
- `posix` module: POSIX compatibility layer for porting Unix applications
- `random` module: Kernel random number generation (`init_random()`, `get_random_bytes()`)
- `defense` module: Kernel defense including stack canaries and ASLR (`init_defense()`)
- `kernel_thread` module: Kernel thread creation and management (`create_kernel_thread()`)
- `perf_tune` module: Runtime performance tuning interface (`init_perf_tune()`)

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `cache`, `cpu`, `defense`, `kernel_thread`, `mempool`, `perf_tune`, `posix`, `random`, `signal`, `time`, `wait`, `workqueue`

---

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
