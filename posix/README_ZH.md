# POSIX — POSIX 兼容层

## 概述

POSIX 兼容层提供 POSIX 标准兼容接口，使 Nuva OS 能够运行符合 POSIX 标准的应用程序。与 syslib (L2) 同层，依赖 kernel (L1) 提供底层实现。

## 子模块

| 子模块 | 说明 |
|--------|------|
| unistd.rs | POSIX 进程和文件操作（read, write, close, fork, exec, pipe 等） |
| fcntl.rs | 文件控制（open, creat, fcntl, dup 等） |
| signal.rs | 信号处理（sigaction, kill, raise, sigprocmask 等） |
| errno.rs | 错误码定义（EPERM, ENOENT, ESRCH 等） |

## 依赖关系

- **下层依赖**：kernel (L1)
- **同层协作**：syslib (L2)
- **上层被依赖**：services (L3)

## 构建配置

POSIX 层随内核一起编译，无需额外 feature flag。

## 公开接口

提供标准 POSIX 系统调用接口，错误码遵循 POSIX 语义。所有函数签名与 POSIX 标准兼容。
