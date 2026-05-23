# Nuva OS Documentation

## Overview

Nuva OS is a modern operating system built from scratch in pure Rust, featuring a microkernel architecture design based on the `#![no_std]` bare-metal environment.

**Version**: v1.0.0
**License**: Apache 2.0
**Toolchain**: Rust nightly (configured in `rust-toolchain.toml` with `channel = "nightly"`, components: `rust-src`, `rustfmt`, `clippy`, targets: `aarch64-unknown-none`, `x86_64-unknown-none`)
**Supported Architectures**: ARM64 (AArch64), x86-64, LoongArch64
**Build Status**: ARM64 ✅ Passing / x86-64 ✅ Passing / LoongArch64 ✅ Passing

---

## Project Structure

```
Nuva/
├── kernel/              # Kernel Layer (L1)
│   ├── arch/            #   Architecture-specific code (arm64, x64, loongarch64)
│   ├── mm/              #   Memory Management (Buddy, SLAB, VMA, NUMA, COW, HugePage, OOM)
│   ├── sched/           #   Process Scheduling (CFS, EAS, RT, Load Balancing)
│   ├── process/         #   Process Management (fork, execve, signal, wait4)
│   ├── fs/              #   Kernel File System (VFS, Page Cache, io_uring)
│   ├── net/             #   Network Stack (TCP/UDP/IPv6/ARP)
│   ├── ipc/             #   Inter-Process Communication (NuvaIPC, Shared Memory, L4, Zero-Copy)
│   ├── syscall/         #   System Call Interface
│   ├── security/        #   Security Module (LSM, ASLR, Sandbox, Stack Canary)
│   ├── driver/          #   Driver Framework (Device Model, Bus, IRQ, DMA, DMA-BUF)
│   │   ├── framework/   #     Driver Core Framework (display, input)
│   │   ├── class/       #     Device Classes (audio, backlight, bluetooth, camera, ...)
│   │   ├── impl/        #     Driver Implementations (irqchip)
│   │   ├── device.rs    #     Device trait definition
│   │   ├── dma.rs       #     DMA channel abstraction
│   │   ├── dmabuf.rs    #     DMA-BUF shared buffer framework
│   │   ├── gpio.rs      #     GPIO subsystem
│   │   ├── i2c.rs       #     I2C bus driver
│   │   ├── spi.rs       #     SPI bus driver
│   │   ├── irq.rs       #     Interrupt request management
│   │   ├── pm.rs        #     Driver power management integration
│   │   ├── dt.rs        #     Device tree matching and parsing
│   │   └── ...          #     (clk, freq, opp, phy, pinctrl, pwm, regulator, ...)
│   ├── drivers/         #   Concrete Driver Implementations (irqchip/GIC)
│   ├── quantum/         #   Quantum Scheduler
│   ├── plugin/          #   Plugin System (ELF Loader, Manager, Sandbox, Registry)
│   ├── sync/            #   Synchronization Primitives (Spinlock, Mutex)
│   ├── interrupt/       #   Interrupt Management
│   ├── timer/           #   Timer
│   ├── perf/            #   Performance Monitoring
│   ├── debug/           #   Debug Support (printk)
│   └── bsd/             #   BSD Compatibility Layer
├── hal/                 # Hardware Abstraction Layer (L0)
│   ├── cpu/             #   CPU Abstraction (DVFS, Kirin SoC, Loongson SoC, Thermal)
│   ├── gpu/             #   GPU Abstraction (Maleoon GPU, Command Queue)
│   ├── npu/             #   NPU Abstraction (Da Vinci NPU HAL, ONNX Runtime, AI Scheduler, Predictor)
│   ├── quantum/         #   Quantum Cryptography (PQC: Kyber/Dilithium, QRNG, QKD)
│   ├── power/           #   Power Management (PMIC, Suspend/Resume)
│   ├── ffi/             #   Foreign Function Interface (C API, C++ API, API Stability)
│   ├── input.rs         #   Input Device
│   ├── platform.rs      #   Platform Detection
│   ├── dt.rs            #   Device Tree Parser (ARM64)
│   ├── acpi.rs          #   ACPI Table Parser (x86_64)
│   ├── arm64/           #   ARM64 Architecture-Specific Implementation
│   ├── x64/             #   x86_64 Architecture-Specific Implementation
│   ├── loongarch64/     #   LoongArch64 Architecture-Specific Implementation (Page Tables, Interrupts, SIMD)
│   └── snapdragon/      #   Qualcomm Snapdragon Platform
├── syslib/              # System Library Layer (L2)
│   ├── core/            #   Core Library (Allocator, Sync Primitives)
│   ├── brain/           #   Nuva Brain AI Engine
│   ├── ai/              #   AI Library
│   ├── lang/            #   NuvaLang Compiler and Runtime
│   ├── ml/              #   Machine Learning Library (Tensor, Model, Inference Engine)
│   ├── net/             #   Network Library (HTTP, WebSocket, JSON)
│   ├── data/            #   Data Structure Library
│   ├── gfx/             #   Graphics Library
│   ├── ui/              #   UI Library (Layout, View, Window)
│   ├── std/             #   Standard Library (Collections, Basic Types, IO)
│   ├── runtime/         #   Runtime Library (Arc, Metadata, Protocol)
│   └── dispatch/        #   Concurrency Framework (GCD-style, Thread Pool)
├── services/            # System Services Layer (L3)
│   ├── app/             #   Application Service (Activity, Package Manager)
│   ├── ipc/             #   IPC Service (Binder, Channel)
│   ├── net/             #   Network Service (DNS, TCP/UDP)
│   ├── power/           #   Power Service (Policy, Wake Lock)
│   ├── security/        #   Security Service (Gatekeeper, Keymaster, TEE)
│   └── form_factor.rs   #   Form Factor Manager
├── application/         # Application Framework Layer (L4)
│   ├── ui/              #   UI Framework (Adaptive Layout, Components)
│   ├── window/          #   Window Management
│   ├── event/           #   Event System
│   ├── render/          #   Rendering Engine (Compositor, Brush)
│   └── resource/        #   Resource Management (Cache, Decoder)
├── posix/               # POSIX Compatibility Layer
│   ├── unistd.rs        #   POSIX Process and File Operations
│   ├── fcntl.rs         #   File Control
│   ├── signal.rs        #   Signal Handling
│   └── errno.rs         #   Error Codes
├── fs/                  # File System Implementations
│   ├── ext4/            #   ext4 File System
│   ├── fat32/           #   FAT32 File System
│   └── nuvafs/          #   NuvaFS Custom File System
├── sdk/                 # Software Development Kit
│   ├── cli/             #   Command Line Interface
│   ├── build/           #   Build System
│   ├── debug/           #   Debugger (DAP Protocol)
│   ├── package/         #   Package Manager (HTTP)
│   └── profiler/        #   Profiler (/proc)
├── tools/               #   Toolchain Collection
│   ├── dep_analyzer/    #   Dependency Analyzer (Layer Compliance Check, build.rs Integration)
│   ├── compiler/        #   Compiler Tools
│   ├── linker/          #   Linker Tools
│   ├── lsp/             #   Language Server Protocol
│   └── toolchain/       #   Toolchain Management
├── configs/             #   Architecture Layer Compliance Configs (layers/{hal,kernel,lib})
├── sysroot/             #   System Root (C Header Files)
├── toolchains/          #   Cross-Compilation Toolchain Configs
├── scripts/             #   Build and Documentation Generation Scripts
├── tests/               #   Test Suites
├── benches/             #   Performance Benchmarks
├── examples/            #   C/C++ Example Programs
└── docs/                #   Project Documentation
```

---

## Module Completion Status

| Module | Framework | Functionality | Overall | Notes |
|--------|-----------|---------------|---------|-------|
| Memory Management | 95% | 95% | 95% | Buddy/SLAB/VMA/NUMA/COW/HugePage/OOM |
| Process Scheduling | 90% | 90% | 90% | CFS/EAS/RT/Deadline/Load Balancing |
| File System | 90% | 90% | 90% | VFS/NuvaFS/ext4/FAT32/io_uring/NFS/SMB |
| Network Stack | 80% | 85% | 82% | TCP/UDP/IPv6/ARP/Congestion Control |
| Device Drivers | 75% | 70% | 72% | Device Model/Bus/IRQ/DMA/GPIO/I2C/SPI/DMA-BUF |
| System Calls | 90% | 90% | 90% | POSIX Interface Coverage |
| Security Module | 90% | 85% | 87% | LSM/ASLR/Sandbox/Stack Canary/Secure Boot/SHA-256 |
| Power Management | 85% | 60% | 72% | PMIC/Suspend/Resume/Driver PM Integration |
| Quantum Security | 85% | 80% | 82% | Kyber/Dilithium/QRNG/QKD |
| AI/NPU | 90% | 85% | 87% | Da Vinci NPU HAL/ONNX/Predictor/AI Scheduler |
| Plugin System | 100% | 100% | 100% | ELF Loader/Manager/Sandbox/Registry |
| LoongArch64 | 95% | 90% | 92% | HAL/Page Tables/Interrupts/SIMD/Extension Detection |
| SDK | 100% | 100% | 100% | Debugger(DAP)/Profiler(/proc)/PackageManager(HTTP)/CLI/Build |

---

## Documentation Index

### Core Documents

| Document | Chinese | English | Description |
|----------|---------|---------|-------------|
| Architecture | [ARCHITECTURE_zh.md](ARCHITECTURE_zh.md) | [ARCHITECTURE.md](ARCHITECTURE.md) | Microkernel architecture, module design |
| Memory | [MEMORY_zh.md](MEMORY_zh.md) | [MEMORY.md](MEMORY.md) | Physical memory, virtual memory, NUMA, COW |
| Process | [PROCESS_zh.md](PROCESS_zh.md) | [PROCESS.md](PROCESS.md) | Scheduling, process control, load balancing |
| Filesystem | [FILESYSTEM_zh.md](FILESYSTEM_zh.md) | [FILESYSTEM.md](FILESYSTEM.md) | VFS, NuvaFS |
| Syscall | [SYSCALL_zh.md](SYSCALL_zh.md) | [SYSCALL.md](SYSCALL.md) | POSIX interface, error codes |
| API | [API_zh.md](API_zh.md) | [API.md](API.md) | Kernel API, filesystem API, IPC API |

### Development Documents

| Document | Chinese | English | Description |
|----------|---------|---------|-------------|
| Quick Start | [QUICK_START_zh.md](QUICK_START_zh.md) | [QUICK_START.md](QUICK_START.md) | Setup, build, run |
| Coding Standard | [CODING_STANDARD_zh.md](CODING_STANDARD_zh.md) | [CODING_STANDARD.md](CODING_STANDARD.md) | Coding conventions |

### Planning Documents

| Document | Chinese | English | Description |
|----------|---------|---------|-------------|
| Roadmap | [ROADMAP_zh.md](ROADMAP_zh.md) | [ROADMAP.md](ROADMAP.md) | TODO, priorities |

### Architecture & Standards

| Document | Chinese | English | Description |
|----------|---------|---------|-------------|
| Layer Rules | [architecture/LAYER_RULES_zh.md](architecture/LAYER_RULES_zh.md) | [architecture/LAYER_RULES.md](architecture/LAYER_RULES.md) | Layer dependency constraints |
| Doc Standard | [standards/DOCUMENTATION_STANDARD_zh.md](standards/DOCUMENTATION_STANDARD_zh.md) | [standards/DOCUMENTATION_STANDARD.md](standards/DOCUMENTATION_STANDARD.md) | Documentation standards |
| Module Template | — | — | Module template (pending creation) |
| Driver Guide | [development/DRIVER_DEVELOPMENT_GUIDE_zh.md](development/DRIVER_DEVELOPMENT_GUIDE_zh.md) | [development/DRIVER_DEVELOPMENT_GUIDE.md](development/DRIVER_DEVELOPMENT_GUIDE.md) | Driver development |
| API Reference | [api/API_REFERENCE_zh.md](api/API_REFERENCE_zh.md) | [api/API_REFERENCE.md](api/API_REFERENCE.md) | HAL API reference |

---

## Quick Links

- [快速入门](QUICK_START_zh.md) / [Quick Start](QUICK_START.md)
- [系统架构](ARCHITECTURE_zh.md) / [Architecture](ARCHITECTURE.md)
- [内存管理](MEMORY_zh.md) / [Memory](MEMORY.md)
- [进程管理](PROCESS_zh.md) / [Process](PROCESS.md)
- [文件系统](FILESYSTEM_zh.md) / [Filesystem](FILESYSTEM.md)
- [系统调用](SYSCALL_zh.md) / [Syscall](SYSCALL.md)
- [API 参考](API_zh.md) / [API](API.md)
- [开发路线图](ROADMAP_zh.md) / [Roadmap](ROADMAP.md)
- [层级规则](architecture/LAYER_RULES_zh.md) / [Layer Rules](architecture/LAYER_RULES.md)
- [驱动开发指南](development/DRIVER_DEVELOPMENT_GUIDE_zh.md) / [Driver Guide](development/DRIVER_DEVELOPMENT_GUIDE.md)
- [API 参考手册](api/API_REFERENCE_zh.md) / [API Reference](api/API_REFERENCE.md)

---

## Design Philosophy

Nuva OS adheres to the following core principles:

1. **Simplicity & Elegance**: Clean interfaces, modular design, single responsibility
2. **Policy/Mechanism Separation**: Kernel provides mechanisms, userspace decides policy
3. **Everything is a File**: Devices, pipes, sockets use unified VFS interface
4. **Memory Safety**: Leveraging Rust's safety guarantees
5. **High Performance**: Modern optimization techniques for critical paths
6. **Post-Quantum Security**: Integrated NIST PQC standard algorithms (Kyber/Dilithium)
7. **AI Native**: NPU abstraction layer and intelligent optimization

---

## Core Features

### Memory Management
- Physical memory management with Buddy and SLAB allocators
- Virtual memory management with VMA and page tables
- Advanced features: NUMA support, COW mechanism, huge pages, OOM killer
- Page migration and memory compaction

### Process Management
- Complete process lifecycle management
- CFS (Completely Fair Scheduler) for normal processes
- EAS (Energy-Aware Scheduling)
- RT (Real-Time) scheduler with FIFO and RR policies
- Multi-core load balancing and CPU affinity control

### File System
- VFS (Virtual File System) abstraction
- NuvaFS native file system (journaling, snapshots, POSIX compatible)
- ext4 and FAT32 file system support
- NFS and SMB network file system clients
- Page cache and buffer cache for file caching
- io_uring async I/O support

### System Services
- Power management with multiple sleep states
- Security services with Gatekeeper/Keymaster/TEE
- Network services with DNS and TCP/IP stack
- Application services with Binder IPC

### Quantum Security
- CRYSTALS-Kyber key encapsulation (NIST standard)
- CRYSTALS-Dilithium digital signatures (NIST standard)
- SHA-256 secure hash (FIPS 180-4)
- QRNG quantum random number generation
- QKD quantum key distribution

### AI Engine (Nuva Brain)
- ML model inference engine
- NPU scheduling and management (DaVinci architecture, Da Vinci NPU HAL implemented)
- AI scheduler for intelligent task scheduling and load balancing
- Tensor operation support
- ONNX runtime integration

### Application Framework
- UI framework with adaptive layout
- Window management system
- Event processing system
- Rendering engine with hardware acceleration
- Resource management (JPEG/PNG/TTF/WAV decoding)

---

## Quick Start

### Prerequisites

- Rust **nightly** toolchain (see `rust-toolchain.toml`)
- QEMU >= 7.0
- Git >= 2.0

### Build

```bash
# Install toolchain
rustup install nightly
rustup override set nightly
rustup target add --toolchain nightly aarch64-unknown-none
rustup target add --toolchain nightly x86_64-unknown-none
rustup component add rust-src

# ARM64 target (Kirin)
cargo build --target aarch64-unknown-none --features kirin9020

# x86-64 target
cargo build --target x86_64-unknown-none --features x64
```

### Dependency Compliance Check

```bash
# Run layer dependency analyzer
cargo run --bin dep_analyzer -- .
```

### Run

```bash
# ARM64
qemu-system-aarch64 -machine virt -cpu cortex-a76 -nographic \
    -kernel target/aarch64-unknown-none/debug/nuva_kernel

# x86-64
qemu-system-x86_64 -nographic \
    -kernel target/x86_64-unknown-none/debug/nuva_kernel
```

See [QUICK_START.md](QUICK_START.md) for detailed instructions.

---

## Contributing

Contributions are welcome! Please follow the coding conventions in [CODING_STANDARD.md](CODING_STANDARD.md).

---

## Support

- **Issues**: https://github.com/nuva-os/nuva/issues
- **Docs**: [docs/](.) directory
- **Email**: zhangyujie_china@163.com

---

**Last Updated**: May 15, 2026
**Maintainer**: Nuva OS Team
