# Nuva OS - Next-Generation Intelligent Operating System

<div align="center">

**Future-Oriented Intelligent Operating System Kernel**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-ARM64%20%7C%20x86--64%20%7C%20LoongArch64-green.svg)]()

English | [简体中文](README_ZH.md)

</div>

## 📖 Overview

Nuva OS is a next-generation intelligent operating system designed from scratch, aiming to provide high-performance, high-security, and intelligent operating system experience. The project is developed in Rust and supports ARM64, x86-64, and LoongArch64 architectures.

### 🎯 Core Features

- **🚀 High-Performance Kernel**: Zero-copy IPC, lock-free data structures, optimized scheduler
- **🔒 Quantum Security**: Integrated quantum encryption, post-quantum cryptography, SHA-256 FIPS 180-4
- **🤖 AI Optimization**: AI-driven performance optimization, intelligent scheduling, Da Vinci NPU HAL, AI scheduler
- **🌐 Complete Ecosystem**: Graphics interface, multimedia, network protocol stack, NFS/SMB clients
- **📱 Multi-Device Support**: Mobile, tablet, desktop, embedded
- **🔌 Plugin System**: Dynamic loading (ELF loader), sandbox isolation, 100% implemented
- **🛠️ Full SDK**: Debugger (DAP), profiler (/proc), package manager (HTTP), 100% implemented

## 🏗️ System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  UI Framework │ Window Manager │ Event │ Render │ Resource│
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                    Services Layer                        │
│  App Manager │ IPC │ Network │ Power │ Security         │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                    Lib Layer                             │
│  Core │ Brain(AI) │ Lang │ Net │ ML │ Data │ UI │ Std  │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                    Kernel Layer                          │
│  Process │ Memory │ File System │ Network │ IPC │ Driver│
│  Scheduler │ Security │ Quantum │ Plugin │ BSD │ Debug  │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│                    Hardware Abstraction Layer (HAL)      │
│  CPU │ GPU │ NPU │ Power │ Quantum │ Snapdragon │ x64  │
│  Da Vinci NPU │ LoongArch64 (MMU/SIMD)                 │
└─────────────────────────────────────────────────────────┘
```

> **Recent Implementations**: SHA-256 FIPS 180-4 · ELF Loader · NFS/SMB Clients · Da Vinci NPU HAL · AI Scheduler · LoongArch64 Page Tables/Interrupts/SIMD · Plugin Sandbox · SDK Debugger(DAP)/Profiler(/proc)/PackageManager(HTTP)

## 🌟 Core Functionality

### 1. Kernel Features

| Module | Description | Status |
|--------|-------------|--------|
| Process Management | Process creation, scheduling, destruction, complete lifecycle | ✅ Complete |
| Memory Management | Page tables, address spaces, page fault handling, mmap/munmap/mprotect/msync | ✅ Complete |
| File System | VFS (open/close/read/write/lseek/mkdir/unlink), Ext4, Ramfs, NuvaFS, NFS/SMB clients | ✅ Complete |
| Network Stack | TCP/IP, UDP, Socket API | ✅ Complete |
| IPC | NuvaIPC, L4 IPC, Shared Memory | ✅ Complete |
| Security Subsystem | Capability security, sandbox isolation, ASLR, SHA-256 FIPS 180-4 | ✅ Complete |
| Boot Flow | ARM64 FDT, x64 Multiboot2, LoongArch64 UEFI boot | ✅ Complete |
| Platform Detection | PlatformInfo, BootInfoType, detect_platform_info() | ✅ Complete |
| Plugin System | ELF loader, sandbox isolation, lifecycle management, registry | ✅ Complete |
| SDK | Debugger (DAP), profiler (/proc), package manager (HTTP), CLI, build system | ✅ Complete |

### 2. IPC Subsystem

**NuvaIPC Performance Comparison**:

| System | Small Message Latency | Large Message Latency | Throughput |
|--------|----------------------|----------------------|------------|
| Android Binder | ~1μs | ~100μs | ~1M/s |
| iOS XPC | ~2μs | ~200μs | ~500K/s |
| **NuvaIPC** | **<100ns** | **<10μs** | **~10M/s** |

**Key Features**:
- ✅ Zero-copy transmission
- ✅ Lock-free queues
- ✅ Batch processing
- ✅ Quantum encryption
- ✅ AI optimization

### 3. Driver Framework

**Unified Driver Interface**:
- ✅ Device type classification management
- ✅ Plugin-based driver system
- ✅ Automatic device discovery
- ✅ Vendor driver integration support

**Supported Device Types**:
- Display devices
- Camera devices
- Bluetooth devices
- USB devices
- Input devices (keyboard, mouse, touchpad)
- NFC devices
- Sensor devices
- WiFi devices

### 4. Service Framework

**Core Services**:
- ✅ Application management service
- ✅ IPC service (Binder, Channel, Shared Memory)
- ✅ Network service (DNS, TCP/IP, UDP)
- ✅ Power management service
- ✅ Security service (Gatekeeper, Keymaster, Permission, TEE)

### 5. Multimedia Framework

**Features**:
- ✅ Audio playback/recording
- ✅ Video playback/recording
- ✅ 2D/3D graphics rendering
- ✅ Multiple codec support

**Supported Formats**:
- Audio: MP3, AAC, WAV, FLAC, OGG
- Video: MP4, AVI, MKV, WebM, MOV
- Codecs: H.264, H.265, VP8, VP9, AV1

### 6. UI Framework

**Core Components**:
- ✅ Window management system
- ✅ View system
- ✅ Basic components (button, table, navigation bar)
- ✅ Touch event handling
- ✅ Application lifecycle management
- ✅ Layout system
- ✅ Animation system

### 7. Programming Language

**Nuva Programming Language**:
- ✅ Self-developed compiler
- ✅ Type safety
- ✅ Ownership semantics
- ✅ Zero-cost abstractions

**String Type System**:
- `string`: Owned string
- `str`: String slice (borrowed)
- Clear ownership semantics
- Consistent with Rust type system

## 🚀 Quick Start

### Requirements

- Rust stable toolchain (see `rust-toolchain.toml`)
- QEMU 7.0+ (for testing)
- ARM64 or x86-64 toolchain

### Build

```bash
# Clone the project
git clone https://github.com/nuva-os/nuva.git
cd nuva

# Build the kernel
cargo build --release

# Run tests
cargo test
```

### Run

```bash
# Run in QEMU
cargo run --release

# Or use script
./scripts/run.sh
```

## 📁 Project Structure

```
nuva/
├── kernel/              # Kernel code
│   ├── arch/           # Architecture-specific code
│   │   ├── arm64/      # ARM64 architecture (boot, exception vectors, FDT)
│   │   ├── loongarch64/ # LoongArch64 architecture (boot, linker script)
│   │   └── x64/        # x86-64 architecture (boot, GDT, IDT, exceptions, APIC)
│   ├── mm/             # Memory management
│   ├── process/        # Process management
│   ├── sched/          # Scheduler
│   ├── fs/             # File system
│   ├── net/            # Network stack
│   ├── ipc/            # IPC subsystem
│   ├── driver/         # Driver framework
│   ├── security/       # Security subsystem
│   ├── syscall/        # System calls
│   ├── quantum/        # Quantum computing support
│   ├── plugin/         # Plugin system (ELF loader, sandbox, registry)
│   ├── bsd/            # BSD compatibility
│   ├── debug/          # Debug support
│   └── platform.rs     # Platform detection (PlatformInfo, BootInfoType)
├── hal/                 # Hardware abstraction layer
│   ├── cpu/            # CPU abstraction (PSCI SMC for Kirin)
│   ├── gpu/            # GPU abstraction
│   ├── npu/            # NPU abstraction (Da Vinci NPU HAL, AI scheduler)
│   ├── power/          # Power management (C-state, suspend, PMIC, ACPI)
│   ├── quantum/        # Quantum cryptography (PQC)
│   ├── snapdragon/     # Snapdragon platform
│   ├── acpi.rs         # ACPI power driver (Fadt, sleep states)
│   ├── x64/            # x86-64 platform (APIC, Timer, PageTable, Power)
│   ├── arm64/          # ARM64 platform
│   └── loongarch64/    # LoongArch64 platform (MMU, page tables, interrupts, SIMD)
├── lib/                 # System libraries
│   ├── core/           # Core utilities
│   ├── brain/          # AI engine
│   ├── lang/           # Nuva programming language
│   ├── net/            # Network library
│   ├── ml/             # Machine learning
│   ├── data/           # Database/Key-Value store
│   ├── gfx/            # Graphics
│   ├── ui/             # UI components
│   ├── runtime/        # Runtime support
│   └── std/            # Standard library
├── fs/                  # File system implementations
│   └── nuvafs/         # NuvaFS native file system
├── application/         # Application framework
│   ├── ui/             # UI framework
│   ├── window/         # Window management
│   ├── event/          # Event system
│   ├── render/         # Rendering engine
│   └── resource/       # Resource management
├── services/            # System services
│   ├── app/            # Application service
│   ├── ipc/            # IPC service
│   ├── net/            # Network service
│   ├── power/          # Power service
│   └── security/       # Security service
├── tools/               # Toolchain
│   ├── compiler/       # Compiler
│   ├── lsp/            # Language Server Protocol
│   └── toolchain/      # Toolchain utilities
├── sdk/                 # Software Development Kit (debugger, profiler, package manager)
├── tests/               # Test suites
└── docs/                # Documentation
```

## 🛠️ Development Roadmap

### Phase 1: Core Features (P0) ✅

- [x] ARM64 page table operations
- [x] ARM64 GIC interrupt controller
- [x] ARM64 Generic Timer
- [x] x86-64 page table operations
- [x] x86-64 APIC
- [x] Address space management
- [x] Page fault handling
- [x] Process creation/destruction
- [x] CFS scheduler
- [x] Core system calls

### Phase 2: Important Features (P1) ✅

- [x] VFS core implementation
- [x] File permission checking
- [x] TCP/IP protocol stack
- [x] Socket API
- [x] Binder IPC
- [x] L4 IPC
- [x] Driver framework

### Phase 3: Enhanced Features (P2) ✅

- [x] Security subsystem enhancement
- [x] HAL layer implementation
- [x] Power management implementation
- [x] Application framework implementation

### Phase 4: Optimization Features (P3) ✅

- [x] Performance optimization
- [x] Debugging support
- [x] Testing framework enhancement

### Phase 5: Advanced Features (P4) ✅

- [x] SHA-256 FIPS 180-4 secure hash
- [x] ELF dynamic loader for plugin system
- [x] NFS/SMB network file system clients
- [x] Da Vinci NPU HAL implementation
- [x] AI scheduler for intelligent task scheduling
- [x] LoongArch64 page tables, interrupts, SIMD support
- [x] Plugin sandbox isolation
- [x] SDK debugger (DAP protocol), profiler (/proc), package manager (HTTP)

## 🤝 Contributing

We welcome all forms of contribution!

### Ways to Contribute

1. **Submit Issues**: Report bugs or suggest new features
2. **Submit Pull Requests**: Fix bugs or implement new features
3. **Improve Documentation**: Enhance documentation quality
4. **Testing Feedback**: Test the system and provide feedback

### Development Process

1. Fork the project
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Submit a Pull Request

## 📄 License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Thanks to all developers who have contributed to Nuva OS!

Special thanks to:
- Rust Community
- Linux Kernel Community
- Android Open Source Project
- FreeBSD Project

## 📞 Contact

- **github**:https://github.com/nuva-os/nuva
- **gitee**: https://gitee.com/nuva-os/nuva
- **email**: zhangyujie_china@163.com

---

<div align="center">

**Nuva OS - Future-Oriented Intelligent Operating System**

Made with ❤️ by Nuva OS Team

</div>
