# Nuva OS 快速入门指南

本指南帮助您开始 Nuva OS 的开发和测试。

---

## 目录

- [前置条件](#前置条件)
- [快速开始](#快速开始)
- [构建](#构建)
- [运行测试](#运行测试)
- [使用 QEMU 运行](#使用-qemu-运行)
- [调试](#调试)
- [常见问题](#常见问题)

---

## 前置条件

### 必需工具

| 工具 | 版本 | 用途 |
|------|------|------|
| Rust | nightly | 编译 kernel（参见 `rust-toolchain.toml`） |
| rust-src | nightly 组件 | `no_std` 构建所需的 Rust 源码 |
| QEMU | >= 7.0 | 仿真和运行 |
| Git | >= 2.0 | 版本控制 |

### 可选工具

| 工具 | 用途 |
|------|------|
| GDB / gdb-multiarch | 调试 kernel |
| VS Code | 代码编辑与 DAP 调试 |
| cargo-binutils | 二进制分析（`cargo size`、`cargo objdump`） |
| rust-analyzer | IDE 语言服务 |

---

## 快速开始

### 1. 安装 Rust nightly 工具链

```bash
# 安装 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 nightly 工具链并设为默认
rustup install nightly
rustup override set nightly
rustup update
```

### 2. 安装目标平台与必需组件

```bash
# ARM64 目标
rustup target add --toolchain nightly aarch64-unknown-none

# x86-64 目标
rustup target add --toolchain nightly x86_64-unknown-none

# LoongArch64 目标（需要自定义 target JSON）
rustup target add --toolchain nightly loongarch64-unknown-none

# 安装 rust-src 组件（no_std 构建必需）
rustup component add --toolchain nightly rust-src

# 安装辅助组件
rustup component add --toolchain nightly rustfmt
rustup component add --toolchain nightly clippy
```

> **注意**：`rust-src` 是 `no_std` 项目构建 `core`/`alloc` 的必需组件。项目 `.cargo/config.toml` 中配置了 `build-std = ["core", "compiler_builtins", "alloc"]`，因此必须安装此组件。

### 3. 安装 QEMU

**Ubuntu/Debian：**
```bash
sudo apt update
sudo apt install qemu-system-arm qemu-system-x86
```

**macOS：**
```bash
brew install qemu
```

**Windows：**
从 [QEMU 官方网站](https://www.qemu.org/download/) 下载并添加到 PATH。

**Arch Linux：**
```bash
sudo pacman -S qemu-emulation-full
```

### 4. 克隆仓库

```bash
git clone https://github.com/nuva-os/nuva.git
cd nuva
```

### 5. 验证环境

```bash
# 确认 nightly 工具链
rustup show

# 确认目标已安装
rustup target list --installed

# 确认 rust-src 已安装
rustup component list --installed | grep rust-src
```

---

## 构建

Nuva OS 使用 Cargo feature flag 选择目标平台和硬件配置。所有 feature 定义在根 `Cargo.toml` 中。

### Feature Flag 总览

| Feature | 架构 | 说明 |
|---------|------|------|
| `arm64` | AArch64 | 通用 ARM64 |
| `x64` | x86_64 | 通用 x86-64 |
| `loongarch64` | LoongArch64 | 通用 LoongArch64 |
| `kirin` | AArch64 | 鲲鹏平台基类 |
| `kirin9000` | AArch64 | 鲲鹏 9000 |
| `kirin9010` | AArch64 | 鲲鹏 9010 |
| `kirin9020` | AArch64 | 鲲鹏 9020（含 `kirin`） |
| `snapdragon8gen4` | AArch64 | 骁龙 8 Gen 4 |
| `intel_core` | x86_64 | Intel Core 系列 |
| `amd_ryzen` | x86_64 | AMD Ryzen 系列 |
| `loongson3a6000` | LoongArch64 | 龙芯 3A6000 |
| `loongson3c6000` | LoongArch64 | 龙芯 3C6000 |
| `smp` | 通用 | SMP 多核支持 |
| `debug` | 通用 | 调试模式 |

### 构建 ARM64 版本

```bash
# Kirin9020 平台
cargo build --target aarch64-unknown-none --features kirin9020

# Kirin9000 平台
cargo build --target aarch64-unknown-none --features kirin9000

# Kirin9010 平台
cargo build --target aarch64-unknown-none --features kirin9010

# Snapdragon 8 Gen 4 平台
cargo build --target aarch64-unknown-none --features snapdragon8gen4

# 通用 ARM64 + SMP
cargo build --target aarch64-unknown-none --features "arm64,smp"
```

### 构建 x86-64 版本

```bash
# 通用 x86-64
cargo build --target x86_64-unknown-none --features x64

# Intel Core 平台
cargo build --target x86_64-unknown-none --features intel_core

# AMD Ryzen 平台
cargo build --target x86_64-unknown-none --features amd_ryzen
```

### 构建 LoongArch64 版本

```bash
# 龙芯 3A6000 桌面
cargo build --target loongarch64-unknown-none --features loongson3a6000

# 龙芯 3C6000 服务器
cargo build --target loongarch64-unknown-none --features loongson3c6000
```

### 发布构建

```bash
cargo build --target aarch64-unknown-none --features kirin9020 --release
```

---

## 运行测试

### 运行所有测试

```bash
cargo test
```

### 运行指定测试

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

### 运行性能基准测试

```bash
cargo bench
```

### 查看测试输出

```bash
cargo test -- --nocapture
```

---

## 使用 QEMU 运行

### ARM64（virt 平台）

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

> **注意**：kernel 二进制名称为 `nuva_kernel`（对应 `Cargo.toml` 中 `[[bin]] name = "nuva_kernel"`）。

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

# Run with QEMU (需要 loongarch64 版本 QEMU)
qemu-system-loongarch64 \
    -m 1G \
    -nographic \
    -kernel target/loongarch64-unknown-none/debug/nuva_kernel
```

### QEMU 常用参数

| 参数 | 描述 |
|------|------|
| `-M virt` | 使用 virt 虚拟机（ARM64） |
| `-cpu cortex-a57` | 指定 CPU 类型（ARM64） |
| `-m 1G` | 分配 1GB 内存 |
| `-nographic` | 无 GUI，使用串口输出 |
| `-kernel` | 指定 kernel 镜像 |
| `-s` | 启动 GDB 服务器（端口 1234） |
| `-S` | 启动时暂停 CPU |
| `-device loader,file=dtb,addr=0x40000000` | 加载设备树 |

---

## 调试

### 使用 GDB

**终端 1 - 启动 QEMU：**
```bash
qemu-system-aarch64 -M virt -cpu cortex-a57 -m 1G -nographic \
    -kernel target/aarch64-unknown-none/debug/nuva_kernel \
    -s -S
```

**终端 2 - 启动 GDB：**
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

**x86-64 调试：**
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

### 常用 GDB 命令

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

### VS Code 调试配置（GDB）

创建 `.vscode/launch.json`：

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

### VS Code DAP 调试配置

Nuva OS SDK 内置 DAP（Debug Adapter Protocol）服务器，支持通过 VS Code 的 DAP 接口进行调试（`sdk/debug/dap/`）。

创建 `.vscode/launch.json` 使用 DAP：

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

## 常见问题

### 问：编译错误 "can't find crate for std"

**答：** Nuva OS 是 `no_std` 项目，需要使用 `*-unknown-none` 目标，且必须安装 `rust-src` 组件：

```bash
rustup component add --toolchain nightly rust-src
rustup target add --toolchain nightly aarch64-unknown-none
cargo build --target aarch64-unknown-none
```

### 问：编译错误 "error[E0554]: `#![feature]` may not be used on the stable release channel"

**答：** Nuva OS 需要 nightly 工具链。确认已切换到 nightly：

```bash
rustup override set nightly
rustup show
```

### 问：QEMU 没有输出

**答：** 检查以下内容：
1. 确保使用了 `-nographic` 参数
2. 确保 kernel 已正确初始化串口（ARM64 使用 PL011 UART at `0x0900_0000`）
3. 检查 kernel 是否正确加载
4. 尝试添加 `-serial mon:stdio` 参数

### 问：如何查看 kernel 日志？

**答：** kernel 使用串口输出日志。在 QEMU 中使用 `-nographic` 参数后，日志将输出到终端。

### 问：如何添加新平台支持？

**答：**
1. 在 `Cargo.toml` 的 `[features]` 中添加新 feature
2. 在 `hal/` 目录中创建平台 HAL
3. 在 `kernel/arch/` 中添加架构支持代码
4. 在 `sdk/build-config.toml` 中添加平台构建配置

### 问：测试失败怎么办？

**答：**
1. 检查错误信息
2. 确保所有依赖已安装（特别是 `rust-src`）
3. 运行 `cargo clean` 并重新构建
4. 检查 Rust 版本是否与 `rust-toolchain.toml` 匹配（nightly）

### 问：如何贡献代码？

**答：** 请参见 [CONTRIBUTING.md](CONTRIBUTING.md)。

---

## 下一步

- 阅读[架构文档](ARCHITECTURE.md)了解系统设计
- 查看 [API 文档](API.md)了解接口定义
- 阅读[编码规范](CODING_STANDARD_zh.md)了解代码风格
- 查看[贡献指南](CONTRIBUTING.md)参与开发
- 查看[开发路线图](ROADMAP_zh.md)了解项目规划

---

## 获取帮助

- **GitHub Issues**：https://github.com/nuva-os/nuva/issues
- **文档**：[docs/](docs/) 目录
- **邮箱**：team@nuva-os.org

---

**最后更新**：2026 年 5 月 15 日
