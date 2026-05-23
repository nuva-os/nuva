# Power Management Subsystem

## Overview

The `kernel/power_mgmt/` module provides kernel-level power management, including ACPI power states, PM subsystem, and hot-plug support.

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point and power management framework |
| `pm.rs` | Power management subsystem (suspend/resume, C-states) |
| `power.rs` | ACPI power driver (Fadt, S3/S5) |
| `hotplug.rs` | Hot-plug support for CPU and memory |

## Initialization Order

1. Hot-plug support (`hotplug::init_hotplug`)
2. PM subsystem (`pm::init_pm`)
3. ACPI power (`power::init_acpi`)

Hot-plug and PM initialize in Phase 4 (Infrastructure), while ACPI power initializes in Phase 8 (Platform & Diagnostics) after APIC operations are configured.

## Dependencies

- **Internal dependencies**: `kernel/core` (CPU, workqueue), `kernel/device` (device model for hot-plug events), `kernel/irq_mgmt` (APIC ops for ACPI), `kernel/sync`
- **Depended by**: Device power management, CPU hot-plug, suspend/resume infrastructure, system services (L3)

## Public Interface

- `pm` module: Power management subsystem for suspend, resume, and CPU C-states (`init_pm()`, `suspend()`, `resume()`, `set_cpu_cstate()`)
- `power` module: ACPI power driver for system power states (`init_acpi()`, `acpi_enter_sleep_state()`, `acpi_shutdown()`)
- `hotplug` module: CPU and memory hot-plug support (`init_hotplug()`, `hotplug_cpu()`, `hotplug_memory()`)

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `hotplug`, `pm`, `power`

---

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
