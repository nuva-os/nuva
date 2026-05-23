# Nuva OS 操作系统

<div align="center">

**一个现代化的、抗量子的、AI 原生操作系统**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-nightly-orange?style=flat-square)](https://www.rust-lang.org)

[English](README.md) | 简体中文

[特性](#特性) • [架构](#架构) • [快速开始](#快速开始) • [文档](#文档) • [模块](#模块文档) • [贡献](#贡献)

</div>

---

## 概述

Nuva OS (女娲)是一款为现代移动和嵌入式设备设计的尖端操作系统，基于 Rust `#![no_std]` 裸机环境构建，具有以下特性：

- **抗量子安全**：NIST PQC 标准算法（CRYSTALS-Kyber、CRYSTALS-Dilithium）
- **AI 原生设计**：NPU 抽象和智能优化
- **高性能**：零拷贝 IPC、无锁数据结构、内存池
- **插件架构**：动态加载、生命周期管理、沙箱隔离（100% 已实现）
- **多架构支持**：ARM64、x86_64、LoongArch64（页表/中断/SIMD 已实现）
- **开发者友好**：完整 C/C++ API、SDK 100% 已实现（调试/分析/包管理）

## 特性

### 🔐 抗量子安全

- **CRYSTALS-Kyber**：NIST 标准化的密钥封装机制
- **CRYSTALS-Dilithium**：NIST 标准化的数字签名方案
- **SHA-256**：FIPS 180-4 标准安全哈希算法
- **QRNG**：量子随机数生成接口
- **QKD**：量子密钥分发接口
- **硬件加速**：支持加密加速器

### 🤖 AI 原生设计

- **NPU 抽象**：针对不同 NPU 架构的统一接口（达芬奇 NPU HAL 已实现）
- **AI 调度器**：智能任务调度与负载均衡
- **模型管理**：缓存、版本控制和优化
- **智能调度**：AI 驱动的任务调度（EAS 能耗感知调度）
- **性能优化器**：在线学习和瓶颈检测

### ⚡ 高性能

- **零拷贝 IPC**：小消息延迟 <100ns
- **无锁结构**：MPSC/SPSC 队列、无锁栈
- **内存池**：<10ns 分配，零碎片
- **Buddy + SLAB 分配器**：高效内存管理
- **性能监控**：实时指标和告警

### 🔌 插件系统

- **动态加载**：运行时插件加载（ELF 加载器已实现）
- **依赖解析**：自动依赖排序
- **沙箱执行**：资源限制和安全隔离（插件沙箱已实现）
- **热插拔**：无需重启即可加载/卸载插件

### 🛠️ 开发者友好

- **C API**：完整的 C99 兼容 API（`nuva_hal.h`）
- **C++ API**：RAII、异常安全、移动语义（`nuva_hal.hpp`）
- **API 稳定性**：语义版本控制和 ABI 兼容性
- **SDK 工具链**：CLI、构建系统、调试器（DAP 协议）、包管理器（HTTP）、性能分析器（/proc）
- **文档**：全面的指南和示例

## 架构

```
┌─────────────────────────────────────────────────┐
│              Application Layer (L4)              │
│         UI │ Window │ Event │ Render │ Resource  │
├─────────────────────────────────────────────────┤
│              Services Layer (L3)                 │
│     App │ IPC │ Net │ Power │ Security │ Form   │
├─────────────────────────────────────────────────┤
│              Syslib Layer (L2)                   │
│  ┌──────────┬──────────┬──────────┬──────────┐  │
│  │  Core    │  Brain   │   Lang   │   ML     │  │
│  ├──────────┼──────────┼──────────┼──────────┤  │
│  │   Net    │   Data   │   GFX    │   UI     │  │
│  ├──────────┼──────────┼──────────┼──────────┤  │
│  │ Runtime  │   Std    │ Dispatch │   AI     │  │
│  └──────────┴──────────┴──────────┴──────────┘  │
├─────────────────────────────────────────────────┤
│              Kernel Layer (L1)                   │
│  ┌──────────┬──────────┬──────────┬──────────┐  │
│  │  Sched   │   IPC    │ Security │ Quantum  │  │
│  ├──────────┼──────────┼──────────┼──────────┤  │
│  │   MM     │   FS     │  Driver  │  Plugin  │  │
│  ├──────────┼──────────┼──────────┼──────────┤  │
│  │ Process  │   Net    │  Sync    │  Syscall │  │
│  └──────────┴──────────┴──────────┴──────────┘  │
├─────────────────────────────────────────────────┤
│               HAL Layer (L0)                     │
│  ┌──────────┬──────────┬──────────┬──────────┐  │
│  │   CPU    │   GPU    │   NPU    │ Quantum  │  │
│  ├──────────┼──────────┼──────────┼──────────┤  │
│  │  Power   │   FFI    │  Input   │ Platform │  │
│  ├──────────┼──────────┼──────────┼──────────┤  │
│  │  ARM64   │   x64    │LoongArch │Snapdragon│  │
│  └──────────┴──────────┴──────────┴──────────┘  │
└─────────────────────────────────────────────────┘
```

> **最新实现**：SHA-256 FIPS 180-4 · ELF 加载器 · NFS/SMB 客户端 · Da Vinci NPU HAL · AI 调度器 · LoongArch64 页表/中断/SIMD · 插件沙箱 · SDK 调试(DAP)/性能分析(/proc)/包管理(HTTP)

### 分层架构约束

| 层级 | 依赖 | 说明 |
|------|------|------|
| L0 - HAL | 无 | 硬件抽象层，最底层，无外部层依赖 |
| L1 - Kernel | L0 | 内核层，仅依赖 HAL |
| L2 - Syslib | L0, L1 | 系统库层，可依赖 Kernel API 和 HAL traits |
| L3 - Services | L0, L1, L2 | 系统服务层 |
| L4 - Application | L0, L1, L2, L3 | 应用框架层 |

## 快速开始

### 前置条件

- Rust **nightly** 工具链（参见 `rust-toolchain.toml`）
- QEMU >= 7.0（用于测试）
- C/C++ 工具链（用于示例编译）

### 安装

```bash
# 克隆仓库
git clone https://github.com/nuva-os/nuva.git
cd nuva

# 安装 Rust nightly 工具链
rustup install nightly
rustup override set nightly

# 安装目标平台
rustup target add --toolchain nightly aarch64-unknown-none
rustup target add --toolchain nightly x86_64-unknown-none

# 安装必要组件
rustup component add rust-src

# 构建（ARM64 + Kirin）
cargo build --target aarch64-unknown-none --features arm64 --release

# 构建（x86_64）
cargo build --target x86_64-unknown-none --features x64 --release

# 检查编译
cargo check --target aarch64-unknown-none --features arm64

# 运行测试
cargo test

# 在 QEMU 中运行
qemu-system-aarch64 -machine virt -cpu cortex-a76 -kernel target/aarch64-unknown-none/release/nuva_kernel
```

### 快速示例

```c
#include <nuva_hal.h>

int main() {
    // 获取 CPU 信息
    nuva_cpu_info_t cpu_info;
    nuva_cpu_get_info(&cpu_info);

    printf("CPU: %u cores @ %u MHz\n",
           cpu_info.core_count,
           cpu_info.frequency_mhz);

    return 0;
}
```

## Feature Flags

| Feature | 依赖 | 说明 |
|---------|------|------|
| `arm64` | — | ARM64 架构支持 |
| `x64` | — | x86_64 架构支持 |
| `loongarch64` | — | LoongArch64 架构支持 |
| `kirin` | `arm64` | 麒麟 SoC 通用支持 |
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
| ARM64 | `aarch64-unknown-none` | ✅ 编译通过 | Kirin 9020、Snapdragon 8 Gen 4 |
| x86_64 | `x86_64-unknown-none` | ✅ 编译通过 | Intel Core、AMD Ryzen |
| LoongArch64 | `loongarch64-unknown-none` | ✅ 编译通过 | 龙芯 3A6000/3C6000 |

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
- [层级规则](docs/architecture/LAYER_RULES_zh.md) / [English](docs/architecture/LAYER_RULES.md)
- [驱动开发指南](docs/development/DRIVER_DEVELOPMENT_GUIDE_zh.md) / [English](docs/development/DRIVER_DEVELOPMENT_GUIDE.md)
- [文档标准](docs/standards/DOCUMENTATION_STANDARD_zh.md) / [English](docs/standards/DOCUMENTATION_STANDARD.md)
- [API 参考手册](docs/api/API_REFERENCE_zh.md) / [English](docs/api/API_REFERENCE.md)

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

## 性能

| 组件 | 指标 | 值 |
|------|------|-----|
| IPC (小消息) | 延迟 | <100ns |
| IPC (大消息) | 延迟 | <10μs |
| 内存池 | 分配/释放 | <10ns |
| 无锁队列 | 推入/弹出 | <50ns |
| Kyber-768 | 密钥生成 | <1ms |
| Dilithium-3 | 签名 | <1ms |

## 贡献

我们欢迎贡献！详情请参见 [CONTRIBUTING.md](CONTRIBUTING.md)。

### 开发流程

1. Fork 仓库
2. 创建特性分支
3. 进行修改
4. 运行测试：`cargo test`
5. 检查格式：`cargo fmt --check`
6. 运行 clippy：`cargo clippy`
7. 提交 Pull Request

## 许可证

Nuva OS 基于 Apache License 2.0 许可。详情请参见 [LICENSE](LICENSE)。

## 致谢

- NIST PQC 标准化过程
- Rust 嵌入式社区

## 联系方式

- **github**:https://github.com/nuva-os/nuva
- **gitee**: https://gitee.com/nuva-os/nuva
- **邮箱**：zhangyujie_china@163.com

---

<div align="center">

**Made with ❤️ by Nuva OS Team**

</div>
