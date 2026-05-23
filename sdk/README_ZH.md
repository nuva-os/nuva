# SDK — 软件开发套件

## 概述

SDK 模块提供 Nuva OS 软件开发工具包，包括命令行界面、构建系统、调试器、包管理器和性能分析器。

## 子模块

| 子模块 | 说明 |
|--------|------|
| cli/ | 命令行界面：参数解析、命令（build/clean/debug/doc/fmt/init/lint/new/pkg/profile/run/test） |
| build/ | 构建系统：缓存、配置、交叉编译、执行器、调度器、目标 |
| debug/ | 调试器：断点、执行控制、内存检查、栈追踪、变量查看、DAP 协议、ptrace 后端、目标进程管理 |
| package/ | 包管理：缓存、依赖解析、锁文件、元数据、HTTP 注册表、解析器、验证器 |
| profiler/ | 性能分析器：CPU 采样、内存分析、火焰图、采样器、I/O 分析、锁竞争分析 |

## 依赖关系

- **同层协作**：tools (工具链)
- **下层依赖**：syslib (L2)

## 构建配置

SDK 在主机工具链下编译（host target），不在裸机目标上运行。

## 公开接口

- `nuva build` — 构建项目
- `nuva run` — 运行项目
- `nuva test` — 运行测试
- `nuva debug` — 启动调试器
- `nuva pkg` — 包管理
- `nuva profile` — 性能分析

## 构建配置文件

- `sdk/build-config.toml` — 多架构构建目标配置
