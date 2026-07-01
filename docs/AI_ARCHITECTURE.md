# Nuva OS AI Architecture

## Overview

Nuva OS is an AI-native operating system where artificial intelligence drives autonomous decision-making at every layer — from scheduling and load balancing to power management and inference. The AI subsystem follows a **Perceive → Reason → Execute → Verify** closed loop with a three-level fallback mechanism ensuring system stability even when AI confidence is low.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    L4 Application                        │
│  AI-assisted apps, NuvaLang runtime, ML workloads        │
├─────────────────────────────────────────────────────────┤
│                    L3 Services                           │
│  Brain AI Service (IPC), Model Registry, AI Profiler     │
├─────────────────────────────────────────────────────────┤
│                    L2 Syslib                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ NvScheduler│  │ NvBalancer│  │ModelManager│            │
│  │ (AI Sched) │  │ (HW Bal) │  │(Quant/Fuse)│            │
│  └─────┬─────┘  └─────┬────┘  └─────┬─────┘             │
│        │              │              │                    │
│  ┌─────┴──────────────┴──────────────┴─────┐            │
│  │           Brain AI Engine                │            │
│  │  Inference │ Learning │ Prediction │ NPU │            │
│  │  Engine    │ Module   │ Module     │ Sched│            │
│  └───────────────────────┬─────────────────┘            │
├──────────────────────────┼──────────────────────────────┤
│                    L1 Kernel                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │NvScheduler│  │NvBalancer│  │NvPowerMgr│              │
│  │ Kernel    │  │ Kernel   │  │(DVFS/    │              │
│  │ Decision  │  │ Migration│  │ Thermal) │              │
│  └──────────┘  └──────────┘  └──────────┘              │
│  Three-Party Cooperation: Sched↔Balancer↔Power           │
├─────────────────────────────────────────────────────────┤
│                    L0 HAL                                │
│  NPU HAL (Da Vinci/Hexagon) │ GPU HAL │ DVFS HW │ Timer │
└─────────────────────────────────────────────────────────┘
```

## AI Decision Closed Loop

Every AI-driven subsystem in Nuva OS follows the same decision pattern:

```
    ┌──────────┐
    │ Perceive │ ← Collect real metrics (CPU util, latency, temperature)
    └────┬─────┘
         │
    ┌────▼─────┐
    │  Reason  │ ← AI inference (NPU preferred, CPU fallback)
    └────┬─────┘
         │
    ┌────▼─────┐     confidence < 50%
    │  Execute │ ─────────────────────┐
    └────┬─────┘                      │
         │                            ▼
    ┌────▼─────┐              ┌──────────────┐
    │  Verify  │              │ Three-Level  │
    └────┬─────┘              │  Fallback    │
         │                    └──────┬───────┘
         │                           │
         ▼                           ▼
    Success/Adjust          1. AI Inference (NPU)
                            2. Declarative Policy
                            3. CFS + RT Traditional
```

### Confidence-Based Fallback

| Confidence | Action |
|------------|--------|
| ≥ 80% | Execute AI decision directly |
| 50%–80% | Execute with monitoring, auto-revert on anomaly |
| < 50% | Fall back to declarative policy engine |
| Policy fails | Fall back to CFS + RT traditional scheduling |

## Core AI Subsystems

### 1. NvScheduler — AI Autonomous Scheduler

**Location**: `syslib/ai/scheduler.rs`, `kernel/sched/nvsched/`

The NvScheduler makes autonomous scheduling decisions using an 8-dimensional feature vector:

| Dimension | Metric |
|-----------|--------|
| 0 | CPU utilization |
| 1 | Run queue length |
| 2 | I/O wait ratio |
| 3 | Cache miss rate |
| 4 | Task priority distribution |
| 5 | Memory pressure |
| 6 | Thermal status |
| 7 | NPU queue depth |

**Decision Flow**:
1. `MetricsCollector` gathers real hardware metrics via HAL callbacks
2. `SchedFeatureVector` encodes the 8-dimensional state
3. AI inference via NPU (or CPU fallback) produces `AiSchedDecision`
4. `FallbackLevel` determines execution path (AI → Policy → CFS+RT)
5. `ActionExecutor` applies the decision (task migration, priority adjustment, CPU selection)
6. `ModelOptimizer` continuously improves the model (quantization, operator fusion, dead code elimination)

**Scheduling Classes**: `NvSchedClass` maps to kernel scheduling policies:
- `NvRealtime` → SCHED_FIFO/SCHED_RR
- `NvInteractive` → Enhanced CFS with latency bonus
- `NvBatch` → CFS with throughput optimization
- `NvAiInference` → Dedicated NPU-aware scheduling
- `NvEnergyAware` → EAS with power budget constraint

### 2. NvBalancer — AI Autonomous Load Balancer

**Location**: `syslib/ai/optimizer.rs`, `kernel/sched/nvbalancer/`

NvBalancer autonomously balances workloads across heterogeneous compute units (big.LITTLE CPUs, GPU, NPU):

- **Metrics Collection**: Real hardware timestamps via HAL callback
- **Load Analysis**: Per-device utilization, thermal state, power efficiency
- **Migration Execution**: Actual task migration between CPU clusters and device power state control
- **Oscillation Detection**: 32-entry ring buffer prevents thrashing
- **Hot-Plug Support**: Dynamic device addition/removal handling

### 3. NvPowerMgr — AI Autonomous Power Manager

**Location**: `kernel/power_mgmt/nvpowermgr/`

NvPowerMgr autonomously optimizes power consumption while maintaining performance:

| Component | Function |
|-----------|----------|
| `budget.rs` | Autonomous power budget allocation across devices |
| `dvfs_controller.rs` | DVFS execution with safe switching sequences, emergency throttling |
| `thermal.rs` | Thermal wall management (85°C proactive, 95°C critical), sensor failure conservative fallback |
| `green_metrics.rs` | PUE, carbon footprint, efficiency metrics, proactive frequency reduction under low load |
| `optimization.rs` | Complete `run_cycle()` and `validate_safety()` implementation |

### 4. Brain AI Engine

**Location**: `syslib/brain/`

The Brain AI Engine is the unified AI inference and learning platform:

| Module | Description |
|--------|-------------|
| `inference/engine.rs` | Model load/unload/infer closed loop, NPU→CPU fallback, `infer_async` |
| `npu/scheduler.rs` | Priority queue NPU scheduling, device selection, CPU fallback |
| `service/server.rs` | IPC service registration (`register_service`), request handling |
| `operators/conv.rs` | ConvOps trait, NPU-priority + CPU fallback, depthwise separable convolution |
| `learning/` | Online learning support |
| `prediction/` | System behavior prediction |
| `scheduler/` | AI-assisted scheduling decisions |

### 5. Model Manager

**Location**: `syslib/ai/model_manager.rs`

Provides autonomous model optimization:
- **INT8/FP16 Quantization**: Reduce model size and inference latency
- **Operator Fusion**: Merge consecutive operators for fewer memory accesses
- **Dead Code Elimination**: Remove unused computation paths

## Three-Party Cooperation

NvScheduler, NvBalancer, and NvPowerMgr operate cooperatively with runtime invariant verification:

```
NvScheduler ←──→ NvBalancer
     ↕                ↕
NvPowerMgr ←──→ (shared invariants)
```

| Cooperation | Invariant |
|-------------|-----------|
| Sched ↔ Power | Scheduling decisions evaluate power impact via NvPowerMgr |
| Sched ↔ Balancer | NvScheduler drives NvBalancer load balancing |
| Balancer ↔ Power | Balance decisions consider device power efficiency |
| Power ↔ Sched | NvPowerMgr never sleeps devices with active high-priority tasks |

## HAL Integration

### NPU HAL

**Location**: `hal/npu/`

Two complementary interfaces:
- `traits.rs` — `NpuHal` trait: high-level inference operations
- `device.rs` — `NpuDevice` trait + `ModelHandle`/`TensorHandle`: low-level device management

Supported NPU hardware:
- Huawei Da Vinci (`hal/npu/davinci.rs`)
- Qualcomm Hexagon DSP (`hal/npu/hexagon.rs`)

### HAL Callback Interface

**Location**: `hal/callback.rs`

The HAL callback mechanism provides kernel-to-HAL function pointer interfaces:
- `page_alloc` — Page allocation from HAL
- `time_ms` — Hardware timestamp
- `ai_wakeup_boost` — AI-driven wakeup priority boost
- `ai_latency_pick` — AI-driven latency-sensitive CPU selection

## Declarative Policy Engine

**Location**: `kernel/sched/nv_policy.rs`

When AI confidence is insufficient, the declarative policy engine provides deterministic fallback:

| Policy Field | Purpose |
|-------------|---------|
| `ai_confidence_threshold` | Minimum confidence for AI decisions (default: 50%) |
| `inference_budget_us` | Maximum time budget for AI inference |
| `power_aware_enabled` | Enable power-aware scheduling |
| `balancer_driven` | Enable balancer-driven load distribution |

## POSIX Compatibility

**Location**: `posix/`

The POSIX compatibility layer maps POSIX interfaces to Nuva AI-native primitives:

| POSIX | Nuva Native |
|-------|-------------|
| `fork()` | `nv_process_spawn()` with COW |
| `execve()` | `nv_process_execute()` with ELF validation |
| `waitpid()` | `nv_event_wait()` with NvEvent |
| `kill()` | `NuvaEvent::Interrupt` delivery |
| `sigaction()` | NvEvent handler registration |
| `pid_t` | `NuvaProcessId` bridge |

## Error Handling

AI subsystem errors use short error codes:

| Code | Description |
|------|-------------|
| E005 | AI inference timeout |
| E006 | NPU unavailable (CPU fallback triggered) |
| E007 | Model load failure |
| E008 | Confidence below threshold |
| E009 | Power budget exceeded |
| E010 | Thermal throttle active |

---

*Last updated: 2026-06-26 | Nuva OS v1.0.0*