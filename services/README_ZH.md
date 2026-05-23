# Services — 系统服务层 (L3)

## 概述

Services 层（Layer 3）提供操作系统级的系统服务，在微内核架构中运行于用户空间。包括应用管理、IPC、网络、电源、安全等核心服务。

## 子模块

| 子模块 | 说明 |
|--------|------|
| app/ | 应用服务：Activity 生命周期、安装器、包管理器 |
| ipc/ | IPC 服务：Binder、通道、共享内存 |
| net/ | 网络服务：DNS 解析、接口管理、TCP/UDP |
| power/ | 电源服务：电源管理器、策略、挂起、唤醒锁 |
| security/ | 安全服务：Gatekeeper、Keymaster、权限管理、TEE 客户端 |
| form_factor.rs | 形态因子管理器（手机/平板/桌面/服务器自适应） |

## 依赖关系

- **下层依赖**：hal (L0)、kernel (L1)、syslib (L2)
- **上层被依赖**：application (L4)

## 构建配置

服务层随内核一起编译，通过条件编译支持不同设备类型：

- 移动设备：启用 app、power、security 服务
- 服务器设备：启用 ipc、net 服务

## 公开接口

各服务通过 Binder IPC 暴露接口，支持同步和异步调用：
- `AppService` — 应用生命周期管理
- `PowerService` — 电源策略和状态管理
- `SecurityService` — 权限和密钥管理
- `NetService` — 网络配置和连接管理
