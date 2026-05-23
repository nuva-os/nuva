# Nuva OS 系统架构

## 概述

Nuva OS 采用微内核架构，具有最小的 kernel 功能、运行在用户空间的服务、以 IPC 作为主要通信机制以及故障隔离。系统采用五层架构设计（L0-L4），支持 ARM64、x86-64 和 LoongArch64 三种处理器架构，并集成量子安全密码学和插件系统。

---

## 系统架构

### 总体架构（五层架构 L0-L4）

```
+------------------+
|   Applications   |
+------------------+
         |
+------------------+
| App Framework    |
| (UI/Window/Event)|
+------------------+  L4 - 应用框架层
         |
+------------------+
| System Services  |
| (Power/Net/IPC)  |
+------------------+  L3 - 系统服务层
         |
+------------------+
|   Syslib         |
| (Core/Brain/ML)  |
+------------------+  L2 - 系统库层
         |
+------------------+
|  Kernel (Micro)  |
| (Sched/MM/IPC)   |
+------------------+  L1 - 内核层
         |
+------------------+
|   HAL (Hardware  |
|   Abstraction)   |
+------------------+  L0 - 硬件抽象层
         |
+------------------+
|    Hardware      |
+------------------+
```

#### L0 - 硬件抽象层 (HAL)

HAL 提供统一的硬件访问接口，屏蔽底层硬件差异：

| 子模块 | 说明 |
|--------|------|
| `cpu` | CPU HAL：频率/电压/温度/空闲状态管理，支持 Kirin（PSCI SMC）、Loongson、DVFS、热管理 |
| `gpu` | GPU HAL：帧管理、命令队列，支持 Maleoon GPU |
| `npu` | NPU HAL：推理引擎、模型管理、AI 调度器（内核集成 notify_kernel_scheduler/select_cpu_for_task）、性能预测器、DaVinci NPU（DAVINCI_NPU_OPS 桥接真实 HAL）、Hexagon DSP、ONNX 运行时 |
| `power` | 电源 HAL：PMIC、休眠/恢复、跨架构 C-state（MWAIT/WFI/idle）、ACPI 电源驱动（Fadt、S3/S5） |
| `input` | 输入设备 HAL |
| `quantum` | 量子技术 HAL：QRNG（量子随机数生成器）、PQC（后量子密码学） |
| `ffi` | C/C++ FFI 接口：API 稳定性检查、ABI 兼容性验证 |
| `platform` | 平台检测与识别（PlatformInfo、BootInfoType：Fdt/Acpi/Multiboot2/LoongArchFw） |

架构特定 HAL 实现：
- `arm64/` — ARM64 架构（CPU、MMU、中断控制器 GIC、定时器、FDT 引导）
- `x64/` — x86-64 架构（CPU、APIC（LAPIC/I/O APIC）、IDT、GDT、MMU、定时器（LAPIC Timer + TSC）、电源（S3/S5/MWAIT）、页表（destroy/protect））
- `loongarch64/` — LoongArch64 架构（CPU、MMU（3级页表、PageTableOps）、EIOINTC 中断控制器、UEFI 引导、LSX 128位 SIMD、LASX 256位 SIMD、LVZ 虚拟化、LBT 二进制翻译）
- `snapdragon/` — Snapdragon 8 Gen 4 SoC（CPU、GPU、NPU）

#### L1 - 内核层 (Kernel)

微内核，提供最小核心功能：

| 组件 | 说明 |
|------|------|
| 统一错误类型 | `KernelError` 枚举覆盖 7 个错误类别，**扩展变体**（DeadlockDetected、InvalidState、WouldBlock、Timeout、Busy、QuotaExceeded） |
| 调度器 | CFS/RT/Deadline/Idle/EAS 调度，负载均衡，调度域，**声明式策略配置**（`SchedPolicyConfig` 热更新），**Per-CPU 运行队列**（`PerCpuRunQueue` 缓存行对齐） |
| 内存管理 | Buddy+SLAB 分配器，VMA，页错误处理，NUMA，热插拔，**VMA 红黑树增强**（`max_end` 加速查找、`VmaMergePolicy` 延迟合并），**OOM 综合评分** |
| IPC | NuvaIPC（Mach 风格端口消息），共享内存，管道，信号量，消息队列，**零拷贝快速路径**（<=256B 寄存器路径） |
| 中断处理 | 硬件中断，异常，GIC/APIC/EIOINTC 自动检测 |
| 进程管理 | 进程生命周期，信号处理，资源限制 |
| 设备管理 | 设备驱动框架，设备类，**声明式驱动模型**（`DeclarativeDriver` trait、`declare_driver!`、`declare_resource!` 宏），**声明式电源管理**（`PmStateMachine`、`declare_pm!` 宏），**兼容字符串哈希表**（`CompatibleHashTable` O(1) 匹配） |
| 网络栈 | TCP/IP 协议栈，Socket API，NFSv3 客户端，SMB2/3 客户端 |
| 文件系统 | VFS，NuvaFS，ext4，FAT32，io_uring，页缓存，dentry 缓存，缓冲区缓存 |
| 定时器 | 内核定时时子系统，tick/no-tick 模式 |
| 插件系统 | ELF 加载器（RELA 重定位），注册表，沙箱（资源限制），SHA-256 指纹，审计（真实时间戳），PluginServices 内核接口 |
| 安全 | ASLR，栈金丝雀，沙箱，防御系统，病毒扫描器 |
| 调试/性能 | 内核调试器，性能监控，性能调优 |
| 电源管理 | ACPI 驱动（Fadt、S3/S5），PM 子系统 |
| Tombstone | 崩溃记录捕获、存储和查询；通过 HAL 收集崩溃上下文、栈回溯、去重、原子文件写入、内存缓存回退 |
| 量子 | QuantumManager、QuantumRng、QKD 会话、PQC 上下文 |
| 平台检测 | PlatformInfo，BootInfoType，detect_platform_info() |
| 引导流程 | ARM64 FDT + 异常向量表，x64 Multiboot2 + GDT/IDT/异常处理，LoongArch64 UEFI 引导 |
| 同步原语 | SpinLock（抢占控制+持有者追踪），Mutex，Semaphore，RwLock，**PreemptCount**（preempt_disable/enable，分配约束检查），**RwLock TOCTOU 修复**（原子版本检查），**RCU**（读-拷贝-更新，用于读多写少路径），**Per-CPU 变量**（缓存行对齐，无锁访问） |

##### 内核功能域子目录

内核已重组为功能域子目录，提升模块化程度：

| 子目录 | 描述 | 关键重导出 |
|--------|------|-----------|
| `kernel/init/` | 初始化子系统 | cmdline、config、elf、platform、resource |
| `kernel/diag/` | 诊断子系统 | journal、kdebug、log、scanner、stats |
| `kernel/irq_mgmt/` | IRQ 管理 | apic_ops、irq、trap |
| `kernel/net_stack/` | 网络协议栈 | socket、tcpip |
| `kernel/storage/` | 存储子系统 | block |
| `kernel/device/` | 设备模型与插件 | device_model、driver_plugin、feature_plugin、module、notifier |
| `kernel/power_mgmt/` | 电源管理 | hotplug、pm、power |
| `kernel/virt/` | 虚拟化子系统 | vmx |
| `kernel/core/` | 核心内核服务 | cache、cpu、defense、kernel_thread、mempool、perf_tune、posix、random、signal、time、wait、workqueue |

#### L2 - 系统库层 (Syslib)

详见下方 [Syslib 系统库层](#syslib-系统库层-l2) 章节。

#### L3 - 系统服务层

| 服务 | 说明 |
|------|------|
| 电源管理 | 电源状态：Active、Idle、Suspend、Off；休眠模式：Freeze、Standby、Suspend-to-RAM、Hibernate；唤醒锁 |
| 安全服务 | 基于能力的权限模型（CapSet），密钥管理（Keymaster），用户认证（Gatekeeper），TEE 客户端 |
| 网络服务 | TCP/IP 协议栈，DNS 解析，网络接口管理，Socket API |
| IPC 服务 | NuvaIPC 服务（Mach 风格端口消息），共享内存管理 |
| 应用服务 | 声明式Screen生命周期管理（四态模型），包管理（NPK 格式） |
| 外形规格 | 设备外形检测与管理（手机/平板/TV/手表/车机） |

#### L4 - 应用框架层

L4 层是完全声明式的 — 所有 UI、窗口、事件、渲染和资源管理均使用 Nuva 原生声明式范式，无任何遗留的 View/Activity/Widget 代码。

| 模块 | 说明 |
|------|------|
| `application/ui` | 声明式 UI：Screen 系统、Component 模型、State\<T\>、Modifier 链、渲染管线、O(n) Reconciler、自适应布局 |
| `application/window` | 声明式窗口管理：基于 screen 生命周期的窗口、DeclarativeSurface |
| `application/event` | 声明式事件系统：基于 Modifier 的事件绑定、冒泡分发 |
| `application/render` | 声明式合成器：VSync 对齐的帧呈现 |
| `application/resource` | 声明式资源管理：Resource\<T\> 自动 UI 更新、缓存 |

| 组件 | 说明 |
|------|------|
| Screen 系统 | 声明式 screen 生命周期（Screen trait）、ScreenLifecycleManager |
| Component 模型 | 9 个内置组件（Text/Column/Row/Stack/Button/Image/ScrollView/Spacer/SizedBox）、Component trait |
| 状态绑定 | 响应式 State\<T\>，原子版本号 + 脏标记 |
| Modifier 链 | 零成本链式修饰器（layout/event/window/resource） |
| 渲染管线 | Reconcile→Layout→Paint→Composite、O(n) diff、AdaptiveLayoutEngine 集成 |
| 窗口管理 | 声明式窗口、Z 序管理、基于 screen 生命周期的可见性 |
| 事件系统 | 声明式事件、基于 Modifier 的事件处理、冒泡分发 |
| 渲染引擎 | 声明式合成器、VSync 对齐的帧呈现 |

---

### Kernel 设计

#### 微内核原则

- 最小的 kernel 功能
- 服务运行在用户空间
- IPC 作为主要通信机制
- 故障隔离

#### Kernel 组件

1. **调度器**：CFS/RT/Deadline/Idle/EAS 进程/线程调度
2. **内存管理**：虚拟内存，物理内存，Buddy+SLAB 分配器
3. **中断处理**：硬件中断，异常
4. **IPC**：进程间通信（Binder、共享内存、管道等）
5. **进程管理**：进程生命周期，信号，资源限制
6. **设备管理**：设备驱动框架，驱动/功能插件系统
7. **网络栈**：TCP/IP，Socket API，NFSv3，SMB2/3
8. **文件系统**：VFS，NuvaFS，ext4，FAT32，io_uring
9. **定时器**：内核定时器子系统
10. **插件系统**：ELF 加载器，注册表，沙箱（资源限制），SHA-256 指纹，审计
11. **安全**：防御系统，病毒扫描器，沙箱

---

## 内存管理

核心内存管理特性（详见 [MEMORY_zh.md](MEMORY_zh.md)）：

- **物理内存**：Buddy+SLAB 两级分配器、Per-CPU 页缓存、内存区域（DMA/Normal/HighMem）
- **虚拟内存**：4 级页表（ARM64/x64/LoongArch64）、VMA、mmap、COW、大页（2MB/1GB）
- **高级特性**：NUMA 支持、内存热插拔、页面迁移、OOM Killer、内存整理

### 内存布局

#### ARM64

```
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : User space (128TB)
0xFFFF_0000_0000_0000 - 0xFFFF_7FFF_FFFF_FFFF : Kernel space (128TB)
```

#### x86-64

```
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : User space (128TB)
0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF : Kernel space (128TB)
```

#### LoongArch64

```
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF : User space (128TB)
0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF : Kernel space (128TB)
```

---

## 进程调度

核心调度特性（详见 [PROCESS_zh.md](PROCESS_zh.md)）：

### 调度类（按优先级排序）

1. **Deadline** — 最早截止时间优先（EDF），优先级最高
2. **RT** — FIFO/RR，优先级 0-99
3. **CFS** — 红黑树，基于 vruntime 的公平调度
4. **EAS** — 面向 big.LITTLE 系统的能耗感知 CPU 选择
5. **Idle** — 最低优先级，空闲任务

---

## 文件系统

核心文件系统特性（详见 [FILESYSTEM_zh.md](FILESYSTEM_zh.md)）：

- **VFS**：统一抽象层，FileSystem/InodeOps/FileOps trait
- **NuvaFS**：日志结构、COW、快照、ZSTD/LZ4 压缩、去重
- **ext4**：日志模式支持（journal/ordered/writeback）、extents
- **FAT32**：VFAT LFN 支持，最大文件 4GB-1
- **NFSv3**：客户端实现，TCP/UDP socket 传输，RPC 发送/接收/重传，XDR 解码
- **SMB2/3**：客户端实现，TCP 传输，Direct TCP 封包
- **io_uring**：零拷贝异步 IO，环形缓冲区

---

## IPC（进程间通信）

- **NuvaIPC**：Mach 风格端口消息，发送/接收权限，零拷贝传输，同步/异步调用
- **共享内存**：匿名和命名共享内存、内存屏障
- **其他**：管道、信号量、消息队列、信号

---

## 系统服务

- **电源管理**：多种休眠状态（Freeze/Standby/Suspend-to-RAM/Hibernate）、唤醒锁
- **安全服务**：权限管理、Keymaster、Gatekeeper、TEE 客户端
- **网络服务**：TCP/IP 协议栈、DNS 解析、网络接口管理

---

## AI 引擎 (Nuva Brain)

### 架构

```
+------------------+
|   AI Service     |
+------------------+
         |
+------------------+
| Inference Engine |
+------------------+
         |
+------------------+
|  Model Manager   |
+------------------+
         |
+------------------+
|  NPU Scheduler   |
+------------------+
         |
+------------------+
|    Operators     |
+------------------+
```

### 推理流程

1. 加载模型
2. 创建推理上下文
3. 准备输入张量
4. 执行推理
5. 获取输出张量

### NPU 调度

- 优先级队列
- 批处理
- 内存池
- 动态频率调整
- 性能预测器
- AI 调度器（`AiScheduler`）内核集成
- `notify_kernel_scheduler()` — 通知内核 AI 任务调度决策
- `select_cpu_for_task()` — AI 驱动的 CPU 选择
- Da Vinci NPU 算子（`DAVINCI_NPU_OPS`）桥接真实 HAL 实现

---

## Nuva 语言

### 编译流程

```
Source Code (.nv) -> Lexer -> Parser -> Semantic Analysis -> IR Generation -> Optimization -> Code Generation
```

#### 编译器管线（全部已实现）

| 阶段 | 状态 | 详情 |
|------|------|------|
| **词法分析** | 完成 | 字符串/字符/数字/标识符读取，多进制支持（0b/0o/0x），声明式关键字（component、signal、effect、async、resource、with） |
| **语法分析** | 完成 | Pratt 优先级解析，component/signal/effect/async/resource/with 构造的声明式语法解析 |
| **优先级表** | 完成 | 算术运算符优先级已修正，指数运算符（`^`）为右结合 |
| **语义分析** | 完成 | 类型检查、类型推断、纯度验证、声明式约束验证 |
| **代码生成** | 完成 | 管线/推导式 IR 生成、异步状态机 IR、响应式 IR |
| **IR 优化** | 完成 | 常量折叠、DCE（死代码消除）、CSE（公共子表达式消除）、复制传播、循环优化、内联 |

### 运行时（全部已实现）

| 组件 | 状态 | 详情 |
|------|------|------|
| **垃圾回收** | 完成 | 标记-清除 GC，根扫描与清除阶段 |
| **虚拟机** | 完成 | 256 寄存器 VM，指令分发与执行循环 |
| **响应式调度器** | 完成 | Effect 调度、依赖追踪、传播 |
| **二进制模块** | 完成 | NEX 格式加载、重定位、原生代码生成 |
| **HashMap** | 完成 | SipHash 哈希、链式碰撞解决、容量增长时 rehash |

### 声明式构造

Nuva 将声明式构造作为一等语言关键字提供：

| 构造 | 关键字 | 用途 |
|------|--------|------|
| 组件 | `component` | 声明式 UI 组件定义 |
| 响应式信号 | `signal` | 响应式状态绑定，自动传播 |
| 副作用 | `effect` | 带依赖追踪的副作用注册 |
| 异步计算 | `async`/`await` | 声明式异步计算 |
| 资源 | `resource` | 声明式资源获取，自动清理 |
| 上下文管理器 | `with` | 作用域资源管理（RAII 风格） |

### 特殊类型

| 类型 | 说明 |
|------|------|
| `Reactive<T>` | 响应式包装器，自动将更改传播到依赖的 effect |
| `Future<T>` | 异步计算结果，通过 await 解析 |
| `Resource<T>` | 托管资源，具有自动获取和释放语义 |

### 标准库

- 集合：Vec、String、HashMap、LinkedList
- IO：Stdin、Stdout、Stderr、File
- 数学：三角函数、指数、对数
- 响应式：Reactive\<T\>、effect、signal
- 异步：Future\<T\>、spawn、await
- 资源：Resource\<T\>、with

---

## 插件系统架构

Nuva OS 内核包含动态插件系统，支持运行时加载和卸载功能模块：

### 核心组件

| 组件 | 说明 |
|------|------|
| `PluginLoader` | ELF 二进制加载器，内存映射、RELA 重定位（x86-64/AARCH64/LoongArch64）、VFS 文件读取、ElfPlugin 实例化 |
| `PluginRegistry` | 插件注册表，维护元数据、名称索引、类型索引和依赖图 |
| `PluginSandbox` | 沙箱隔离，资源限制检查、MemoryPool 真实分配、IPC 通道限制、设备访问控制 |
| `PluginManager` | 生命周期管理，失败计数追踪、加载时间追踪、依赖解析 |
| `PluginSignature` | SHA-256 完整 FIPS 180-4 实现（插件指纹），Dilithium 签名 |
| `PluginAudit` | 安全审查工作流，真实时间戳和 SHA-256 指纹 |
| `PluginServices` | 内核服务接口：内存限制、IPC 通道限制、设备访问 |

### 插件生命周期

1. **加载**：`PluginLoader::load(path)` — 解析 ELF，映射段，应用 RELA 重定位，解析符号，获取入口点 `plugin_entry`
2. **注册**：`PluginRegistry::register(id, meta)` — 注册插件元数据和依赖关系
3. **使用**：通过 `Plugin` trait 接口调用插件功能
4. **卸载**：`PluginLoader::unload(handle)` — 关闭动态库，释放资源
5. **注销**：`PluginRegistry::unregister(id)` — 移除插件记录

### 插件配置

```rust
pub struct LoaderConfig {
    pub verify_signature: bool,    // 验证插件签名（SHA-256 + Dilithium）
    pub enable_cache: bool,        // 缓存已加载插件
    pub max_plugin_size: usize,    // 最大插件大小（默认 10MB）
}
```

---

## 量子安全架构

Nuva OS 在 HAL 层集成量子安全技术，提供抗量子密码学支持：

### QRNG（量子随机数生成器）

- 硬件 QRNG 检测与初始化
- 软件 PRNG 回退
- 随机数质量评估（`RandomnessQuality`）

### PQC（后量子密码学）

基于 NIST PQC 标准化方案：

| 算法 | 类型 | 变体 |
|------|------|------|
| CRYSTALS-Kyber | 密钥封装 (KEM) | Kyber512, Kyber768, Kyber1024 |
| CRYSTALS-Dilithium | 数字签名 | Dilithium2, Dilithium3, Dilithium5 |

`PqcProvider` trait 接口：

```rust
pub trait PqcProvider: Send + Sync {
    fn kyber_keygen(&self, variant: KyberVariant) -> Result<(PublicKey, SecretKey), PqcError>;
    fn kyber_encapsulate(&self, pk: &PublicKey) -> Result<(SharedSecret, Ciphertext), PqcError>;
    fn kyber_decapsulate(&self, sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, PqcError>;
    fn dilithium_keygen(&self, variant: DilithiumVariant) -> Result<(PublicKey, SecretKey), PqcError>;
    fn dilithium_sign(&self, sk: &SecretKey, msg: &[u8]) -> Result<Signature, PqcError>;
    fn dilithium_verify(&self, pk: &PublicKey, msg: &[u8], sig: &Signature) -> Result<bool, PqcError>;
}
```

### 量子安全安全模块

`QuantumSafeSecurity` 整合 QRNG 和 PQC，提供统一的安全配置（`SecurityConfig`）和安全级别（`SecurityLevel`）。

---

## POSIX 兼容性

### 系统调用

- 文件操作：open、close、read、write、lseek、stat...
- 进程操作：fork、execve、exit、waitpid、getpid...
- 内存操作：mmap、munmap、mprotect、mlock...
- IPC：pipe、shmget、semget、msgget...
- 网络：socket、bind、listen、accept、connect...

### 信号

- 标准信号：SIGHUP、SIGINT、SIGTERM、SIGKILL...
- 实时信号：SIGRTMIN - SIGRTMAX
- 信号处理：sigaction、sigprocmask、sigpending...

---

## 性能优化

### Kernel 优化

- 无锁数据结构
- RCU（读-拷贝-更新）— 读多写少数据结构支持无锁读取，延迟回收；通过 `synchronize_rcu()` 宽限期检测
- Per-CPU 变量 — `#[repr(C, align(64))]` 缓存行对齐；每 CPU 数据（运行队列、页缓存、统计）零锁竞争
- 大页支持（2MB/1GB）

### 内存优化

- Slab 着色
- 内存规整
- 页缓存
- 预读
- Per-CPU 页缓存（PCP）— order-0 分配绕过全局 Buddy 锁；带水位线的批量补充/排空

### I/O 优化

- io_uring — 零拷贝异步 I/O，共享环形缓冲区；用户-内核共享内存中的提交/完成队列；支持固定缓冲区和链接操作

### 调度优化

- 调度域
- 调度组
- 负载追踪（PELT）
- 能效感知调度（EAS）
- Per-CPU 运行队列 — `PerCpuRunQueue` 缓存行对齐，无锁本地调度

---

## 安全设计

### 内存安全

- ASLR（地址空间布局随机化）
- DEP（数据执行保护）
- 栈金丝雀
- 安全栈

### 访问控制

- 能力（Capability）
- ACL（访问控制列表）
- NSM（Nuva 安全模块）策略

### 加密

- 磁盘加密
- 文件加密
- 网络加密（TLS）

### 量子安全

- CRYSTALS-Kyber 密钥封装
- CRYSTALS-Dilithium 数字签名
- QRNG 高质量随机数

---

## 硬件抽象层 (HAL)

HAL 通过 Rust trait 提供统一的硬件访问接口。完整的 trait 签名请参阅 [API_zh.md](API_zh.md) 和 [API_REFERENCE_zh.md](api/API_REFERENCE_zh.md)。

关键 trait 接口：
- `CpuHal` — CPU 频率/电压/温度管理
- `GpuHal` — GPU 帧管理和命令队列
- `NpuHal` — NPU 模型加载、推理执行、缓冲区管理
- `PowerHal` — 电源状态管理（挂起/恢复）
- `InputHal` — 输入设备事件读取

---

## Syslib 系统库层 (L2)

Syslib 层提供面向应用和服务的系统库集合，位于 Kernel 之上、Services 之下。

### 核心子模块

| 子模块 | 说明 | 关键文件 |
|--------|------|----------|
| core | 核心库：分配器（pool）、同步原语（lockfree） | `alloc/`, `sync/` |
| brain | Nuva Brain AI 引擎：推理、模型管理、NPU 调度、算子、服务 | `inference/`, `model/`, `npu/`, `operators/`, `service/` |
| ai | AI 库：模型管理器、优化器、调度器 | `model_manager.rs`, `optimizer.rs`, `scheduler.rs` |
| lang | NuvaLang 编译器和运行时：词法分析、语法分析、语义分析、IR、代码生成、GC、VM | `lexer/`, `parser/`, `semantic/`, `codegen/`, `runtime/`, `stdlib/`, `binary/` |
| ml | 机器学习库：张量、模型、推理引擎 | `tensor.rs`, `model.rs`, `engine.rs` |
| net | 网络库：TCP/UDP/IP/ICMP/ARP/以太网、HTTP、WebSocket、JSON | `tcp/`, `udp.rs`, `ip.rs`, `http.rs`, `websocket.rs`, `json.rs` |
| data | 数据结构库：键值存储、数据库 | `kvstore.rs`, `database.rs` |
| gfx | 图形库：FPS 监控 | `fps/` |
| ui | UI 库：布局、视图、窗口 | `view/` |
| std | 标准库：集合、基础类型、IO | `collection.rs`, `foundation.rs` |
| runtime | 运行时库：Arc、元数据、协议 | `arc.rs`, `metadata.rs`, `protocol.rs` |
| dispatch | 并发框架（GCD 风格）：线程池、信号量、调度队列、调度组 | `pool.rs`, `semaphore.rs`, `queue.rs`, `group.rs` |
| posix | POSIX 兼容层：系统调用封装、信号处理、文件描述符管理 | `errno.rs`, `signal.rs` |

---

## LoongArch64 架构支持

Nuva OS 支持龙芯 LoongArch64 架构，目标平台为 `loongarch64-unknown-none`。

### 支持 SoC

| SoC | Feature Flag | 说明 |
|-----|-------------|------|
| 龙芯 3A6000 | `loongson3a6000` | 桌面处理器 |
| 龙芯 3C6000 | `loongson3c6000` | 服务器处理器 |

### HAL 实现

- `hal/loongarch64/` — LoongArch64 架构特定 HAL 实现
  - `cpu.rs` — CPU 操作（频率、电压、温度、空闲状态）
  - `mmu.rs` — 内存管理单元（3级页表 PageTableOps、Pte 结构体、TLB）
  - `lsx.rs` — LSX 128位 SIMD 扩展（原生 vld/vst 内联汇编）
  - `lasx.rs` — LASX 256位 SIMD 扩展
  - `lvz.rs` — LVZ 硬件虚拟化支持
  - `lbt.rs` — LBT 二进制翻译支持
- `hal/cpu/loongson.rs` — 龙芯 SoC 特定实现
- `kernel/arch/loongarch64/` — 内核架构相关代码（boot 模块、parse_boot_info）
- 设备树通过固件（UEFI）传递

### 中断控制器

LoongArch64 使用 EIOINTC（扩展 I/O 中断控制器），完整实现 `IrqControllerOps` 用于中断路由和管理。

### 内存布局

LoongArch64 使用与 x86-64 兼容的内存布局：
- 48 位虚拟地址空间
- 3 级页表（4KB 页），`PageTableOps` trait 实现
- User space: 0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF (128TB)
- Kernel space: 0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF (128TB)

### 编译状态

三架构均已实现 0 error 编译通过：
- ARM64 (kirin9020): ✅
- x86_64 (intel_core): ✅
- LoongArch64 (loongson3a6000): ✅

---

## SDK 层

Nuva SDK 提供面向开发者的工具链，包括构建、调试、性能分析和打包。

### 核心子模块

| 子模块 | 说明 | 关键文件 |
|--------|------|----------|
| build | 构建系统：交叉编译、构建缓存、目标配置、并行构建调度 | `config.rs`, `cache.rs`, `cross.rs`, `target.rs`, `scheduler.rs`, `executor.rs` |
| cli | 命令行界面：init、build、run、test、debug、profile、包管理 | `args.rs`, `commands/` |
| debug | 调试器：fork+execv 启动，ptrace read_registers，process_vm_readv read_memory，断点，栈展开 | `breakpoint.rs`, `memory.rs`, `stack.rs`, `variable.rs`, `execution.rs`, `target.rs` |
| debug/dap | 调试适配协议：变量读取、反汇编处理器、DAP 服务器 | `protocol.rs`, `server.rs` |
| package | 包管理器：传递依赖解析、注册表版本查询、锁文件、SHA-256 校验和验证 | `dependency.rs`, `resolver.rs`, `registry.rs`, `validator.rs`, `lock_file.rs`, `cache.rs` |
| profiler | 性能分析器：/proc 真实调用栈捕获、gettid()、CPU 采样（/proc/stat）、内存采样（/proc/self/statm）、火焰图 | `cpu.rs`, `memory.rs`, `io.rs`, `lock.rs`, `flamegraph.rs`, `sampler.rs` |

---

<!-- 翻译状态：中文翻译 | 最后更新：2026-05-20 | 与英文版同步 -->

**最后更新**：2026 年 5 月 20 日
**许可证**：Apache-2.0
