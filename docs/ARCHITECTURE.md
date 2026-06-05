# Nuva OS System Architecture

## Overview

Nuva OS adopts a **three-level microkernel architecture** with minimal kernel functionality, equipment-mode system services, user-space applications, IPC as the primary communication mechanism, and fault isolation. The system employs a five-layer architecture design (L0-L4) with three privilege levels (EL2/EL1/EL0), supporting ARM64, x86-64, LoongArch64, and RISC-V 64-bit (RV64G) processor architectures, with integrated quantum-safe cryptography and a plugin system.

**Design Philosophy**: nuva is not unix, nuva is not linux. Nuva OS uses its own native type system, system call interface, and capability-based security model. POSIX compatibility is provided as an optional module (feature flag `posix`), not as a core kernel path.

## Three-Level Privilege Architecture

Nuva OS introduces a **three-level privilege architecture** (EL2/EL1/EL0), superior to traditional two-level (kernel/user) designs:

| Level | Name | Components | Hardware Mapping |
|-------|------|------------|-----------------|
| EL2 | Minimal Kernel Mode | Scheduler, IPC, Memory Mgmt, Capability Mgr, IRQ, Timer | ARM64: EL2 / x64: Ring 0 / RISC-V: M-mode / LA64: PLV0 |
| EL1 | Equipment Mode | Filesystem, Network Stack, Device Drivers, Display Server | ARM64: EL1 / x64: Ring 1 / RISC-V: S-mode / LA64: PLV1 |
| EL0 | User Mode | Applications, User Libraries | ARM64: EL0 / x64: Ring 3 / RISC-V: U-mode / LA64: PLV3 |

### Cross-Level Access Rules

- **EL1→EL2**: Only through `NvSupervisorCall` (capability-gated, 14 operations)
- **EL1↔EL0**: Only through NvIPC port message passing
- **EL2→EL1/EL0**: Only through NvIPC port (kernel-mediated delivery)
- **Direct cross-level memory access**: Always denied (`CrossLevelAccessDenied`)

### Equipment Mode Fault Isolation

- Each EL1 service runs in independent `NvEquipmentFaultDomain`
- Service crash **never** affects kernel mode or other services
- Dual-mechanism detection: DeadName (instant) + heartbeat (periodic)
- 7-step automatic recovery: check oscillation → mark restarting → isolate → restart → rebind → rebuild → notify
- Formal invariant: `∀s ∈ EquipmentServices: crash(s) → healthy(KernelMode)`

## Nuva Native System

### Native Type System

Nuva OS defines its own type system in `sysroot/include/nuva/types.h` and `kernel/types.rs`, replacing POSIX/Unix type semantics:

| Nuva Native Type | Replaces | Description |
|-----------------|----------|-------------|
| `nuva_process_id_t` / `NuvaProcessId` | `pid_t` | 64-bit capability-based process ID |
| `nuva_thread_id_t` / `NuvaThreadId` | `tid_t` | 64-bit thread ID |
| `nuva_capability_id_t` / `NuvaCapabilityId` | `uid_t`/`gid_t` | Capability token ID |
| `nuva_file_handle_t` / `NuvaFileHandle` | `fd_t` | 64-bit file handle |
| `nuva_file_offset_t` / `NuvaFileOffset` | `off_t` | 64-bit file offset |
| `nuva_inode_id_t` / `NuvaInodeId` | `ino_t` | 64-bit inode ID |
| `NuvaAccessRight` | `mode_t` | Bitflags for access rights (READ/WRITE/EXECUTE/GRANT/REVOKE/etc.) |
| `NuvaError` | `errno` (i32) | Typed error enum (CapabilityDenied/CapabilityExpired/etc.) |
| `NuvaEvent` | POSIX signal | Native event notification (Interrupt/TimerExpired/IoComplete/etc.) |
| `NuvaDiagnostic` | `/proc`/`/sys` | Native diagnostic query interface |
| `NvPrivilegeLevel` | N/A | Three-level privilege: UserMode(0)/EquipmentMode(1)/KernelMode(2) |
| `NvSupervisorOp` | N/A | EL1→EL2 controlled operations (14 types: MapDeviceMemory, DmaMap, IrqRequest, etc.) |
| `NvAddressSpaceId` | N/A | Independent fault domain address space identifier |
| `NvServiceName` | N/A | Equipment mode service name identifier |

### Native System Call Interface

Nuva native system calls occupy number space `0x0000_0000 - 0x0000_FFFF`, separate from POSIX calls (`0x0001_0000 - 0x0001_FFFF`):

| Category | Call Numbers | Key Interfaces |
|----------|-------------|----------------|
| Process | 0x01-0x0F | `NUVA_PROCESS_CREATE/EXECUTE/TERMINATE/YIELD` |
| Memory | 0x10-0x1F | `NUVA_MEMORY_ALLOCATE/DEALLOCATE/PROTECT/MAP` |
| IPC | 0x20-0x2F | `NUVA_IPC_PORT_CREATE/DESTROY/SEND/RECEIVE/CALL/REPLY/FORWARD` |
| File | 0x30-0x3F | `NUVA_FILE_OPEN/CLOSE/READ/WRITE/SEEK/IOCTL` |
| Capability | 0x40-0x4F | `NUVA_CAPABILITY_GRANT/REVOKE/CHECK/TRANSFER` |
| Event | 0x50-0x5F | `NUVA_EVENT_REGISTER/NOTIFY/WAIT` |
| Diagnostic | 0x60-0x6F | `NUVA_DIAG_QUERY/STATS` |

All native system calls require `NuvaCapability` token verification.

### Native Security Model

The `NuvaSecurityHook` trait replaces the Linux LSM imitation pattern:

| Aspect | Old (LSM-style) | New (Nuva native) |
|--------|-----------------|-------------------|
| Hook signature | `(*mut c_void, u32) -> i32` | `(NuvaCapabilityId, NuvaResourceHandle, NuvaAccessRight) -> Result<(), NuvaError>` |
| Module priority | `u32` stacking | `NuvaSecurityPolicy` with `NuvaPolicyPriority` enum |
| Permission check | `mode_t` bit ops | `NuvaCapability::check()` |
| Return type | `i32` errno | `Result<(), NuvaError>` |

### POSIX Optional Compatibility Module

POSIX support is **optional** and **not required**. It is controlled by the `posix` feature flag:

- **Default build** (`cargo build`): No POSIX support, smaller kernel
- **POSIX build** (`cargo build --features posix`): POSIX adapters bridge to Nuva native interfaces
- **Kernel core path isolation**: Kernel core modules (ipc/mm/security/process/core/sched/fs) must not import POSIX/Unix types or interfaces

### Vulkan Native GPU Integration

Nuva OS integrates Vulkan as its native GPU/compute API with zero-copy direct passthrough:

- **Architecture**: Kernel directly exposes Vulkan-capable GPU devices (no HAL intermediate layer)
- **Superior to Android**: Eliminates Gralloc+HAL chain; single kernel syscall path
- **Superior to Apple**: Uses open Vulkan standard instead of proprietary Metal
- **Security**: NvGpuCapability-based GPU access with fine-grained permissions (Compute/Render/Memory/Present/Video)
- **Memory**: Zero-copy CPU-GPU shared pages (HOST_VISIBLE+HOST_COHERENT)
- **Feature flag**: `vulkan` (optional, default off)

---

## System Architecture

### Overall Architecture (Five Layers L0-L4)

```
+------------------+
|   Applications   |
+------------------+
         |
+------------------+
| App Framework    |
| (UI/Window/Event)|
+------------------+  L4 - Application Framework Layer
         |
+------------------+
| System Services  |
| (Power/Net/IPC)  |
+------------------+  L3 - System Services Layer
         |
+------------------+
|   Syslib         |
| (Core/Brain/ML)  |
+------------------+  L2 - System Libraries Layer
         |
+------------------+
|  Kernel (Micro)  |
| (Sched/MM/IPC)   |
+------------------+  L1 - Kernel Layer
         |
+------------------+
|   HAL (Hardware  |
|   Abstraction)   |
+------------------+  L0 - Hardware Abstraction Layer
         |
+------------------+
|    Hardware      |
+------------------+
```

#### L0 - Hardware Abstraction Layer (HAL)

HAL provides unified hardware access interfaces, abstracting underlying hardware differences:

| Submodule | Description |
|-----------|-------------|
| `cpu` | CPU HAL: frequency/voltage/temperature/idle state management, supporting Kirin (PSCI SMC), Loongson, DVFS, thermal management |
| `gpu` | GPU HAL: frame management, command queue, supporting Maleoon GPU |
| `npu` | NPU HAL: inference engine, model management, AI scheduler with kernel integration (notify_kernel_scheduler/select_cpu_for_task), performance predictor, DaVinci NPU (DAVINCI_NPU_OPS bridging real HAL), Hexagon DSP, ONNX runtime |
| `power` | Power HAL: PMIC, suspend/resume, cross-architecture C-state (MWAIT/WFI/idle), ACPI power driver (Fadt, S3/S5) |
| `input` | Input device HAL |
| `quantum` | Quantum technology HAL: QRNG (Quantum Random Number Generator), PQC (Post-Quantum Cryptography) |
| `ffi` | C/C++ FFI interface: API stability checker, ABI compatibility validation |
| `platform` | Platform detection and identification (PlatformInfo, BootInfoType: Fdt/Acpi/Multiboot2/LoongArchFw) |

Architecture-specific HAL implementations:
- `arm64/` — ARM64 architecture (CPU, MMU, interrupt controller GIC, timer, FDT boot)
- `x64/` — x86-64 architecture (CPU, APIC (LAPIC/I/O APIC), IDT, GDT, MMU, Timer (LAPIC Timer + TSC), Power (S3/S5/MWAIT), PageTable (destroy/protect))
- `loongarch64/` — LoongArch64 architecture (CPU, MMU (3-level page tables, PageTableOps), EIOINTC interrupt controller, UEFI boot, LSX 128-bit SIMD, LASX 256-bit SIMD, LVZ virtualization, LBT binary translation)
- `riscv64/` — RISC-V 64 architecture (CPU, MMU, interrupt controller PLIC, SBI, timer)
- `snapdragon/` — Snapdragon 8 Gen 4 SoC (CPU, GPU, NPU)

#### L1 - Kernel Layer

Microkernel providing minimal core functionality:

| Component | Description |
|-----------|-------------|
| Unified Error Type | `KernelError` enum covering 7 error categories (memory/sched/IPC/driver/fs/sync/security), `KernelResult<T>` alias, POSIX errno mapping, recoverable/user-error classification, **extended variants** (DeadlockDetected, InvalidState, WouldBlock, Timeout, Busy, QuotaExceeded) |
| Scheduler | **NvScheduler** AI intelligent scheduler (NPU inference, four-level AI scheduling classes, three-tier fallback), CFS/RT/Deadline/Idle/EAS scheduling, **NvBalancer** heterogeneous hardware load balancer, scheduling domains, **declarative policy configuration** (`SchedPolicyConfig` with hot-update) |
| Power Management | **NvPowerMgr** AI-driven power optimization (power budget, DVFS, thermal monitor, green metrics), ACPI driver (Fadt, S3/S5), PM subsystem |
| Memory Management | Buddy+SLAB allocators, VMA, page fault handling, NUMA, hotplug, mmap/munmap/mprotect/msync, **Per-CPU page cache (PCP) with watermarks**, **SLAB cache-line alignment (64B)** |
| IPC | NuvaIPC (Mach-style ports), shared memory, pipes, semaphores, message queues, **zero-copy fast path (<=256B register path)** |
| Interrupt Handling | Hardware interrupts, exceptions, GIC/APIC/EIOINTC/PLIC auto-detection |
| Process Management | Process lifecycle, signal handling, resource limits |
| Device Management | Device driver framework, device classes, **declarative driver model** (`DeclarativeDriver` trait, `declare_driver!` macro), **declarative power management** (`PmStateMachine`, `declare_pm!` macro) |
| Synchronization | SpinLock with preemption control and holder tracking, Mutex, Semaphore, RwLock, **PreemptCount** (preempt_disable/enable, allocation constraint check), **RwLock TOCTOU fix** (atomic version check before write), **RCU** (Read-Copy-Update for read-heavy paths), **Per-CPU variables** (cache-line aligned, lock-free access) |
| Network Stack | TCP/IP stack (full TCP state machine RFC 793, 11 states, 3-way handshake, retransmit/keepalive/timewait timers), UDP, socket API, firewall (stateless rules, NAT, rate limiting), NFSv3 client, SMB2/3 client (organized in `kernel/net_stack/`) |
| File System | VFS, NuvaFS, ext4, FAT32, **io_uring** (zero-copy async I/O with ring buffers), page cache, dentry cache, buffer cache |
| Timer | Kernel timer subsystem, tick/no-tick modes |
| Plugin System | ELF loader with RELA relocation, registry, sandbox with resource limits, SHA-256 fingerprint, audit with real timestamps, PluginServices kernel interface |
| Security | ASLR, stack canary, sandbox, defense system, virus scanner |
| Debug/Perf | Kernel debugger, performance monitoring, performance tuning |
| Power Management | ACPI driver (Fadt, S3/S5), PM subsystem |
| Tombstone | Crash record capture, storage, and query; crash context collection via HAL, stack backtrace, deduplication, atomic file writes, memory cache fallback |
| Quantum | QuantumManager, QuantumRng, QKD sessions, PQC context |
| Platform Detection | PlatformInfo, BootInfoType, detect_platform_info() |
| Boot Flow | ARM64 FDT + exception vectors, x64 Multiboot2 + GDT/IDT/exceptions, LoongArch64 UEFI boot, RISC-V 64 SBI boot |
| Architecture Support | ARM64 (`kernel/arch/arm64/`), x86-64 (`kernel/arch/x64/`), LoongArch64 (`kernel/arch/loongarch64/`), RISC-V 64 (`kernel/arch/riscv64/`: boot/SBI, trap, MMU, PLIC, timer, context) |

##### Kernel Functional Domain Subdirectories

The kernel has been reorganized into functional domain subdirectories for improved modularity:

| Subdirectory | Description | Key Re-exports |
|-------------|-------------|----------------|
| `kernel/init/` | Initialization subsystem | cmdline, config, elf, platform, resource |
| `kernel/diag/` | Diagnostics subsystem | journal, kdebug, log, scanner, stats |
| `kernel/irq_mgmt/` | IRQ management | apic_ops, irq, trap |
| `kernel/net_stack/` | Network stack | socket, tcpip |
| `kernel/storage/` | Storage subsystem | block |
| `kernel/device/` | Device model and plugins | device_model, driver_plugin, feature_plugin, module, notifier |
| `kernel/power_mgmt/` | Power management | hotplug, pm, power, **nvpowermgr** |
| `kernel/virt/` | Virtualization subsystem | vmx |
| `kernel/core/` | Core kernel services | cache, cpu, defense, kernel_thread, mempool, perf_tune, posix, random, signal, time, wait, workqueue |

#### L2 - System Libraries Layer (Syslib)

See [Syslib System Libraries Layer](#syslib-system-libraries-layer-l2) section below.

#### L3 - System Services Layer

| Service | Description |
|---------|-------------|
| Power Management | Power states: Active, Idle, Suspend, Off; Sleep modes: Freeze, Standby, Suspend-to-RAM, Hibernate; Wake locks |
| Security Service | Capability-based permission model (CapSet), key management (Keymaster), user authentication (Gatekeeper), TEE client |
| Network Service | TCP/IP protocol stack, DNS resolution, network interface management, Socket API |
| IPC Service | NuvaIPC service (Mach-style port messaging), shared memory management |
| App Service | Declarative screen lifecycle management (four-state model), package management (NPK format) |
| Form Factor | Device form factor detection and management (phone/tablet/TV/watch/car) |

#### L4 - Application Framework Layer

The L4 layer is fully declarative — all UI, window, event, render, and resource
management uses a Nuva-native declarative paradigm with no legacy View/Activity/Widget code.

| Module | Description |
|--------|-------------|
| `application/ui` | Declarative UI: Screen system, Component model, State\<T\>, Modifier chain, Render pipeline, O(n) Reconciler, Adaptive layout |
| `application/window` | Declarative window management: screen-lifecycle-driven windows, DeclarativeSurface |
| `application/event` | Declarative event system: Modifier-bound event handling, bubbling dispatch |
| `application/render` | Declarative compositor: VSync-aligned frame presentation |
| `application/resource` | Declarative resource management: Resource\<T\> auto-UI-update, cache |

| Component | Description |
|-----------|-------------|
| Screen System | Declarative screen lifecycle (Screen trait), ScreenLifecycleManager |
| Component Model | 9 built-in components (Text/Column/Row/Stack/Button/Image/ScrollView/Spacer/SizedBox), Component trait |
| State Binding | Reactive State\<T\> with atomic version + dirty marking |
| Modifier Chain | Zero-cost chainable modifiers (layout/event/window/resource) |
| Render Pipeline | Reconcile→Layout→Paint→Composite, O(n) diff, AdaptiveLayoutEngine integration |
| Window Management | Declarative windows, Z-order, screen-lifecycle-driven visibility |
| Event System | Declarative events, Modifier-bound handling, bubbling dispatch |
| Rendering Engine | Declarative compositor, VSync-aligned presentation |

---

### Kernel Design

#### Microkernel Principles

- Minimal kernel functionality
- Services run in user space
- IPC as the primary communication mechanism
- Fault isolation

#### Kernel Components

1. **Scheduler**: CFS/RT/Deadline/Idle/EAS process/thread scheduling
2. **Memory Management**: Virtual memory, physical memory, Buddy+SLAB allocators
3. **Interrupt Handling**: Hardware interrupts, exceptions
4. **IPC**: Inter-process communication (Binder, shared memory, pipes, etc.)
5. **Process Management**: Process lifecycle, signals, resource limits
6. **Device Management**: Device driver framework, driver/feature plugin system
7. **Network Stack**: TCP/IP, socket API, NFSv3, SMB2/3
8. **File System**: VFS, NuvaFS, ext4, FAT32, io_uring
9. **Timer**: Kernel timer subsystem
10. **Plugin System**: ELF loader, registry, sandbox with resource limits, SHA-256 fingerprint, audit
11. **Security**: Defense system, virus scanner, sandbox

---

## Memory Management

Core memory management features (for details, see [MEMORY.md](MEMORY.md)):

- **Physical Memory**: Buddy+SLAB two-level allocators, Per-CPU page cache, memory zones (DMA/Normal/HighMem)
- **Virtual Memory**: 4-level page tables (ARM64/x64/LoongArch64), Sv39/Sv48 page tables (RISC-V 64), VMA, mmap, COW, huge pages (2MB/1GB)
- **Advanced**: NUMA support, memory hotplug, page migration, OOM killer with comprehensive scoring (memory usage, CPU time, nice value, swap usage, oom_score_adj), memory compaction

### Memory Layout

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

#### RISC-V 64

```
0x0000_0000_0000_0000 - 0x0000_3FFF_FFFF_FFFF : User space (256GB, Sv39)
0xFFFF_C000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF : Kernel space (256GB, Sv39)
```

---

## Process Scheduling

Core scheduling features (for details, see [PROCESS.md](PROCESS.md)):

### Scheduling Classes (Priority Order)

1. **Deadline** — Earliest Deadline First (EDF), highest priority
2. **RT** — FIFO/RR, priority 0-99
3. **CFS** — Red-black tree, vruntime-based fair scheduling
4. **EAS** — Energy-aware CPU selection for big.LITTLE systems
5. **Idle** — Lowest priority, idle task

### NvScheduler AI Intelligent Scheduler

NvScheduler extends the traditional scheduling framework with AI-driven intelligent scheduling:

- **NPU Inference Engine**: Submits 12-dimensional `SchedFeatureVector` to Da Vinci NPU for scheduling decisions
- **Four-Level AI Scheduling Classes**: `AI_REALTIME` (NPU→Big, max boost 0-5) > `AI_NORMAL` (Big→NPU, boost 1-3) > `AI_BATCH` (Little, throughput) > `AI_IDLE` (Little, energy)
- **Three-Tier Fallback**: AI inference → Declarative policy → CFS+RT traditional
- **AI Task Classifier**: Automatic task classification based on compute ratio, NPU access, and memory usage
- **Declarative Policy Engine**: Enhanced `SchedPolicyConfig` with `ai_confidence_threshold`, `inference_budget_us`, `power_aware_enabled`, `balancer_driven` fields
- **Confidence Threshold**: AI decisions with confidence ≥ 50% are used; below threshold triggers fallback

### NvBalancer Heterogeneous Hardware Balancer

NvBalancer distributes workloads across GPU (RTX Spark), NPU (Da Vinci), CPU, and Quantum devices:

- **Device Topology**: `HeteroDeviceTopology` with NUMA mapping, PCIe bandwidth matrix, interconnect latency matrix, generation-based hot-plug tracking
- **Load Collection**: Per-device real-time metrics (utilization, queue depth, temperature, power, data locality) with degraded fallback on timeout
- **Balance Optimizer**: Task-device matching + data locality + power efficiency scoring; triggers when load deviation > 30%
- **Migration Executor**: Checkpoint-save → pause → migrate → resume sequence; overhead limited to ≤ 15% of task execution time
- **Oscillation Detector**: 32-entry ring buffer detects tasks bouncing between devices (≥ 3 times triggers suppression)
- **Hot-Plug Support**: Device add/remove with generation counter, running tasks not disrupted

### NvPowerMgr AI-Driven Power Optimization

NvPowerMgr provides comprehensive power management with AI-driven optimization:

- **Power Budget Manager**: System power budget with 5% overshoot allowance; infeasible budget → minimum power mode
- **DVFS Controller**: Per-device DVFS with safe switching sequences (scale up: voltage→frequency; scale down: frequency→voltage)
- **Device Power Controller**: Per-device independent power control; critical devices never sleep
- **Thermal Monitor**: Per-device temperature monitoring; proactive throttling at 85°C; sensor failure → conservative policy
- **Green Metrics Collector**: Real-time PUE (Power Usage Effectiveness), carbon emission equivalent, power efficiency score
- **AI Power Optimizer**: NPU-based power optimization model generating DVFS + sleep + throttle plans; performance impact ≤ 10%, energy reduction ≥ 15%
- **Fallback**: NPU unavailable → heuristic DVFS lookup + temperature thresholds; PMIC failure → maintain current state

### Three-Party Cooperation (NvScheduler ↔ NvBalancer ↔ NvPowerMgr)

- **Scheduling ↔ Power**: NvScheduler evaluates power impact of decisions via NvPowerMgr, selects most power-efficient option
- **Scheduling ↔ Balance**: NvScheduler drives NvBalancer; balance triggered by AI inference or declarative policy (not fixed thresholds)
- **Balance ↔ Power**: NvBalancer queries NvPowerMgr device power state, prefers power-efficient devices
- **Power ↔ Scheduling**: NvPowerMgr never sleeps devices with active high-priority tasks
- **Invariant Verification**: Runtime checks ensure: (1) scheduling considers power, (2) power considers scheduling, (3) balance is scheduler-driven, (4) AI fallback: perf degradation ≤ 10% and energy reduction ≥ 15%

---

## File System

Core file system features (for details, see [FILESYSTEM.md](FILESYSTEM.md)):

- **VFS**: Unified abstraction layer with FileSystem/InodeOps/FileOps traits
- **NuvaFS**: Log-structured, COW, snapshots, ZSTD/LZ4 compression, deduplication
- **ext4**: Journal mode support (journal/ordered/writeback), extents
- **FAT32**: VFAT LFN support, 4GB-1 max file size
- **NFSv3**: Client implementation with TCP/UDP socket transport, RPC send/receive/retransmit, XDR decode
- **SMB2/3**: Client implementation with TCP transport, Direct TCP packet framing
- **io_uring**: Zero-copy async IO with ring buffers

---

## IPC (Inter-Process Communication)

- **NuvaIPC**: Mach-style port messaging, send/receive rights, zero-copy transfer, sync/async calls
- **Shared Memory**: Anonymous and named, memory barriers
- **Other**: Pipes, semaphores, message queues, signals

---

## System Services

- **Power Management**: Multiple sleep states (Freeze/Standby/Suspend-to-RAM/Hibernate), wake locks
- **Security Service**: Permission management, Keymaster, Gatekeeper, TEE client
- **Network Service**: TCP/IP stack, DNS resolution, network interface management

---

## AI Engine (Nuva Brain)

### Architecture

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

### Inference Flow

1. Load model
2. Create inference context
3. Prepare input tensors
4. Execute inference
5. Get output tensors

### NPU Scheduling

- Priority queue
- Batch processing
- Memory pool
- Dynamic frequency adjustment
- Performance predictor
- AI scheduler (`AiScheduler`) with kernel integration
- `notify_kernel_scheduler()` — notify kernel of AI task scheduling decisions
- `select_cpu_for_task()` — AI-driven CPU selection for compute tasks
- Da Vinci NPU ops (`DAVINCI_NPU_OPS`) bridging real HAL implementation

---

## Nuva Language

### Compilation Flow

```
Source Code (.nv) -> Lexer -> Parser -> Semantic Analysis -> IR Generation -> Optimization -> Code Generation
```

#### Compiler Pipeline (Fully Implemented)

| Stage | Status | Details |
|-------|--------|---------|
| **Lexer** | Complete | String/character/number/identifier reading, multi-radix support (0b/0o/0x), declarative keywords (component, signal, effect, async, resource, with) |
| **Parser** | Complete | Pratt priority parsing, declarative syntax parsing for component/signal/effect/async/resource/with constructs |
| **Precedence Table** | Complete | Arithmetic operator precedence corrected, exponentiation (`^`) right-associative |
| **Semantic Analysis** | Complete | Type checking, type inference, purity verification, declarative constraint validation |
| **Code Generation** | Complete | Pipeline/comprehension IR generation, async state machine IR, reactive IR |
| **IR Optimization** | Complete | Constant folding, DCE (Dead Code Elimination), CSE (Common Subexpression Elimination), copy propagation, loop optimization, inlining |

### Runtime (Fully Implemented)

| Component | Status | Details |
|-----------|--------|---------|
| **Garbage Collection** | Complete | Mark-sweep GC with root scanning and sweep phase |
| **Virtual Machine** | Complete | 256-register VM with instruction dispatch and execution loop |
| **Reactive Scheduler** | Complete | Effect scheduling, dependency tracking, propagation |
| **Binary Module** | Complete | NEX format loading, relocation, native code generation |
| **HashMap** | Complete | SipHash hashing, chaining collision resolution, rehash with capacity growth |

### Declarative Constructs

Nuva provides first-class declarative constructs as language keywords:

| Construct | Keyword | Purpose |
|-----------|---------|---------|
| Component | `component` | Declarative UI component definition |
| Reactive Signal | `signal` | Reactive state binding with automatic propagation |
| Side Effect | `effect` | Effect registration with dependency tracking |
| Async Computation | `async`/`await` | Declarative asynchronous computation |
| Resource | `resource` | Declarative resource acquisition with auto-cleanup |
| Context Manager | `with` | Scoped resource management (RAII-style) |

### Special Types

| Type | Description |
|------|-------------|
| `Reactive<T>` | Reactive wrapper that automatically propagates changes to dependent effects |
| `Future<T>` | Asynchronous computation result, resolved via await |
| `Resource<T>` | Managed resource with automatic acquisition and release semantics |

### Standard Library

- Collections: Vec, String, HashMap, LinkedList
- IO: Stdin, Stdout, Stderr, File
- Math: trigonometric, exponential, logarithmic
- Reactive: Reactive<T>, effect, signal
- Async: Future<T>, spawn, await
- Resource: Resource<T>, with

---

## Plugin System Architecture

The Nuva OS kernel includes a dynamic plugin system that supports runtime loading and unloading of functional modules:

### Core Components

| Component | Description |
|-----------|-------------|
| `PluginLoader` | ELF binary loader with memory mapping, RELA relocation (x86-64/AARCH64/LoongArch64), VFS file reading, ElfPlugin instantiation |
| `PluginRegistry` | Plugin registry maintaining metadata, name index, type index, and dependency graph |
| `PluginSandbox` | Sandbox isolation with resource limit checks, MemoryPool real allocation, IPC channel limits, device access control |
| `PluginManager` | Lifecycle management with failure count tracking, load time tracking, dependency resolution |
| `PluginSignature` | SHA-256 complete FIPS 180-4 implementation for plugin fingerprinting, Dilithium-based signing |
| `PluginAudit` | Security review workflow with real timestamps and SHA-256 fingerprint |
| `PluginServices` | Kernel service interface: memory limits, IPC channel limits, device access |

### Plugin Lifecycle

1. **Load**: `PluginLoader::load(path)` — parse ELF, map segments, apply RELA relocations, resolve symbols, get entry point `plugin_entry`
2. **Register**: `PluginRegistry::register(id, meta)` — register plugin metadata and dependencies
3. **Use**: Call plugin functionality through the `Plugin` trait interface
4. **Unload**: `PluginLoader::unload(handle)` — close dynamic library, release resources
5. **Unregister**: `PluginRegistry::unregister(id)` — remove plugin records

### Plugin Configuration

```rust
pub struct LoaderConfig {
    pub verify_signature: bool,    // Verify plugin signature (SHA-256 + Dilithium)
    pub enable_cache: bool,        // Cache loaded plugins
    pub max_plugin_size: usize,    // Maximum plugin size (default 10MB)
}
```

---

## Quantum-Safe Architecture

Nuva OS integrates quantum-safe technology at the HAL layer, providing post-quantum cryptography support:

### QRNG (Quantum Random Number Generator)

- Hardware QRNG integration: entropy source detection (MMIO/DeviceTree/ACPI/RISC-V seed/ARM RNDR), entropy pool with SHA-256 conditioning, NIST SP 800-90B health tests (Repetition Count + Adaptive Proportion + Restart)
- Software PRNG fallback (Xorshift128+)
- Randomness quality assessment (`RandomnessQuality`)

### QKD (Quantum Key Distribution)

- BB84 protocol implementation with qubit preparation (4 polarization bases) and measurement, basis reconciliation, error estimation, privacy amplification
- Quantum-secure key exchange between trusted nodes

### PQC (Post-Quantum Cryptography)

Based on NIST PQC standardization schemes:

| Algorithm | Type | Variants |
|-----------|------|----------|
| CRYSTALS-Kyber | Key Encapsulation (KEM) | Kyber512, Kyber768, Kyber1024 |
| CRYSTALS-Dilithium | Digital Signature | Dilithium2, Dilithium3, Dilithium5 |

`PqcProvider` trait interface:

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

### Quantum-Safe Security Module

`QuantumSafeSecurity` integrates QRNG and PQC, providing unified security configuration (`SecurityConfig`) and security levels (`SecurityLevel`).

---

## POSIX Compatibility

### System Calls

- File operations: open, close, read, write, lseek, stat...
- Process operations: fork, execve, exit, waitpid, getpid...
- Memory operations: mmap, munmap, mprotect, mlock...
- IPC: pipe, shmget, semget, msgget...
- Network: socket, bind, listen, accept, connect...

### Signals

- Standard signals: SIGHUP, SIGINT, SIGTERM, SIGKILL...
- Real-time signals: SIGRTMIN - SIGRTMAX
- Signal handling: sigaction, sigprocmask, sigpending...

---

## Performance Optimization

### Kernel Optimization

- Lock-free data structures
- RCU (Read-Copy-Update) — read-heavy data structures allow lock-free reads with deferred reclamation; grace period detection via `synchronize_rcu()`
- Per-CPU variables — `#[repr(C, align(64))]` cache-line-aligned; zero lock contention for per-CPU data (run queues, page caches, statistics)
- Huge page support (2MB/1GB)

### Memory Optimization

- Slab coloring
- Memory compaction
- Page cache
- Readahead
- Per-CPU page cache (PCP) — order-0 allocations bypass global Buddy lock; batch refill/drain with watermarks

### I/O Optimization

- io_uring — zero-copy async I/O with shared ring buffers; submission/completion queues in user-kernel shared memory; supports fixed buffers and linked operations

### Scheduling Optimization

- Scheduling domains
- Scheduling groups
- Per-Entity Load Tracking (PELT)
- Energy Aware Scheduling (EAS)
- Per-CPU run queues — `PerCpuRunQueue` cache-line aligned, lock-free local scheduling

---

## Security Design

### Memory Safety

- ASLR (Address Space Layout Randomization)
- DEP (Data Execution Prevention)
- Stack canary
- Safe stack

### Access Control

- Capability (NvCapability tokens, permission monotonicity, cascading revocation)
- NvCapability-LSM bridge: capability tokens bridged with Linux Security Module hooks (`kernel/security/security_hook.rs`)
- ACL (Access Control List)
- NSM (Nuva Security Module) policy

### Encryption

- Disk encryption
- File encryption
- Network encryption (TLS)

### Quantum Safety

- CRYSTALS-Kyber key encapsulation
- CRYSTALS-Dilithium digital signatures
- QRNG high-quality random numbers

---

## Hardware Abstraction Layer (HAL)

HAL provides unified hardware access interfaces through Rust traits. For detailed API signatures, see [API.md](API.md) and [api/API_REFERENCE.md](api/API_REFERENCE.md).

Key trait interfaces:
- `CpuHal` — CPU frequency/voltage/temperature management
- `GpuHal` — GPU frame management and command queue
- `NpuHal` — NPU model loading, inference execution, buffer management
- `PowerHal` — Power state management (suspend/resume)
- `InputHal` — Input device event reading

---

## Syslib System Libraries Layer (L2)

The Syslib layer provides a collection of system libraries for applications and services, located above the Kernel and below Services.

### Core Submodules

| Submodule | Description | Key Files |
|-----------|-------------|-----------|
| core | Core library: allocator (pool), synchronization primitives (lockfree) | `alloc/`, `sync/` |
| brain | Nuva Brain AI engine: inference, model management, NPU scheduling, operators, service | `inference/`, `model/`, `npu/`, `operators/`, `service/` |
| ai | AI library: model manager, optimizer, scheduler | `model_manager.rs`, `optimizer.rs`, `scheduler.rs` |
| lang | NuvaLang compiler and runtime: lexer, parser, semantic analysis, IR, code generation, GC, VM | `lexer/`, `parser/`, `semantic/`, `codegen/`, `runtime/`, `stdlib/`, `binary/` |
| ml | Machine learning library: tensor, model, inference engine | `tensor.rs`, `model.rs`, `engine.rs` |
| net | Network library: TCP/UDP/IP/ICMP/ARP/Ethernet, HTTP, WebSocket, JSON | `tcp/`, `udp.rs`, `ip.rs`, `http.rs`, `websocket.rs`, `json.rs` |
| data | Data structure library: key-value store, database | `kvstore.rs`, `database.rs` |
| gfx | Graphics library: FPS monitoring | `fps/` |
| ui | UI library: layout, view, window | `view/` |
| std | Standard library: collections, basic types, IO | `collection.rs`, `foundation.rs` |
| runtime | Runtime library: Arc, metadata, protocol | `arc.rs`, `metadata.rs`, `protocol.rs` |
| dispatch | Concurrency framework (GCD-style): thread pool, semaphore, dispatch queue, dispatch group | `pool.rs`, `semaphore.rs`, `queue.rs`, `group.rs` |
| posix | POSIX compatibility layer: system call wrappers, signal handling, file descriptor management | `errno.rs`, `signal.rs` |

---

## LoongArch64 Architecture Support

Nuva OS supports the Loongson LoongArch64 architecture with the target platform `loongarch64-unknown-none`.

### Supported SoCs

| SoC | Feature Flag | Description |
|-----|-------------|-------------|
| Loongson 3A6000 | `loongson3a6000` | Desktop processor |
| Loongson 3C6000 | `loongson3c6000` | Server processor |

### HAL Implementation

- `hal/loongarch64/` — LoongArch64 architecture-specific HAL implementation
  - `cpu.rs` — CPU operations (frequency, voltage, temperature, idle state)
  - `mmu.rs` — Memory management unit (3-level page tables with PageTableOps, Pte struct, TLB)
  - `lsx.rs` — LSX 128-bit SIMD extension (native vld/vst inline assembly)
  - `lasx.rs` — LASX 256-bit SIMD extension
  - `lvz.rs` — LVZ hardware virtualization support
  - `lbt.rs` — LBT binary translation support
- `hal/cpu/loongson.rs` — Loongson SoC-specific implementation
- `kernel/arch/loongarch64/` — Kernel architecture-related code (boot module, parse_boot_info)
- Device tree passed via firmware (UEFI)

### Interrupt Controller

LoongArch64 uses the EIOINTC (Extended I/O Interrupt Controller) with complete `IrqControllerOps` implementation for interrupt routing and management.

### Memory Layout

LoongArch64 uses a memory layout compatible with x86-64:
- 48-bit virtual address space
- 3-level page tables (4KB pages) with `PageTableOps` trait implementation
- User space: 0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF (128TB)
- Kernel space: 0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF (128TB)

### Build Status

All four architectures compile with 0 errors:
- ARM64 (kirin9020): ✅
- x86_64 (intel_core): ✅
- LoongArch64 (loongson3a6000): ✅
- RISC-V 64 (qemu_virt): ✅

---

## SDK Layer

The Nuva SDK provides developer tooling for building, debugging, profiling, and packaging Nuva OS applications.

### Core Submodules

| Submodule | Description | Key Files |
|-----------|-------------|-----------|
| build | Build system: cross-compilation, build cache, target configuration, parallel build scheduling | `config.rs`, `cache.rs`, `cross.rs`, `target.rs`, `scheduler.rs`, `executor.rs` |
| cli | Command-line interface: init, build, run, test, debug, profile, package management | `args.rs`, `commands/` |
| debug | Debugger: fork+execv launch, ptrace read_registers, process_vm_readv read_memory, breakpoints, stack unwinding | `breakpoint.rs`, `memory.rs`, `stack.rs`, `variable.rs`, `execution.rs`, `target.rs` |
| debug/dap | Debug Adapter Protocol: variable reading, disassembly processor, DAP server | `protocol.rs`, `server.rs` |
| package | Package manager: dependency resolver (transitive), registry version queries, lock file, SHA-256 checksum validation | `dependency.rs`, `resolver.rs`, `registry.rs`, `validator.rs`, `lock_file.rs`, `cache.rs` |
| profiler | Performance profiler: /proc real callstack capture, gettid(), CPU sampling (/proc/stat), memory sampling (/proc/self/statm), flamegraph | `cpu.rs`, `memory.rs`, `io.rs`, `lock.rs`, `flamegraph.rs`, `sampler.rs` |

---

<!-- Translation Status: Source (English) | Last Updated: 2026-05-30 -->

**Last Updated**: 2026-05-30
**License**: Apache-2.0
