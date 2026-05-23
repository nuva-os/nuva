# 电源管理子系统

## 概述

`kernel/power_mgmt/` 模块提供内核级电源管理，包括 ACPI 电源状态、PM 子系统和热插拔支持。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口和电源管理框架 |
| `pm.rs` | 电源管理子系统（挂起/恢复、C-states） |
| `power.rs` | ACPI 电源驱动（Fadt、S3/S5） |
| `hotplug.rs` | CPU 和内存热插拔支持 |

## 初始化顺序

1. 热插拔支持（`hotplug::init_hotplug`）
2. PM 子系统（`pm::init_pm`）
3. ACPI 电源（`power::init_acpi`）

热插拔和 PM 在阶段 4（基础设施）中初始化，而 ACPI 电源在阶段 8（平台与诊断）中初始化，在 APIC 操作配置完成后进行。

## 依赖关系

- **内部依赖**：`kernel/core`（CPU、workqueue）、`kernel/device`（热插拔事件的设备模型）、`kernel/irq_mgmt`（ACPI 的 APIC ops）、`kernel/sync`
- **上层被依赖**：设备电源管理、CPU 热插拔、挂起/恢复基础设施、系统服务（L3）

## 公开接口

- `pm` 模块：电源管理子系统，用于挂起、恢复和 CPU C-states（`init_pm()`、`suspend()`、`resume()`、`set_cpu_cstate()`）
- `power` 模块：ACPI 电源驱动，用于系统电源状态管理（`init_acpi()`、`acpi_enter_sleep_state()`、`acpi_shutdown()`）
- `hotplug` 模块：CPU 和内存热插拔支持（`init_hotplug()`、`hotplug_cpu()`、`hotplug_memory()`）

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `hotplug`、`pm`、`power`

---

*最后更新：2026-05-20 | Nuva OS v1.0.0*
