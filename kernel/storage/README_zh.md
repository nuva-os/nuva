# 存储子系统

## 概述

`kernel/storage/` 模块提供块设备存储管理，包括块设备抽象和 I/O 调度。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口和存储框架 |
| `block.rs` | 块设备管理和 I/O 操作 |

## 初始化

- 块设备子系统初始化（`block::init_block_device`）— 阶段 7（I/O 与网络），在核心内核服务就绪之后进行

## 依赖关系

- **内部依赖**：`kernel/core`（workqueue、mempool、time）、`kernel/irq_mgmt`（存储中断 IRQ）、`kernel/device`（块设备注册的设备模型）
- **上层被依赖**：文件系统（VFS、ext4、FAT32、Nuvafs）、日志系统、交换子系统

## 公开接口

- `block` 模块：块设备抽象和 I/O 调度（`init_block_device()`、`register_block_device()`、`submit_bio()`、`blk_queue_rq()`）

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `block`

---

*最后更新：2026-05-20 | Nuva OS v1.0.0*
