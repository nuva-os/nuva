# Nuva OS 文档

## 概述

Nuva OS 是一个从零开始使用纯 Rust 构建的现代化操作系统，采用微内核架构设计，基于 `#![no_std]` 裸机环境。

**版本**：v1.0.0
**许可证**：Apache 2.0
**工具链**：Rust nightly（`rust-toolchain.toml` 中配置 `channel = "nightly"`，组件：`rust-src`、`rustfmt`、`clippy`，目标：`aarch64-unknown-none`、`x86_64-unknown-none`）
**支持的架构**：ARM64 (AArch64)、x86-64、LoongArch64
**编译状态**：ARM64 ✅ 通过 / x86-64 ✅ 通过 / LoongArch64 ✅ 通过

---

## 项目结构

```
Nuva/
├── kernel/              # 内核层 (L1)
│   ├── arch/            #   架构相关代码 (arm64, x64, loongarch64)
│   ├── mm/              #   内存管理 (Buddy, SLAB, VMA, NUMA, COW, 大页, OOM)
│   ├── sched/           #   进程调度 (CFS, EAS, RT, 负载均衡)
│   ├── process/         #   进程管理 (fork, execve, signal, wait4)
│   ├── fs/              #   内核文件系统 (VFS, 页缓存, io_uring)
│   ├── net/             #   网络协议栈 (TCP/UDP/IPv6/ARP)
│   ├── ipc/             #   进程间通信 (NuvaIPC, 共享内存, L4, 零拷贝)
│   ├── syscall/         #   系统调用接口
│   ├── security/        #   安全模块 (LSM, ASLR, 沙箱, 栈金丝雀)
│   ├── driver/          #   驱动框架 (设备模型, 总线, IRQ, DMA, DMA-BUF)
│   │   ├── framework/   #     驱动核心框架 (display, input)
│   │   ├── class/       #     设备类 (audio, backlight, bluetooth, camera, ...)
│   │   ├── impl/        #     驱动实现 (irqchip)
│   │   ├── device.rs    #     Device trait 定义
│   │   ├── dma.rs       #     DMA 通道抽象
│   │   ├── dmabuf.rs    #     DMA-BUF 共享缓冲区框架
│   │   ├── gpio.rs      #     GPIO 子系统
│   │   ├── i2c.rs       #     I2C 总线驱动
│   │   ├── spi.rs       #     SPI 总线驱动
│   │   ├── irq.rs       #     中断请求管理
│   │   ├── pm.rs        #     驱动电源管理集成
│   │   ├── dt.rs        #     设备树匹配与解析
│   │   └── ...          #     (clk, freq, opp, phy, pinctrl, pwm, regulator, ...)
│   ├── drivers/         #   具体驱动实现 (irqchip/GIC)
│   ├── quantum/         #   量子调度器
│   ├── plugin/          #   插件系统 (ELF 加载器, 管理器, 沙箱, 注册表)
│   ├── sync/            #   同步原语 (自旋锁, 互斥锁)
│   ├── interrupt/       #   中断管理
│   ├── timer/           #   定时器
│   ├── perf/            #   性能监控
│   ├── debug/           #   调试支持 (printk)
│   └── bsd/             #   BSD 兼容层
├── hal/                 # 硬件抽象层 (L0)
│   ├── cpu/             #   CPU 抽象 (DVFS, 麒麟 SoC, 龙芯 SoC, 热管理)
│   ├── gpu/             #   GPU 抽象 (Maleoon GPU, 命令队列)
│   ├── npu/             #   NPU 抽象 (达芬奇 NPU HAL, ONNX 运行时, AI 调度器, 推理器)
│   ├── quantum/         #   量子密码 (PQC: Kyber/Dilithium, QRNG, QKD)
│   ├── power/           #   电源管理 (PMIC, 挂起/恢复)
│   ├── ffi/             #   外部函数接口 (C API, C++ API, API 稳定性保证)
│   ├── input.rs         #   输入设备
│   ├── platform.rs      #   平台检测
│   ├── dt.rs            #   设备树解析器 (ARM64)
│   ├── acpi.rs          #   ACPI 表解析器 (x86_64)
│   ├── arm64/           #   ARM64 架构特定实现
│   ├── x64/             #   x86_64 架构特定实现
│   ├── loongarch64/     #   LoongArch64 架构特定实现 (页表, 中断, SIMD)
│   └── snapdragon/      #   高通骁龙平台
├── syslib/              # 系统库层 (L2)
│   ├── core/            #   核心库 (分配器, 同步原语)
│   ├── brain/           #   Nuva Brain AI 引擎
│   ├── ai/              #   AI 库
│   ├── lang/            #   NuvaLang 编译器和运行时
│   ├── ml/              #   机器学习库 (张量, 模型, 推理引擎)
│   ├── net/             #   网络库 (HTTP, WebSocket, JSON)
│   ├── data/            #   数据结构库
│   ├── gfx/             #   图形库
│   ├── ui/              #   UI 库 (布局, 视图, 窗口)
│   ├── std/             #   标准库 (集合, 基础类型, IO)
│   ├── runtime/         #   运行时库 (Arc, 元数据, 协议)
│   └── dispatch/        #   并发框架 (GCD 风格, 线程池)
├── services/            # 系统服务层 (L3)
│   ├── app/             #   应用服务 (Activity, 包管理器)
│   ├── ipc/             #   IPC 服务 (Binder, 通道)
│   ├── net/             #   网络服务 (DNS, TCP/UDP)
│   ├── power/           #   电源服务 (策略, 唤醒锁)
│   ├── security/        #   安全服务 (Gatekeeper, Keymaster, TEE)
│   └── form_factor.rs   #   形态因子管理器
├── application/         # 应用框架层 (L4)
│   ├── ui/              #   UI 框架 (自适应布局, 组件)
│   ├── window/          #   窗口管理
│   ├── event/           #   事件系统
│   ├── render/          #   渲染引擎 (合成器, 画笔)
│   └── resource/        #   资源管理 (缓存, 解码器)
├── posix/               # POSIX 兼容层
│   ├── unistd.rs        #   POSIX 进程和文件操作
│   ├── fcntl.rs         #   文件控制
│   ├── signal.rs        #   信号处理
│   └── errno.rs         #   错误码
├── fs/                  # 文件系统实现
│   ├── ext4/            #   ext4 文件系统
│   ├── fat32/           #   FAT32 文件系统
│   └── nuvafs/          #   NuvaFS 自研文件系统
├── sdk/                 # 软件开发套件
│   ├── cli/             #   命令行界面
│   ├── build/           #   构建系统
│   ├── debug/           #   调试器 (DAP 协议)
│   ├── package/         #   包管理器 (HTTP)
│   └── profiler/        #   性能分析器 (/proc)
├── tools/               #   工具链集合
│   ├── dep_analyzer/    #   依赖分析器 (层级合规检查, build.rs 集成)
│   ├── compiler/        #   编译器工具
│   ├── linker/          #   链接器工具
│   ├── lsp/             #   语言服务器协议
│   └── toolchain/       #   工具链管理
├── configs/             #   架构层合规配置 (layers/{hal,kernel,lib})
├── sysroot/             #   系统根目录 (C 头文件)
├── toolchains/          #   交叉编译工具链配置
├── scripts/             #   构建和文档生成脚本
├── tests/               #   测试套件
├── benches/             #   性能基准测试
├── examples/            #   C/C++ 示例程序
└── docs/                #   项目文档
```

---

## 模块完成状态

| 模块 | 框架 | 功能 | 总体 | 说明 |
|------|------|------|------|------|
| 内存管理 | 95% | 95% | 95% | Buddy/SLAB/VMA/NUMA/COW/大页/OOM |
| 进程调度 | 90% | 90% | 90% | CFS/EAS/RT/Deadline/负载均衡 |
| 文件系统 | 90% | 90% | 90% | VFS/NuvaFS/ext4/FAT32/io_uring/NFS/SMB |
| 网络协议栈 | 80% | 85% | 82% | TCP/UDP/IPv6/ARP/拥塞控制 |
| 设备驱动 | 75% | 70% | 72% | 设备模型/总线/IRQ/DMA/GPIO/I2C/SPI/DMA-BUF |
| 系统调用 | 90% | 90% | 90% | POSIX 接口覆盖 |
| 安全模块 | 90% | 85% | 87% | LSM/ASLR/沙箱/栈金丝雀/安全启动/SHA-256 |
| 电源管理 | 85% | 60% | 72% | PMIC/挂起/恢复/驱动 PM 集成 |
| 量子安全 | 85% | 80% | 82% | Kyber/Dilithium/QRNG/QKD |
| AI/NPU | 90% | 85% | 87% | 达芬奇 NPU HAL/ONNX/推理器/AI 调度器 |
| 插件系统 | 100% | 100% | 100% | ELF 加载器/管理器/沙箱/注册表 |
| 龙架构 | 95% | 90% | 92% | HAL/页表/中断/SIMD/扩展检测 |
| SDK | 100% | 100% | 100% | 调试器(DAP)/性能分析(/proc)/包管理(HTTP)/CLI/构建 |

---

## 文档索引

### 核心文档

| 文档 | 中文版 | 英文版 | 说明 |
|------|--------|--------|------|
| 系统架构 | [ARCHITECTURE_zh.md](ARCHITECTURE_zh.md) | [ARCHITECTURE.md](ARCHITECTURE.md) | 微内核架构，模块设计 |
| 内存管理 | [MEMORY_zh.md](MEMORY_zh.md) | [MEMORY.md](MEMORY.md) | 物理内存，虚拟内存，NUMA，COW |
| 进程管理 | [PROCESS_zh.md](PROCESS_zh.md) | [PROCESS.md](PROCESS.md) | 进程调度，进程控制，负载均衡 |
| 文件系统 | [FILESYSTEM_zh.md](FILESYSTEM_zh.md) | [FILESYSTEM.md](FILESYSTEM.md) | VFS，NuvaFS |
| 系统调用 | [SYSCALL_zh.md](SYSCALL_zh.md) | [SYSCALL.md](SYSCALL.md) | POSIX 接口，错误码 |
| API 文档 | [API_zh.md](API_zh.md) | [API.md](API.md) | kernel API，文件系统 API，IPC API |

### 开发文档

| 文档 | 中文版 | 英文版 | 说明 |
|------|--------|--------|------|
| 快速入门 | [QUICK_START_zh.md](QUICK_START_zh.md) | [QUICK_START.md](QUICK_START.md) | 环境搭建，构建，运行 |
| 编码规范 | [CODING_STANDARD_zh.md](CODING_STANDARD_zh.md) | [CODING_STANDARD.md](CODING_STANDARD.md) | 编码规范和约定 |

### 规划文档

| 文档 | 中文版 | 英文版 | 说明 |
|------|--------|--------|------|
| 开发路线图 | [ROADMAP_zh.md](ROADMAP_zh.md) | [ROADMAP.md](ROADMAP.md) | 待办工作，优先级 |

### 架构与标准文档

| 文档 | 中文版 | 英文版 | 说明 |
|------|--------|--------|------|
| 分层架构规则 | [architecture/LAYER_RULES_zh.md](architecture/LAYER_RULES_zh.md) | [architecture/LAYER_RULES.md](architecture/LAYER_RULES.md) | 分层依赖约束 |
| 文档编写标准 | [standards/DOCUMENTATION_STANDARD_zh.md](standards/DOCUMENTATION_STANDARD_zh.md) | [standards/DOCUMENTATION_STANDARD.md](standards/DOCUMENTATION_STANDARD.md) | 文档规范 |
| 模块文档模板 | — | — | 模块模板（待创建） |
| 驱动开发指南 | [development/DRIVER_DEVELOPMENT_GUIDE_zh.md](development/DRIVER_DEVELOPMENT_GUIDE_zh.md) | [development/DRIVER_DEVELOPMENT_GUIDE.md](development/DRIVER_DEVELOPMENT_GUIDE.md) | 驱动开发 |
| API 参考手册 | [api/API_REFERENCE_zh.md](api/API_REFERENCE_zh.md) | [api/API_REFERENCE.md](api/API_REFERENCE.md) | HAL API 参考 |

---

## 快速链接

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

## 设计哲学

Nuva OS 遵循以下核心原则：

1. **简洁与优雅**：清晰的接口，模块化设计，单一职责
2. **策略与机制分离**：kernel 提供机制，用户空间决定策略
3. **一切皆文件**：设备、管道、套接字使用统一的 VFS 接口
4. **内存安全**：利用 Rust 的安全特性
5. **高性能**：使用现代技术优化关键路径
6. **抗量子安全**：集成 NIST PQC 标准算法（Kyber/Dilithium）
7. **AI 原生**：NPU 抽象层和智能优化

---

## 核心特性

### 内存管理
- 使用 Buddy 和 SLAB 分配器的物理内存管理
- 使用 VMA 和页表的虚拟内存管理
- 高级特性：NUMA 支持、COW 机制、大页、OOM killer
- 页迁移和内存规整

### 进程管理
- 完整的进程生命周期管理
- 普通进程的 CFS（完全公平调度器）
- EAS（能耗感知调度）
- 支持 FIFO 和 RR 策略的 RT（实时）调度器
- 多核负载均衡与 CPU 亲和性控制

### 文件系统
- VFS（虚拟文件系统）抽象
- NuvaFS 原生文件系统（日志、快照、POSIX 兼容）
- ext4 和 FAT32 文件系统支持
- NFS 和 SMB 网络文件系统客户端
- 文件缓存的页缓存与缓冲区缓存
- io_uring 异步 I/O 支持

### 系统服务
- 支持多种休眠状态的电源管理
- 带 Gatekeeper/Keymaster/TEE 的安全服务
- 带 DNS 和 TCP/IP 协议栈的网络服务
- 带 Binder IPC 的应用服务

### 量子安全
- CRYSTALS-Kyber 密钥封装（NIST 标准）
- CRYSTALS-Dilithium 数字签名（NIST 标准）
- SHA-256 安全哈希（FIPS 180-4）
- QRNG 量子随机数生成
- QKD 量子密钥分发

### AI 引擎 (Nuva Brain)
- ML 模型推理引擎
- NPU 调度和管理（达芬奇架构，Da Vinci NPU HAL 已实现）
- AI 调度器（智能任务调度与负载均衡）
- 张量操作支持
- ONNX 运行时集成

### 应用框架
- 带自适应布局的 UI 框架
- 窗口管理系统
- 事件处理系统
- 带硬件加速的渲染引擎
- 资源管理（JPEG/PNG/TTF/WAV 解码）

---

## 快速开始

### 前置条件

- Rust **nightly** 工具链（参见 `rust-toolchain.toml`）
- QEMU >= 7.0
- Git >= 2.0

### 构建

```bash
# 安装工具链
rustup install nightly
rustup override set nightly
rustup target add --toolchain nightly aarch64-unknown-none
rustup target add --toolchain nightly x86_64-unknown-none
rustup component add rust-src

# ARM64 目标（麒麟）
cargo build --target aarch64-unknown-none --features kirin9020

# x86-64 目标
cargo build --target x86_64-unknown-none --features x64
```

### 依赖合规检查

```bash
# 运行层级依赖分析器
cargo run --bin dep_analyzer -- .
```

### 运行

```bash
# ARM64
qemu-system-aarch64 -machine virt -cpu cortex-a76 -nographic \
    -kernel target/aarch64-unknown-none/debug/nuva_kernel

# x86-64
qemu-system-x86_64 -nographic \
    -kernel target/x86_64-unknown-none/debug/nuva_kernel
```

详细说明请参见 [QUICK_START_zh.md](QUICK_START_zh.md)。

---

## 贡献

欢迎贡献！请遵循 [CODING_STANDARD_zh.md](CODING_STANDARD_zh.md) 中的编码规范。

---

## 支持

- **Issue**：https://github.com/nuva-os/nuva/issues
- **文档**：[docs/](.) 目录
- **邮箱**：team@nuva-os.org

---

**最后更新**：2026 年 5 月 15 日
**维护者**：Nuva OS Team
