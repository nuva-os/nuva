# Syslib — 系统库层 (L2)

## 概述

Syslib 层（Layer 2）提供面向应用和服务的系统库集合。可依赖 Kernel API 和 HAL traits，为上层提供高级抽象和功能库。

## 子模块

| 子模块 | 说明 |
|--------|------|
| core/ | 核心库：分配器、同步原语（无锁数据结构） |
| brain/ | Nuva Brain AI 引擎：学习、预测、NPU 调度、推理引擎、算子（卷积/激活/池化） |
| ai/ | AI 库：模型管理、优化器、调度器 |
| lang/ | NuvaLang 编译器和运行时：词法分析、语法分析、语义分析、类型推断、代码生成、优化器、标准库、二进制格式（Native/NEX）、GC、VM |
| ml/ | 机器学习库：张量、模型、推理引擎 |
| net/ | 网络库：HTTP、WebSocket、JSON、TCP/UDP 协议栈、ARP、以太网 |
| data/ | 数据结构库：数据库、键值存储 |
| gfx/ | 图形库：FPS 监控 |
| ui/ | UI 库：布局、视图、窗口 |
| std/ | 标准库：集合、基础类型、IO |
| runtime/ | 运行时库：Arc、元数据、协议 |
| dispatch/ | 并发框架（GCD 风格）：线程池、信号量、队列、分组 |
| posix/ | POSIX 兼容层：系统调用封装 |

## 依赖关系

- **下层依赖**：hal (L0)、kernel (L1)
- **上层被依赖**：services (L3)、application (L4)

## 构建配置

Syslib 库随内核一起编译，无需额外 feature flag。

## 公开接口

各子模块通过 `pub mod` 暴露公共 API，包括：
- `brain` — AI 推理和训练接口
- `ml` — 张量操作和模型管理
- `net` — 网络协议和 HTTP 客户端
- `dispatch` — 异步任务分发
