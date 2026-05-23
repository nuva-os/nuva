# 内核初始化子系统

## 概述

`kernel/init/` 模块负责早期内核初始化，包括命令行解析、平台配置、ELF 加载和资源管理。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口和初始化编排 |
| `cmdline.rs` | 内核命令行解析 |
| `config.rs` | 内核配置管理 |
| `elf.rs` | ELF 二进制加载器（用于内核模块） |
| `platform.rs` | 平台特定初始化 |
| `resource.rs` | 资源管理和分配 |

## 初始化顺序

init 模块提供启动引导组件，触发内核初始化。完整的内核引导序列分为 8 个阶段，由 `kernel::init_subsystems()` 编排：

**阶段 1 — 引导**（无依赖）：
1. 命令行解析（`cmdline::init_cmdline`）
2. 配置设置（`config::init_config`）
3. 日志系统（`log::init_log`）
4. CPU 管理（`cpu::init_cpu`）
5. 调试子系统（`debug::init_debug`）

**阶段 2 — 内存与 IRQ**（依赖阶段 1）：
6. 内存池（`mempool::init_mempool`）
7. 资源管理器（`resource::init_resource`）
8. 随机数生成器（`random::init_random`）
9. IRQ 管理（`irq::init_irq`）
10. 时间保持（`time::init_time`）

**阶段 3 — 设备与插件**（依赖阶段 2）：
11. 设备模型（`device_model::init_device_model`）
12. 插件系统（`plugin::init_plugin`）
13. 驱动插件（`driver_plugin::init_driver_plugin`）
14. 功能插件（`feature_plugin::init_feature_plugin`）
15. 模块加载器（`module::init_module`）
16. 通知链（`notifier::init_notifier`）

**阶段 4 — 基础设施**（依赖阶段 3）：
17. 统计信息（`stats::init_stats`）
18. 热插拔（`hotplug::init_hotplug`）
19. 电源管理（`pm::init_pm`）
20. 性能监控（`perf::init_perf`）
21. 定时器子系统（`timer::init_timer`）
22. 工作队列（`workqueue::init_workqueue`）

**阶段 5 — 核心内核服务**（依赖阶段 4）：
23. 进程管理（`process::init_process`）
24. 调度器（`sched::init_scheduler`）
25. 信号处理（`signal::init_signal`）
26. 安全子系统（`security::init_security`）

**阶段 6 — 韧性与性能**（依赖阶段 5）：
27. 崩溃记录（`tombstone::init_tombstone`）
28. 防御机制（`defense::init_defense`）
29. 病毒扫描器（`scanner::init_virus_scanner`）
30. 内核缓存（`cache::init_cache`）
31. 性能调优（`perf_tune::init_perf_tune`）

**阶段 7 — I/O 与网络**（依赖阶段 6）：
32. 块设备（`block::init_block_device`）
33. TCP/IP 协议栈（`tcpip::init_tcpip`）
34. Socket API（`socket::init_socket_api`）
35. 网络子系统（`net::init_net`）

**阶段 8 — 平台与诊断**（依赖阶段 7）：
36. APIC 操作（`apic_ops::init_apic_ops`）
37. 虚拟化（`vmx::init_vmx`）
38. ACPI 电源（`power::init_acpi`）
39. 内核调试器（`kdebug::init_kdebug`）
40. 日志记录（`journal::init_journal`）

## 依赖关系

- **下层依赖**：`kernel/core`（CPU、mempool）、HAL（L0）
- **上层被依赖**：所有其他内核子系统 — init 提供所有内核组件依赖的基础引导配置和资源管理

## 公开接口

- `cmdline` 模块：内核命令行解析（`init_cmdline()`、`get_cmdline()`、`get_boot_arg()`）
- `config` 模块：内核配置管理（`init_config()`、`get_config()`、`set_config()`）
- `elf` 模块：ELF 二进制加载器，用于解析和加载内核模块与插件
- `platform` 模块：平台特定初始化和检测（`detect_platform_info()`）
- `resource` 模块：资源分配和管理（`init_resource()`、`allocate_resource()`、`free_resource()`）

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `cmdline`、`config`、`elf`、`platform`、`resource`

---

*最后更新：2026-05-20 | Nuva OS v1.0.0*
