# Storage Subsystem

## Overview

The `kernel/storage/` module provides block device storage management, including block device abstraction and I/O scheduling.

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point and storage framework |
| `block.rs` | Block device management and I/O operations |

## Initialization

- Block device subsystem initialization (`block::init_block_device`) — Phase 7 (I/O & Networking), after core kernel services are ready

## Dependencies

- **Internal dependencies**: `kernel/core` (workqueue, mempool, time), `kernel/irq_mgmt` (IRQ for storage interrupts), `kernel/device` (device model for block device registration)
- **Depended by**: File systems (VFS, ext4, FAT32, Nuvafs), journaling, swap subsystem

## Public Interface

- `block` module: Block device abstraction and I/O scheduling (`init_block_device()`, `register_block_device()`, `submit_bio()`, `blk_queue_rq()`)

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `block`

---

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
