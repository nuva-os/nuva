# Nuva OS Performance Optimization

> This document describes the performance optimizations applied to Nuva OS
> and their measured or projected impact.

## Overview

Nuva OS implements a multi-layered performance optimization strategy
targeting the kernel hot paths: memory allocation, IPC, scheduling,
and driver management.

## 1. Per-CPU Page Cache (PCP)

### Optimization
- Order-0 page allocations bypass the global Buddy allocator spinlock
  by maintaining per-CPU free page caches.
- High/low watermarks control automatic refill and drain.
- Batch operations reduce lock contention on the global allocator.

### Configuration
| Parameter | Value | Description |
|-----------|-------|-------------|
| `PCP_CACHE_SIZE` | 128 | Pages per cache per order |
| `PCP_HIGH` | 32 | Drain threshold |
| `PCP_LOW` | 8 | Refill threshold |
| `PCP_BATCH` | 16 | Bulk operation size |
| `PCP_NR_ORDERS` | 3 | Cached orders (4KB, 8KB, 16KB) |

### Target Improvement
- Order-0 allocation latency: **>= 15% reduction**
- Cache line aligned structure (`#[repr(C, align(64))]`)

## 2. SLAB Allocator Cache Line Alignment

### Optimization
- All SLAB object sizes are aligned to 64-byte cache lines.
- Prevents false sharing between objects accessed by different CPUs.
- `KmemCache` structure itself is cache-line aligned.

### Target Improvement
- Hot path cache hit rate: **>= 90%**
- Reduced cross-CPU cache coherency traffic

## 3. IPC Fast Path Optimization

### Optimization
- Small messages (<= 256 bytes) use inline/register fast path.
- Bypasses shared memory mapping for latency-critical IPC.
- SPSC lock-free queue with cache-line-aligned slots.

### Configuration
| Parameter | Old Value | New Value | Description |
|-----------|-----------|-----------|-------------|
| `SMALL_MESSAGE_SIZE` | 64 B | 256 B | Register fast path threshold |
| `MEDIUM_MESSAGE_SIZE` | 4 KB | 4 KB | Direct transfer threshold |

### Target Improvement
- Small message IPC latency: **>= 20% reduction**
- Fast path covers ~95th percentile of IPC messages

## 4. SpinLock Optimization

### Optimization
- Preemption disable integrated into lock/unlock to prevent
  deadlock from scheduler invocation during critical section.
- Holder CPU tracking for deadlock detection.
- `#[inline(always)]` on hot path methods.
- Architecture-specific IRQ-safe locking (ARM64 DAIF, x86_64 RFLAGS, LA64 CRMD).

### Target Improvement
- Lock acquisition overhead: **minimal** (single CAS + preempt_disable)
- Context switch while lock held: **eliminated**

## 5. Atomic Operation Ordering

### Optimization
- All synchronization atomics use `AcqRel` ordering (was inconsistent).
- Statistics counters retain `Relaxed` ordering (no synchronization needed).
- Each Ordering choice is documented with a comment explaining the rationale.
- Fixed `ref_count.fetch_sub` in IPC zero_copy from `Release` to `AcqRel`.

### Target Improvement
- Correctness improvement (eliminates potential memory ordering bugs)
- No performance regression (AcqRel is the minimum for correctness)

## 6. Hot Path Inlining

### Strategy
- `#[inline(always)]` applied to:
  - `SpinLock::lock()`, `SpinLock::unlock()`, `SpinLock::try_lock()`
  - `preempt_disable()`, `preempt_enable()`, `preempt_count()`, `preemptible()`
  - `allocation_allowed()`
  - `PerCpuPageCache::alloc()`, `PerCpuPageCache::free()`
  - All atomic operation wrappers
  - `SchedPolicyConfig` parameter accessors

- `#[inline]` (hint) applied to:
  - Non-critical path functions that may benefit from inlining

### Monitoring
- Kernel image size growth from inlining must not exceed 5%.
- If exceeded, downgrade `#[inline(always)]` to `#[inline]` for
  less critical paths.

## 7. Declarative Paradigm Performance

### Driver Registration
- `DriverDescriptor` is `&'static` — zero-cost at runtime.
- Device matching uses pre-sorted priority order.
- No heap allocation during driver probe.

### Power State Machine
- Current state stored in `AtomicU8` — lock-free reads.
- Transition validation is O(n) on transitions table (typically <= 6 entries).
- `#[inline(always)]` on `current_state()`.

### Scheduler Policy
- `SchedPolicyConfig` uses `AtomicU32` per parameter — lock-free reads.
- Hot-update is atomic with generation counter for ABA prevention.
- Cache-line aligned to prevent false sharing.

## 8. RCU (Read-Copy-Update)

### Optimization
- Read-side access is lock-free and wait-free.
- Writers create new copies and use `synchronize_rcu()` for grace period.
- Target use cases: module list traversal, device tree lookups, security policy reads.
- Read path overhead: single memory barrier only.

## 9. Per-CPU Infrastructure

### Optimization
- `PerCpuRunQueue` — cache-line-aligned per-CPU run queues for lock-free local scheduling.
- `PerCpuPageCache` — per-CPU free page caches bypassing global Buddy lock.
- Per-CPU statistics — `AtomicU64` counters with `Relaxed` ordering, aggregated on demand.
- All Per-CPU data: `#[repr(C, align(64))]` to prevent false sharing.

## 10. io_uring Async I/O

### Optimization
- Zero-copy I/O via shared ring buffers between user space and kernel.
- Submission queue (SQ) and completion queue (CQ) in shared memory.
- Fixed buffer registration avoids per-IO page pinning.
- Linked operations enable atomic multi-step I/O sequences.
- Batched submission reduces syscall overhead.

## Benchmarks

### Methodology
- QEMU `-cpu cortex-a57 -m 1G` for ARM64
- `criterion` benchmark harness for micro-benchmarks
- `perf` counters for cache hit rates

### Key Metrics
| Metric | Baseline | Target | Method |
|--------|----------|--------|--------|
| Page alloc latency (order-0) | TBD | -15% | PCP cache |
| SLAB cache hit rate | TBD | >= 90% | Cache line alignment |
| IPC small msg latency | TBD | -20% | Register fast path |
| SpinLock acquire | TBD | Minimal | Inline + preempt |
| Context switch latency | TBD | -10% | Minimal save/restore |

---

<!-- Translation Status: Source (English) | Last Updated: 2026-05-20 -->

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
