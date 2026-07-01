# Nuva OS AI 架构

## 概述

Nuva OS 是一个AI原生操作系统，人工智能驱动从调度、负载均衡到功耗管理和推理的每一层自主决策。AI子系统遵循**感知→推理→执行→验证**闭环，并配备三级回退机制，确保AI置信度不足时系统依然稳定。

## 架构图

```
┌─────────────────────────────────────────────────────────┐
│                    L4 应用层                             │
│  AI辅助应用、NuvaLang运行时、ML工作负载                    │
├─────────────────────────────────────────────────────────┤
│                    L3 服务层                             │
│  Brain AI服务(IPC)、模型注册表、AI性能分析器                │
├─────────────────────────────────────────────────────────┤
│                    L2 系统库层                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ NvScheduler│  │ NvBalancer│  │ModelManager│            │
│  │ (AI调度)   │  │ (硬件均衡) │  │(量化/融合) │            │
│  └─────┬─────┘  └─────┬────┘  └─────┬─────┘             │
│        │              │              │                    │
│  ┌─────┴──────────────┴──────────────┴─────┐            │
│  │           Brain AI 引擎                  │            │
│  │  推理引擎 │ 学习模块 │ 预测模块 │ NPU调度 │            │
│  └───────────────────────┬─────────────────┘            │
├──────────────────────────┼──────────────────────────────┤
│                    L1 内核层                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │NvScheduler│  │NvBalancer│  │NvPowerMgr│              │
│  │ 内核决策  │  │ 迁移执行  │  │(DVFS/    │              │
│  │          │  │          │  │ 温度管理) │              │
│  └──────────┘  └──────────┘  └──────────┘              │
│  三方协同：调度↔均衡↔功耗                                 │
├─────────────────────────────────────────────────────────┤
│                    L0 HAL层                             │
│  NPU HAL(达芬奇/Hexagon) │ GPU HAL │ DVFS硬件 │ 定时器   │
└─────────────────────────────────────────────────────────┘
```

## AI决策闭环

Nuva OS中每个AI驱动的子系统遵循相同的决策模式：

```
    ┌──────────┐
    │   感知   │ ← 收集真实指标（CPU利用率、延迟、温度）
    └────┬─────┘
         │
    ┌────▼─────┐
    │   推理   │ ← AI推理（NPU优先，CPU回退）
    └────┬─────┘
         │
    ┌────▼─────┐     置信度 < 50%
    │   执行   │ ─────────────────────┐
    └────┬─────┘                      │
         │                            ▼
    ┌────▼─────┐              ┌──────────────┐
    │   验证   │              │   三级回退    │
    └────┬─────┘              └──────┬───────┘
         │                           │
         ▼                           ▼
    成功/调整                1. AI推理（NPU）
                            2. 声明式策略引擎
                            3. CFS + RT传统调度
```

### 基于置信度的回退

| 置信度 | 动作 |
|--------|------|
| ≥ 80% | 直接执行AI决策 |
| 50%–80% | 执行并监控，异常时自动回退 |
| < 50% | 回退到声明式策略引擎 |
| 策略失败 | 回退到CFS + RT传统调度 |

## 核心AI子系统

### 1. NvScheduler — AI自主调度器

**位置**：`syslib/ai/scheduler.rs`、`kernel/sched/nvsched/`

NvScheduler使用8维特征向量进行自主调度决策：

| 维度 | 指标 |
|------|------|
| 0 | CPU利用率 |
| 1 | 运行队列长度 |
| 2 | I/O等待比 |
| 3 | 缓存缺失率 |
| 4 | 任务优先级分布 |
| 5 | 内存压力 |
| 6 | 温度状态 |
| 7 | NPU队列深度 |

**决策流程**：
1. `MetricsCollector`通过HAL回调收集真实硬件指标
2. `SchedFeatureVector`编码8维状态
3. AI推理通过NPU（或CPU回退）产生`AiSchedDecision`
4. `FallbackLevel`确定执行路径（AI→策略→CFS+RT）
5. `ActionExecutor`应用决策（任务迁移、优先级调整、CPU选择）
6. `ModelOptimizer`持续优化模型（量化、算子融合、死代码消除）

**调度类别**：`NvSchedClass`映射到内核调度策略：
- `NvRealtime` → SCHED_FIFO/SCHED_RR
- `NvInteractive` → 增强CFS（延迟奖励）
- `NvBatch` → CFS（吞吐量优化）
- `NvAiInference` → 专用NPU感知调度
- `NvEnergyAware` → EAS（功耗预算约束）

### 2. NvBalancer — AI自主负载均衡器

**位置**：`syslib/ai/optimizer.rs`、`kernel/sched/nvbalancer/`

NvBalancer自主平衡异构计算单元（big.LITTLE CPU、GPU、NPU）的工作负载：

- **指标收集**：通过HAL回调获取真实硬件时间戳
- **负载分析**：每设备利用率、温度状态、功耗效率
- **迁移执行**：实际任务迁移和设备功耗状态控制
- **震荡检测**：32项环形缓冲区防止抖动
- **热插拔支持**：动态设备增删处理

### 3. NvPowerMgr — AI自主功耗管理器

**位置**：`kernel/power_mgmt/nvpowermgr/`

NvPowerMgr在维持性能的同时自主优化功耗：

| 组件 | 功能 |
|------|------|
| `budget.rs` | 自主功耗预算分配 |
| `dvfs_controller.rs` | DVFS执行+安全切换序列+紧急节流 |
| `thermal.rs` | 温度墙管理（85°C主动/95°C紧急）、传感器故障保守回退 |
| `green_metrics.rs` | PUE、碳足迹、效率指标、低负载主动降频 |
| `optimization.rs` | 完整`run_cycle()`和`validate_safety()`实现 |

### 4. Brain AI引擎

**位置**：`syslib/brain/`

Brain AI引擎是统一的AI推理与学习平台：

| 模块 | 描述 |
|------|------|
| `inference/engine.rs` | 模型加载/卸载/推理闭环，NPU→CPU回退，`infer_async` |
| `npu/scheduler.rs` | 优先级队列NPU调度，设备选择，CPU回退 |
| `service/server.rs` | IPC服务注册（`register_service`），请求处理 |
| `operators/conv.rs` | ConvOps trait，NPU优先+CPU回退，深度可分离卷积 |
| `learning/` | 在线学习支持 |
| `prediction/` | 系统行为预测 |
| `scheduler/` | AI辅助调度决策 |

### 5. 模型管理器

**位置**：`syslib/ai/model_manager.rs`

提供自主模型优化：
- **INT8/FP16量化**：减小模型尺寸和推理延迟
- **算子融合**：合并连续算子减少内存访问
- **死代码消除**：移除未使用的计算路径

## 三方协同

NvScheduler、NvBalancer和NvPowerMgr协同运行，带运行时不变量验证：

```
NvScheduler ←──→ NvBalancer
     ↕                ↕
NvPowerMgr ←──→ (共享不变量)
```

| 协同关系 | 不变量 |
|----------|--------|
| 调度↔功耗 | 调度决策通过NvPowerMgr评估功耗影响 |
| 调度↔均衡 | NvScheduler驱动NvBalancer负载均衡 |
| 均衡↔功耗 | 均衡决策考虑设备功耗效率 |
| 功耗↔调度 | NvPowerMgr不休眠有活跃高优先级任务的设备 |

## HAL集成

### NPU HAL

**位置**：`hal/npu/`

两套互补接口：
- `traits.rs` — `NpuHal` trait：高层推理操作
- `device.rs` — `NpuDevice` trait + `ModelHandle`/`TensorHandle`：低层设备管理

支持的NPU硬件：
- 华为达芬奇（`hal/npu/davinci.rs`）
- 高通Hexagon DSP（`hal/npu/hexagon.rs`）

### HAL回调接口

**位置**：`hal/callback.rs`

HAL回调机制提供内核到HAL的函数指针接口：
- `page_alloc` — HAL页分配
- `time_ms` — 硬件时间戳
- `ai_wakeup_boost` — AI驱动唤醒优先级提升
- `ai_latency_pick` — AI驱动延迟敏感CPU选择

## 声明式策略引擎

**位置**：`kernel/sched/nv_policy.rs`

当AI置信度不足时，声明式策略引擎提供确定性回退：

| 策略字段 | 用途 |
|----------|------|
| `ai_confidence_threshold` | AI决策最低置信度（默认：50%） |
| `inference_budget_us` | AI推理最大时间预算 |
| `power_aware_enabled` | 启用功耗感知调度 |
| `balancer_driven` | 启用均衡器驱动负载分配 |

## POSIX兼容性

**位置**：`posix/`

POSIX兼容层将POSIX接口映射到Nuva AI原生原语：

| POSIX | Nuva原生 |
|-------|----------|
| `fork()` | `nv_process_spawn()`（COW） |
| `execve()` | `nv_process_execute()`（ELF验证） |
| `waitpid()` | `nv_event_wait()`（NvEvent） |
| `kill()` | `NuvaEvent::Interrupt`投递 |
| `sigaction()` | NvEvent处理器注册 |
| `pid_t` | `NuvaProcessId`桥接 |

## 错误处理

AI子系统错误使用短错误码：

| 错误码 | 描述 |
|--------|------|
| E005 | AI推理超时 |
| E006 | NPU不可用（触发CPU回退） |
| E007 | 模型加载失败 |
| E008 | 置信度低于阈值 |
| E009 | 功耗预算超限 |
| E010 | 温度节流激活 |

---

*最后更新：2026-06-26 | Nuva OS v1.0.0*