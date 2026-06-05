# Changelog

This file records all important changes to Nuva OS.

Format based on [Keep a Changelog](https://keepachangelog.com/),
version numbers follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- NvScheduler AI intelligent scheduler with NPU inference, four-level scheduling classes (AI_REALTIME/AI_NORMAL/AI_BATCH/AI_IDLE), three-tier fallback (AI→Declarative→CFS+RT), and 12-dimensional feature vector
- NvBalancer heterogeneous hardware load balancer with device topology management, load collection, balance optimization, migration execution, oscillation detection (32-entry ring buffer), and hot-plug support
- NvPowerMgr AI-driven power optimization with power budget management (5% overshoot allowance), DVFS controller (safe switching sequences), device power control (critical device protection), thermal monitor (85°C proactive throttling), green computing metrics (PUE/carbon/efficiency), and AI power optimizer
- Three-party cooperation mechanism: NvScheduler↔NvBalancer↔NvPowerMgr with runtime invariant verification
- Sched-Power cooperation: scheduling decisions evaluate power impact via NvPowerMgr
- Sched-Balancer cooperation: NvScheduler drives NvBalancer load balancing
- Balancer-Power cooperation: balance decisions consider device power efficiency
- Power-Sched cooperation: NvPowerMgr never sleeps devices with active high-priority tasks
- Declarative policy engine enhancement with ai_confidence_threshold, inference_budget_us, power_aware_enabled, balancer_driven fields
- RISC-V 64 (RV64G) architecture support (SBI boot, page tables, PLIC, trap handling, timer)
- `riscv64` and `qemu_virt` feature flags
- `skip_dep_check` feature flag
- HAL RISC-V 64 platform module (CPU, MMU, interrupt controller)
- Kernel RISC-V 64 architecture module (boot/SBI, trap, MMU, PLIC, timer, context)
- RISC-V Sv39 3-level page table walk (map/unmap/translate/protect) with page table allocation/deallocation and superpage support
- Maleoon GPU interrupt handler (fence/GART fault/hang/cmd-complete) and VRAM allocator (best-fit with coalescing)
- Da Vinci NPU interrupt handler (inference-done/error/model-loaded/hang) and recyclable model memory manager (best-fit with per-model free and coalescing)
- DVFS hardware interface (dvfs_set_frequency/dvfs_set_voltage) with register-level implementation
- Thermal management with passive throttling at 85°C and critical shutdown at 105°C

### Changed

- Updated project structure to include RISC-V 64 architecture directories
- Updated supported platforms to include RISC-V 64 (QEMU virt)
- Updated build system with RISC-V 64 build and run targets
- MALEOON_GPU_OPS now bridges to actual MaleoonGpuHal methods (was placeholder stubs)
- PMIC_POWER_OPS now bridges to actual PmicDriver methods (was placeholder stubs)
- CpuFreqInfo::set_freq() now calls DVFS hardware via dvfs_set_frequency()
- PowerManager::power_off()/reboot() now iterate power domains and call platform ops
- PowerManager::register_default_domains() now properly registers 3 default domains

### Fixed

- Fix layer dependency analyzer false positives for FFI boundary modules
- Fix POSIX compatibility feature gating consistency across build targets

## [1.0.0] - 2026-05-27

### Added

- Full multi-architecture support (ARM64, x86-64, LoongArch64, RISC-V 64)
- Quantum-safe security (CRYSTALS-Kyber, CRYSTALS-Dilithium, SHA-256 FIPS 180-4)
- AI-native design (Da Vinci NPU HAL, AI-driven scheduler with EAS)
- Zero-copy IPC (NuvaIPC <100ns small message latency)
- Plugin architecture (ELF dynamic loader, sandbox isolation, hot-plug)
- Full SDK (debugger DAP, profiler /proc, package manager HTTP, CLI, build system)
- LoongArch64 page tables, interrupts, SIMD support
- Nuva programming language (.nv) with declarative paradigm

## [0.1.0] - 2026-01-01

### Added

- Initial project scaffold with `#![no_std]` Rust bare-metal kernel
- ARM64 (AArch64) architecture support (boot, GIC, MMU, exception vectors)
- x86-64 architecture support (boot, GDT, IDT, APIC, exceptions)
- Basic memory management (buddy allocator, SLAB allocator, page tables)
- Basic process management (process creation, O(1) scheduler, context switching)
- Basic system call interface and handler framework
- Basic VFS layer with file operations (open/close/read/write)
- HAL layer with CPU, GPU, and interrupt abstractions
- Build system with Cargo and Makefile

## Version Notes

### Version Number Format

- **Major**: Incompatible API changes
- **Minor**: Backward-compatible functionality additions
- **Patch**: Backward-compatible bug fixes

### Change Types

- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Features to be removed soon
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security-related fixes
