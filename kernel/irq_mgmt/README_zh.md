# IRQ 管理子系统

## 概述

`kernel/irq_mgmt/` 模块提供中断请求（IRQ）管理，包括 IRQ 分发、陷阱处理和 APIC 操作，支持 ARM64、x86-64 和 LoongArch64 架构。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口和 IRQ 框架 |
| `irq.rs` | IRQ 分发和管理 |
| `trap.rs` | 陷阱和异常处理 |
| `apic_ops.rs` | APIC 操作（x86-64 LAPIC/I/O APIC、ARM64 GIC、LA64 EIOINTC） |

## 架构支持

| 架构 | 中断控制器 |
|------|-----------|
| ARM64 | GIC（通用中断控制器） |
| x86-64 | LAPIC + I/O APIC |
| LoongArch64 | EIOINTC（扩展 I/O 中断控制器） |
| RISC-V 64 | PLIC | 平台级中断控制器 |

## 初始化顺序

IRQ 管理组件根据其硬件依赖关系在引导阶段中初始化：

| 阶段 | 组件 | 初始化函数 |
|------|------|-----------|
| 2 — 内存与 IRQ | irq | `irq::init_irq()` |
| 8 — 平台 | apic_ops | `apic_ops::init_apic_ops()` |

`trap` 模块通过 HAL 层在架构特定的早期引导（阶段 1）中隐式初始化。

## 依赖关系

- **内部依赖**：`kernel/core`（CPU）、HAL（L0 — GIC/APIC/EIOINTC 硬件抽象）
- **上层被依赖**：所有中断驱动的子系统（设备驱动、定时器、IPC、网络）

## 公开接口

- `irq` 模块：通用 IRQ 分发和管理（`init_irq()`、`request_irq()`、`free_irq()`、`enable_irq()`、`disable_irq()`）
- `trap` 模块：所有架构的陷阱和异常处理，包括故障、中止和系统调用
- `apic_ops` 模块：架构抽象的中断控制器操作（`init_apic_ops()`、`send_ipi()`、`eoi()`）

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `apic_ops`、`irq`、`trap`

---

*最后更新：2026-05-20 | Nuva OS v1.0.0*
