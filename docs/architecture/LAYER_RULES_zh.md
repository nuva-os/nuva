# Nuva OS 分层架构规则

**文档编号**: ARCH-LAYER-RULES-001
**版本**: 1.3.0
**创建日期**: 2026-04-03
**最后更新**: 2026-05-30

---

## 一、架构层次定义

### 1.1 层次结构

```
┌─────────────────────────────────────────┐
│  Layer 4: Application (应用层)          │
│  - UI 框架 (application/ui)             │
│  - 窗口管理 (application/window)        │
│  - 事件系统 (application/event)         │
│  - 渲染引擎 (application/render)        │
│  - 资源管理 (application/resource)      │
└─────────────────────────────────────────┘
              ↓ 只能依赖
┌─────────────────────────────────────────┐
│  Layer 3: Services (服务层)             │
│  - 应用服务 (services/app)              │
│  - IPC 服务 (services/ipc)              │
│  - 网络服务 (services/net)              │
│  - 电源服务 (services/power)            │
│  - 安全服务 (services/security)         │
└─────────────────────────────────────────┘
              ↓ 只能依赖
┌─────────────────────────────────────────┐
│  Layer 2: Lib (库层)                    │
│  - AI 库 (syslib/ai)                    │
│  - 核心库 (syslib/core)                 │
│  - 语言库 (syslib/lang)                 │
│  - 网络库 (syslib/net)                  │
│  - 图形库 (syslib/gfx)                  │
│  - ML 库 (syslib/ml)                    │
│  - UI 库 (syslib/ui)                    │
│  - 数据库 (syslib/data)                 │
│  - 运行时 (syslib/runtime)              │
│  - 并发框架 (syslib/dispatch)           │
│  - POSIX 兼容层 (syslib/posix)          │
└─────────────────────────────────────────┘
              ↓ 只能依赖
┌─────────────────────────────────────────┐
│  Layer 1: Kernel (内核层)               │
│  - 架构抽象 (kernel/arch)               │
│  - 内存管理 (kernel/mm)                 │
│  - 进程管理 (kernel/process)            │
│  - 调度器 (kernel/sched)                │
│  - 文件系统 (kernel/fs)                 │
│  - IPC (kernel/ipc)                     │
│  - 驱动框架 (kernel/driver)             │
│  - 插件系统 (kernel/plugin)             │
│  - 安全模块 (kernel/security)           │
│  - 量子支持 (kernel/quantum)            │
│  - 网络栈 (kernel/net)                  │
│  - 系统调用 (kernel/syscall)            │
│  - 同步原语 (kernel/sync)               │
└─────────────────────────────────────────┘
              ↓ 只能依赖
┌─────────────────────────────────────────┐
│  Layer 0: HAL (硬件抽象层)              │
│  - CPU 抽象 (hal/cpu)                   │
│  - GPU 抽象 (hal/gpu)                   │
│  - NPU 抽象 (hal/npu)                   │
│  - 量子设备 (hal/quantum)               │
│  - 电源管理 (hal/power)                 │
│  - 平台 HAL (hal/platform, dt, acpi)    │
│  - FFI 接口 (hal/ffi)                   │
│  - 输入设备 (hal/input)                 │
└─────────────────────────────────────────┘
```

---

## 二、完整分层依赖规则 (L0→L4)

### 2.1 允许的依赖

| 源层 | 目标层 | 允许 | 说明 |
|------|--------|------|------|
| L4 Application | L3 Services | ✅ | 应用可使用服务 |
| L4 Application | L2 Lib | ✅ | 应用可使用库 |
| L4 Application | L1 Kernel | ✅ | 应用可调用内核 API（通过系统调用） |
| L3 Services | L2 Lib | ✅ | 服务可使用库 |
| L3 Services | L1 Kernel | ✅ | 服务可调用内核 API |
| L2 Lib | L1 Kernel | ✅ | 库可调用内核 API |
| L2 Lib | L0 HAL | ⚠️ | 仅通过 trait 抽象接口 |
| L1 Kernel | L0 HAL | ✅ | 内核使用 HAL |
| L0 HAL | 无 | ✅ | HAL 不依赖其他任何层 |

### 2.2 禁止的依赖

| 源层 | 目标层 | 禁止 | 原因 |
|------|--------|------|------|
| L0 HAL | L1 Kernel | ❌ | HAL 不能依赖内核 |
| L0 HAL | L2 Lib | ❌ | HAL 不能依赖库 |
| L0 HAL | L3 Services | ❌ | HAL 不能依赖服务 |
| L0 HAL | L4 Application | ❌ | HAL 不能依赖应用 |
| L1 Kernel | L2 Lib | ❌ | 内核不能依赖库 |
| L1 Kernel | L3 Services | ❌ | 内核不能依赖服务 |
| L1 Kernel | L4 Application | ❌ | 内核不能依赖应用 |
| L2 Lib | L3 Services | ❌ | 库不能依赖服务 |
| L2 Lib | L4 Application | ❌ | 库不能依赖应用 |
| L3 Services | L4 Application | ❌ | 服务不能依赖应用 |

### 2.3 同层依赖

| 源层 | 同层依赖 | 规则 |
|------|----------|------|
| L0 HAL | HAL ↔ HAL | 允许，HAL 模块之间可相互依赖 |
| L1 Kernel | Kernel ↔ Kernel | 允许，但需代码审查 |
| L2 Lib | Lib ↔ Lib | 允许，库模块之间可相互依赖 |
| L3 Services | Services ↔ Services | 允许，服务模块之间可相互依赖 |
| L4 Application | App ↔ App | 允许，应用模块之间可相互依赖 |

### 2.4 特殊规则

#### 2.4.1 库层访问 HAL

库层可以通过 **trait 抽象接口** 访问 HAL，但不能直接依赖具体实现：

```rust
// ✅ 正确：通过 trait 抽象
pub struct AiEngine<N: NpuHal> {
    npu: N,
}

// ❌ 错误：直接依赖 HAL 实现
use crate::hal::npu::davinci::DaVinciNpu;
```

在 `configs/layers/lib/Cargo.toml` 中通过 `nuva-hal-traits` 依赖实现此约束：

```toml
[dependencies]
nuva-hal-traits = { path = "../hal/traits" }
# 注意：HAL 访问必须仅通过 traits
```

#### 2.4.2 内核插件系统

插件系统允许动态加载模块，但必须遵守：

1. 插件必须实现 `Plugin` trait
2. 插件通过 `PluginContext` 访问系统服务
3. 插件不能直接访问内核内部结构
4. 插件在沙箱中运行（可选）

插件系统所在层级（L1 Kernel）的模块包括：`core`、`loader`、`manager`、`registry`、`sandbox`、`legacy`。

#### 2.4.3 声明式驱动模型

声明式驱动模型位于 L1 内核层（`kernel/driver/declarative`、`kernel/driver/declarative_pm`、`kernel/driver/matching`），遵守以下约束：

1. `declare_driver!` 宏生成 `DriverDescriptor` 静态变量 — 注册时不进行运行时堆分配
2. `declare_pm!` 宏生成 `PmStateMachine` 静态变量 — 不进行运行时堆分配
3. `declare_resource!` 宏生成 `DeclarativeResource` 静态变量 — 不进行运行时堆分配
4. `CompatibleHashTable`（位于 `kernel/driver/matching`）可使用 `alloc` 进行动态哈希桶分配，但不得依赖 L2/L3/L4
5. `DeclarativeDriver` trait 实现只能依赖 L1 内核和 L0 HAL
6. 启动期间的驱动匹配使用 `CompatibleHashTable` 进行 O(1) 查找，替代线性扫描

```rust
// ✅ 正确：声明式驱动注册
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

// ❌ 错误：手动构造 DriverDescriptor（绕过声明式模型）
static MY_DRIVER: DriverDescriptor = DriverDescriptor {
    name: "my_driver",
    compatible: &["vendor,my-device"],
    // ...
};
```

#### 2.4.4 跨平台代码

平台特定代码必须隔离：

```
hal/
├── arm64/           # ARM64 特定代码
├── x64/             # x86-64 特定代码
├── loongarch64/     # LoongArch64 特定代码
├── snapdragon/      # 高通骁龙平台特定
├── dt.rs            # 设备树（ARM64）
├── acpi.rs          # ACPI（x86_64）
└── platform.rs      # 跨平台检测与分发
```

---

## 三、模块边界定义

### 3.1 HAL 层模块 (L0)

| 模块 | 职责 | 可见性 |
|------|------|--------|
| `hal::cpu` | CPU 抽象 (DVFS, 热管理) | pub trait |
| `hal::gpu` | GPU 抽象 (Maleoon, 命令队列) | pub trait |
| `hal::npu` | NPU 抽象 (达芬奇, ONNX, 推理器) | pub trait |
| `hal::quantum` | 量子密码 (Kyber, Dilithium, QRNG, QKD) | pub trait |
| `hal::power` | 电源管理 (PMIC, 挂起/恢复) | pub trait |
| `hal::platform` | 平台 HAL (dt, acpi) | pub (平台特定) |
| `hal::ffi` | C/C++ FFI 接口 + API 稳定性 | pub (extern "C") |
| `hal::input` | 输入设备 | pub trait |

**边界规则**:
- HAL 模块之间可以相互依赖
- HAL 模块不能依赖 kernel、lib、services、application
- HAL trait 必须定义清晰的接口
- HAL `Cargo.toml` 中 `allowed_deps = []`

### 3.2 内核层模块 (L1)

| 模块 | 职责 | 可见性 |
|------|------|--------|
| `kernel::arch` | 架构抽象 (arm64, x64, loongarch64) | pub (内部) |
| `kernel::mm` | 内存管理 (Buddy, SLAB, VMA, NUMA) | pub API |
| `kernel::process` | 进程管理 (fork, execve, signal) | pub API |
| `kernel::sched` | 调度器 (CFS, EAS, RT) | pub API |
| `kernel::fs` | 文件系统 (VFS, NuvaFS) | pub API |
| `kernel::ipc` | IPC (NuvaIPC, L4, 共享内存) | pub API |
| `kernel::driver` | 驱动框架 (设备模型, DMA, GPIO, I2C, SPI) | pub API |
| `kernel::plugin` | 插件系统 (加载器, 管理器, 沙箱) | pub API |
| `kernel::security` | 安全 (LSM, ASLR, 沙箱, 栈金丝雀) | pub API |
| `kernel::quantum` | 量子支持 (调度器) | pub API |
| `kernel::net` | 网络协议栈 | pub API |
| `kernel::syscall` | 系统调用 | pub API |
| `kernel::sync` | 同步原语 | pub (内部) |

**边界规则**:
- 内核模块之间可以相互依赖（需审查）
- 内核模块可以依赖 HAL
- 内核模块不能依赖 lib、services、application
- 内核 `Cargo.toml` 中 `allowed_deps = ["hal"]`

### 3.3 库层模块 (L2)

| 模块 | 职责 | 可见性 |
|------|------|--------|
| `syslib::ai` | AI/ML 库 | pub |
| `syslib::brain` | Nuva Brain AI 引擎 | pub |
| `syslib::core` | 核心库 | pub |
| `syslib::lang` | NuvaLang 编译器和运行时 | pub |
| `syslib::ml` | 机器学习库 | pub |
| `syslib::net` | 网络库 | pub |
| `syslib::gfx` | 图形库 | pub |
| `syslib::ui` | UI 库 | pub |
| `syslib::data` | 数据结构库 | pub |
| `syslib::std` | 标准库 | pub |
| `syslib::runtime` | 运行时库 | pub |
| `syslib::dispatch` | 并发框架 (GCD 风格) | pub |
| `syslib::posix` | POSIX 兼容层 | pub |

**边界规则**:
- 库模块之间可以相互依赖
- 库模块可以依赖内核 API
- 库模块通过 trait 访问 HAL（不能直接依赖实现）
- 库 `Cargo.toml` 中 `allowed_deps = ["kernel", "hal"]`（HAL 仅限 traits）

### 3.4 服务层模块 (L3)

| 模块 | 职责 | 可见性 |
|------|------|--------|
| `services::app` | 应用服务 (Activity, 包管理器) | pub |
| `services::ipc` | IPC 服务 (Binder, 通道) | pub |
| `services::net` | 网络服务 (DNS, TCP/UDP) | pub |
| `services::power` | 电源服务 (策略, 唤醒锁) | pub |
| `services::security` | 安全服务 (Gatekeeper, Keymaster, TEE) | pub |

**边界规则**:
- 服务模块之间可以相互依赖
- 服务模块可以依赖库和内核
- 服务模块不能依赖应用

### 3.5 应用层模块 (L4)

| 模块 | 职责 | 可见性 |
|------|------|--------|
| `application::ui` | UI 框架 (自适应布局, 组件) | pub |
| `application::window` | 窗口管理 | pub |
| `application::event` | 事件系统 | pub |
| `application::render` | 渲染引擎 (合成器, 画笔) | pub |
| `application::resource` | 资源管理 (缓存, 解码器) | pub |

**边界规则**:
- 应用模块之间可以相互依赖
- 应用模块可以依赖服务、库和内核
- 应用模块是最高层，无反向依赖

---

## 四、dep_analyzer 工具说明

### 4.1 工具概述

`dep_analyzer` 是 Nuva OS 的架构合规检查工具，位于 `tools/dep_analyzer/`，用于分析模块依赖并强制执行分层架构边界。

**功能**：
- 遍历项目中所有 `.rs` 文件，解析 `use` 语句
- 构建模块依赖图 (`DependencyGraph`)
- 检测**层级违规**（低层依赖高层）
- 检测**循环依赖**（DFS 算法）
- 检测**直接跨层依赖**（未通过抽象接口）
- **编译时强制执行**：在 release 模式下通过 `build.rs` 强制执行
- **内核功能域子目录**边界验证
- **重导出验证** — 确保向后兼容的重导出遵循分层规则

### 4.2 违规类型

| 类型 | 说明 | 严重程度 |
|------|------|----------|
| `LayerViolation` | 低层级依赖高层级（如 HAL → Kernel） | P0/P1 |
| `CircularDependency` | 模块间循环依赖 | P1 |
| `DirectDependency` | 跨层直接依赖（未通过 trait） | P2 |
| `SubdirectoryViolation` | 功能域子目录违反层级边界 | P1 |
| `ReExportViolation` | 重导出绕过层级抽象 | P2 |

### 4.3 使用方法

```bash
# 分析项目依赖
cargo run --bin dep_analyzer -- /path/to/nuva

# 成功时输出
# ✅ No dependency violations found!

# 失败时输出具体违规信息并返回非零退出码
```

### 4.4 build.rs 集成

`dep_analyzer` 可通过 `build.rs` 在编译时自动执行依赖检查：

```rust
// build.rs (简化示例)
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

项目 `Cargo.toml` 中提供了 `skip_dep_check` feature 以跳过检查：

```toml
[features]
skip_dep_check = []
```

### 4.5 CI 集成

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

## 五、层级配置文件

各层在 `configs/layers/` 目录下维护独立的 `Cargo.toml` 配置：

| 配置文件 | 层级 | `allowed_deps` | 说明 |
|----------|------|----------------|------|
| `configs/layers/hal/Cargo.toml` | L0 | `[]` | HAL 无外部层依赖 |
| `configs/layers/kernel/Cargo.toml` | L1 | `["hal"]` | 内核仅依赖 HAL |
| `configs/layers/lib/Cargo.toml` | L2 | `["kernel", "hal"]` | 库依赖内核 API + HAL traits |

每个配置文件中通过 `[package.metadata.layer]` 声明层级元数据：

```toml
[package.metadata.layer]
level = 0              # 层级编号
visibility = "public"  # 可见性
allowed_deps = []      # 允许的跨层依赖
```

---

## 六、违规检测与处理

### 6.1 违规处理流程

| 优先级 | 违规类型 | 示例 | 处理时限 |
|--------|----------|------|----------|
| P0 | HAL 依赖内核/库/服务 | `hal::xxx` → `kernel::yyy` | 立即修复 |
| P1 | 内核依赖库/服务 | `kernel::xxx` → `syslib::yyy` | 优先修复 |
| P1 | 循环依赖 | A → B → A | 优先修复 |
| P2 | 库直接依赖 HAL 实现 | `syslib::ai` → `hal::npu::davinci` | 计划修复 |
| P2 | 跨层直接依赖 | 未通过抽象接口 | 计划修复 |

### 6.2 违规示例与修复

#### 违规示例

```rust
// ❌ 错误：syslib/brain 直接依赖 kernel/sched 内部结构
use crate::kernel::sched::task::TaskStruct;

pub fn schedule_task(task: TaskStruct) {
    // ...
}
```

#### 修复方案

```rust
// ✅ 正确：通过抽象接口
pub trait Scheduler {
    fn schedule(&self, task: &TaskInfo) -> Result<(), Error>;
}

pub fn schedule_task<S: Scheduler>(scheduler: &S, task: &TaskInfo) {
    scheduler.schedule(task)?;
}
```

---

## 七、抽象接口规范

### 7.1 HAL 抽象接口

所有 HAL 模块必须提供 trait 接口：

```rust
// hal/npu/traits.rs
pub trait NpuHal: Send + Sync {
    fn initialize(&mut self) -> Result<(), NpuError>;
    fn load_model(&mut self, model: &ModelData) -> Result<ModelId, NpuError>;
    fn execute(&self, model: ModelId, inputs: &[Buffer], outputs: &mut [Buffer]) -> Result<(), NpuError>;
    fn unload_model(&mut self, model: ModelId) -> Result<(), NpuError>;
}
```

### 7.2 内核 API 接口

内核对外提供稳定的 API：

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

### 7.3 服务接口

服务通过接口暴露功能：

```rust
// services/security/interface.rs
pub trait SecurityService {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
}
```

### 7.4 FFI API 稳定性

HAL FFI 层通过 `hal/ffi/stability.rs` 提供 API 版本稳定保证：

- C API (`hal/ffi/c_api/`)：遵循 C99 标准，ABI 稳定
- C++ API (`hal/ffi/cpp_api/`)：RAII 包装，异常安全

---

## 八、依赖图可视化

### 8.1 合规依赖图

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
    └─→ (无外部层依赖)
```

### 8.2 禁止的依赖

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

## 九、实施检查清单

### 9.1 代码审查

- [ ] 所有 `use` 语句符合分层规则
- [ ] 无循环依赖
- [ ] HAL 访问通过 trait 抽象
- [ ] 内核 API 稳定且文档化
- [ ] 服务接口清晰定义
- [ ] FFI API 版本兼容

### 9.2 构建验证

- [ ] `dep_analyzer` 通过
- [ ] 层级配置 `Cargo.toml` 正确
- [ ] 无编译警告
- [ ] 所有测试通过
- [ ] CI 检查通过

### 9.3 文档验证

- [ ] 模块职责文档化
- [ ] 依赖关系文档化
- [ ] 接口文档化
- [ ] 架构图更新

---

**文档状态**: 已定义
**执行状态**: dep_analyzer 已实现并集成到 build.rs (release 模式)；CI 集成待完成
**下一步**: 完整 CI 流水线集成及子目录边界强制执行

---

## 附录 A：代码中额外存在的 L1 内核子模块

以下内核子模块在代码 (`kernel/mod.rs`) 中存在，但未纳入上方核心层结构图（属于辅助/基础设施模块）：

### 功能域子目录（重组后）

| 子目录 | 职责 |
|--------|------|
| `kernel::init` | 初始化（cmdline, config, elf, platform, resource） |
| `kernel::diag` | 诊断（journal, kdebug, log, scanner, stats） |
| `kernel::irq_mgmt` | IRQ 管理（apic_ops, irq, trap） |
| `kernel::net_stack` | 网络协议栈（socket, tcpip） |
| `kernel::storage` | 存储子系统（block） |
| `kernel::device` | 设备模型与插件（device_model, driver_plugin, feature_plugin, module, notifier） |
| `kernel::power_mgmt` | 电源管理（hotplug, pm, power） |
| `kernel::virt` | 虚拟化（vmx） |
| `kernel::core` | 核心服务（cache, cpu, defense, kernel_thread, mempool, perf_tune, posix, random, signal, time, wait, workqueue） |

### 遗留/其他子模块

| 模块 | 职责 |
|------|------|
| `kernel::debug` | 内核调试支持 |
| `kernel::interrupt` | 中断处理 |
| `kernel::timer` | 定时器 |
| `kernel::perf` | 性能监控 |
| `kernel::bsd` | BSD 兼容 |
| `kernel::user` | 用户管理 |
| `kernel::apic_ops` | APIC 操作 |

这些模块均属于 L1 内核层，遵守 L1 边界规则（仅依赖 L0 HAL，不依赖 L2/L3/L4）。
