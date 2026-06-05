# Nuva OS — Next-Generation Intelligent Operating System

<div align="center">

**A Modern, Quantum-Safe, AI-Native Operating System**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-ARM64%20%7C%20x86--64%20%7C%20LoongArch64%20%7C%20RISC--V%2064-green.svg)]()

English | [简体中文](README_ZH.md)

</div>

## Overview

Nuva OS is a next-generation operating system built from scratch in Rust (`#![no_std]` bare-metal), designed for modern mobile and embedded devices. It delivers high performance, quantum-safe security, and AI-native intelligence across ARM64, x86-64, LoongArch64, and RISC-V 64 (RV64G) architectures.

### Core Pillars

- **Quantum-Safe Security**: NIST PQC standards (CRYSTALS-Kyber, CRYSTALS-Dilithium), SHA-256 FIPS 180-4, hardware QRNG integration (MMIO/DT/ACPI entropy), QKD BB84 protocol implementation
- **AI-Native Design**: Unified NPU abstraction (Da Vinci NPU HAL), AI-driven scheduler with EAS
- **Three-Level Microkernel**: EL2 (min kernel) / EL1 (equipment mode) / EL0 (user mode), capability-gated NvSupervisorCall, equipment fault isolation
- **High Performance**: Zero-copy IPC (<50ns small message, <5μs large), O(1) port lookup, lock-free data structures, buddy+SLAB allocators
- **Capability Security**: NvCapability tokens replace uid/gid, permission monotonicity, cascading revocation
- **Plugin Architecture**: ELF dynamic loader, sandbox isolation, hot-plug, lifecycle management (100% implemented)
- **Multi-Architecture**: ARM64, x86-64, LoongArch64, RISC-V 64 (page tables, interrupts, SIMD)
- **Full SDK**: Debugger (DAP protocol), profiler (/proc), package manager (HTTP), CLI, build system (100% implemented)

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                 Application Layer (L4)                   │
│    UI Framework │ Window Manager │ Event │ Render │ Resource │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  Services Layer (L3)                     │
│  App │ IPC │ Net │ Power │ Security │ Audio │ Video │ Web │
│  OpenGL │ SQLite │ Image │ FormFactor │ CoreProcessing   │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   Syslib Layer (L2)                      │
│  Core │ Brain(AI) │ Lang │ Net │ ML │ Data │ GFX │ UI    │
│  Runtime │ Std │ Dispatch │ AI │ Posix                   │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   Kernel Layer (L1)                      │
│  Process │ Memory │ FileSystem │ Network │ IPC │ Driver  │
│  Scheduler │ Security │ Quantum │ Plugin │ BSD │ Debug   │
│  Syscall │ Sync │ Interrupt │ Timer │ Virt │ Device      │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                Hardware Abstraction Layer (L0)           │
│  CPU │ GPU │ NPU │ Power │ Quantum │ FFI │ Input │ DT    │
│  ARM64 │ x64 │ LoongArch64 │ RISC-V 64 │ Snapdragon │ ACPI │
└─────────────────────────────────────────────────────────┘
```

### Layer Dependency Constraints

| Layer | Depends On | Description |
|-------|-----------|-------------|
| L0 — HAL | None | Hardware abstraction; no cross-layer dependencies |
| L1 — Kernel | L0 | Core kernel; depends only on HAL |
| L2 — Syslib | L0, L1 | System libraries; may use Kernel API and HAL traits |
| L3 — Services | L0, L1, L2 | System services layer |
| L4 — Application | L0, L1, L2, L3 | Application framework |

> **Recent implementations**: SHA-256 FIPS 180-4 · ELF Loader · NFS/SMB Clients · io_uring · TCP State Machine · Firewall · NvCapability-LSM Bridge · Hardware QRNG · QKD BB84 · Da Vinci NPU HAL · AI Scheduler · LoongArch64 Page Tables / Interrupts / SIMD · Plugin Sandbox · SDK Debugger (DAP) / Profiler (/proc) / Package Manager (HTTP) · RISC-V Sv39 3-Level Page Table Walk · GPU/NPU Interrupt Handlers + VRAM/Model Memory Allocators · DVFS Hardware + Thermal Management · NvScheduler/NvBalancer/NvPowerMgr + Three-Party Cooperation

## Core Functionality

### 1. Kernel Features

| Module | Description | Status |
|--------|-------------|--------|
| Process Management | Process creation, scheduling, destruction, full lifecycle | Done |
| Memory Management | Page tables, address spaces, page fault handling, mmap/munmap/mprotect/msync, OOM killer | Done |
| File System | VFS (open/close/read/write/lseek/mkdir/unlink), Ext4, Ramfs, NuvaFS, NFS/SMB clients, io_uring async I/O | Done |
| Network Stack | TCP/IP (full state machine RFC 793), UDP, Socket API, Firewall (stateless rules, NAT, rate limiting) | Done |
| IPC | NuvaIPC, L4 IPC, Shared Memory | Done |
| Security Subsystem | Capability security, sandbox isolation, ASLR, SHA-256 FIPS 180-4 | Done |
| Boot Flow | ARM64 FDT, x64 Multiboot2, LoongArch64 UEFI boot, RISC-V 64 SBI boot | Done |
| Platform Detection | PlatformInfo, BootInfoType, detect_platform_info() | Done |
| Plugin System | ELF loader, sandbox isolation, lifecycle management, registry | Done |
| SDK | Debugger (DAP), profiler (/proc), package manager (HTTP), CLI, build system | Done |

### 2. IPC Subsystem

**NuvaIPC Performance Comparison**:

| System | Small Message Latency | Large Message Latency | Throughput |
|--------|----------------------|----------------------|------------|
| Android Binder | ~1μs | ~100μs | ~1M/s |
| iOS XPC | ~2μs | ~200μs | ~500K/s |
| **NuvaIPC** | **<100ns** | **<10μs** | **~10M/s** |

**Key Features**:
- Zero-copy transmission
- Lock-free queues (MPSC/SPSC)
- Batch processing
- Quantum encryption
- AI optimization

### 3. Driver Framework

**Unified Driver Interface**:
- Device type classification management
- Plugin-based driver system
- Automatic device discovery
- Vendor driver integration support

**Supported Device Types**:
- Display, Camera, Bluetooth, USB
- Input devices (keyboard, mouse, touchpad)
- NFC, Sensor, WiFi

### 4. Service Framework

**Core Services**:
- Application management service
- IPC service (Binder, Channel, Shared Memory)
- Network service (DNS, TCP/IP, UDP)
- Power management service
- Security service (Gatekeeper, Keymaster, Permission, TEE)
- Audio / Video services
- OpenGL / SQLite / Image services
- Web service
- Form factor & core processing services

### 5. Multimedia Framework

**Features**: Audio playback/recording, video playback/recording, 2D/3D graphics rendering, multiple codec support

**Supported Formats**:
- Audio: MP3, AAC, WAV, FLAC, OGG
- Video: MP4, AVI, MKV, WebM, MOV
- Codecs: H.264, H.265, VP8, VP9, AV1

### 6. UI Framework

**Core Components**: Window management, view system, basic components (button, table, navigation bar), touch event handling, application lifecycle management, layout system, animation system

### 7. Nuva Programming Language

**Nuva Language** (`.nv` files):
- Self-developed compiler with type safety
- Ownership semantics and zero-cost abstractions
- Declarative paradigm — `component` for UI, `signal` for reactive state, `effect` for side effects
- `async`/`await` concurrency, `resource`/`with` for resource management
- `string` (owned) / `str` (borrowed) type system, consistent with Rust

## Quick Start

### Prerequisites

- Rust **nightly** toolchain (see `rust-toolchain.toml`)
- QEMU >= 7.0 (for testing)
- C/C++ toolchain (for example compilation)

### Build

```bash
# Clone the project
git clone https://github.com/nuva-os/nuva.git
cd nuva

# Install Rust nightly toolchain
rustup install nightly
rustup override set nightly

# Install target platforms
rustup target add --toolchain nightly aarch64-unknown-none
rustup target add --toolchain nightly x86_64-unknown-none
rustup target add --toolchain nightly riscv64-unknown-none

# Install required components
rustup component add rust-src

# Build for ARM64 + Kirin
cargo build --target aarch64-unknown-none --features arm64 --release

# Build for x86-64
cargo build --target x86_64-unknown-none --features x64 --release

# Build for RISC-V 64
cargo build --target riscv64-unknown-none --features riscv64 --release

# Run tests
cargo test
```

### Run in QEMU

```bash
# ARM64
qemu-system-aarch64 -machine virt -cpu cortex-a76 \
  -kernel target/aarch64-unknown-none/release/nuva_kernel

# x86-64
qemu-system-x86_64 -kernel target/x86_64-unknown-none/release/nuva_kernel

# RISC-V 64
qemu-system-riscv64 -machine virt -nographic -bios default \
  -kernel target/riscv64-unknown-none/release/nuva_kernel
```

### Quick Example

```c
#include <nuva_hal.h>

int main() {
    nuva_cpu_info_t cpu_info;
    nuva_cpu_get_info(&cpu_info);

    printf("CPU: %u cores @ %u MHz\n",
           cpu_info.core_count,
           cpu_info.frequency_mhz);
    return 0;
}
```

## Feature Flags

| Feature | Requires | Description |
|---------|----------|-------------|
| `arm64` | — | ARM64 architecture support |
| `x64` | — | x86-64 architecture support |
| `loongarch64` | — | LoongArch64 architecture support |
| `riscv64` | — | RISC-V 64 architecture support |
| `qemu_virt` | `riscv64` | QEMU virt machine for RISC-V 64 |
| `kirin` | `arm64` | HiSilicon Kirin SoC family |
| `kirin9000` | `arm64` | Kirin 9000 |
| `kirin9010` | `arm64` | Kirin 9010 |
| `kirin9020` | `arm64`, `kirin` | Kirin 9020 |
| `snapdragon8gen4` | `arm64` | Qualcomm Snapdragon 8 Gen 4 |
| `loongson3a6000` | `loongarch64` | Loongson 3A6000 |
| `loongson3c6000` | `loongarch64` | Loongson 3C6000 |
| `intel_core` | `x64` | Intel Core processors |
| `amd_ryzen` | `x64` | AMD Ryzen processors |
| `debug` | — | Debug mode |
| `smp` | — | Symmetric Multi-Processor support |
| `skip_dep_check` | — | Skip dependency checking |

## Supported Platforms

| Platform | Target Triple | Status | Notes |
|----------|-------------|--------|-------|
| ARM64 | `aarch64-unknown-none` | Builds | Kirin 9020, Snapdragon 8 Gen 4 |
| x86-64 | `x86_64-unknown-none` | Builds | Intel Core, AMD Ryzen |
| LoongArch64 | `loongarch64-unknown-none` | Builds | Loongson 3A6000 / 3C6000 |
| RISC-V 64 | `riscv64-unknown-none` | Builds | QEMU virt machine |

## Performance

| Component | Metric | Value |
|-----------|--------|-------|
| IPC (small message) | Latency | <100ns |
| IPC (large message) | Latency | <10μs |
| Memory Pool | Alloc/Free | <10ns |
| Lock-Free Queue | Push/Pop | <50ns |
| Kyber-768 | Key Generation | <1ms |
| Dilithium-3 | Signing | <1ms |

## Project Structure

```
nuva/
├── kernel/                # Kernel implementation (L1)
│   ├── arch/              # Architecture-specific code
│   │   ├── arm64/         # ARM64 (boot, exception vectors, GIC, MMU, FDT)
│   │   ├── loongarch64/   # LoongArch64 (boot, linker, MMU, interrupts, SIMD)
│   │   ├── riscv64/       # RISC-V 64 (boot/SBI, trap, MMU, PLIC, timer, context)
│   │   └── x64/           # x86-64 (boot, GDT, IDT, exceptions, APIC)
│   ├── mm/                # Memory management (buddy, SLAB, mmap, OOM)
│   ├── process/           # Process management
│   ├── sched/             # Scheduler (CFS, AI scheduler, EAS)
│   ├── fs/                # File system (VFS, ext4, ramfs)
│   ├── net/               # Network stack (TCP/IP, UDP, socket)
│   ├── ipc/               # IPC subsystem (NuvaIPC, L4, shared memory)
│   ├── driver/            # Driver framework
│   ├── security/          # Security subsystem (capability, sandbox, ASLR)
│   ├── syscall/           # System call interface
│   ├── quantum/           # Quantum computing support
│   ├── plugin/            # Plugin system (ELF loader, sandbox, registry)
│   ├── bsd/               # BSD compatibility layer
│   ├── debug/             # Debug & diagnostics
│   ├── device/            # Device management
│   ├── init/              # Kernel initialization
│   ├── diag/              # Diagnostics subsystem
│   ├── irq_mgmt/          # IRQ management
│   ├── net_stack/         # Network stack
│   ├── storage/           # Storage subsystem
│   ├── power_mgmt/        # Power management
│   ├── core/              # Core kernel services
│   └── virt/              # Virtualization support
├── hal/                   # Hardware Abstraction Layer (L0)
│   ├── cpu/               # CPU abstraction (PSCI SMC for Kirin)
│   ├── gpu/               # GPU abstraction
│   ├── npu/               # NPU abstraction (Da Vinci NPU HAL, AI scheduler)
│   ├── power/             # Power management (C-state, suspend, PMIC, ACPI)
│   ├── quantum/           # Quantum cryptography (PQC)
│   ├── ffi/               # C/C++ FFI bindings
│   ├── snapdragon/        # Snapdragon platform support
│   ├── arm64/             # ARM64 platform
│   ├── x64/               # x86-64 platform (APIC, Timer, PageTable, Power)
│   ├── loongarch64/       # LoongArch64 platform (MMU, page tables, interrupts, SIMD)
│   ├── riscv64/           # RISC-V 64 platform (CPU, MMU, interrupt controller)
│   ├── acpi.rs            # ACPI power driver (Fadt, sleep states)
│   ├── dt.rs              # Device tree parser
│   ├── input.rs           # Input subsystem
│   └── platform.rs        # Platform abstraction
├── syslib/                # System libraries (L2)
│   ├── core/              # Core utilities
│   ├── brain/             # AI engine
│   ├── lang/              # Nuva programming language compiler & runtime
│   ├── net/               # Network library
│   ├── ml/                # Machine learning
│   ├── data/              # Database / key-value store
│   ├── gfx/               # Graphics library
│   ├── ui/                # UI components
│   ├── runtime/           # Runtime support
│   ├── std/               # Standard library
│   ├── dispatch/          # Task dispatch
│   ├── ai/                # AI subsystem
│   └── posix/             # POSIX compatibility
├── application/           # Application framework (L4)
│   ├── ui/                # UI framework
│   ├── window/            # Window management
│   ├── event/             # Event system
│   ├── render/            # Rendering engine
│   └── resource/          # Resource management
├── services/              # System services (L3)
│   ├── app/               # Application service
│   ├── ipc/               # IPC service
│   ├── net/               # Network service
│   ├── power/             # Power service
│   ├── security/          # Security service
│   ├── audio/             # Audio service
│   ├── video/             # Video service
│   ├── web/               # Web service
│   ├── opengl/            # OpenGL service
│   ├── sqlite/            # SQLite service
│   ├── image/             # Image service
│   ├── form_factor/       # Form factor manager
│   └── core_processing/   # Core processing service
├── fs/                    # File system implementations
│   ├── nuvafs/            # NuvaFS native file system
│   ├── ext4/              # Ext4 file system
│   └── fat32/             # FAT32 file system
├── posix/                 # POSIX compatibility layer
├── tools/                 # Development toolchain
│   ├── compiler/          # Nuva language compiler
│   ├── lsp/               # Language Server Protocol
│   ├── linker/            # Linker
│   ├── dep_analyzer/      # Dependency analyzer
│   └── toolchain/         # Toolchain utilities
├── sdk/                   # Software Development Kit
│   ├── cli/               # CLI tools
│   ├── build/             # Build system
│   ├── debug/             # Debugger (DAP protocol)
│   ├── profiler/          # Profiler (/proc integration)
│   └── package/           # Package manager (HTTP)
├── scripts/               # Build & utility scripts
├── configs/               # Configuration files (layer rules)
├── docs/                  # Documentation
├── examples/              # Example code (C, C++, crypto)
├── editors/               # Editor integrations
├── tests/                 # Test suites
├── benches/               # Benchmarks
├── sysroot/               # System root filesystem
├── toolchains/            # Cross-compilation toolchains
├── build.rs               # Build script
├── Cargo.toml             # Cargo manifest
└── Makefile               # Make build system
```

## Development Roadmap

### Phase 1: Core Features (P0) — Done
- [x] ARM64 page table operations, GIC interrupt controller, Generic Timer
- [x] x86-64 page table operations, APIC
- [x] Address space management, page fault handling
- [x] Process creation/destruction, CFS scheduler
- [x] Core system calls

### Phase 2: Important Features (P1) — Done
- [x] VFS core implementation, file permission checking
- [x] TCP/IP protocol stack, Socket API
- [x] Binder IPC, L4 IPC
- [x] Driver framework

### Phase 3: Enhanced Features (P2) — Done
- [x] Security subsystem, HAL layer
- [x] Power management, application framework

### Phase 4: Optimization Features (P3) — Done
- [x] Performance optimization, debugging support, testing framework

### Phase 5: Advanced Features (P4) — Done
- [x] SHA-256 FIPS 180-4 secure hash
- [x] ELF dynamic loader for plugin system
- [x] NFS/SMB network file system clients
- [x] Da Vinci NPU HAL implementation
- [x] AI scheduler for intelligent task scheduling
- [x] LoongArch64 page tables, interrupts, SIMD support
- [x] RISC-V 64 SBI boot, page tables, PLIC, trap handling
- [x] Plugin sandbox isolation
- [x] SDK debugger (DAP protocol), profiler (/proc), package manager (HTTP)

### Phase 6: AI-Native Core (P5) — Done
- [x] NvScheduler AI intelligent scheduler (NPU inference, 4-level classes, 3-tier fallback)
- [x] NvBalancer heterogeneous hardware load balancer (topology, oscillation detection, hot-plug)
- [x] NvPowerMgr AI-driven power optimization (budget, DVFS, thermal, green metrics)
- [x] Three-party cooperation: NvScheduler↔NvBalancer↔NvPowerMgr with runtime invariants
- [x] NuvaFS WAL/COW/Snapshot, IPv6 neighbor discovery, secure boot, PQC compliance

### Phase 7: Hardware Integration (P6) — In Progress
- [x] RISC-V Sv39 3-level page table walk (map/unmap/translate/protect)
- [x] Maleoon GPU ops bridging + interrupt handler + VRAM allocator
- [x] Da Vinci NPU interrupt handler + recyclable model memory manager
- [x] CPU DVFS hardware calls + thermal management (85°C throttle / 105°C shutdown)
- [x] System power_off/reboot platform calls + domain registration
- [x] PMIC ops bridging to actual driver methods
- [ ] USB host controller driver
- [ ] LoongArch64 QEMU/LBT support

## Documentation

> Chinese documentation uses the `_zh` suffix; English documentation keeps the base filename.

- [Architecture Design](docs/ARCHITECTURE.md) / [架构设计](docs/ARCHITECTURE_zh.md)
- [Memory Management](docs/MEMORY.md) / [内存管理](docs/MEMORY_zh.md)
- [Process Management](docs/PROCESS.md) / [进程管理](docs/PROCESS_zh.md)
- [File System](docs/FILESYSTEM.md) / [文件系统](docs/FILESYSTEM_zh.md)
- [System Calls](docs/SYSCALL.md) / [系统调用](docs/SYSCALL_zh.md)
- [API Reference](docs/API.md) / [API 参考](docs/API_zh.md)
- [Quick Start Guide](docs/QUICK_START.md) / [快速入门](docs/QUICK_START_zh.md)
- [Roadmap](docs/ROADMAP.md) / [开发路线图](docs/ROADMAP_zh.md)
- [Coding Standards](docs/CODING_STANDARD.md) / [编码规范](docs/CODING_STANDARD_zh.md)
- [Performance](docs/PERFORMANCE.md) / [性能](docs/PERFORMANCE_zh.md)
- [Nuva Language Reference](docs/NUVA_LANG.md) / [Nuva 语言参考](docs/NUVA_LANG_zh.md)
- [Layer Rules](docs/architecture/LAYER_RULES.md) / [层级规则](docs/architecture/LAYER_RULES_zh.md)
- [Driver Development Guide](docs/development/DRIVER_DEVELOPMENT_GUIDE.md) / [驱动开发指南](docs/development/DRIVER_DEVELOPMENT_GUIDE_zh.md)
- [Documentation Standards](docs/standards/DOCUMENTATION_STANDARD.md) / [文档标准](docs/standards/DOCUMENTATION_STANDARD_zh.md)

## Module Documentation

| Module | Layer | Description |
|--------|-------|-------------|
| [hal](hal/README.md) | L0 | Hardware Abstraction Layer |
| [kernel](kernel/README.md) | L1 | Kernel |
| [syslib](syslib/README.md) | L2 | System Libraries |
| [services](services/README.md) | L3 | System Services |
| [application](application/README.md) | L4 | Application Framework |
| [posix](posix/README.md) | Aux | POSIX Compatibility Layer |
| [fs](fs/README.md) | Aux | File System Implementations |
| [sdk](sdk/README.md) | Aux | Software Development Kit |
| [tools](tools/README.md) | Aux | Toolchain Collection |
| [sysroot](sysroot/README.md) | Aux | System Root |

## Contributing

We welcome all forms of contribution! See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Ways to Contribute

1. **Submit Issues**: Report bugs or suggest new features
2. **Submit Pull Requests**: Fix bugs or implement new features
3. **Improve Documentation**: Enhance documentation quality
4. **Testing Feedback**: Test the system and provide feedback

### Development Process

1. Fork the project
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'feat: add AmazingFeature'`)
4. Run tests: `cargo test` and lint: `cargo clippy && cargo fmt --check`
5. Push to the branch (`git push origin feature/AmazingFeature`)
6. Submit a Pull Request

## License

This project is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

Thanks to all developers who have contributed to Nuva OS.

Special thanks to:
- Rust Embedded Community
- FreeBSD Project
- NIST PQC Standardization Process

## Contact

- **GitHub**: [https://github.com/nuva-os/nuva](https://github.com/nuva-os/nuva)
- **Email**: [kellen9903@gmail.com](mailto:kellen9903@gmail.com)

---

<div align="center">

**Nuva OS — Future-Oriented Intelligent Operating System**

Made with ❤️ by Nuva OS Team

</div>
