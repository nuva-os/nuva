# Nuva OS Quick Start Guide

This guide helps you get started with Nuva OS development and testing.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Building](#building)
- [Running Tests](#running-tests)
- [Running with QEMU](#running-with-qemu)
- [Debugging](#debugging)
- [FAQ](#faq)

---

## Prerequisites

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | nightly | Compile the kernel (see `rust-toolchain.toml`) |
| rust-src | nightly component | Rust source for `no_std` builds |
| QEMU | >= 7.0 | Emulation and execution |
| Git | >= 2.0 | Version control |

### Optional Tools

| Tool | Purpose |
|------|---------|
| GDB / gdb-multiarch | Debug the kernel |
| VS Code | Code editing and DAP debugging |
| cargo-binutils | Binary analysis (`cargo size`, `cargo objdump`) |
| rust-analyzer | IDE language service |

---

## Quick Start

### 1. Install the Rust Nightly Toolchain

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly toolchain and set as default
rustup install nightly
rustup override set nightly
rustup update
```

### 2. Install Targets and Required Components

```bash
# ARM64 target
rustup target add --toolchain nightly aarch64-unknown-none

# x86-64 target
rustup target add --toolchain nightly x86_64-unknown-none

# LoongArch64 target (requires custom target JSON)
rustup target add --toolchain nightly loongarch64-unknown-none

# RISC-V 64 target
rustup target add --toolchain nightly riscv64-unknown-none

# Install rust-src component (required for no_std builds)
rustup component add --toolchain nightly rust-src

# Install auxiliary components
rustup component add --toolchain nightly rustfmt
rustup component add --toolchain nightly clippy
```

> **Note**: `rust-src` is required for `no_std` projects to build `core`/`alloc`. The project's `.cargo/config.toml` sets `build-std = ["core", "compiler_builtins", "alloc"]`, so this component must be installed.

### 3. Install QEMU

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install qemu-system-arm qemu-system-x86
```

**macOS:**
```bash
brew install qemu
```

**Windows:**
Download from the [QEMU official website](https://www.qemu.org/download/) and add to PATH.

**Arch Linux:**
```bash
sudo pacman -S qemu-emulation-full
```

### 4. Clone the Repository

```bash
git clone https://github.com/nuva-os/nuva.git
cd nuva
```

### 5. Verify the Environment

```bash
# Confirm nightly toolchain
rustup show

# Confirm targets are installed
rustup target list --installed

# Confirm rust-src is installed
rustup component list --installed | grep rust-src
```

---

## Building

Nuva OS uses Cargo feature flags to select the target platform and hardware configuration. All features are defined in the root `Cargo.toml`.

### Feature Flag Overview

| Feature | Architecture | Description |
|---------|-------------|-------------|
| `arm64` | AArch64 | Generic ARM64 |
| `x64` | x86_64 | Generic x86-64 |
| `loongarch64` | LoongArch64 | Generic LoongArch64 |
| `kirin` | AArch64 | Kirin platform base |
| `kirin9000` | AArch64 | Kirin 9000 |
| `kirin9010` | AArch64 | Kirin 9010 |
| `kirin9020` | AArch64 | Kirin 9020 (implies `kirin`) |
| `snapdragon8gen4` | AArch64 | Snapdragon 8 Gen 4 |
| `intel_core` | x86_64 | Intel Core series |
| `amd_ryzen` | x86_64 | AMD Ryzen series |
| `loongson3a6000` | LoongArch64 | Loongson 3A6000 |
| `loongson3c6000` | LoongArch64 | Loongson 3C6000 |
| `riscv64` | RISC-V 64 | Generic RISC-V 64-bit (RV64G) |
| `qemu_virt` | RISC-V 64 | QEMU virt machine (implies `riscv64`) |
| `smp` | Generic | SMP multi-core support |
| `debug` | Generic | Debug mode |

### Build for ARM64

```bash
# Kirin9020 platform
cargo build --target aarch64-unknown-none --features kirin9020

# Kirin9000 platform
cargo build --target aarch64-unknown-none --features kirin9000

# Kirin9010 platform
cargo build --target aarch64-unknown-none --features kirin9010

# Snapdragon 8 Gen 4 platform
cargo build --target aarch64-unknown-none --features snapdragon8gen4

# Generic ARM64 + SMP
cargo build --target aarch64-unknown-none --features "arm64,smp"
```

### Build for x86-64

```bash
# Generic x86-64
cargo build --target x86_64-unknown-none --features x64

# Intel Core platform
cargo build --target x86_64-unknown-none --features intel_core

# AMD Ryzen platform
cargo build --target x86_64-unknown-none --features amd_ryzen
```

### Build for LoongArch64

```bash
# Loongson 3A6000 desktop
cargo build --target loongarch64-unknown-none --features loongson3a6000

# Loongson 3C6000 server
cargo build --target loongarch64-unknown-none --features loongson3c6000
```

### Build for RISC-V 64

```bash
# Generic RISC-V 64
cargo build --target riscv64-unknown-none --features riscv64

# QEMU virt machine
cargo build --target riscv64-unknown-none --features qemu_virt

# Using Makefile
make build-riscv
```

### Release Build

```bash
cargo build --target aarch64-unknown-none --features kirin9020 --release
```

### Release Build for RISC-V 64

```bash
cargo build --target riscv64-unknown-none --features riscv64 --release
```

---

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Specific Tests

```bash
# Run memory management tests
cargo test --test kernel_tests -- memory

# Run scheduler tests
cargo test --test kernel_tests -- scheduler

# Run quantum PQC tests
cargo test --test quantum_tests

# Run NPU tests
cargo test --test npu_tests

# Run plugin tests
cargo test --test plugin_tests
```

### Run Performance Benchmarks

```bash
cargo bench
```

### View Test Output

```bash
cargo test -- --nocapture
```

---

## Running with QEMU

### ARM64 (virt Platform)

```bash
# Build
cargo build --target aarch64-unknown-none --features kirin9020

# Run with QEMU virt machine
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a57 \
    -m 1G \
    -nographic \
    -kernel target/aarch64-unknown-none/debug/nuva_kernel
```

> **Note**: The kernel binary name is `nuva_kernel` (corresponding to `[[bin]] name = "nuva_kernel"` in `Cargo.toml`).

### x86-64

```bash
# Build
cargo build --target x86_64-unknown-none --features x64

# Run with QEMU
qemu-system-x86_64 \
    -m 1G \
    -nographic \
    -kernel target/x86_64-unknown-none/debug/nuva_kernel
```

### LoongArch64

```bash
# Build
cargo build --target loongarch64-unknown-none --features loongson3a6000

# Run with QEMU (requires loongarch64 version of QEMU)
qemu-system-loongarch64 \
    -m 1G \
    -nographic \
    -kernel target/loongarch64-unknown-none/debug/nuva_kernel
```

### RISC-V 64 (QEMU virt)

```bash
# Build
cargo build --target riscv64-unknown-none --features qemu_virt

# Run with QEMU virt machine (OpenSBI firmware)
qemu-system-riscv64 \
    -machine virt \
    -m 1G \
    -nographic \
    -bios default \
    -kernel target/riscv64-unknown-none/debug/nuva_kernel

# Run release build
qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios default \
    -kernel target/riscv64-unknown-none/release/nuva_kernel

# Or using Makefile
make run-riscv
```

### Common QEMU Arguments

| Argument | Description |
|----------|-------------|
| `-M virt` | Use virt virtual machine (ARM64) |
| `-cpu cortex-a57` | Specify CPU type (ARM64) |
| `-m 1G` | Allocate 1GB of memory |
| `-nographic` | No GUI, use serial output |
| `-kernel` | Specify kernel image |
| `-s` | Start GDB server (port 1234) |
| `-S` | Pause CPU at startup |
| `-device loader,file=dtb,addr=0x40000000` | Load device tree |

---

## Debugging

### Using GDB

**Terminal 1 - Start QEMU:**
```bash
qemu-system-aarch64 -M virt -cpu cortex-a57 -m 1G -nographic \
    -kernel target/aarch64-unknown-none/debug/nuva_kernel \
    -s -S
```

**Terminal 2 - Start GDB:**
```bash
# Install multi-architecture GDB (Ubuntu)
sudo apt install gdb-multiarch

# Start debugging
gdb-multiarch target/aarch64-unknown-none/debug/nuva_kernel

# Connect in GDB
(gdb) target remote :1234
(gdb) break kmain
(gdb) continue
```

**x86-64 Debugging:**
```bash
# Terminal 1
qemu-system-x86_64 -m 1G -nographic \
    -kernel target/x86_64-unknown-none/debug/nuva_kernel \
    -s -S

# Terminal 2
gdb target/x86_64-unknown-none/debug/nuva_kernel
(gdb) target remote :1234
(gdb) break _start
(gdb) continue
```

### Common GDB Commands

```
# Breakpoints
break kmain          # Set breakpoint at function
break *0x400000      # Set breakpoint at address
info breakpoints     # View breakpoints
delete 1             # Delete breakpoint 1

# Execution
continue             # Continue execution
step                 # Single step (enter function)
next                 # Single step (don't enter function)
finish               # Execute until function returns

# View
backtrace            # Call stack
info registers       # Registers
x/10i $pc            # Disassemble 10 instructions
print variable       # Print variable
```

### VS Code Debug Configuration (GDB)

Create `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug Kernel (ARM64)",
            "type": "cppdbg",
            "request": "launch",
            "program": "${workspaceFolder}/target/aarch64-unknown-none/debug/nuva_kernel",
            "miDebuggerServerAddress": "localhost:1234",
            "miDebuggerPath": "/usr/bin/gdb-multiarch",
            "stopAtEntry": true,
            "externalConsole": false,
            "MIMode": "gdb",
            "setupCommands": [
                {
                    "description": "Enable pretty-printing",
                    "text": "-enable-pretty-printing",
                    "ignoreFailures": true
                }
            ]
        },
        {
            "name": "Debug Kernel (x86-64)",
            "type": "cppdbg",
            "request": "launch",
            "program": "${workspaceFolder}/target/x86_64-unknown-none/debug/nuva_kernel",
            "miDebuggerServerAddress": "localhost:1234",
            "miDebuggerPath": "/usr/bin/gdb",
            "stopAtEntry": true,
            "externalConsole": false,
            "MIMode": "gdb"
        }
    ]
}
```

### VS Code DAP Debug Configuration

The Nuva OS SDK includes a built-in DAP (Debug Adapter Protocol) server for debugging through VS Code's DAP interface (`sdk/debug/dap/`).

Create `.vscode/launch.json` using DAP:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "DAP Debug Kernel (ARM64)",
            "type": "nuva-dap",
            "request": "launch",
            "target": "aarch64-unknown-none",
            "features": ["kirin9020"],
            "program": "${workspaceFolder}/target/aarch64-unknown-none/debug/nuva_kernel",
            "qemuPath": "qemu-system-aarch64",
            "qemuArgs": ["-M", "virt", "-cpu", "cortex-a57", "-m", "1G", "-nographic"],
            "stopAtEntry": true
        },
        {
            "name": "DAP Debug Kernel (x86-64)",
            "type": "nuva-dap",
            "request": "launch",
            "target": "x86_64-unknown-none",
            "features": ["x64"],
            "program": "${workspaceFolder}/target/x86_64-unknown-none/debug/nuva_kernel",
            "qemuPath": "qemu-system-x86_64",
            "qemuArgs": ["-m", "1G", "-nographic"],
            "stopAtEntry": true
        }
    ]
}
```

---

## FAQ

### Q: Compilation error "can't find crate for std"

**A:** Nuva OS is a `no_std` project. You must use `*-unknown-none` targets and install the `rust-src` component:

```bash
rustup component add --toolchain nightly rust-src
rustup target add --toolchain nightly aarch64-unknown-none
cargo build --target aarch64-unknown-none
```

### Q: Compilation error "error[E0554]: `#![feature]` may not be used on the stable release channel"

**A:** Nuva OS requires the nightly toolchain. Confirm the switch:

```bash
rustup override set nightly
rustup show
```

### Q: QEMU has no output

**A:** Check the following:
1. Ensure the `-nographic` argument is used
2. Ensure the kernel has properly initialized the serial port (ARM64 uses PL011 UART at `0x0900_0000`)
3. Verify the kernel is loaded correctly
4. Try adding the `-serial mon:stdio` argument

### Q: How to view kernel logs?

**A:** The kernel uses serial port output for logs. With the `-nographic` argument in QEMU, logs will appear in the terminal.

### Q: How to add new platform support?

**A:**
1. Add a new feature in `Cargo.toml` under `[features]`
2. Create platform HAL in the `hal/` directory
3. Add architecture support code in `kernel/arch/`
4. Add platform build configuration in `sdk/build-config.toml`

### Q: Tests are failing, what to do?

**A:**
1. Check error messages
2. Ensure all dependencies are installed (especially `rust-src`)
3. Run `cargo clean` and rebuild
4. Verify the Rust version matches `rust-toolchain.toml` (nightly)

### Q: How to contribute?

**A:** See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## Next Steps

- Read the [Architecture document](ARCHITECTURE.md) for system design
- See the [API document](API.md) for interface definitions
- Read the [Coding Standard](CODING_STANDARD.md) for code style
- See the [Contributing Guide](CONTRIBUTING.md) to participate in development
- See the [Roadmap](ROADMAP.md) for project planning

---

## Getting Help

- **GitHub Issues**: https://github.com/nuva-os/nuva/issues
- **Docs**: [docs/](docs/) directory
- **Email**: kellen9903@gmail.com

---

**Last Updated**: May 30, 2026
