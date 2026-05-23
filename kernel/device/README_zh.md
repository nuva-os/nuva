# 设备模型与插件

## 概述

`kernel/device/` 模块提供设备模型框架、驱动/功能插件系统、模块加载器和设备事件传播的通知链。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口 |
| `device_model.rs` | 设备模型抽象和总线/类/驱动模型 |
| `driver_plugin.rs` | 驱动插件系统（可扩展驱动注册） |
| `feature_plugin.rs` | 功能插件系统（运行时功能加载） |
| `module.rs` | 内核模块加载器 |
| `notifier.rs` | 通知链（设备事件传播） |

## 初始化顺序

1. 设备模型初始化（`device_model::init_device_model`）
2. 驱动插件系统（`driver_plugin::init_driver_plugin`）
3. 功能插件系统（`feature_plugin::init_feature_plugin`）
4. 模块加载器（`module::init_module`）
5. 通知链（`notifier::init_notifier`）

所有设备子系统组件在阶段 3（设备与插件）中初始化，在内存管理和 IRQ 可用之后进行。

## 依赖关系

- **内部依赖**：`kernel/core`（CPU、mempool、workqueue）、`kernel/init`（elf）、`kernel/irq_mgmt`（IRQ）、`kernel/sync`
- **上层被依赖**：所有设备驱动、电源管理、存储、网络、插件扩展

## 公开接口

- `device_model` 模块：统一的设备模型，包含总线/类/驱动抽象（`init_device_model()`、`register_device()`、`register_driver()`、`device_create()`）
- `driver_plugin` 模块：驱动插件系统，支持可扩展的驱动注册（`init_driver_plugin()`、`register_driver_plugin()`）
- `feature_plugin` 模块：功能插件系统，支持运行时功能加载和激活（`init_feature_plugin()`、`load_feature_plugin()`）
- `module` 模块：内核模块加载器，支持动态模块插入和移除（`init_module()`、`load_module()`、`unload_module()`）
- `notifier` 模块：通知链，用于设备事件传播（`init_notifier()`、`register_notifier()`、`notify_event()`）

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `device_model`、`driver_plugin`、`feature_plugin`、`module`、`notifier`

---

*最后更新：2026-05-20 | Nuva OS v1.0.0*
