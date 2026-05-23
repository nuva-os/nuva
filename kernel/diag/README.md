# Kernel Diagnostics Subsystem

## Overview

The `kernel/diag/` module provides kernel diagnostics infrastructure, including logging, kernel debugging, journaling, virus scanning, and statistics collection.

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point |
| `log.rs` | Kernel logging subsystem |
| `kdebug.rs` | Kernel debugger interface |
| `journal.rs` | Journaling subsystem for persistent diagnostics |
| `scanner.rs` | Virus scanner for security diagnostics |
| `stats.rs` | Diagnostic statistics collection |

## Initialization Order

Diagnostics components initialize across multiple boot phases to satisfy their dependencies:

| Phase | Component | Init Function | Dependencies |
|-------|-----------|---------------|--------------|
| 1 — Bootstrap | log | `log::init_log()` | CPU |
| 4 — Infrastructure | stats | `stats::init_stats()` | Device model, plugin system |
| 6 — Resilience | scanner | `scanner::init_virus_scanner()` | Process, scheduler, security, VFS |
| 8 — Platform & Diag | kdebug | `kdebug::init_kdebug()` | APIC, platform |
| 8 — Platform & Diag | journal | `journal::init_journal()` | kdebug, block device, VFS |

## Dependencies

- **Internal dependencies**: `kernel/core` (CPU, mempool, time, workqueue), `kernel/init` (config, cmdline), `kernel/irq_mgmt` (APIC ops), `kernel/device` (device model), `kernel/process`, `kernel/sched`, `kernel/security`, `kernel/storage` (block)
- **Depended by**: Upper-layer diagnostic tools, system services (L3)

## Public Interface

- `log` module: Kernel logging with severity levels (emerg, alert, crit, err, warn, notice, info, debug) and `pr_*!` macros
- `kdebug` module: Kernel debugger interface for runtime breakpoints, state inspection, and tracing
- `journal` module: Persistent journaling subsystem for storing and replaying diagnostic records
- `scanner` module: Virus scanning engine for security diagnostics and threat detection
- `stats` module: Statistics collection, aggregation, and reporting framework

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `journal`, `kdebug`, `log`, `scanner`, `stats`

---

*Last updated: 2026-05-22 | Nuva OS v1.0.0*
