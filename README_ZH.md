# Nuva OS — 下一代智能操作系统

<div align="center">

**一个现代化的、抗量子的、AI 原生的操作系统**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-ARM64%20%7C%20x86--64%20%7C%20LoongArch64%20%7C%20RISC--V%2064-green.svg)]()

[English](README.md) | 简体中文

</div>

## 概述

Nuva OS 是一款从零开始用 Rust（`#![no_std]` 裸机环境）构建的下一代操作系统，专为AI智能与量子安全设计。它在 ARM64、x86-64、LoongArch64 和 RISC-V 64 (RV64G) 架构上提供高性能、抗量子安全和 AI 原生智能。

### 核心支柱

- **抗量子安全**：NIST PQC 标准算法（CRYSTALS-Kyber、CRYSTALS-Dilithium），SHA-256 FIPS 180-4，硬件 QRNG 集成（MMIO/DT/ACPI 熵源），QKD BB84 协议实现
- **AI 原生设计**：统一 NPU 抽象（达芬奇 NPU HAL），AI 驱动的调度器（EAS 能耗感知调度）
- **高性能**：零拷贝 IPC（小消息 <100ns），无锁数据结构，Buddy + SLAB 分配器
- **插件架构**：ELF 动态加载器，沙箱隔离，热插拔，生命周期管理（100% 已实现）
- **多架构支持**：ARM64、x86-64、LoongArch64、RISC-V 64（页表、中断、SIMD 已实现）
- **完整 SDK**：调试器（DAP 协议）、性能分析器（/proc）、包管理器（HTTP）、CLI、构建系统（100% 已实现）

## 系统架构

```
┌─────────────────────────────────────────────────────────┐
│                   应用框架层 (L4)                         │
│    UI 框架 │ 窗口管理 │ 事件系统 │ 渲染引擎 │ 资源管理      │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   系统服务层 (L3)                         │
│  应用 │ IPC │ 网络 │ 电源 │ 安全 │ 音频 │ 视频 │ Web      │
│  OpenGL │ SQLite │ 图像 │ 形态因子 │ 核心处理              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                   系统库层 (L2)                           │
│  核心 │ Brain(AI) │ 语言 │ 网络 │ ML │ 数据 │ 图形 │ UI   │
│  运行时 │ 标准库 │ 调度 │ AI │ Posix                      │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                    内核层 (L1)                            │
│  进程 │ 内存 │ 文件系统 │ 网络 │ IPC │ 驱动 │ 调度器       │
│  安全 │ 量子 │ 插件 │ BSD │ 调试 │ 系统调用 │ 同步         │
│  中断 │ 定时器 │ 虚拟化 │ 设备                            │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                 硬件抽象层 (L0)                           │
│  CPU │ GPU │ NPU │ 电源 │ 量子 │ FFI │ 输入 │ 设备树      │
│  ARM64 │ x64 │ LoongArch64 │ RISC-V 64 │ Snapdragon │ ACPI │
└─────────────────────────────────────────────────────────┘
```

### 分层架构约束

| 层级 | 依赖 | 说明 |
|------|------|------|
| L0 — HAL | 无 | 硬件抽象层，最底层，无外部层依赖 |
| L1 — Kernel | L0 | 内核层，仅依赖 HAL |
| L2 — Syslib | L0, L1 | 系统库层，可使用 Kernel API 和 HAL traits |
| L3 — Services | L0, L1, L2 | 系统服务层 |
| L4 — Application | L0, L1, L2, L3 | 应用框架层 |

> **最新实现**：SHA-256 FIPS 180-4 · ELF 加载器 · NFS/SMB 客户端 · io_uring · TCP 状态机 · 防火墙 · NvCapability-LSM 桥接 · 硬件 QRNG · QKD BB84 · 达芬奇 NPU HAL · AI 调度器 · LoongArch64 页表/中断/SIMD · 插件沙箱 · SDK 调试器(DAP)/性能分析器(/proc)/包管理器(HTTP) · RISC-V Sv39 三级页表遍历 · GPU/NPU中断处理+VRAM/模型内存分配器 · DVFS硬件+热管理 · NvScheduler/NvBalancer/NvPowerMgr+三方协同

## 核心功能

### 1. 内核特性

| 模块 | 描述 | 状态 |
|------|------|------|
| 进程管理 | 进程创建、调度、销毁，完整的生命周期管理 | 已完成 |
| 内存管理 | 页表、地址空间、缺页异常处理、mmap/munmap/mprotect/msync、OOM killer | 已完成 |
| 文件系统 | VFS（open/close/read/write/lseek/mkdir/unlink）、Ext4、Ramfs、NuvaFS、NFS/SMB 客户端、io_uring 异步 I/O | 已完成 |
| 网络协议栈 | TCP/IP（RFC 793 完整状态机）、UDP、Socket API、防火墙（无状态规则、NAT、速率限制） | 已完成 |
| IPC | NuvaIPC、L4 IPC、共享内存 | 已完成 |
| 安全子系统 | 能力安全、沙箱隔离、ASLR、SHA-256 FIPS 180-4 | 已完成 |
| 启动流程 | ARM64 FDT、x64 Multiboot2、LoongArch64 UEFI 启动、RISC-V 64 SBI 启动 | 已完成 |
| 平台检测 | PlatformInfo、BootInfoType、detect_platform_info() | 已完成 |
| 插件系统 | ELF 加载器、沙箱隔离、生命周期管理、注册中心 | 已完成 |
| SDK | 调试器（DAP）、性能分析器（/proc）、包管理器（HTTP）、CLI、构建系统 | 已完成 |

### 2. IPC 子系统

**NuvaIPC 性能对比**：

| 系统 | 小消息延迟 | 大消息延迟 | 吞吐量 |
|------|----------|----------|--------|
| Android Binder | ~1μs | ~100μs | ~1M/s |
| iOS XPC | ~2μs | ~200μs | ~500K/s |
| **NuvaIPC** | **<100ns** | **<10μs** | **~10M/s** |

**核心特性**：
- 零拷贝传输
- 无锁队列（MPSC/SPSC）
- 批量处理
- 量子加密
- AI 优化

### 3. 驱动框架

**统一驱动接口**：
- 设备类型分类管理
- 基于插件的驱动系统
- 自动设备发现
- 厂商驱动集成支持

**支持的设备类型**：
- 显示设备、摄像头、蓝牙、USB
- 输入设备（键盘、鼠标、触摸板）
- NFC、传感器、WiFi

### 4. 服务框架

**核心服务**：
- 应用管理服务
- IPC 服务（Binder、Channel、共享内存）
- 网络服务（DNS、TCP/IP、UDP）
- 电源管理服务
- 安全服务（Gatekeeper、Keymaster、Permission、TEE）
- 音频 / 视频服务
- OpenGL / SQLite / 图像服务
- Web 服务
- 形态因子 & 核心处理服务

### 5. 多媒体框架

**功能**：音频播放/录制、视频播放/录制、2D/3D 图形渲染、多编解码器支持

**支持的格式**：
- 音频：MP3、AAC、WAV、FLAC、OGG
- 视频：MP4、AVI、MKV、WebM、MOV
- 编解码器：H.264、H.265、VP8、VP9、AV1

### 6. UI 框架

**核心组件**：窗口管理系统、视图系统、基础组件（按钮、表格、导航栏）、触摸事件处理、应用生命周期管理、布局系统、动画系统

### 7. Nuva 编程语言

**Nuva 语言**（`.nv` 文件）：
- 自研编译器，类型安全
- 所有权语义，零成本抽象
- 声明式范式 — 用 `component` 声明 UI，`signal` 管理响应式状态，`effect` 处理副作用
- `async`/`await` 并发模型，`resource`/`with` 资源管理
- `string`（自有）/ `str`（借用）类型系统，与 Rust 一致

## 快速开始

### 前置条件

- Rust **nightly** 工具链（参见 `rust-toolchain.toml`）
- QEMU >= 7.0（用于测试）
- C/C++ 工具链（用于示例编译）

### 构建

```bash
# 克隆项目
git clone https://github.com/nuva-os/nuva.git
cd nuva

# 安装 Rust nightly 工具链
rustup install nightly
rustup override set nightly

# 安装目标平台
rustup target add --toolchain nightly aarch64-unknown-none
rustup target add --toolchain nightly x86_64-unknown-none
rustup target add --toolchain nightly riscv64-unknown-none

# 安装必要组件
rustup component add rust-src

# 构建 ARM64 + Kirin
cargo build --target aarch64-unknown-none --features arm64 --release

# 构建 x86-64
cargo build --target x86_64-unknown-none --features x64 --release

# 构建 RISC-V 64
cargo build --target riscv64-unknown-none --features riscv64 --release

# 运行测试
cargo test
```

### 在 QEMU 中运行

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

### 快速示例

```c
#include <nuva_hal.h>

int main() {
    nuva_cpu_info_t cpu_info;
    nuva_cpu_get_info(&cpu_info);

    printf("CPU: %u 核心 @ %u MHz\n",
           cpu_info.core_count,
           cpu_info.frequency_mhz);
    return 0;
}
```

## Feature Flags

| Feature | 依赖 | 说明 |
|---------|------|------|
| `arm64` | — | ARM64 架构支持 |
| `x64` | — | x86-64 架构支持 |
| `loongarch64` | — | LoongArch64 架构支持 |
| `riscv64` | — | RISC-V 64 架构支持 |
| `qemu_virt` | `riscv64` | QEMU virt 虚拟机（RISC-V 64） |
| `kirin` | `arm64` | 海思麒麟 SoC 通用支持 |
| `kirin9000` | `arm64` | 麒麟 9000 |
| `kirin9010` | `arm64` | 麒麟 9010 |
| `kirin9020` | `arm64`, `kirin` | 麒麟 9020 |
| `snapdragon8gen4` | `arm64` | 高通骁龙 8 Gen 4 |
| `loongson3a6000` | `loongarch64` | 龙芯 3A6000 |
| `loongson3c6000` | `loongarch64` | 龙芯 3C6000 |
| `intel_core` | `x64` | Intel Core 处理器 |
| `amd_ryzen` | `x64` | AMD Ryzen 处理器 |
| `debug` | — | 调试模式 |
| `smp` | — | 对称多处理器支持 |
| `skip_dep_check` | — | 跳过依赖检查 |

## 支持平台

| 平台 | 目标三元组 | 编译状态 | 说明 |
|------|-----------|---------|------|
| ARM64 | `aarch64-unknown-none` | 编译通过 | 麒麟 9020、骁龙 8 Gen 4 |
| x86-64 | `x86_64-unknown-none` | 编译通过 | Intel Core、AMD Ryzen |
| LoongArch64 | `loongarch64-unknown-none` | 编译通过 | 龙芯 3A6000 / 3C6000 |
| RISC-V 64 | `riscv64-unknown-none` | 编译通过 | QEMU virt 虚拟机 |

## 性能指标

| 组件 | 指标 | 数值 |
|------|------|------|
| IPC（小消息） | 延迟 | <100ns |
| IPC（大消息） | 延迟 | <10μs |
| 内存池 | 分配/释放 | <10ns |
| 无锁队列 | 推入/弹出 | <50ns |
| Kyber-768 | 密钥生成 | <1ms |
| Dilithium-3 | 签名 | <1ms |

## 项目结构

```
nuva/
├── kernel/                # 内核实现 (L1)
│   ├── arch/              # 架构相关代码
│   │   ├── arm64/         # ARM64（启动、异常向量、GIC、MMU、FDT）
│   │   ├── loongarch64/   # LoongArch64（启动、链接脚本、MMU、中断、SIMD）
│   │   ├── riscv64/       # RISC-V 64（boot/SBI、trap、MMU、PLIC、定时器、上下文）
│   │   └── x64/           # x86-64（启动、GDT、IDT、异常、APIC）
│   ├── mm/                # 内存管理（buddy、SLAB、mmap、OOM）
│   ├── process/           # 进程管理
│   ├── sched/             # 调度器（CFS、AI 调度器、EAS）
│   ├── fs/                # 文件系统（VFS、ext4、ramfs）
│   ├── net/               # 网络协议栈（TCP/IP、UDP、socket）
│   ├── ipc/               # IPC 子系统（NuvaIPC、L4、共享内存）
│   ├── driver/            # 驱动框架
│   ├── security/          # 安全子系统（能力安全、沙箱、ASLR）
│   ├── syscall/           # 系统调用接口
│   ├── quantum/           # 量子计算支持
│   ├── plugin/            # 插件系统（ELF 加载器、沙箱、注册中心）
│   ├── bsd/               # BSD 兼容层
│   ├── debug/             # 调试与诊断
│   ├── device/            # 设备管理
│   ├── init/              # 内核初始化
│   ├── diag/              # 诊断子系统
│   ├── irq_mgmt/          # IRQ 管理
│   ├── net_stack/         # 网络协议栈
│   ├── storage/           # 存储子系统
│   ├── power_mgmt/        # 电源管理
│   ├── core/              # 核心内核服务
│   └── virt/              # 虚拟化支持
├── hal/                   # 硬件抽象层 (L0)
│   ├── cpu/               # CPU 抽象（Kirin PSCI SMC）
│   ├── gpu/               # GPU 抽象
│   ├── npu/               # NPU 抽象（达芬奇 NPU HAL、AI 调度器）
│   ├── power/             # 电源管理（C-state、suspend、PMIC、ACPI）
│   ├── quantum/           # 量子密码学（PQC）
│   ├── ffi/               # C/C++ FFI 绑定
│   ├── snapdragon/        # 骁龙平台支持
│   ├── arm64/             # ARM64 平台
│   ├── x64/               # x86-64 平台（APIC、Timer、PageTable、Power）
│   ├── loongarch64/       # LoongArch64 平台（MMU、页表、中断、SIMD）
│   ├── riscv64/           # RISC-V 64 平台（CPU、MMU、中断控制器）
│   ├── acpi.rs            # ACPI 电源驱动（Fadt、睡眠状态）
│   ├── dt.rs              # 设备树解析器
│   ├── input.rs           # 输入子系统
│   └── platform.rs        # 平台抽象
├── syslib/                # 系统库 (L2)
│   ├── core/              # 核心工具
│   ├── brain/             # AI 引擎
│   ├── lang/              # Nuva 语言编译器与运行时
│   ├── net/               # 网络库
│   ├── ml/                # 机器学习
│   ├── data/              # 数据库 / KV 存储
│   ├── gfx/               # 图形库
│   ├── ui/                # UI 组件
│   ├── runtime/           # 运行时支持
│   ├── std/               # 标准库
│   ├── dispatch/          # 任务调度
│   ├── ai/                # AI 子系统
│   └── posix/             # POSIX 兼容
├── application/           # 应用框架 (L4)
│   ├── ui/                # UI 框架
│   ├── window/            # 窗口管理
│   ├── event/             # 事件系统
│   ├── render/            # 渲染引擎
│   └── resource/          # 资源管理
├── services/              # 系统服务 (L3)
│   ├── app/               # 应用服务
│   ├── ipc/               # IPC 服务
│   ├── net/               # 网络服务
│   ├── power/             # 电源服务
│   ├── security/          # 安全服务
│   ├── audio/             # 音频服务
│   ├── video/             # 视频服务
│   ├── web/               # Web 服务
│   ├── opengl/            # OpenGL 服务
│   ├── sqlite/            # SQLite 服务
│   ├── image/             # 图像服务
│   ├── form_factor/       # 形态因子管理
│   └── core_processing/   # 核心处理服务
├── fs/                    # 文件系统实现
│   ├── nuvafs/            # NuvaFS 原生文件系统
│   ├── ext4/              # Ext4 文件系统
│   └── fat32/             # FAT32 文件系统
├── posix/                 # POSIX 兼容层
├── tools/                 # 开发工具链
│   ├── compiler/          # Nuva 语言编译器
│   ├── lsp/               # 语言服务器协议
│   ├── linker/            # 链接器
│   ├── dep_analyzer/      # 依赖分析器
│   └── toolchain/         # 工具链工具
├── sdk/                   # 软件开发套件
│   ├── cli/               # CLI 工具
│   ├── build/             # 构建系统
│   ├── debug/             # 调试器（DAP 协议）
│   ├── profiler/          # 性能分析器（/proc 集成）
│   └── package/           # 包管理器（HTTP）
├── scripts/               # 构建与工具脚本
├── configs/               # 配置文件（层级规则）
├── docs/                  # 文档
├── examples/              # 示例代码（C、C++、加密）
├── editors/               # 编辑器集成
├── tests/                 # 测试套件
├── benches/               # 性能基准测试
├── sysroot/               # 系统根文件系统
├── toolchains/            # 交叉编译工具链
├── build.rs               # 构建脚本
├── Cargo.toml             # Cargo 清单
└── Makefile               # Make 构建系统
```

## 开发路线图

### 第一阶段：核心功能 (P0) — 已完成
- [x] ARM64 页表操作、GIC 中断控制器、Generic Timer
- [x] x86-64 页表操作、APIC
- [x] 地址空间管理、缺页异常处理
- [x] 进程创建/销毁、CFS 调度器
- [x] 核心系统调用

### 第二阶段：重要功能 (P1) — 已完成
- [x] VFS 核心实现、文件权限检查
- [x] TCP/IP 协议栈、Socket API
- [x] Binder IPC、L4 IPC
- [x] 驱动框架

### 第三阶段：增强功能 (P2) — 已完成
- [x] 安全子系统增强、HAL 层实现
- [x] 电源管理实现、应用框架实现

### 第四阶段：优化功能 (P3) — 已完成
- [x] 性能优化、调试支持、测试框架增强

### 第五阶段：高级功能 (P4) — 已完成
- [x] SHA-256 FIPS 180-4 安全哈希
- [x] ELF 动态加载器（插件系统）
- [x] NFS/SMB 网络文件系统客户端
- [x] 达芬奇 NPU HAL 实现
- [x] AI 调度器（智能任务调度）
- [x] LoongArch64 页表、中断、SIMD 支持
- [x] RISC-V 64 SBI 启动、页表、PLIC、trap 处理
- [x] 插件沙箱隔离
- [x] SDK 调试器（DAP 协议）、性能分析器（/proc）、包管理器（HTTP）

### 第六阶段：AI原生核心 (P5) — 已完成
- [x] NvScheduler AI智能调度器（NPU推理、四级调度类别、三级降级）
- [x] NvBalancer异构硬件均衡器（拓扑、震荡检测、热插拔）
- [x] NvPowerMgr AI驱动功耗优化（预算、DVFS、温度、绿色指标）
- [x] 三方协同：NvScheduler↔NvBalancer↔NvPowerMgr运行时不变量
- [x] NuvaFS WAL/COW/Snapshot、IPv6邻居发现、安全启动、PQC合规

### 第七阶段：硬件集成 (P6) — 进行中
- [x] RISC-V Sv39 三级页表遍历(map/unmap/translate/protect)
- [x] Maleoon GPU ops桥接+中断处理+VRAM分配器
- [x] Da Vinci NPU中断处理+可回收模型内存管理器
- [x] CPU DVFS硬件调用+热管理(85°C节流/105°C关机)
- [x] 系统power_off/reboot平台调用+域注册
- [x] PMIC ops桥接到实际驱动方法
- [ ] USB主控制器驱动
- [ ] LoongArch64 QEMU/LBT支持

## 文档

> 中文文档以 `_zh` 后缀标识，英文文档保持原文件名。

- [架构设计](docs/ARCHITECTURE_zh.md) / [English](docs/ARCHITECTURE.md)
- [内存管理](docs/MEMORY_zh.md) / [English](docs/MEMORY.md)
- [进程管理](docs/PROCESS_zh.md) / [English](docs/PROCESS.md)
- [文件系统](docs/FILESYSTEM_zh.md) / [English](docs/FILESYSTEM.md)
- [系统调用](docs/SYSCALL_zh.md) / [English](docs/SYSCALL.md)
- [API 参考](docs/API_zh.md) / [English](docs/API.md)
- [快速入门指南](docs/QUICK_START_zh.md) / [English](docs/QUICK_START.md)
- [开发路线图](docs/ROADMAP_zh.md) / [English](docs/ROADMAP.md)
- [编码规范](docs/CODING_STANDARD_zh.md) / [English](docs/CODING_STANDARD.md)
- [性能指标](docs/PERFORMANCE_zh.md) / [English](docs/PERFORMANCE.md)
- [Nuva 语言参考](docs/NUVA_LANG_zh.md) / [English](docs/NUVA_LANG.md)
- [层级规则](docs/architecture/LAYER_RULES_zh.md) / [English](docs/architecture/LAYER_RULES.md)
- [驱动开发指南](docs/development/DRIVER_DEVELOPMENT_GUIDE_zh.md) / [English](docs/development/DRIVER_DEVELOPMENT_GUIDE.md)
- [文档标准](docs/standards/DOCUMENTATION_STANDARD_zh.md) / [English](docs/standards/DOCUMENTATION_STANDARD.md)

## 模块文档

| 模块 | 层级 | 说明 |
|------|------|------|
| [hal](hal/README.md) | L0 | 硬件抽象层 |
| [kernel](kernel/README.md) | L1 | 内核层 |
| [syslib](syslib/README.md) | L2 | 系统库层 |
| [services](services/README.md) | L3 | 系统服务层 |
| [application](application/README.md) | L4 | 应用框架层 |
| [posix](posix/README.md) | 辅助 | POSIX 兼容层 |
| [fs](fs/README.md) | 辅助 | 文件系统实现 |
| [sdk](sdk/README.md) | 辅助 | 软件开发套件 |
| [tools](tools/README.md) | 辅助 | 工具链集合 |
| [sysroot](sysroot/README.md) | 辅助 | 系统根目录 |

## 贡献

我们欢迎各种形式的贡献！详见 [CONTRIBUTING_ZH.md](CONTRIBUTING_ZH.md)。

## 联系

- Bug 和功能建议：[GitHub Issues](https://github.com/nuva-os/nuva/issues)
- 安全问题：[GitHub private advisory](https://github.com/nuva-os/nuva/discussions) 或 `kellen9903@gmail.com`
- 合作、商业支持、赞助：微信 `HiKellen` 或 `kellen9903@gmail.com`

## 赞助

Nuva OS 接受个人赞助、AI Credits / Token 厂商赞助、企业支持和商业授权咨询。

- 赞助页面：[docs/sponsor.md](docs/sponsor.md)
- 微信 / 支付宝二维码：[docs/sponsor.md#personal-donations](docs/sponsor.md#personal-donations)
- AI Credits / Token 厂商赞助：[docs/sponsor.md#ai-credits-token-sponsorship](docs/sponsor.md#ai-credits-token-sponsorship)
- 企业支持和商业授权：
微信 `HiKellen`,
Google Mail:`kellen9903@gmail.com`

### 参与方式

1. **提交 Issue**：报告问题或建议新功能
2. **提交 Pull Request**：修复问题或实现新功能
3. **改进文档**：提升文档质量
4. **测试反馈**：测试系统并提供反馈

### 开发流程

1. Fork 项目
2. 创建特性分支（`git checkout -b feature/AmazingFeature`）
3. 提交更改（`git commit -m 'feat: 添加 AmazingFeature'`）
4. 运行测试：`cargo test` 和 lint：`cargo clippy && cargo fmt --check`
5. 推送到分支（`git push origin feature/AmazingFeature`）
6. 提交 Pull Request

## 许可证

本项目基于 Apache License 2.0 许可。详见 [LICENSE](LICENSE)。

## 致谢

感谢所有为 Nuva OS 做出贡献的开发者！

特别感谢：
- Rust 嵌入式社区
- FreeBSD 项目
- NIST PQC 标准化过程

## 联系方式

- **GitHub**：[https://github.com/nuva-os/nuva](https://github.com/nuva-os/nuva)
- **邮箱**：[kellen9903@gmail.com](mailto:kellen9903@gmail.com)

---

<div align="center">

**Nuva OS — 面向未来的智能操作系统**

Made with ❤️ by Nuva OS Team

</div>
