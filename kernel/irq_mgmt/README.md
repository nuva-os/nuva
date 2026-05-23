# IRQ Management Subsystem

## Overview

The `kernel/irq_mgmt/` module provides interrupt request (IRQ) management, including IRQ dispatch, trap handling, and APIC operations across ARM64, x86-64, and LoongArch64 architectures.

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point and IRQ framework |
| `irq.rs` | IRQ dispatch and management |
| `trap.rs` | Trap and exception handling |
| `apic_ops.rs` | APIC operations (x86-64 LAPIC/I/O APIC, ARM64 GIC, LA64 EIOINTC) |

## Architecture Support

| Architecture | Interrupt Controller |
|--------------|---------------------|
| ARM64 | GIC (Generic Interrupt Controller) |
| x86-64 | LAPIC + I/O APIC |
| LoongArch64 | EIOINTC (Extended I/O Interrupt Controller) |

## Initialization Order

IRQ management components initialize across boot phases according to their hardware dependencies:

| Phase | Component | Init Function |
|-------|-----------|---------------|
| 2 — Memory & IRQ | irq | `irq::init_irq()` |
| 8 — Platform | apic_ops | `apic_ops::init_apic_ops()` |

The `trap` module is initialized implicitly as part of architecture-specific early boot (Phase 1) via the HAL layer.

## Dependencies

- **Internal dependencies**: `kernel/core` (CPU), HAL (L0 — GIC/APIC/EIOINTC hardware abstraction)
- **Depended by**: All interrupt-driven subsystems (device drivers, timers, IPC, networking)

## Public Interface

- `irq` module: Generic IRQ dispatch and management (`init_irq()`, `request_irq()`, `free_irq()`, `enable_irq()`, `disable_irq()`)
- `trap` module: Trap and exception handling for faults, aborts, and system calls across all architectures
- `apic_ops` module: Architecture-abstracted interrupt controller operations (`init_apic_ops()`, `send_ipi()`, `eoi()`)

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `apic_ops`, `irq`, `trap`

---

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
