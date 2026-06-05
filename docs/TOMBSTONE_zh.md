# 墓碑机制

## 概述

墓碑机制是 Nuva OS 内核（L1 内核层）中的子系统，用于在进程或任务异常终止时捕获、存储和查询崩溃记录。

当崩溃发生时，该机制将：

1. 从 HAL 层采集 CPU 上下文（寄存器、栈指针、程序计数器）
2. 执行调用栈回溯（最多 32 帧）
3. 组装包含完整崩溃元数据的墓碑记录
4. 通过原子写入将记录持久化到文件系统
5. 若文件系统不可用，回退到内存环形缓冲区

## 架构

```
kernel/tombstone/
├── mod.rs              # 模块入口，TombstoneManager，初始化，崩溃回调
├── record.rs           # 核心数据结构，序列化，CRC32
├── crash_context.rs    # 从 HAL 采集 CrashContext，寄存器脱敏
├── arch_adapter.rs     # CrashArchAdapter trait + ARM64/x64/LoongArch64 适配器
├── store.rs            # TombstoneStore，MemoryCache，文件 I/O，索引
├── query.rs            # 查询引擎（按 PID、时间范围、崩溃原因、最新 N 条）
├── prune.rs            # 清理引擎（按 PID、时间范围、全量）
├── config.rs           # TombstoneStoreConfig
├── stats.rs            # 原子统计计数器
└── syscall.rs          # 系统调用接口（500-503）
```

## 关键数据结构

| 结构体 | 描述 |
|--------|------|
| `TombstoneRecord` | 包含 CPU 上下文、栈回溯、元数据的完整崩溃记录 |
| `CrashReason` | 崩溃分类枚举（FatalSignal、IllegalAccess 等） |
| `ArchId` | 架构标识（Arm64、X64、LoongArch64） |
| `TombstoneError` | 所有墓碑操作的错误类型 |
| `TombstoneStoreConfig` | 存储配置（路径、限制、缓存大小） |
| `TombstoneStats` | 原子统计计数器 |

## 崩溃触发源

| 触发源 | 回调 | 来源 |
|--------|------|------|
| 致命信号（SIGSEGV、SIGABRT 等） | `on_fatal_signal()` | kernel::signal |
| 任务异常终止 | `on_task_crash()` | kernel::sched |
| 看门狗超时 | `on_watchdog_timeout()` | kernel::sched |

## 系统调用

| 编号 | 名称 | 权限 | 描述 |
|------|------|------|------|
| 500 | `tombstone_query` | CAP_SYS_PTRACE 或 CAP_SYS_ADMIN | 查询墓碑记录 |
| 501 | `tombstone_read` | CAP_SYS_PTRACE 或 CAP_SYS_ADMIN | 读取单条记录详情 |
| 502 | `tombstone_clear` | CAP_SYS_ADMIN | 清理墓碑记录 |
| 503 | `tombstone_stats` | CAP_SYS_PTRACE 或 CAP_SYS_ADMIN | 获取统计信息 |

## 存储

- **路径**：`/data/tombstones/`（可配置）
- **命名**：`tombstone_XX.pb`（XX = 00-99，循环使用）
- **容量**：最多 100 条记录，FIFO 自动淘汰
- **原子写入**：临时文件 → fsync → 原子重命名
- **降级模式**：文件系统不可用时使用内存环形缓冲区（4 个槽位）

## 性能

- 墓碑生成：≤ 5ms（HAL ≤ 1ms + 栈回溯 ≤ 2ms + 序列化 + 异步写入）
- 查询（索引命中）：≤ 1ms
- 内存开销：~13.5 KB（索引 + 环形缓冲区 + 统计）
- 正常执行路径：零开销

## 二进制格式

```
偏移   大小  字段
0      4     魔数（0x5442534E = "TBSN"）
4      4     格式版本（1）
8      4     体长度（N）
12     N     体（序列化的 TombstoneRecord 字段）
12+N   4     CRC32 校验和（覆盖字节 0..12+N）
```

## 配置

| 参数 | 默认值 | 范围 |
|------|--------|------|
| `store_dir` | `/data/tombstones/` | 非空路径 |
| `max_count` | 100 | 1-1000 |
| `max_file_size` | 8192 字节 | > 0 |
| `memory_cache_size` | 4 | ≥ 2 |
| `auto_prune_enabled` | true | - |

## 崩溃去重

同一 PID 在 5 秒窗口内的崩溃会被去重：仅保留第一条和最后一条墓碑，中间崩溃递增合并记录的 `crash_count`。

## 安全性

- 寄存器脱敏：可能包含密钥的 callee-saved 寄存器在存储前被清零
- 查询需要 `CAP_SYS_PTRACE` 或 `CAP_SYS_ADMIN`
- 清理需要 `CAP_SYS_ADMIN`
- 生产路径中无 panic/unwrap/expect，所有错误通过 `Result<T, TombstoneError>` 返回

---

**最后更新**：2026 年 5 月 30 日
