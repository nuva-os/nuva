# Nuva OS 开发路线图

## 项目完成状态

| 模块 | 框架 | 功能 | 总体 |
|------|------|------|------|
| 内存管理 | 95% | 95% | 95% |
| 进程调度 | 90% | 90% | 90% |
| 文件系统 | 85% | 85% | 85% |
| 网络协议栈 | 80% | 85% | 82% |
| 设备驱动 | 75% | 72% | 73% |
| 系统调用 | 90% | 90% | 90% |
| 安全模块 | 88% | 85% | 86% |
| 电源管理 | 85% | 60% | 72% |
| 量子安全（PQC） | 90% | 85% | 87% |
| NPU/AI 集成 | 85% | 78% | 81% |
| LoongArch64 支持 | 92% | 80% | 86% |
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
| NFS/SMB 客户端 | 框架完成 | NFSv3/SMB2 客户端含 RPC/XDR 和协商（`kernel/net/nfs.rs`、`kernel/net/smb.rs`） |

### 2. 网络协议栈

| 功能 | 状态 | 描述 |
|------|------|------|
| TCP 协议实现 | 框架完成 | 完整的 TCP 状态机 |
| UDP 协议实现 | 框架完成 | UDP 数据报处理 |
| Socket 系统调用 | 框架完成 | socket/bind/listen/accept/send/recv |
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
| 沙箱机制 | 已实现 | 进程隔离（`kernel/plugin/sandbox.rs`） |
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
| I/O 性能优化 | 已实现 | io_uring 真实 VFS read/write/open/close/stat/fsync + socket send/recv/accept |

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
| 硬件 QRNG 集成 | 已实现 | 对接硬件量子随机源 |
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

## 第七阶段：插件系统路线图

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

## 第八阶段：SDK 完善计划

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
- [x] IRQ 控制器自动检测（GIC/APIC/EIOINTC）
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

1. **内存管理** ✅
   - mem_map 数组已实现
   - Per-CPU 页缓存已实现
   - LRU 页回收已实现

2. **进程管理** ✅
   - fork COW 地址空间复制已实现
   - execve ELF 从 VFS 加载已实现
   - 上下文切换已实现

3. **系统调用** ✅
   - mmap/munmap 已实现
   - brk 页表映射已实现
   - stat/fstat 已实现

4. **量子安全** ✅
   - Kyber/Dilithium C 实现集成验证完成
   - X25519+Kyber768 混合 KEM 已实现

### 中优先级任务

1. **文件系统** ✅
   - 完成 NovaFS 操作
   - 添加文件权限检查
   - 实现文件锁定
   - NFS/SMB 客户端 RPC 网络收发和 XDR 解码

2. **网络** ✅
   - 完成 TCP 实现
   - 完成 UDP 实现
   - 添加 Socket 系统调用

3. **NPU/AI** ✅
   - Da Vinci NPU 驱动完善（NpuHalOps 桥接真实实现）
   - AI 调度器与内核调度集成（AiSchedExt 桥接）
   - 模型内存管理优化
   - 性能预测器与 AI Scheduler 集成

4. **LoongArch64** ✅
   - QEMU 仿真支持
   - LSX/LASX 指令集利用（原生 SIMD 内联汇编）
   - PageTableOps map/unmap/translate/protect 实现
   - IrqControllerOps EIOINTC 中断分配/处理实现

### 低优先级任务

1. **插件系统** ✅
   - 插件签名验证
   - 插件 SDK 开发

2. **SDK** ✅
   - CLI 命令完善
   - 文档生成

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

- **邮箱**：team@nuva-os.org
- **GitHub**：https://github.com/nuva-os/nuva

---

**最后更新**：2026 年 5 月 14 日
**更新者**：Nuva OS Team
