# Nuva OS Layered Architecture Rules

**Document ID**: ARCH-LAYER-RULES-001
**Version**: 1.3.0
**Created**: 2026-04-03
**Last Updated**: 2026-05-30

---

## 1. Architecture Layer Definition

### 1.1 Layer Structure

```
┌─────────────────────────────────────────┐
│  Layer 4: Application                   │
│  - UI Framework (application/ui)        │
│  - Window Management (application/window)│
│  - Event System (application/event)     │
│  - Rendering Engine (application/render)│
│  - Resource Mgmt (application/resource) │
└─────────────────────────────────────────┘
              ↓ depends only on
┌─────────────────────────────────────────┐
│  Layer 3: Services                      │
│  - App Service (services/app)           │
│  - IPC Service (services/ipc)           │
│  - Net Service (services/net)           │
│  - Power Service (services/power)       │
│  - Security Service (services/security) │
└─────────────────────────────────────────┘
              ↓ depends only on
┌─────────────────────────────────────────┐
│  Layer 2: Lib                           │
│  - AI Lib (syslib/ai)                   │
│  - Core Lib (syslib/core)               │
│  - Lang Lib (syslib/lang)               │
│  - Net Lib (syslib/net)                 │
│  - Gfx Lib (syslib/gfx)                 │
│  - ML Lib (syslib/ml)                   │
│  - UI Lib (syslib/ui)                   │
│  - Data Lib (syslib/data)               │
│  - Runtime (syslib/runtime)             │
│  - Dispatch (syslib/dispatch)           │
│  - POSIX Compat (syslib/posix)          │
└─────────────────────────────────────────┘
              ↓ depends only on
┌─────────────────────────────────────────┐
│  Layer 1: Kernel                        │
│  - Arch (kernel/arch)                   │
│  - Memory Mgmt (kernel/mm)              │
│  - Process Mgmt (kernel/process)        │
│  - Scheduler (kernel/sched)             │
│  - File System (kernel/fs)              │
│  - IPC (kernel/ipc)                     │
│  - Driver Framework (kernel/driver)     │
│  - Plugin System (kernel/plugin)        │
│  - Security (kernel/security)           │
│  - Quantum (kernel/quantum)             │
│  - Net Stack (kernel/net)               │
│  - Syscall (kernel/syscall)             │
│  - Sync Primitives (kernel/sync)        │
└─────────────────────────────────────────┘
              ↓ depends only on
┌─────────────────────────────────────────┐
│  Layer 0: HAL                           │
│  - CPU Abstraction (hal/cpu)            │
│  - GPU Abstraction (hal/gpu)            │
│  - NPU Abstraction (hal/npu)            │
│  - Quantum Devices (hal/quantum)        │
│  - Power Mgmt (hal/power)               │
│  - Platform HAL (hal/platform, dt, acpi)│
│  - FFI Interface (hal/ffi)              │
│  - Input Devices (hal/input)            │
└─────────────────────────────────────────┘
```

---

## 2. Complete Layer Dependency Rules (L0→L4)

### 2.1 Allowed Dependencies

| Source Layer | Target Layer | Allowed | Description |
|--------------|--------------|---------|-------------|
| L4 Application | L3 Services | ✅ | Application can use services |
| L4 Application | L2 Lib | ✅ | Application can use libraries |
| L4 Application | L1 Kernel | ✅ | Application can call kernel API (via syscall) |
| L3 Services | L2 Lib | ✅ | Services can use libraries |
| L3 Services | L1 Kernel | ✅ | Services can call kernel API |
| L2 Lib | L1 Kernel | ✅ | Libraries can call kernel API |
| L2 Lib | L0 HAL | ⚠️ | Only through trait abstraction |
| L1 Kernel | L0 HAL | ✅ | Kernel uses HAL |
| L0 HAL | None | ✅ | HAL has no layer dependencies |

### 2.2 Prohibited Dependencies

| Source Layer | Target Layer | Prohibited | Reason |
|--------------|--------------|------------|--------|
| L0 HAL | L1 Kernel | ❌ | HAL cannot depend on kernel |
| L0 HAL | L2 Lib | ❌ | HAL cannot depend on libraries |
| L0 HAL | L3 Services | ❌ | HAL cannot depend on services |
| L0 HAL | L4 Application | ❌ | HAL cannot depend on application |
| L1 Kernel | L2 Lib | ❌ | Kernel cannot depend on libraries |
| L1 Kernel | L3 Services | ❌ | Kernel cannot depend on services |
| L1 Kernel | L4 Application | ❌ | Kernel cannot depend on application |
| L2 Lib | L3 Services | ❌ | Libraries cannot depend on services |
| L2 Lib | L4 Application | ❌ | Libraries cannot depend on application |
| L3 Services | L4 Application | ❌ | Services cannot depend on application |

### 2.3 Same-Layer Dependencies

| Source Layer | Same-Layer | Rule |
|--------------|------------|------|
| L0 HAL | HAL ↔ HAL | Allowed, HAL modules may depend on each other |
| L1 Kernel | Kernel ↔ Kernel | Allowed, but requires code review |
| L2 Lib | Lib ↔ Lib | Allowed, library modules may depend on each other |
| L3 Services | Services ↔ Services | Allowed, service modules may depend on each other |
| L4 Application | App ↔ App | Allowed, application modules may depend on each other |

### 2.4 Special Rules

#### 2.4.1 Library Layer Accessing HAL

The library layer may access HAL through **trait abstraction** only, never through concrete implementations:

```rust
// ✅ Correct: through trait abstraction
pub struct AiEngine<N: NpuHal> {
    npu: N,
}

// ❌ Wrong: direct dependency on HAL implementation
use crate::hal::npu::davinci::DaVinciNpu;
```

This constraint is enforced in `configs/layers/lib/Cargo.toml` via `nuva-hal-traits`:

```toml
[dependencies]
nuva-hal-traits = { path = "../hal/traits" }
# Note: HAL access must be through traits only
```

#### 2.4.2 Kernel Plugin System

The plugin system allows dynamic module loading with the following constraints:

1. Plugins must implement the `Plugin` trait
2. Plugins access system services through `PluginContext`
3. Plugins cannot directly access kernel internal structures
4. Plugins run in a sandbox (optional)

Plugin system modules (L1 Kernel): `core`, `loader`, `manager`, `registry`, `sandbox`, `legacy`.

#### 2.4.3 Declarative Driver Model

The declarative driver model resides in L1 Kernel (`kernel/driver/declarative`, `kernel/driver/declarative_pm`, `kernel/driver/matching`) and follows these constraints:

1. `declare_driver!` macro generates `DriverDescriptor` statics — no runtime heap allocation at registration time
2. `declare_pm!` macro generates `PmStateMachine` statics — no runtime heap allocation
3. `declare_resource!` macro generates `DeclarativeResource` statics — no runtime heap allocation
4. `CompatibleHashTable` (in `kernel/driver/matching`) may use `alloc` for dynamic hash buckets, but must not depend on L2/L3/L4
5. `DeclarativeDriver` trait implementations may only depend on L1 kernel and L0 HAL
6. Driver matching during boot uses `CompatibleHashTable` for O(1) lookup, replacing linear scan

```rust
// ✅ Correct: declarative driver registration
declare_driver! {
    MY_DRIVER {
        name: "my_driver",
        compatible: &["vendor,my-device"],
        resources: &[ResourceDescriptor::Irq { number: 42 }],
        capabilities: READ | WRITE,
        priority: 0,
        hotplug: false,
    }
}

// ❌ Wrong: manual DriverDescriptor construction (bypasses declarative model)
static MY_DRIVER: DriverDescriptor = DriverDescriptor {
    name: "my_driver",
    compatible: &["vendor,my-device"],
    // ...
};
```

#### 2.4.4 Cross-Platform Code

Platform-specific code must be isolated:

```
hal/
├── arm64/           # ARM64-specific code
├── x64/             # x86-64-specific code
├── loongarch64/     # LoongArch64-specific code
├── snapdragon/      # Qualcomm Snapdragon platform
├── dt.rs            # Device Tree (ARM64)
├── acpi.rs          # ACPI (x86_64)
└── platform.rs      # Cross-platform detection and dispatch
```

---

## 3. Module Boundary Definitions

### 3.1 HAL Layer Modules (L0)

| Module | Responsibility | Visibility |
|--------|---------------|------------|
| `hal::cpu` | CPU Abstraction (DVFS, Thermal) | pub trait |
| `hal::gpu` | GPU Abstraction (Maleoon, Command Queue) | pub trait |
| `hal::npu` | NPU Abstraction (DaVinci, ONNX, Predictor) | pub trait |
| `hal::quantum` | Quantum Cryptography (Kyber, Dilithium, QRNG, QKD) | pub trait |
| `hal::power` | Power Management (PMIC, Suspend/Resume) | pub trait |
| `hal::platform` | Platform HAL (dt, acpi) | pub (platform-specific) |
| `hal::ffi` | C/C++ FFI + API Stability | pub (extern "C") |
| `hal::input` | Input Devices | pub trait |

**Boundary Rules**:
- HAL modules may depend on each other
- HAL modules cannot depend on kernel, lib, services, application
- HAL traits must define clear interfaces
- HAL `Cargo.toml` has `allowed_deps = []`

### 3.2 Kernel Layer Modules (L1)

| Module | Responsibility | Visibility |
|--------|---------------|------------|
| `kernel::arch` | Architecture Abstraction (arm64, x64, loongarch64) | pub (internal) |
| `kernel::mm` | Memory Management (Buddy, SLAB, VMA, NUMA) | pub API |
| `kernel::process` | Process Management (fork, execve, signal) | pub API |
| `kernel::sched` | Scheduler (CFS, EAS, RT) | pub API |
| `kernel::fs` | File System (VFS, NuvaFS) | pub API |
| `kernel::ipc` | IPC (NuvaIPC, L4, Shared Memory) | pub API |
| `kernel::driver` | Driver Framework (Device Model, DMA, GPIO, I2C, SPI) | pub API |
| `kernel::plugin` | Plugin System (Loader, Manager, Sandbox) | pub API |
| `kernel::security` | Security (LSM, ASLR, Sandbox, Stack Canary) | pub API |
| `kernel::quantum` | Quantum Support (Scheduler) | pub API |
| `kernel::net` | Network Stack | pub API |
| `kernel::syscall` | System Calls | pub API |
| `kernel::sync` | Sync Primitives | pub (internal) |

**Boundary Rules**:
- Kernel modules may depend on each other (requires review)
- Kernel modules can depend on HAL
- Kernel modules cannot depend on lib, services, application
- Kernel `Cargo.toml` has `allowed_deps = ["hal"]`

### 3.3 Library Layer Modules (L2)

| Module | Responsibility | Visibility |
|--------|---------------|------------|
| `syslib::ai` | AI/ML Library | pub |
| `syslib::brain` | Nuva Brain AI Engine | pub |
| `syslib::core` | Core Library | pub |
| `syslib::lang` | NuvaLang Compiler & Runtime | pub |
| `syslib::ml` | Machine Learning Library | pub |
| `syslib::net` | Network Library | pub |
| `syslib::gfx` | Graphics Library | pub |
| `syslib::ui` | UI Library | pub |
| `syslib::data` | Data Structure Library | pub |
| `syslib::std` | Standard Library | pub |
| `syslib::runtime` | Runtime Library | pub |
| `syslib::dispatch` | Concurrency Framework (GCD-style) | pub |
| `syslib::posix` | POSIX Compatibility Layer | pub |

**Boundary Rules**:
- Library modules may depend on each other
- Library modules can depend on kernel API
- Library modules access HAL through traits only
- Library `Cargo.toml` has `allowed_deps = ["kernel", "hal"]` (HAL traits only)

### 3.4 Service Layer Modules (L3)

| Module | Responsibility | Visibility |
|--------|---------------|------------|
| `services::app` | Application Service (Activity, Package Manager) | pub |
| `services::ipc` | IPC Service (Binder, Channel) | pub |
| `services::net` | Network Service (DNS, TCP/UDP) | pub |
| `services::power` | Power Service (Policy, Wake Lock) | pub |
| `services::security` | Security Service (Gatekeeper, Keymaster, TEE) | pub |

**Boundary Rules**:
- Service modules may depend on each other
- Service modules can depend on libraries and kernel
- Service modules cannot depend on application

### 3.5 Application Layer Modules (L4)

| Module | Responsibility | Visibility |
|--------|---------------|------------|
| `application::ui` | UI Framework (Adaptive Layout, Components) | pub |
| `application::window` | Window Management | pub |
| `application::event` | Event System | pub |
| `application::render` | Rendering Engine (Compositor, Brush) | pub |
| `application::resource` | Resource Management (Cache, Decoder) | pub |

**Boundary Rules**:
- Application modules may depend on each other
- Application modules can depend on services, libraries, and kernel
- Application is the highest layer with no reverse dependencies

---

## 4. dep_analyzer Tool

### 4.1 Overview

`dep_analyzer` is Nuva OS's architecture compliance tool, located at `tools/dep_analyzer/`. It analyzes module dependencies and enforces architectural layer boundaries.

**Features**:
- Walks all `.rs` files and parses `use` statements
- Builds a dependency graph (`DependencyGraph`)
- Detects **layer violations** (lower layer depending on higher)
- Detects **circular dependencies** (DFS algorithm)
- Detects **direct cross-layer dependencies** (without abstraction)
- **Enforced at build time** in release mode via `build.rs`
- **Kernel functional domain subdirectory** boundary validation
- **Re-export validation** — ensures backward-compatible re-exports follow layer rules

### 4.2 Violation Types

| Type | Description | Severity |
|------|-------------|----------|
| `LayerViolation` | Lower layer depends on higher layer (e.g., HAL → Kernel) | P0/P1 |
| `CircularDependency` | Circular dependency between modules | P1 |
| `DirectDependency` | Cross-layer dependency without abstraction | P2 |
| `SubdirectoryViolation` | Functional domain subdirectory violates layer boundary | P1 |
| `ReExportViolation` | Re-export bypasses layer abstraction | P2 |

### 4.3 Usage

```bash
# Analyze project dependencies
cargo run --bin dep_analyzer -- /path/to/nuva

# On success:
# ✅ No dependency violations found!

# On failure: outputs specific violations and returns non-zero exit code
```

### 4.4 build.rs Integration

`dep_analyzer` can be integrated via `build.rs` for compile-time dependency checking:

```rust
// build.rs (simplified example)
fn main() {
    if std::env::var("SKIP_DEP_CHECK").is_err() {
        let status = std::process::Command::new("cargo")
            .args(["run", "--bin", "dep_analyzer", "--", "."])
            .status()
            .expect("Failed to run dep_analyzer");
        if !status.success() {
            panic!("Dependency violations detected! Fix or set SKIP_DEP_CHECK=1");
        }
    }
}
```

The project `Cargo.toml` provides a `skip_dep_check` feature to bypass checks:

```toml
[features]
skip_dep_check = []
```

### 4.5 CI Integration

```yaml
# .github/workflows/check-deps.yml
- name: Check Dependencies
  run: |
    cargo run --bin dep_analyzer -- .
    if [ $? -ne 0 ]; then
      echo "Dependency violations found!"
      exit 1
    fi
```

---

## 5. Layer Configuration Files

Each layer maintains an independent `Cargo.toml` in `configs/layers/`:

| Config File | Layer | `allowed_deps` | Description |
|-------------|-------|----------------|-------------|
| `configs/layers/hal/Cargo.toml` | L0 | `[]` | HAL has no layer dependencies |
| `configs/layers/kernel/Cargo.toml` | L1 | `["hal"]` | Kernel depends only on HAL |
| `configs/layers/lib/Cargo.toml` | L2 | `["kernel", "hal"]` | Lib depends on kernel API + HAL traits |

Each config declares layer metadata via `[package.metadata.layer]`:

```toml
[package.metadata.layer]
level = 0              # Layer number
visibility = "public"  # Visibility
allowed_deps = []      # Allowed cross-layer dependencies
```

---

## 6. Violation Detection and Handling

### 6.1 Violation Handling Flow

| Priority | Violation Type | Example | Deadline |
|----------|---------------|---------|----------|
| P0 | HAL depends on kernel/lib/services | `hal::xxx` → `kernel::yyy` | Fix immediately |
| P1 | Kernel depends on lib/services | `kernel::xxx` → `syslib::yyy` | Fix ASAP |
| P1 | Circular dependency | A → B → A | Fix ASAP |
| P2 | Lib directly depends on HAL impl | `syslib::ai` → `hal::npu::davinci` | Plan fix |
| P2 | Cross-layer dependency without abstraction | Direct dependency | Plan fix |

### 6.2 Violation Example and Fix

#### Violation

```rust
// ❌ Wrong: syslib/brain directly depends on kernel/sched internals
use crate::kernel::sched::task::TaskStruct;

pub fn schedule_task(task: TaskStruct) {
    // ...
}
```

#### Fix

```rust
// ✅ Correct: through abstraction
pub trait Scheduler {
    fn schedule(&self, task: &TaskInfo) -> Result<(), Error>;
}

pub fn schedule_task<S: Scheduler>(scheduler: &S, task: &TaskInfo) {
    scheduler.schedule(task)?;
}
```

---

## 7. Abstraction Interface Specification

### 7.1 HAL Abstraction Interface

All HAL modules must provide trait interfaces:

```rust
// hal/npu/traits.rs
pub trait NpuHal: Send + Sync {
    fn initialize(&mut self) -> Result<(), NpuError>;
    fn load_model(&mut self, model: &ModelData) -> Result<ModelId, NpuError>;
    fn execute(&self, model: ModelId, inputs: &[Buffer], outputs: &mut [Buffer]) -> Result<(), NpuError>;
    fn unload_model(&mut self, model: ModelId) -> Result<(), NpuError>;
}
```

### 7.2 Kernel API Interface

The kernel exposes a stable API:

```rust
// kernel/api/mod.rs
pub mod process;
pub mod memory;
pub mod fs;
pub mod ipc;
pub mod driver;
pub mod security;
pub mod quantum;
```

### 7.3 Service Interface

Services expose functionality through interfaces:

```rust
// services/security/interface.rs
pub trait SecurityService {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
}
```

### 7.4 FFI API Stability

The HAL FFI layer provides API version stability guarantees via `hal/ffi/stability.rs`:

- C API (`hal/ffi/c_api/`): C99 compliant, ABI stable
- C++ API (`hal/ffi/cpp_api/`): RAII wrappers, exception-safe

---

## 8. Dependency Graph Visualization

### 8.1 Compliant Dependency Graph

```
L4 Application
    ├─→ L3 Services
    ├─→ L2 Lib
    └─→ L1 Kernel (syscall)

L3 Services
    ├─→ L2 Lib
    └─→ L1 Kernel (API)

L2 Lib
    ├─→ L1 Kernel (API)
    └─→ L0 HAL (trait only)

L1 Kernel
    └─→ L0 HAL

L0 HAL
    └─→ (no external layer dependencies)
```

### 8.2 Prohibited Dependencies

```
❌ L0 HAL → L1 Kernel
❌ L0 HAL → L2 Lib
❌ L0 HAL → L3 Services
❌ L0 HAL → L4 Application
❌ L1 Kernel → L2 Lib
❌ L1 Kernel → L3 Services
❌ L1 Kernel → L4 Application
❌ L2 Lib → L3 Services
❌ L2 Lib → L4 Application
❌ L3 Services → L4 Application
```

---

## 9. Implementation Checklist

### 9.1 Code Review

- [ ] All `use` statements comply with layer rules
- [ ] No circular dependencies
- [ ] HAL access through trait abstraction
- [ ] Kernel API stable and documented
- [ ] Service interfaces clearly defined
- [ ] FFI API version compatible

### 9.2 Build Verification

- [ ] `dep_analyzer` passes
- [ ] Layer config `Cargo.toml` correct
- [ ] No compiler warnings
- [ ] All tests pass
- [ ] CI checks pass

### 9.3 Documentation Verification

- [ ] Module responsibilities documented
- [ ] Dependency relationships documented
- [ ] Interfaces documented
- [ ] Architecture diagrams updated

---

**Document Status**: Defined
**Implementation Status**: dep_analyzer implemented and integrated in build.rs (release mode); CI integration pending
**Next Step**: Full CI pipeline integration and subdirectory boundary enforcement

---

## Appendix A: Additional L1 Kernel Sub-modules in Code

The following kernel sub-modules exist in code (`kernel/mod.rs`) but are not listed in the core layer structure diagram above (auxiliary/infrastructure modules):

### Functional Domain Subdirectories (Reorganized)

| Subdirectory | Responsibility |
|-------------|---------------|
| `kernel::init` | Initialization (cmdline, config, elf, platform, resource) |
| `kernel::diag` | Diagnostics (journal, kdebug, log, scanner, stats) |
| `kernel::irq_mgmt` | IRQ management (apic_ops, irq, trap) |
| `kernel::net_stack` | Network stack (socket, tcpip) |
| `kernel::storage` | Storage subsystem (block) |
| `kernel::device` | Device model & plugins (device_model, driver_plugin, feature_plugin, module, notifier) |
| `kernel::power_mgmt` | Power management (hotplug, pm, power) |
| `kernel::virt` | Virtualization (vmx) |
| `kernel::core` | Core services (cache, cpu, defense, kernel_thread, mempool, perf_tune, posix, random, signal, time, wait, workqueue) |

### Legacy/Other Sub-modules

| Module | Responsibility |
|--------|---------------|
| `kernel::debug` | Kernel debug support |
| `kernel::interrupt` | Interrupt handling |
| `kernel::timer` | Timer |
| `kernel::perf` | Performance monitoring |
| `kernel::bsd` | BSD compatibility |
| `kernel::user` | User management |
| `kernel::apic_ops` | APIC operations |

These modules all belong to L1 Kernel layer and follow L1 boundary rules (depend only on L0 HAL, never on L2/L3/L4).
