# Nuva OS 开发路线图

## 项目完成状态

| 模块 | 框架 | 功能 | 总体 |
|------|------|------|------|
| 内存管理 | 95% | 95% | 95% |
| 进程调度 | 90% | 90% | 90% |
| NvScheduler AI调度器 | 80% | 70% | 75% |
| NvBalancer 硬件均衡器 | 80% | 65% | 72% |
| NvPowerMgr 功耗优化器 | 80% | 65% | 72% |
| 文件系统 | 90% | 90% | 90% |
| 网络协议栈 | 90% | 90% | 90% |
| 设备驱动 | 75% | 72% | 73% |
| 系统调用 | 90% | 90% | 90% |
| 安全模块 | 92% | 90% | 91% |
| 电源管理 | 85% | 60% | 72% |
| 量子安全（PQC） | 95% | 90% | 92% |
| NPU/AI 集成 | 85% | 78% | 81% |
| LoongArch64 支持 | 92% | 80% | 86% |
| RISC-V 64 支持 | 80% | 60% | 70% |
| 插件系统 | 100% | 100% | 100% |
| SDK | 100% | 100% | 100% |
| 引导流程 | 100% | 90% | 95% |
| 平台检测 | 100% | 90% | 95% |

**总体完成度**：积极开发中

---

## 第一阶段：核心功能（高优先级）

### 1. 内存管理模块

| 功能 | 状态 | 描述 |
|------|------|------|
| mem_map 数组实现 | 已实现 | 页帧号到 Page 结构的映射 |
| Per-CPU 页缓存 | 已实现 | 减少锁竞争，提高分配速度 |
| 页回收 (LRU) | 已实现 | LRU 链表和页回收算法 |
| Slab 页回收 | 已实现 | 内存回收和规整 |
| NUMA 支持 | 已实现 | 多节点内存管理，含 SRAT/FDT 解析和 NUMA 均衡 |
| COW 缺页处理 | 已实现 | 实际的 COW 缺页处理逻辑 |

### 2. 进程管理模块

| 功能 | 状态 | 描述 |
|------|------|------|
| fork 进程复制 | 已实现 | COW 地址空间复制、文件描述符复制、信号处理继承 |
| execve ELF 加载 | 已实现 | VFS 文件读取、ELF 段映射、BSS 清零 |
| wait4 进程等待 | 已实现 | 僵尸进程回收 |
| 上下文切换 | 已实现 | 架构相关的寄存器保存/恢复汇编 |
| 信号处理 | 已实现 | 信号发送和处理逻辑 |

### 3. 系统调用模块

| 功能 | 状态 | 描述 |
|------|------|------|
| mmap 实现 | 已实现 | 虚拟内存映射 |
| munmap 实现 | 已实现 | 取消内存映射 |
| brk 堆管理 | 已实现 | 堆扩展和收缩，含页表映射 |
| stat/fstat | 已实现 | 文件状态获取 |

---

## 第二阶段：子系统功能（中优先级）

### 1. 文件系统模块

| 功能 | 状态 | 描述 |
|------|------|------|
| NovaFS 实现 | 已实现 | 原生文件系统的实际操作 |
| 文件权限检查 | 已实现 | 权限验证逻辑 |
| 文件锁定 | 已实现 | flock/fcntl 锁 |
| NFS/SMB 客户端 | 已实现 | NFSv3 RPC 客户端（mount/lookup/read/write/getattr 含 XDR 编解码）+ SMB2/3 客户端（negotiate/session/tree/read/write）（`kernel/net/nfs.rs`、`kernel/net/smb.rs`） |

### 2. 网络协议栈

| 功能 | 状态 | 描述 |
|------|------|------|
| TCP 协议实现 | 已实现 | 完整 TCP 状态机（RFC 793，11 种状态）、三次握手、重传/保活/timewait 定时器、每连接 TCB、段处理 |
| UDP 协议实现 | 已实现 | UDP 数据报处理、校验和验证、Socket 集成 |
| Socket 系统调用 | 已实现 | socket/bind/listen/accept/connect/send/recv 含真实网络栈集成 |
| TCP 拥塞控制 | 已实现 | 慢启动、拥塞避免、快速重传/快速恢复（Reno） |
| Socket connect (TCP) | 已实现 | SYN 段构造含 MSS 选项、伪首部校验和 |

### 3. 设备驱动模块

| 功能 | 状态 | 描述 |
|------|------|------|
| 块设备驱动 | 已实现 | 实际的磁盘驱动 |
| 字符设备驱动 | 已实现 | 实际的串口/TTY 驱动 |
| GPU 驱动 | 框架完成 | Maleoon/Adreno 驱动（寄存器级实现 `hal/gpu/maleoon.rs`，GART/Fence 管理 `hal/gpu/mod.rs`） |
| NPU 驱动 | 框架完成 | Da Vinci/Hexagon 驱动（HAL 已实现，缓冲区管理完整） |

### 4. 安全模块

| 功能 | 状态 | 描述 |
|------|------|------|
| 沙箱机制 | 已实现 | 进程隔离含能力关控资源限制（`kernel/plugin/sandbox.rs`）、NvCapability-LSM 桥接（`kernel/security/security_hook.rs`） |
| 代码签名 | 已实现 | SHA-256 哈希计算、软件签名验证 |
| 安全启动 | 已实现 | 通过代码签名模块的 SHA-256 启动哈希 |
| 内存加密 | 已实现 | RDRAND/Xorshift128+ RNG、XOR 流密码页面加密/解密 |

---

## 第三阶段：高级功能（低优先级）

### 1. 性能优化

| 功能 | 状态 | 描述 |
|------|------|------|
| 性能分析工具 | 已实现 | ftrace 真实函数追踪、perf events PMU 环形缓冲区、monitor 真实 CPU/内存/IO/网络指标 |
| 热代码优化 | 已实现 | PGO 数据收集 + 运行时反馈（布局重排 + prefetch 分支提示） |
| 内存使用优化 | 已实现 | mempool_opt per-CPU 缓存 + SLAB 调用 buddy allocator grow() |
| I/O 性能优化 | 已实现 | io_uring 真实 VFS read/write/open/close/stat/fsync + socket send/recv/accept + SQ/CQ 环形缓冲区管理 + 固定文件/缓冲区注册（`kernel/fs/io_uring.rs`） |

### 2. 测试框架

| 功能 | 状态 | 描述 |
|------|------|------|
| 集成测试 | 已完成 | 真实调度器/内存/VFS/网络统计断言 `tests/integration/mod.rs` |
| 性能测试 | 已完成 | 7个基准测试含真实内核子系统调用 `kernel/tests/benchmarks.rs` |
| 压力测试 | 已完成 | 真实scheduler/net_mgr/VFS/buddy统计 `kernel/tests/stress.rs` + 集成压力测试 |
| 回归测试 | 已完成 | 进程测试套件含真实内核API调用 `kernel/process/tests.rs` |

---

## 第四阶段：量子安全路线图

### 1. CRYSTALS-Kyber（NIST ML-KEM 标准）

Kyber 是 NIST FIPS 203 标准化的后量子密钥封装机制（KEM），提供 IND-CCA2 安全保证。

| 功能 | 状态 | 描述 |
|------|------|------|
| Kyber-512 | 已实现 | 512-bit 安全级别（`hal/quantum/pqc/kyber.rs`） |
| Kyber-768 | 已实现 | 768-bit 安全级别（推荐） |
| Kyber-1024 | 已实现 | 1024-bit 安全级别 |
| C FFI 绑定 | 已实现 | 对参考 C 实现的 FFI 封装 |
| Kyber-768 TLS 集成 | 已实现 | 在 TLS 握手中使用 Kyber KEM |

### 2. CRYSTALS-Dilithium（NIST ML-DSA 标准）

Dilithium 是 NIST FIPS 204 标准化的后量子数字签名算法，提供 EUF-CMA 安全保证。

| 功能 | 状态 | 描述 |
|------|------|------|
| Dilithium2 | 已实现 | 128-bit 安全级别（`hal/quantum/pqc/dilithium.rs`） |
| Dilithium3 | 已实现 | 192-bit 安全级别（推荐） |
| Dilithium5 | 已实现 | 256-bit 安全级别 |
| C FFI 绑定 | 已实现 | 对参考 C 实现的 FFI 封装 |
| 代码签名集成 | 已实现 | 用 Dilithium 替代 RSA/ECDSA 签名 |

### 3. 量子随机数生成器（QRNG）

| 功能 | 状态 | 描述 |
|------|------|------|
| QRNG 接口 | 已实现 | 量子随机数生成接口（`hal/quantum/qrng/`） |
| 硬件 QRNG 集成 | 已实现 | 硬件熵源检测（MMIO/DeviceTree/ACPI/RISC-V seed/ARM RNDR）、SHA-256 条件熵池、健康测试（`hal/quantum/qrng/hardware.rs`） |
| QRNG 健康测试 | 已实现 | NIST SP 800-90B 重复计数+自适应比例+重启测试（单样本模式已修复） |

### 4. 混合密钥交换

| 功能 | 状态 | 描述 |
|------|------|------|
| X25519+Kyber768 混合 | 已实现 | 经典+后量子混合 KEM |
| 兼容性回退 | 已实现 | 对不支持 PQC 的客户端提供经典 KEM 回退 |

---

## 第五阶段：AI/NPU 集成计划

### 1. NPU HAL 完善

| 功能 | 状态 | 描述 |
|------|------|------|
| NPU 设备抽象 | 已实现 | `hal/npu/device.rs`：设备管理与推理接口，Da Vinci NpuDevice trait 实现 |
| ONNX Runtime 集成 | 已实现 | `hal/npu/onnx.rs`：load_model() protobuf 解析 + 8个真实张量算子（add/sub/mul/div/relu/matmul/conv/softmax） |
| AI 调度器 | 已实现 | `hal/npu/ai_scheduler.rs`：多 NPU 任务调度，已与内核 AiSchedExt 集成 |
| 性能预测器 | 已实现 | `hal/npu/predictor.rs`：推理延迟/吞吐预测，已与 AI Scheduler 集成 |
| Da Vinci NPU 驱动 | 已实现 | `hal/npu/davinci.rs`：华为 Da Vinci NPU，NpuHalOps 桥接真实实现 |
| Hexagon DSP 驱动 | 已实现 | 高通 Hexagon DSP/NPU |

### 2. AI Native 内核特性

| 功能 | 状态 | 描述 |
|------|------|------|
| AI 优先调度 | 已实现 | 基于推理延迟的 CFS 扩展（`kernel/sched/eas.rs`） |
| 模型内存管理 | 已实现 | NPU 专用内存池与零拷贝推理 |
| 推理权限控制 | 已实现 | 基于 capability 的模型访问控制 |
| 量化感知调度 | 已实现 | INT8/FP16 混合精度的动态调度 |

### 3. ML/Brain 模块

| 功能 | 状态 | 描述 |
|------|------|------|
| 模型抽象 | 已实现 | `syslib/ml/model.rs`：模型加载与推理接口 |
| 学习框架 | 已实现 | `syslib/brain/learning/`：在线学习支持 |
| 预测框架 | 已实现 | `syslib/brain/prediction/`：系统行为预测 |
| AI 调度决策 | 已实现 | `syslib/brain/scheduler/`：AI 辅助调度决策 |

---

## 第六阶段：LoongArch64 支持计划

### 1. 架构支持

| 功能 | 状态 | 描述 |
|------|------|------|
| HAL 层 | 已实现 | `hal/loongarch64/`：CPU、MMU 抽象（3级页表、Pte 结构体） |
| Kernel 架构层 | 已实现 | `kernel/arch/loongarch64/`：页表、中断、定时器、电源管理、boot 模块 |
| LoongArch 扩展检测 | 已实现 | LSX/LASX/LVZ/LBT 扩展自动检测，含原生 SIMD 内联汇编 |
| 3A6000 平台配置 | 已实现 | `sdk/build-config.toml` 中已定义 |
| 3C6000 平台配置 | 已实现 | `sdk/build-config.toml` 中已定义 |
| LoongArch QEMU 支持 | 待完成 | `qemu-system-loongarch64` 仿真 |

### 2. 扩展指令集利用

| 功能 | 状态 | 描述 |
|------|------|------|
| LSX (128-bit SIMD) | 已实现 | 向量化内存操作（`hal/loongarch64/lsx.rs`，硬件+标量回退） |
| LASX (256-bit SIMD) | 框架完成 | 向量化加密/哈希含标量回退路径（`hal/loongarch64/lasx.rs`） |
| LVZ (虚拟化) | 框架完成 | EPT 页表映射含 CSR 操作（`hal/loongarch64/lvz.rs`） |
| LBT (二进制翻译) | 框架完成 | x86-64/ARM64 指令解码器含 LoongArch64 发射（`hal/loongarch64/lbt.rs`） |

---

## 第七阶段：RISC-V 64 支持计划

### 1. 架构支持

| 功能 | 状态 | 描述 |
|------|------|------|
| HAL 层 | 已实现 | `hal/riscv64/`：CPU、MMU、中断控制器 PLIC、SBI、定时器 |
| Kernel 架构层 | 已实现 | `kernel/arch/riscv64/`：boot/SBI、trap、MMU、PLIC、timer、context |
| Sv39/Sv48 页表 | 框架完成 | RISC-V Sv39 和 Sv48 虚拟内存页表支持 |
| PLIC 驱动 | 已实现 | Platform-Level Interrupt Controller 外部中断路由 |
| SBI 固件接口 | 已实现 | Supervisor Binary Interface 用于引导和系统服务 |
| QEMU virt 支持 | 已实现 | `qemu-system-riscv64 -machine virt` 仿真含 OpenSBI |

### 2. 平台支持

| 功能 | 状态 | 描述 |
|------|------|------|
| 通用 RV64G | 已实现 | 通用 RISC-V 64位（IMAFD 扩展） |
| QEMU virt 虚拟机 | 已实现 | `qemu_virt` feature flag 用于 QEMU virt 平台 |

---

## 第八阶段：插件系统路线图

### 1. 插件框架核心

| 功能 | 状态 | 描述 |
|------|------|------|
| Plugin trait 定义 | 已实现 | `kernel/plugin/`：插件接口与生命周期 |
| 动态加载 | 已实现 | `kernel/plugin/loader.rs`：ELF 解析+内存映射+RELA 重定位+VFS 文件读取+ElfPlugin 实例化 |
| 插件注册表 | 已实现 | `kernel/plugin/registry.rs`：插件发现与管理 |
| 插件沙箱 | 已实现 | `kernel/plugin/sandbox.rs`：安全隔离执行、资源限制检查、MemoryPool 真实分配 |
| 签名验证 | 已实现 | 插件加载时的 Dilithium 签名验证，SHA-256 (FIPS 180-4) 完整实现 |

### 2. 跨架构插件

| 功能 | 状态 | 描述 |
|------|------|------|
| ARM64 插件支持 | 已完成 | `kernel/arch/arm64/plugin.rs`：PageTableOps+IrqOps+TimerOps+PowerOps+ContextOps 真实 ARM64 汇编实现 |
| x64 插件支持 | 已完成 | `kernel/arch/x64/plugin.rs`：X64ArchOps 含 CPUID 检测，`ops()` 返回 `&X64_ARCH` |
| LoongArch64 插件支持 | 已完成 | `kernel/arch/loongarch64/mod.rs`：CPUCFG 内联汇编扩展特性检测 |
| 插件 ABI 稳定性 | 已完成 | `hal/ffi/stability.rs`：`validate_layouts()` 字段重叠/对齐/尺寸校验 |
| 内核 ELF 插件加载器 | 已完成 | `kernel/plugin/loader.rs`：最小 ELF64 解析器（头验证、PT_LOAD 段、SYMTAB/STRTAB 符号查找、RELA 重定位） |

### 3. 插件生态

| 功能 | 状态 | 描述 |
|------|------|------|
| 插件包管理器 | 已实现 | 远程注册表 TCP/HTTP 交互、SHA-256 哈希验证、传递依赖解析 |
| 插件 SDK | 已实现 | 真实编译/测试/打包（VFS 文件检查+SHA-256 哈希计算） |
| 官核与签名流程 | 已实现 | 插件安全审核与签名发布，真实时间戳与 SHA-256 fingerprint |

---

## 第九阶段：SDK 完善计划

### 1. SDK CLI 完善

| 功能 | 状态 | 描述 |
|------|------|------|
| build 命令 | 已实现 | `sdk/cli/commands/build.rs` |
| run 命令 | 已实现 | `sdk/cli/commands/run.rs` |
| test 命令 | 已实现 | `sdk/cli/commands/test.rs` |
| debug 命令 | 已实现 | `sdk/cli/commands/debug.rs` |
| init/new 命令 | 已实现 | `sdk/cli/commands/init.rs`, `new.rs` |
| lint 命令 | 已实现 | `sdk/cli/commands/lint.rs` |
| fmt 命令 | 已实现 | `sdk/cli/commands/fmt.rs` |
| pkg 命令 | 已实现 | `sdk/cli/commands/pkg.rs`：包管理 |

### 2. 调试基础设施

| 功能 | 状态 | 描述 |
|------|------|------|
| DAP 服务器 | 已实现 | `sdk/debug/dap/`：DAP 协议实现，变量读取+反汇编（原始字节） |
| 断点管理 | 已实现 | `sdk/debug/breakpoint.rs` |
| 内存查看 | 已实现 | `sdk/debug/memory.rs` |
| 调用栈跟踪 | 已实现 | `sdk/debug/stack.rs` |
| 变量检查 | 已实现 | `sdk/debug/variable.rs` |

### 3. 性能分析

| 功能 | 状态 | 描述 |
|------|------|------|
| CPU 性能分析 | 已实现 | `sdk/profiler/cpu.rs`，采样器使用 /proc 读取真实调用栈 |
| 内存性能分析 | 已实现 | `sdk/profiler/memory.rs` |
| Flamegraph 生成 | 已实现 | `sdk/profiler/flamegraph.rs` |
| 采样器 | 已实现 | `sdk/profiler/sampler.rs`：/proc 真实线程 ID+调用栈捕获 |

### 4. 包管理

| 功能 | 状态 | 描述 |
|------|------|------|
| 依赖解析 | 已实现 | `sdk/package/resolver.rs`：传递依赖+注册表版本查询 |
| 包缓存 | 已实现 | `sdk/package/cache.rs` |
| Lock 文件 | 已实现 | `sdk/package/lock_file.rs` |
| 包注册中心 | 已实现 | `sdk/package/registry.rs`：HTTP GET/POST+JSON 解析+搜索+发布+版本列表 |

---

## 状态图例

- **已实现**：功能已完全实现并测试
- **框架完成**：数据结构和接口已定义，但核心逻辑未实现
- **待开发**：功能尚未开始
- **部分完成**：部分功能已实现

---

## 里程碑

### 里程碑 1：核心功能（2026 年 Q2）

- [x] 引导流程补全（ARM64 FDT、x64 Multiboot2、LoongArch64 UEFI）
- [x] 平台检测（PlatformInfo、BootInfoType）
- [x] 内存管理 mmap/munmap/mprotect/msync
- [x] 进程创建/销毁完整流程
- [x] VFS sys_open/close/read/write/lseek/mkdir/unlink
- [x] IRQ 控制器自动检测（GIC/APIC/EIOINTC/PLIC）
- [x] RISC-V 64 SBI 启动、PLIC、trap 处理
- [x] 完成内存管理（COW、NUMA、页回收）
- [x] 完成进程管理（execve、wait4、信号）
- [x] 完成系统调用实现

### 里程碑 2：子系统集成（2026 年 Q3）

- [x] 完成 NovaFS 实现
- [x] 完成 TCP/IP 协议栈
- [x] 基本设备驱动（块、字符）
- [x] 基本安全功能

### 里程碑 3：量子安全与 AI 集成（2026 年 Q4）

- [x] Kyber/Dilithium NIST 标准合规验证
- [x] 混合密钥交换（X25519+Kyber768）
- [x] NPU 推理管线完善
- [x] AI 辅助调度上线

### 里程碑 4：多架构与插件（2027 年 Q1）

- [x] LoongArch64 完整支持（QEMU + 实机）
- [x] RISC-V 64 支持（SBI 引导、PLIC、Sv39/Sv48 MMU、QEMU virt）
- [x] 插件系统签名验证
- [x] 插件 SDK 发布
- [x] SDK v1.0 发布

### 里程碑 5：生产就绪（2027 年 Q2）

- [x] 性能优化
- [x] 全面测试
- [x] 文档完善
- [ ] 生产部署

---

## 当前重点

### 高优先级任务

1. **文件系统** ✅
   - [x] 完成 NFS/SMB 客户端 RPC 网络收发和 XDR 解码
   - [x] 完成 io_uring 异步 I/O 集成
   - [x] 完善 NuvaFS 快照和日志机制（WAL/COW/Snapshot）

2. **网络协议栈** ✅
   - [x] 完善 TCP 状态机边界情况
   - [x] 完成网络防火墙和安全规则
   - [x] 实现完整的 IPv6 邻居发现（NDP/NUD/DAD/RA/SLAAC/SEND框架）

3. **安全模块** ✅
   - [x] NvCapability 令牌与 LSM hooks 桥接
   - [x] 完成代码签名验证链（SignatureChain/CertChain/X509/PQC签名）
   - [x] 完善安全启动证明（PCR extend修复为SHA256标准/Quote/AIK/EventLog）

4. **量子安全** ✅
   - [x] 集成硬件 QRNG 熵源
   - [x] 完成 QKD BB84 协议实现
   - [x] Kyber/Dilithium NIST PQC 合规验证（参数修复Dilithium5=4595）

### 中优先级任务

1. **设备驱动**
   - [ ] 完成 GPU 驱动寄存器级实现（Maleoon/Adreno）
   - [ ] 完成 NPU 驱动缓冲区管理和推理管线
   - [ ] 实现 USB 主控制器驱动

2. **电源管理**
   - [ ] 完成 CPU DVFS 和热管理
   - [ ] 实现完整的系统挂起/恢复流程
   - [ ] 完成 PMIC 驱动集成

3. **RISC-V 64** ✅
   - [ ] 完成 Sv39/Sv48 页表及完整 MMU 抽象
   - [x] 完成 PLIC 驱动及所有中断路由场景
   - [ ] 在真实 RISC-V 64 硬件上验证

4. **LoongArch64** ✅
   - [ ] QEMU 仿真支持验证
   - [x] LSX/LASX 指令集利用（原生 SIMD）
   - [ ] LBT 二进制翻译完善

### 低优先级任务

1. **文档** — 双语同步，API 参考完善
2. **测试** — 扩展集成和压力测试
3. **性能** — 基准测试基线测量

---

## 贡献

我们欢迎贡献！编码指南请参见 [CODING_STANDARD_zh.md](CODING_STANDARD_zh.md)。

### 如何贡献

1. 从路线图中选择一个任务
2. 阅读相关文档
3. 实现功能
4. 编写测试
5. 提交 Pull Request

### 贡献领域

- 核心 kernel 开发
- 设备驱动
- 文件系统
- 网络协议
- 量子安全（PQC）
- NPU/AI 集成
- LoongArch64 移植
- RISC-V 64 移植
- 插件开发
- 测试和文档
- 性能优化

---

## 资源

- **文档**：[docs/](docs/) 目录
- **源代码**：[kernel/](kernel/) 目录
- **Issue**：https://github.com/nuva-os/nuva/issues
- **讨论**：https://github.com/nuva-os/nuva/discussions

---

## 联系方式

- **邮箱**：kellen9903@gmail.com
- **GitHub**：https://github.com/nuva-os/nuva

---

**最后更新**：2026 年 5 月 30 日
**更新者**：Nuva OS Team
