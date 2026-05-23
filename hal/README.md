# HAL — Hardware Abstraction Layer (L0)

## Overview

HAL (Hardware Abstraction Layer) is the lowest layer (Layer 0) of Nuva OS, providing a unified hardware access interface. HAL has no dependencies on any other layer; all hardware-related operations are abstracted through HAL for use by upper layers.

## Submodules

| Submodule | Description |
|-----------|-------------|
| cpu/ | CPU abstraction: DVFS, Kirin SoC (PSCI SMC CPU_ON/CPU_OFF), Loongson SoC, thermal management |
| gpu/ | GPU abstraction: Maleoon GPU, command queue |
| npu/ | NPU abstraction: Da Vinci NPU HAL bridge, ONNX runtime, AI scheduler, inference predictor, device management |
| quantum/ | Quantum cryptography: PQC (Kyber KEM, Dilithium signature), QRNG quantum random number, QKD quantum key distribution |
| power/ | Power management: PMIC, suspend/resume, cross-architecture C-state (MWAIT/WFI/idle) |
| ffi/ | Foreign function interface: C API (nuva_hal.h), C++ API (nuva_hal.hpp), ABI stability |
| input.rs | Input device HAL |
| platform.rs | Platform detection and identification (architecture, SoC, form factor, BootInfoType) |
| dt.rs | Device tree parser (ARM64 FDT/DTB) |
| acpi.rs | ACPI table parser (x86_64), AcpiPowerDriver (Fadt, enter_sleep_state, S3/S5) |
| arm64/ | ARM64 architecture-specific HAL implementation (FDT boot, exception vectors) |
| x64/ | x86_64 architecture-specific HAL (LAPIC/I/O APIC, GDT, IDT, CPU, MMU, Timer, Power, PageTable) |
| loongarch64/ | LoongArch64 architecture-specific HAL (UEFI boot, 3-level page tables, Pte struct, LSX SIMD, LASX, LBT, LVZ) |
| snapdragon/ | Qualcomm Snapdragon platform HAL |

## Dependencies

- **Lower dependencies**: None (lowest layer)
- **Depended by**: kernel (L1), syslib (L2)

## Build Configuration

| Feature | Condition | Description |
|---------|-----------|-------------|
| `arm64` | arch = arm64 | Enable ARM64 HAL |
| `x64` | arch = x86_64 | Enable x86_64 HAL |
| `loongarch64` | arch = loongarch64 | Enable LoongArch64 HAL |
| `snapdragon8gen4` | arm64 | Enable Snapdragon 8 Gen 4 specific implementation |
| `kirin9020` | arm64, kirin | Enable Kirin 9020 specific implementation |

## Public Interface

HAL defines unified hardware interfaces through traits: `CpuHal`, `GpuHal`, `NpuHal`, `PowerHal`, etc. Each architecture provides concrete implementations via conditional compilation.
