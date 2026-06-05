# Nuva OS 进程管理模块

## 概述

进程管理模块提供完整的进程生命周期管理功能，包括进程创建、调度、终止等。调度系统支持 CFS/RT/Deadline/Idle/EAS 五种调度策略，具备完整的负载追踪和负载均衡机制。

---

## 目录

1. [进程调度](#1-进程调度)
2. [进程控制块](#2-进程控制块)
3. [进程系统调用](#3-进程系统调用)
4. [负载均衡](#4-负载均衡)
5. [调度策略](#5-调度策略)
6. [EAS 能耗感知调度](#6-eas-能耗感知调度)
7. [Deadline 调度](#7-deadline-调度)
8. [负载追踪机制](#8-负载追踪机制)
9. [信号处理](#9-信号处理)
10. [文件结构](#10-文件结构)

---

## 1. 进程调度

### 1.1 调度实体

```rust
pub struct SchedEntity {
    pub vruntime: AtomicU64,      // Virtual runtime
    pub runtime: AtomicU64,       // Actual runtime
    pub wait_time: AtomicU64,     // Wait time
    pub time_slice: AtomicU32,    // Time slice
    pub prio: AtomicU32,          // Dynamic priority
    pub static_prio: Priority,    // Static priority
    pub normal_prio: Priority,    // Normal priority
    pub policy: AtomicU32,        // Scheduling policy
    pub flags: AtomicU32,         // Flags
    pub cpus_allowed: AtomicU64,  // Allowed CPUs
    pub cpu: AtomicU32,           // Current CPU
    pub last_ran: AtomicU64,      // Last run time
    pub switches: AtomicU64,      // Context switch count
}
```

### 1.2 运行队列

```rust
pub struct RunQueue {
    pub cpu: CpuId,               // CPU ID
    pub lock: AtomicU32,          // Spinlock
    pub nr_running: AtomicU32,    // Running process count
    pub rt_nr_running: AtomicU32, // Real-time process count
    pub min_vruntime: AtomicU64,  // Minimum virtual runtime
    pub curr: u64,                // Current process
    pub idle: u64,                // Idle process
    pub clock: AtomicU64,         // Scheduler clock
    pub load: AtomicU64,          // Load
    pub switches: AtomicU64,      // Switch count
}
```

### 1.3 调度器

```rust
pub struct Scheduler {
    pub nr_cpus: u32,                 // CPU count
    pub run_queues: [RunQueue; 8],    // Run queues
    pub nr_running: AtomicU32,        // Total running processes
    pub nr_switches: AtomicU64,       // Total switches
    pub sched_count: AtomicU64,       // Schedule count
    pub lb_interval: AtomicU64,       // Load balance interval
}
```

---

## 2. 进程控制块

### 2.1 进程结构

```rust
pub struct ProcessControlBlock {
    pub process: Process,                     // Process structure
    pub files: FilesStruct,                   // File descriptor table
    pub parent: *mut ProcessControlBlock,     // Parent process
    pub children: *mut ProcessControlBlock,   // Child process list
    pub sibling: *mut ProcessControlBlock,    // Sibling process list
}
```

### 2.2 进程状态

| 状态 | 描述 |
|-------|-------------|
| `Created` | 新创建 |
| `Ready` | 就绪 |
| `Running` | 运行中 |
| `Blocked` | 阻塞 |
| `Zombie` | 僵尸 |
| `Terminated` | 已终止 |

---

## 3. 进程系统调用

### 3.1 进程创建

#### fork - 创建子进程

| 项目 | 描述 |
|------|-------------|
| 功能 | 创建当前进程的副本 |
| 返回 | 父进程返回子进程 PID，子进程返回 0 |

**实现步骤**：
1. 获取当前进程 ID
2. 分配新进程 ID
3. 分配进程控制块
4. 复制父进程地址空间（COW）
5. 复制文件描述符表
6. 复制信号处理
7. 设置父子关系
8. 将子进程加入调度器队列

#### vfork - 创建子进程（共享地址空间）

| 项目 | 描述 |
|------|-------------|
| 功能 | 创建共享父进程地址空间的子进程 |
| 特性 | 父进程阻塞直到子进程调用 exec 或 exit |

#### clone - 创建进程或线程

| 项目 | 描述 |
|------|-------------|
| 功能 | 根据标志创建进程或线程 |
| 参数 | flags, child_stack, ptid, ctid, newtls |

**Clone 标志**：

| 标志 | 描述 |
|------|-------------|
| `CLONE_VM` | 共享地址空间（线程） |
| `CLONE_FS` | 共享文件系统信息 |
| `CLONE_FILES` | 共享文件描述符表 |
| `CLONE_SIGHAND` | 共享信号处理 |
| `CLONE_THREAD` | 同一线程组 |
| `CLONE_SETTLS` | 设置 TLS |

### 3.2 程序执行

#### execve - 执行新程序

| 项目 | 描述 |
|------|-------------|
| 功能 | 加载并执行新程序 |
| 参数 | filename, argv, envp |

**实现步骤**：
1. 打开可执行文件
2. 检查文件权限
3. 读取文件头
4. 解析 ELF 格式
5. 检查解释器（如 #!）
6. 释放旧地址空间
7. 创建新地址空间
8. 加载程序段（.text、.data、.bss）
9. 设置栈
10. 设置参数和环境变量
11. 设置入口点
12. 切换到用户空间执行

### 3.3 进程终止

#### exit - 终止当前进程

| 项目 | 描述 |
|------|-------------|
| 功能 | 终止当前进程 |
| 参数 | status（退出状态） |

**实现步骤**：
1. 设置退出状态
2. 关闭所有打开的文件
3. 释放地址空间
4. 向父进程发送 SIGCHLD
5. 如果父进程在等待，唤醒父进程
6. 将子进程重新分配给 init
7. 进入僵尸状态
8. 调度其他进程

### 3.4 进程等待

#### wait4 - 等待子进程状态变化

| 项目 | 描述 |
|------|-------------|
| 功能 | 等待子进程状态变化，回收僵尸子进程 |
| 参数 | pid, status, options, rusage |

**pid 参数**：
- `-1`：等待任意子进程
- `0`：等待同一进程组中的任意子进程
- `> 0`：等待指定子进程
- `< -1`：等待指定进程组中的任意子进程

**options 参数**：
- `WNOHANG`：非阻塞
- `WUNTRACED`：报告已停止的子进程
- `WCONTINUED`：报告已继续的子进程

### 3.5 信号发送

#### kill - 向进程发送信号

| 项目 | 描述 |
|------|-------------|
| 功能 | 向进程或进程组发送信号 |
| 参数 | pid, sig |

### 3.6 进程信息获取

| 系统调用 | 功能 |
|-------------|----------|
| `getpid()` | 获取当前进程 ID |
| `getppid()` | 获取父进程 ID |
| `gettid()` | 获取当前线程 ID |
| `getpgid(pid)` | 获取进程组 ID |

### 3.7 会话和进程组管理

| 系统调用 | 功能 |
|-------------|----------|
| `setsid()` | 创建新会话 |
| `setpgid(pid, pgid)` | 设置进程组 |

### 3.8 调度控制

#### sched_yield - 让出 CPU

| 项目 | 描述 |
|------|-------------|
| 功能 | 主动让出 CPU |
| 实现 | 调用调度器的调度函数 |

---

## 4. 负载均衡

### 4.1 负载统计

```rust
pub struct LoadStats {
    pub load: AtomicU64,          // Load
    pub nr_running: AtomicU32,    // Running process count
    pub nr_runnable: AtomicU32,   // Runnable process count
    pub nr_waiting: AtomicU32,    // Waiting process count
    pub avg_load: AtomicU64,      // Average load
}
```

### 4.2 调度域

调度域是负载均衡的基本单位，形成层次结构：

```rust
pub struct SchedDomain {
    pub span: AtomicU64,              // CPU mask
    pub parent: *mut SchedDomain,     // Parent scheduling domain
    pub child: *mut SchedDomain,      // Child scheduling domain
    pub load: LoadStats,              // Load statistics
    pub balance_interval: AtomicU64,  // Balance interval
    pub last_balance: AtomicU64,      // Last balance time
    pub flags: AtomicU32,             // Flags
    pub level: u32,                   // Level
}
```

**调度域层次**：

| 层级 | 说明 |
|------|------|
| SMT（超线程） | 同一物理核心的超线程 sibling |
| MC（多核心） | 同一芯片上的核心 |
| NUMA | 同一 NUMA 节点的核心 |
| ALL | 全系统 |

### 4.3 调度组

调度组是调度域内的分组，用于负载均衡计算：

```rust
pub struct SchedGroup {
    pub cpus: AtomicU64,              // CPU mask
    pub next: *mut SchedGroup,        // Next group (circular list)
    pub load: LoadStats,              // Group load statistics
    pub imbalance_pct: u32,           // Imbalance percentage threshold
    pub sched_domain: *mut SchedDomain, // Parent domain
}
```

- 每个调度域包含一组调度组（循环链表）
- 负载均衡在组间比较负载，选择最忙组进行迁移
- `imbalance_pct` 控制触发迁移的负载不平衡阈值

### 4.4 负载均衡器

```rust
pub struct LoadBalancer {
    pub balance_count: AtomicU64,     // Balance count
    pub migration_count: AtomicU64,   // Migration count
    pub fail_count: AtomicU64,        // Failure count
    pub max_migrations: AtomicU32,    // Max migrations
    pub interval: AtomicU64,          // Balance interval
}
```

### 4.5 CPU 亲和性

```rust
pub struct CpuAffinity {
    pub default_mask: AtomicU64,      // Default CPU mask
}
```

---

## 5. 调度策略

### 5.1 调度策略类型

```rust
pub enum SchedPolicy {
    Normal = 0,      // Normal scheduling (CFS)
    Fifo = 1,        // First-in-first-out (RT)
    RoundRobin = 2,  // Round-robin (RT)
    Batch = 3,       // Batch processing
    Idle = 4,        // Idle
    Deadline = 5,    // Deadline
}
```

### 5.2 CFS（完全公平调度器）

**特性**：
- 使用红黑树组织进程
- 基于虚拟运行时间（vruntime）
- 公平的 CPU 时间分配

**vruntime 计算**：
```
vruntime += delta_exec * NICE_0_LOAD / se->load
```

**时间片计算**：
```
time_slice = target_latency * (se->load / total_load)
```

- `target_latency`：目标调度延迟（默认 6ms）
- 最小粒度：`sched_min_granularity`（默认 0.75ms）

### 5.3 RT（实时调度器）

**特性**：
- 优先级高于普通进程
- FIFO：先进先出，不主动让出
- RR：轮转调度，时间片用完后让出
- 优先级范围：0（最高）到 99（最低）

---

## 6. EAS 能耗感知调度

EAS（Energy Aware Scheduling）在系统未过载时，基于能量模型为唤醒任务选择能效最优的 CPU。

### 6.1 能量模型

```rust
pub struct EnergyModel {
    pub domains: [Option<PerfDomain>; MAX_NR_PERF_DOMAINS],
    pub nr_domains: u32,
    pub total_capacity: AtomicU64,
    pub enabled: AtomicBool,
    pub overutilization_threshold: AtomicU32,
}
```

### 6.2 性能域

```rust
pub struct PerfDomain {
    pub cpus: u64,
    pub nr_cpus: u32,
    pub states: [Option<PerfState>; 16],
    pub nr_states: u32,
    pub current_state: AtomicU32,
    pub name: [u8; 16],
}

pub struct PerfState {
    pub frequency: u32,   // 频率（kHz）
    pub voltage: u32,     // 电压（微伏）
    pub power: u32,       // 功耗（微瓦）
    pub cost: u64,        // 成本系数
}
```

### 6.3 EAS 调度流程

1. 检查 EAS 是否启用及系统是否过载
2. 若过载（>80%），回退到 CFS 负载均衡
3. 计算每个 CPU 的能量代价
4. 考虑迁移代价（非本地 CPU 额外 100mW 惩罚）
5. 选择总能量最小的 CPU

### 6.4 EAS 统计

```rust
pub struct EasStats {
    pub eas_wakeups: AtomicU64,     // EAS 唤醒次数
    pub eas_migrations: AtomicU64,  // EAS 迁移次数
    pub eas_fallbacks: AtomicU64,   // 回退到 CFS 的次数
    pub energy_saved: AtomicU64,    // 节省的能量（估计值）
}
```

---

## 7. Deadline 调度

Deadline 调度器为实时任务提供保证性的 CPU 带宽和截止时间支持。

### 7.1 Deadline 参数

| 参数 | 说明 |
|------|------|
| `runtime` | 每个 period 内保证的 CPU 运行时间 |
| `period` | 调度周期 |
| `deadline` | 绝对截止时间 |

### 7.2 Deadline 调度规则

- 采用最早截止时间优先（EDF）算法
- 优先级高于 RT 调度器
- 带宽验证：确保所有 Deadline 任务的总带宽不超过 100%
  - `sum(runtime_i / period_i) <= 1`
- 超过带宽时拒绝新任务（`sched_setattr` 返回 EBUSY）

### 7.3 Deadline 与 EAS 的关系

- Deadline 任务具有最高调度优先级
- EAS 不对 Deadline 任务生效（Deadline 忽略能量模型）
- 系统过载时 Deadline 任务仍受保障

---

## 8. 负载追踪机制

### 8.1 Per-Entity Load Tracking (PELT)

PELT 对每个调度实体独立追踪负载贡献：

**负载计算**：
```
load_sum = load_sum * y + running
load_avg = load_sum / (LOAD_AVG_MAX - 1024)
```

- `y`：衰减因子，约 0.98（对应 1ms 衰减周期）
- `running`：当前是否在运行（1 或 0）
- 1024 周期（约 1 秒）内的历史负载按指数衰减

### 8.2 运行时间追踪

```
runnable_sum = runnable_sum * y + runnable
runnable_avg = runnable_sum / (LOAD_AVG_MAX - 1024)
```

- `load_avg`：反映任务在 CPU 上的负载贡献
- `runnable_avg`：反映任务在运行队列中的等待时间

### 8.3 CPU 负载更新

每次时钟 tick 更新：
1. 更新当前任务的 PELT 负载
2. 更新运行队列的 CPU 负载
3. 更新调度域的负载统计
4. 触发周期性负载均衡检查

---

## 9. 信号处理

### 9.1 信号类型

| 类别 | 信号 | 说明 |
|------|------|------|
| 终止 | SIGTERM (15) | 请求终止（可捕获） |
| 强制终止 | SIGKILL (9) | 强制终止（不可捕获） |
| 中断 | SIGINT (2) | 中断（Ctrl+C） |
| 挂断 | SIGHUP (1) | 终端挂断 |
| 段错误 | SIGSEGV (11) | 无效内存访问 |
| 总线错误 | SIGBUS (7) | 总线错误 |
| 子进程 | SIGCHLD (17) | 子进程状态变化 |
| 管道 | SIGPIPE (13) | 管道破裂 |
| 实时 | SIGRTMIN-SIGRTMAX | 实时信号（可排队） |

### 9.2 信号处理流程

1. **信号发送**：`kill(pid, sig)` 或 `raise(sig)`
2. **信号挂起**：加入目标进程的挂起信号集
3. **信号检测**：在从内核返回用户空间前检查挂起信号
4. **信号交付**：
   - 默认处理：执行默认动作（终止/忽略/停止/继续）
   - 自定义处理：跳转到 `sigaction` 注册的处理函数
   - 忽略：丢弃信号

### 9.3 信号掩码

- `sigprocmask`：阻塞/解除阻塞信号
- `sigpending`：查询挂起的阻塞信号
- 阻塞信号不会丢失（挂起等待），但实时信号有排队上限

### 9.4 信号相关系统调用

| 系统调用 | 功能 |
|----------|------|
| `kill(pid, sig)` | 向进程发送信号 |
| `tgkill(tgid, tid, sig)` | 向线程发送信号 |
| `raise(sig)` | 向自身发送信号 |
| `sigaction(sig, act, oact)` | 设置信号处理函数 |
| `sigprocmask(how, set, oset)` | 设置信号掩码 |
| `sigpending(set)` | 获取挂起信号集 |
| `sigsuspend(mask)` | 等待信号（原子替换掩码并暂停） |
| `pause()` | 等待任意信号 |
| `alarm(seconds)` | 设置定时器（SIGALRM） |

---

## 10. 文件结构

```
kernel/sched/
├── scheduler.rs        # Scheduler basics
├── cfs.rs              # CFS scheduler
├── rt.rs               # RT scheduler
├── eas.rs              # Energy Aware Scheduling
├── core.rs             # Core scheduling
├── load_balance.rs     # Load balancing
├── sched_domain.rs     # Scheduling domains and groups
├── rbtree.rs           # Red-black tree (CFS)
├── task.rs             # Task management
├── quant_sched.rs      # Quantum scheduler
├── ai_sched.rs         # AI scheduler

kernel/syscall/
└── process_integration.rs  # Process management system calls
```

---

**最后更新**：2026 年 5 月 30 日
**许可证**：Apache-2.0
