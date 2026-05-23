# 内核诊断子系统

## 概述

`kernel/diag/` 模块提供内核诊断基础设施，包括日志、内核调试、日志记录、病毒扫描和统计信息收集。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口 |
| `log.rs` | 内核日志子系统 |
| `kdebug.rs` | 内核调试器接口 |
| `journal.rs` | 日志记录子系统（持久化诊断） |
| `scanner.rs` | 病毒扫描器（安全诊断） |
| `stats.rs` | 诊断统计信息收集 |

## 初始化顺序

诊断组件分布在多个引导阶段进行初始化，以满足各自的依赖关系：

| 阶段 | 组件 | 初始化函数 | 依赖 |
|------|------|-----------|------|
| 1 — 引导 | log | `log::init_log()` | CPU |
| 4 — 基础设施 | stats | `stats::init_stats()` | 设备模型、插件系统 |
| 6 — 韧性 | scanner | `scanner::init_virus_scanner()` | 进程、调度器、安全、VFS |
| 8 — 平台与诊断 | kdebug | `kdebug::init_kdebug()` | APIC、平台 |
| 8 — 平台与诊断 | journal | `journal::init_journal()` | kdebug、块设备、VFS |

## 依赖关系

- **内部依赖**：`kernel/core`（CPU、mempool、time、workqueue）、`kernel/init`（config、cmdline）、`kernel/irq_mgmt`（APIC ops）、`kernel/device`（device_model）、`kernel/process`、`kernel/sched`、`kernel/security`、`kernel/storage`（block）
- **上层被依赖**：上层诊断工具、系统服务（L3）

## 公开接口

- `log` 模块：内核日志系统，支持严重级别（emerg、alert、crit、err、warn、notice、info、debug）和 `pr_*!` 宏
- `kdebug` 模块：内核调试器接口，支持运行时断点、状态检查和跟踪
- `journal` 模块：持久化日志子系统，用于存储和重放诊断记录
- `scanner` 模块：病毒扫描引擎，用于安全诊断和威胁检测
- `stats` 模块：统计信息收集、聚合和报告框架

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `journal`、`kdebug`、`log`、`scanner`、`stats`

---

*最后更新：2026-05-22 | Nuva OS v1.0.0*
