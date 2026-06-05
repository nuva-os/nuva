# Tombstone Mechanism

## Overview

The Tombstone Mechanism is a kernel subsystem in Nuva OS (L1 Kernel layer) that captures, stores, and provides query access to crash records when processes or tasks terminate abnormally.

When a crash occurs, the mechanism:

1. Collects CPU context (registers, stack pointer, program counter) from the HAL layer
2. Performs a stack backtrace (up to 32 frames)
3. Assembles a tombstone record with full crash metadata
4. Persists the record to the filesystem with atomic writes
5. Falls back to an in-memory ring buffer if the filesystem is unavailable

## Architecture

```
kernel/tombstone/
├── mod.rs              # Module entry, TombstoneManager, initialization, crash callbacks
├── record.rs           # Core data structures, serialization, CRC32
├── crash_context.rs    # CrashContext collection from HAL, register masking
├── arch_adapter.rs     # CrashArchAdapter trait + ARM64/x64/LoongArch64 adapters
├── store.rs            # TombstoneStore, MemoryCache, file I/O, index
├── query.rs            # Query engine (by PID, time range, crash reason, latest N)
├── prune.rs            # Prune/cleanup engine (by PID, time range, all)
├── config.rs           # TombstoneStoreConfig
├── stats.rs            # Atomic statistics counters
└── syscall.rs          # System call interface (500-503)
```

## Key Data Structures

| Structure | Description |
|-----------|-------------|
| `TombstoneRecord` | Complete crash record with CPU context, stack trace, metadata |
| `CrashReason` | Crash classification enum (FatalSignal, IllegalAccess, etc.) |
| `ArchId` | Architecture identifier (Arm64, X64, LoongArch64) |
| `TombstoneError` | Error type for all tombstone operations |
| `TombstoneStoreConfig` | Storage configuration (path, limits, cache size) |
| `TombstoneStats` | Atomic statistics counters |

## Crash Triggers

| Trigger | Callback | Source |
|---------|----------|--------|
| Fatal signal (SIGSEGV, SIGABRT, etc.) | `on_fatal_signal()` | kernel::signal |
| Task abnormal termination | `on_task_crash()` | kernel::sched |
| Watchdog timeout | `on_watchdog_timeout()` | kernel::sched |

## System Calls

| Number | Name | Permission | Description |
|--------|------|------------|-------------|
| 500 | `tombstone_query` | CAP_SYS_PTRACE or CAP_SYS_ADMIN | Query tombstone records |
| 501 | `tombstone_read` | CAP_SYS_PTRACE or CAP_SYS_ADMIN | Read single record detail |
| 502 | `tombstone_clear` | CAP_SYS_ADMIN | Clear tombstone records |
| 503 | `tombstone_stats` | CAP_SYS_PTRACE or CAP_SYS_ADMIN | Get statistics |

## Storage

- **Path**: `/data/tombstones/` (configurable)
- **Naming**: `tombstone_XX.pb` (XX = 00-99, cyclic)
- **Capacity**: 100 records maximum, FIFO auto-pruning
- **Atomic writes**: temp file → fsync → atomic rename
- **Degraded mode**: In-memory ring buffer (4 slots) when FS unavailable

## Performance

- Tombstone generation: ≤ 5ms (HAL ≤ 1ms + unwind ≤ 2ms + serialize + async write)
- Query (index hit): ≤ 1ms
- Memory overhead: ~13.5 KB (index + ring buffer + statistics)
- Normal execution path: zero overhead

## Binary Format

```
Offset  Size  Field
0       4     Magic (0x5442534E = "TBSN")
4       4     Format version (1)
8       4     Body length (N)
12      N     Body (serialized TombstoneRecord fields)
12+N    4     CRC32 checksum (over bytes 0..12+N)
```

## Configuration

| Parameter | Default | Range |
|-----------|---------|-------|
| `store_dir` | `/data/tombstones/` | Non-empty path |
| `max_count` | 100 | 1-1000 |
| `max_file_size` | 8192 bytes | > 0 |
| `memory_cache_size` | 4 | ≥ 2 |
| `auto_prune_enabled` | true | - |

## Crash Deduplication

Same-PID crashes within a 5-second window are deduplicated: only the first and last tombstone are kept, intermediate crashes increment `crash_count` on the merged record.

## Security

- Register masking: callee-saved registers that may contain secrets are zeroed before storage
- Query requires `CAP_SYS_PTRACE` or `CAP_SYS_ADMIN`
- Clear requires `CAP_SYS_ADMIN`
- No panic/unwrap/expect in production paths; all errors via `Result<T, TombstoneError>`

---

**Last Updated**: May 30, 2026
