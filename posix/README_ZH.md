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

POSIX 层作为**可选的 feature gate** 编译。通过以下方式启用：

```toml
[features]
default = ["nuva_native"]
posix = []
bsd_compat = ["posix"]
```

当 `posix` feature 禁用时：
- `posix/` 目录不参与编译
- `kernel/process/fork.rs`、`signal.rs`、`execve.rs`、`wait4.rs` 不参与编译
- `kernel/ipc/shm.rs`、`shm_ipc.rs`（System V 兼容）不参与编译
- 内核核心路径对 POSIX 零依赖

启用后，通过适配器模式将 POSIX 标准接口映射到 Nuva 原生内核原语。

## 公开接口

提供标准 POSIX 系统调用接口，错误码遵循 POSIX 语义。所有函数签名与 POSIX 标准兼容。
