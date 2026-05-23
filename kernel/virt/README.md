# Virtualization Subsystem

## Overview

The `kernel/virt/` module provides kernel-level virtualization support across all supported architectures: VMX (x86-64 Intel VT-x / AMD-V), VHE (ARM64 Virtualization Host Extensions), and LVZ (LoongArch64 Virtualization Extension including LBT for binary translation).

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point |
| `vmx.rs` | VMX virtualization support (x86-64 VMX, ARM64 VHE, LoongArch64 LVZ) |

## Architecture Support

| Architecture | Virtualization Technology |
|--------------|--------------------------|
| x86-64 | VMX (Intel VT-x / AMD-V) |
| ARM64 | VHE (Virtualization Host Extensions) |
| LoongArch64 | LVZ (LoongArch Virtualization Extension) + LBT (Binary Translation) |

## Initialization Order

Virtualization initializes in Phase 8 (Platform & Diagnostics), after APIC operations have been configured:

1. APIC operations (`apic_ops::init_apic_ops`) — Phase 8 prerequisite
2. VMX/VHE/LVZ initialization (`vmx::init_vmx`) — Phase 8

## Dependencies

- **Internal dependencies**: `kernel/core` (CPU), `kernel/irq_mgmt` (APIC ops)
- **Depended by**: Virtual machine monitors, hypervisor services (L3)

## Public Interface

- `vmx` module: Unified virtualization API abstracting over x86-64 VMX, ARM64 VHE, and LoongArch64 LVZ
  - `init_vmx()`: Initialize the hardware virtualization extension for the current architecture
  - VM entry/exit control, EPT/NPT page table management, VMCS/VMCB state configuration

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `vmx`

---

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
