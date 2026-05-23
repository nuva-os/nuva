# 核心内核服务

## 概述

`kernel/core/` 模块提供内核中使用的基础内核服务，包括 CPU 管理、缓存、内存池、工作队列、时间保持、信号处理、POSIX 兼容性、随机数生成、防御机制、内核线程、性能调优和等待队列。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口 |
| `cpu.rs` | CPU 管理和 Per-CPU 数据结构 |
| `cache.rs` | 内核缓存子系统 |
| `mempool.rs` | 内存池分配器 |
| `workqueue.rs` | 工作队列（延迟工作执行） |
| `wait.rs` | 等待队列实现 |
| `time.rs` | 时间保持和定时器 |
| `signal.rs` | 信号处理 |
| `posix.rs` | POSIX 兼容层 |
| `random.rs` | 随机数生成 |
| `defense.rs` | 内核防御机制 |
| `kernel_thread.rs` | 内核线程管理 |
| `perf_tune.rs` | 性能调优子系统 |

## 关键特性

- **Per-CPU 变量**：缓存行对齐的每 CPU 数据，支持跨 CPU 无锁访问
- **内存池**：使用预分配大小的池分配器（`mempool`）进行高效内核内存分配
- **内核线程**：轻量级内核线程创建和生命周期管理
- **工作队列**：在进程上下文中执行延迟工作，支持优先级调度
- **等待队列**：阻塞等待与唤醒通知，用于内核任务间同步
- **时间保持**：高精度定时器和系统时间管理
- **信号处理**：POSIX 信号传递和处理基础设施
- **内核防御**：栈金丝雀校验、ASLR 支持和内核加固机制

## 初始化顺序

核心服务在多个引导阶段中初始化：

| 阶段 | 组件 | 初始化函数 |
|------|------|-----------|
| 1 — 引导 | cpu | `cpu::init_cpu()` |
| 2 — 内存与 IRQ | mempool、random、time | `mempool::init_mempool()`、`random::init_random()`、`time::init_time()` |
| 4 — 基础设施 | workqueue | `workqueue::init_workqueue()` |
| 5 — 核心内核 | signal | `signal::init_signal()` |
| 6 — 韧性 | defense、cache、perf_tune | `defense::init_defense()`、`cache::init_cache()`、`perf_tune::init_perf_tune()` |

## 依赖关系

- **内部依赖**：HAL（L0）提供 CPU 拓扑，`kernel/sync` 提供 RCU 和同步原语
- **上层被依赖**：所有其他内核子系统 — core 提供了每个内核组件都依赖的基础服务（CPU、mempool、time、workqueue）

## 公开接口

- `cpu` 模块：Per-CPU 数据结构、CPU 拓扑检测、CPU 状态管理（`init_cpu()`、`current_cpu_id()`）
- `cache` 模块：内核热数据缓存子系统（`init_cache()`、`cache_get()`、`cache_put()`）
- `mempool` 模块：多池大小的内存池分配器（`init_mempool()`、`pool_alloc()`、`pool_free()`）
- `workqueue` 模块：延迟执行工作队列（`init_workqueue()`、`schedule_work()`、`flush_work()`）
- `wait` 模块：阻塞同步等待队列（`wait_event()`、`wake_up()`）
- `time` 模块：高精度定时器时间保持（`init_time()`、`get_time()`、`set_timer()`）
- `signal` 模块：信号传递框架（`init_signal()`、`send_signal()`、`handle_signal()`）
- `posix` 模块：POSIX 兼容层，用于移植 Unix 应用程序
- `random` 模块：内核随机数生成（`init_random()`、`get_random_bytes()`）
- `defense` 模块：内核防御机制，包括栈金丝雀和 ASLR（`init_defense()`）
- `kernel_thread` 模块：内核线程创建和管理（`create_kernel_thread()`）
- `perf_tune` 模块：运行时性能调优接口（`init_perf_tune()`）

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `cache`、`cpu`、`defense`、`kernel_thread`、`mempool`、`perf_tune`、`posix`、`random`、`signal`、`time`、`wait`、`workqueue`

---

*最后更新：2026-05-20 | Nuva OS v1.0.0*
