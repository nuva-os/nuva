# Nuva OS 编码规范

## 概述

本文档定义了 Nuva OS 开发的编码规范和约定。遵循这些规范可确保代码的一致性、可读性和可维护性。Nuva OS 是 `no_std` 内核项目，有额外的编码约束。

---

## 目录

1. [通用原则](#1-通用原则)
2. [Rust 风格指南](#2-rust-风格指南)
3. [no_std 编码规范](#3-no_std-编码规范)
4. [命名约定](#4-命名约定)
5. [代码组织](#5-代码组织)
6. [条件编译风格](#6-条件编译风格)
7. [文档与注释](#7-文档与注释)
8. [错误处理](#8-错误处理)
9. [unsafe 使用准则](#9-unsafe-使用准则)
10. [内存安全](#10-内存安全)
11. [并发](#11-并发)
12. [分层架构编码约束](#12-分层架构编码约束)
13. [性能考量](#13-性能考量)
14. [测试](#14-测试)
15. [Nuva 语言声明式编程范式](#15-nuva-语言声明式编程范式)
16. [最佳实践](#16-最佳实践)
17. [工具](#17-工具)

---

## 1. 通用原则

### 1.1 代码质量

- **可读性**：代码应自文档化且易于理解
- **简洁性**：避免过度工程和不必要的复杂性
- **一致性**：遵循既有的模式和约定
- **正确性**：确保代码正确并处理边界情况

### 1.2 设计哲学

- **Unix 原则**：小而专注的模块，做好一件事
- **策略与机制分离**：kernel 提供机制，用户空间决定策略
- **一切皆文件**：尽可能使用统一接口
- **快速失败**：尽早检测和报告错误

### 1.3 禁止模式 — Android 兼容代码

以下 Android 风格的模式在新增代码中**禁止**使用，必须替换为 Nuva 原生等价实现：

| 禁止模式 | Nuva 原生替代 |
|----------|--------------|
| `BinderService`、`BinderNode`、`BinderTransaction` | `NuvaIpcService` 配合 `PortManager` |
| `ActivityState`（6 状态生命周期） | `NuvaLifecycleState`（4 状态） |
| `ActivityManager`、`start_activity()` | `NuvaAppLifecycleManager`、`launch_app()` |
| `Permission` 枚举、`PermissionManagerService` | `NuvaCapabilityManager` 配合 `CapSet` |
| `SecurityOps` C 函数指针表 | `SecurityHook` trait |
| `LegacySecurityModule` | `SecurityModule`（来自 `security_hook`） |
| `NetlinkProtocol::Selinux` | `NetlinkProtocol::NsmAudit` |
| `PolicyType::CpufreqLimit` 等 | `NuvaPolicyType::CpuFreqThrottle` 等 |
| `PackageFormat::AndroidApk` | 仅 NPK（`.npk`） |
| `INSTALL_FLAG_FROM_ADB` | `INSTALL_FLAG_FROM_CLI` |
| `UnifiedKey::Back`（Android 风格） | `UnifiedKey::NavigateBack` |
| `static mut` 全局变量配合 `unsafe &mut` | `OnceLock` + 内部可变性 |
| Android DPI 缩写（ldpi/mdpi/hdpi） | Nuva 原生密度层级描述 |
| `View` trait、`BaseView`、`RenderContext`（UI） | `Component` trait + `Element`（声明式） |
| `Button`（命令式）、`ButtonState` | `Button` 组件（声明式，ComponentProps） |
| `Widget`、`WidgetTree`、`WidgetId` | `Element` 树 + `Reconciler` diff |
| `ActivityManager`、`ActivityState` | `ScreenLifecycleManager`、`NuvaScreenState` |
| `WindowManager`、`Window`（命令式） | `DeclarativeWindowManager`、`DeclarativeWindow` |
| `EventDispatcher`、`EventHandler`（命令式） | `DeclarativeEventDispatcher`（Modifier 绑定） |
| `RenderContext`/`Painter`/`Compositor`（命令式） | `RenderPipeline` + `DeclarativeCompositor` |
| `ResourceManager`/`ResourceCache`（命令式） | `DeclarativeResourceManager`（声明式） |

---

## 2. Rust 风格指南

### 2.1 格式化

使用 `rustfmt` 默认设置：

```bash
cargo fmt
```

### 2.2 代码检查

使用 `clippy` 捕获常见错误，`no_std` 项目需指定目标：

```bash
cargo clippy --target aarch64-unknown-none
```

### 2.3 缩进

- 使用 4 个空格缩进
- 不使用 Tab

### 2.4 行长度

- 最大行长度：100 个字符
- 在逻辑断点处断行

### 2.5 大括号

- 函数、结构体、枚举的左大括号在同一行
- 代码块的左大括号在新行

```rust
pub fn function_name() -> Result<()> {
    // ...
}

if condition {
    // ...
} else {
    // ...
}
```

---

## 3. no_std 编码规范

### 3.1 禁止标准库依赖

所有 kernel 和 HAL 代码必须使用 `#![no_std]`。禁止引入依赖 `std` 的 crate。

```rust
#![no_std]
```

### 3.2 使用 core 和 alloc 替代 std

| std 类型 | no_std 替代 |
|----------|-------------|
| `std::sync::Arc` | `alloc::sync::Arc` |
| `std::vec::Vec` | `alloc::vec::Vec` |
| `std::string::String` | `alloc::string::String` |
| `std::boxed::Box` | `alloc::boxed::Box` |
| `std::sync::atomic` | `core::sync::atomic` |
| `std::mem` | `core::mem` |
| `std::ptr` | `core::ptr` |
| `std::fmt` | `core::fmt` |

### 3.3 自定义全局分配器

Nuva OS 在 `kernel/main.rs` 中定义了 `#[global_allocator]`，使用 Slab + Buddy 混合分配器。禁止使用 `Box::new([...])` 等隐式依赖 `std` 分配器的写法。

### 3.4 Panic Handler

`no_std` 项目必须定义 `#[panic_handler]`：

```rust
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    log_error!("KERNEL PANIC!");
    loop {
        core::hint::spin_loop();
    }
}
```

### 3.5 构建标准库

项目 `.cargo/config.toml` 配置了 `build-std`：

```toml
[unstable]
build-std = ["core", "compiler_builtins", "alloc"]
build-std-features = ["compiler-builtins-mem"]
```

修改此配置需谨慎，确保所有目标平台均可构建。

---

## 4. 命名约定

### 4.1 通用规则

- 使用能传达意图的描述性名称
- 避免缩写，除非广为人知
- 保持命名风格一致

### 4.2 变量和函数

- 变量和函数使用 `snake_case`
- 使用描述性名称

```rust
let page_count = calculate_page_count(size);

let pc = calc_pg_cnt(s);
```

### 4.3 类型

- 结构体、枚举和类型别名使用 `PascalCase`

```rust
pub struct PageTable {}

pub enum PageFaultResult {
    Success,
    Failure,
}
```

### 4.4 常量

- 常量使用 `SCREAMING_SNAKE_CASE`

```rust
pub const PAGE_SIZE: usize = 4096;
pub const MAX_CPUS: usize = 256;
```

### 4.5 缩写

- 将缩写视为普通单词（例如 `Http` 而非 `HTTP`）

```rust
pub struct HttpClient {}
pub fn parse_xml() {}

pub struct HTTPClient {}
pub fn parseXML() {}
```

### 4.6 Feature Flag 命名

Feature flag 使用 `snake_case`，与平台/硬件名称对应：

```toml
kirin9020 = ["arm64", "kirin"]
snapdragon8gen4 = ["arm64"]
intel_core = ["x64"]
amd_ryzen = ["x64"]
loongson3a6000 = ["loongarch64"]
```

---

## 5. 代码组织

### 5.1 模块结构

Nuva OS 采用分层架构，代码组织如下：

```
nuva/
├── kernel/          # 内核核心（no_std）
│   ├── arch/        # 架构相关代码
│   │   ├── arm64/
│   │   ├── x64/
│   │   └── loongarch64/
│   ├── mm/          # 内存管理
│   ├── sched/       # 进程调度
│   ├── fs/          # 文件系统
│   ├── net/         # 网络协议栈
│   ├── sync/        # 同步原语
│   ├── syscall/     # 系统调用
│   ├── security/    # 安全模块
│   ├── plugin/      # 插件系统
│   └── quantum/     # 量子安全调度
├── hal/             # 硬件抽象层
│   ├── arm64/
│   ├── x64/
│   ├── loongarch64/
│   ├── npu/         # NPU 驱动
│   ├── gpu/         # GPU 驱动
│   ├── quantum/     # 量子安全（PQC/QRNG）
│   └── power/       # 电源管理
├── syslib/          # 系统库
├── posix/           # POSIX 兼容层
├── fs/              # 文件系统实现（NovaFS）
├── sdk/             # 开发工具链 SDK
└── services/        # 用户空间服务
```

### 5.2 模块声明

在父模块的 `mod.rs` 中使用 `mod` 声明：

```rust
pub mod memory;
pub mod buddy;
pub mod slab;

pub use self::memory::*;
```

### 5.3 可见性

- 谨慎使用 `pub`，仅在必要时使用
- 默认优先私有
- 将相关的 pub 项放在一起

### 5.4 导入

- 在文件顶部分组导入
- 外部 crate 使用绝对路径
- 内部模块使用相对路径

```rust
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::mm::memory::*;
use super::scheduler::*;
```

---

## 6. 条件编译风格

### 6.1 架构条件编译

使用 `cfg(target_arch)` 进行架构分支：

```rust
#[cfg(target_arch = "aarch64")]
fn arch_specific_code() {
    // ARM64 implementation
}

#[cfg(target_arch = "x86_64")]
fn arch_specific_code() {
    // x86-64 implementation
}

#[cfg(target_arch = "loongarch64")]
fn arch_specific_code() {
    // LoongArch64 implementation
}
```

### 6.2 Feature 条件编译

使用 `cfg(feature = "...")` 进行平台/功能分支：

```rust
#[cfg(feature = "kirin9020")]
fn init_platform() {
    init_kirin9020();
}

#[cfg(feature = "smp")]
fn init_smp() {
    // Multi-core initialization
}
```

### 6.3 条件编译规则

- **禁止**在条件编译中使用 `not(...)` 做默认分支（除非有明确的 fallback）
- 每个架构分支都必须有完整实现，不能留空
- 条件编译块应保持简短，复杂逻辑提取到独立函数
- 优先使用 trait 对象和泛型替代条件编译

---

## 7. 文档与注释

### 7.1 注释语言：英文

所有代码注释、文档注释、commit message 必须使用**英文**。这是国际化协作项目的基本要求。

```rust
/// Allocates a single physical page.
///
/// # Arguments
///
/// * `flags` - Allocation flags (e.g., GFP_KERNEL, GFP_ATOMIC)
pub fn alloc_page(flags: GfpFlags) -> Option<PhysAddr> {
    // Check free list first
}
```

### 7.2 模块文档

使用 `//!` 为每个模块编写文档：

```rust
//! # Memory Management Module
//!
//! This module provides physical and virtual memory management
//! functionality including page allocation, memory mapping, and
//! memory protection.
```

### 7.3 函数文档

使用 `///` 为公共函数编写文档：

```rust
/// Allocates a single physical page.
///
/// # Arguments
///
/// * `flags` - Allocation flags (e.g., GFP_KERNEL, GFP_ATOMIC)
///
/// # Returns
///
/// * `Some(PhysAddr)` - Physical address of allocated page
/// * `None` - Allocation failed
///
/// # Examples
///
/// ```
/// if let Some(page) = alloc_page(GFP_KERNEL) {
///     // Use the page
/// }
/// ```
pub fn alloc_page(flags: GfpFlags) -> Option<PhysAddr> {
    // ...
}
```

### 7.4 结构体文档

为公共结构体编写文档：

```rust
/// Represents a physical memory page.
///
/// Each page is 4KB in size and contains metadata
/// for tracking its state and usage.
pub struct Page {
    /// Page flags (dirty, referenced, etc.)
    pub flags: AtomicU32,

    /// Reference count for shared pages
    pub ref_count: AtomicU32,

    /// Physical address of this page
    pub phys_addr: PhysAddr,
}
```

### 7.5 行内注释

谨慎使用行内注释：

```rust
// Calculate virtual runtime for CFS
let vruntime = delta_exec * NICE_0_LOAD / se.load;
```

### 7.6 SAFETY 注释

所有 `unsafe` 块必须附带 `// SAFETY:` 注释说明安全不变量：

```rust
// SAFETY: The caller has verified that the address is mapped
// and properly aligned. No concurrent access is possible
// because interrupts are disabled.
unsafe {
    core::ptr::write_volatile(ptr, value);
}
```

---

## 8. 错误处理

### 8.1 使用 Result 类型

可失败的操作使用 `Result<T, E>`：

```rust
pub fn allocate_page() -> Result<PhysAddr, AllocError> {
    // ...
}
```

### 8.2 自定义错误类型

定义自定义错误类型，使用 `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
    InvalidArgument,
    PermissionDenied,
}
```

### 8.3 错误传播

使用 `?` 运算符进行错误传播：

```rust
pub fn allocate_pages(count: usize) -> Result<Vec<PhysAddr>, AllocError> {
    let mut pages = Vec::new();
    for _ in 0..count {
        pages.push(alloc_page()?);
    }
    Ok(pages)
}
```

### 8.4 Panic 的使用

- **严禁**在 kernel 生产代码中使用 `panic!`、`unwrap()` 或 `expect()`
- 仅在测试代码和不变量必定成立时可使用
- 优先使用优雅的错误处理

```rust
let page = alloc_page().ok_or(AllocError::OutOfMemory)?;

let page = alloc_page().unwrap();
```

#### 8.4.1 Panic 消除规范

所有内核生产路径必须无 panic。以下替换是强制性的：

| 禁止用法 | 替换方法 | 示例 |
|---------|---------|------|
| `unwrap()` | `ok_or()` + `?` | `x.ok_or(KernelError::InvalidState)?` |
| `expect(msg)` | `ok_or()` + `?` + log | `x.ok_or_else(|| { log_error!("..."); KernelError::InvalidState })?` |
| `panic!(msg)` | `return Err(...)` | `return Err(KernelError::InternalError)` |
| `assert!(cond)` | `if !cond { return Err }` | `if !valid { return Err(KernelError::InvalidArgument) }` |
| `unreachable!()` | `return Err(KernelError::InvalidState)` | 防御性错误返回 |

**执行要求**：代码审查必须拒绝非测试内核路径中的任何 `panic!`/`unwrap()`/`expect()`。`main.rs` 中的 `#[panic_handler]` 是唯一允许的 panic 定义。

### 8.5 统一内核错误类型

对所有内核操作使用统一的 `KernelError` 枚举（定义在 `kernel/error.rs` 中）：

```rust
use crate::kernel::error::{KernelError, KernelResult};

/** 使用统一错误处理分配页面 */
pub fn alloc_pages(order: u32) -> KernelResult<*mut Page> {
    if order > MAX_ORDER {
        return Err(KernelError::InvalidArgument);
    }
    // ...
}
```

`KernelError` 提供：
- 7 个错误分类：内存、调度器、IPC、驱动、文件系统、同步、安全
- 用于 POSIX errno 映射的 `to_errno()` 方法
- `is_recoverable()` 和 `is_user_error()` 分类方法
- 便捷的 `KernelResult<T>` 类型别名

### 8.6 分配约束检查

持有 SpinLock 时或在 IRQ 上下文中**禁止**进行内存分配。请使用 `kmalloc!` 或 `kmalloc_result!` 宏：

```rust
// 如果分配被禁止则返回 None
let ptr = kmalloc!(alloc::alloc::alloc(layout));

// 如果分配被禁止则返回 Err(KernelError::DeadlockDetected)
let ptr = kmalloc_result!(alloc::alloc::alloc(layout))?;
```

---

## 9. unsafe 使用准则

### 9.1 最小化 unsafe

- 尽量减少 `unsafe` 的使用范围
- 将 `unsafe` 块限制在最小必要范围内
- 每个 `unsafe` 块都必须有 `// SAFETY:` 注释

### 9.2 unsafe 函数文档

`unsafe` 函数必须使用 `# Safety` 段记录安全前提：

```rust
/// Reads a byte from the given physical address.
///
/// # Safety
///
/// The caller must ensure that:
/// - The address is valid and mapped
/// - The address is aligned to 1 byte
/// - No concurrent mutable access exists
pub unsafe fn read_phys_byte(addr: PhysAddr) -> u8 {
    // ...
}
```

### 9.3 典型 unsafe 使用场景

在 Nuva OS 中，`unsafe` 合理使用的场景包括：

- 硬件 MMIO 寄存器读写（`core::ptr::read_volatile` / `write_volatile`）
- 内联汇编（`core::arch::asm!`）
- 页表操作
- 全局静态可变状态访问
- FFI 调用（如 PQC C 库绑定）
- DMA 缓冲区管理

### 9.4 禁止的 unsafe 用法

- 禁止使用 `unsafe` 绕过借用检查器（除非有充分文档说明）
- 禁止从 `unsafe` 函数返回裸指针而不封装安全接口
- 禁止在 `unsafe` 块中进行不相关的操作

### 9.5 SAFETY 注释标准

每个 `unsafe` 块**必须**在其前面有 `// SAFETY:` 注释，解释所保持的具体安全不变量。通用占位注释如"unsafe block required for low-level memory or hardware access"**不可接受**。

SAFETY 注释必须说明：
1. **为什么** unsafe 操作在此特定上下文中是安全的
2. **哪些不变量**正在被保持（指针有效性、无别名、无数据竞争等）
3. **哪些前置条件**必须成立（例如"调用者确保 ptr 非空"）

附加要求：
- 每个 `unsafe fn` 必须包含 `# Safety` 文档节说明前置条件
- `unsafe impl` 必须包含 `// SAFETY:` 注释说明如何保持 trait 契约
- SAFETY 注释必须引用具体不变量（例如"指针非空""无并发可变访问"），而非模糊理由
- 当 unsafe 块依赖外部状态时，注释必须标识哪个状态以及为何处于预期配置

**可接受的示例：**
```rust
// SAFETY: free_list 指针在 grow() 中或前一次 free() 调用时设置。
// 它指向一个有效的 slab 对象。我们读取该对象的第一个字，其中存储
// 下一个空闲指针（空闲链表链接模式）。这是安全的因为：
// 1. obj 非空（上面已检查）
// 2. obj 指向有效的 slab 对象（free_list 的不变量）
// 3. 第一个指针大小的字存储了下一个指针
unsafe {
    self.free_list = *(obj as *const *mut u8);
}
```

**不可接受的示例：**
```rust
// SAFETY: unsafe block required for low-level memory or hardware access
unsafe {
    self.free_list = *(obj as *const *mut u8);
}
```

### 9.6 SAFETY Lint 工具

运行 `tools/safety_lint.sh` 扫描 `kernel/` 和 `hal/` 中的所有 `.rs` 文件，检查缺失的 SAFETY 注释。该脚本以 `file:line` 格式报告违规项，如果发现任何违规则以非零状态退出。

---

## 10. 内存安全

### 10.1 裸指针

- 优先使用引用而非裸指针
- 使用裸指针时，记录安全要求

```rust
pub fn get_page(page_num: usize) -> Option<&'static Page> {
    // ...
}

pub unsafe fn get_page_raw(page_num: usize) -> *mut Page {
    // ...
}
```

### 10.2 内存分配

- 使用 kernel 分配器，而非标准库
- 检查分配结果

```rust
let page = alloc_page(GFP_KERNEL)?;
let buffer = kmalloc(size, GFP_KERNEL)?;

let buffer = Box::new([0u8; 4096]);
```

### 10.3 Volatile 访问

MMIO 寄存器必须使用 `read_volatile` / `write_volatile`：

```rust
// SAFETY: MMIO register access, address is fixed by hardware spec
unsafe {
    core::ptr::write_volatile(uart_base.add(UART_CR as usize / 4), 0x301);
}
```

---

## 11. 并发

### 11.1 锁顺序

- 建立并记录锁顺序
- 始终以相同顺序获取锁
- 尽可能避免嵌套锁

### 11.2 原子操作

- 对简单计数器使用原子操作
- 使用正确的内存序

```rust
use core::sync::atomic::{AtomicU32, Ordering};

pub struct Page {
    pub ref_count: AtomicU32,
}

impl Page {
    pub fn increment_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }
}
```

### 11.3 自旋锁

- 对短临界区使用自旋锁
- 持有自旋锁时禁止睡眠或调度

```rust
use crate::sync::SpinLock;

pub struct PageTable {
    lock: SpinLock,
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn lookup(&self, index: usize) -> PageTableEntry {
        let _guard = self.lock.lock();
        self.entries[index]
    }
}
```

### 11.4 中断安全

- 在必要时禁用中断
- 使用中断安全锁

```rust
use crate::arch::interrupts::disable_irqs;
use crate::arch::interrupts::enable_irqs;

pub fn critical_section() {
    let flags = disable_irqs();
    // Critical code here
    enable_irqs(flags);
}
```

---

## 12. 分层架构编码约束

Nuva OS 采用分层架构，各层之间有严格的依赖规则。

### 12.1 层级定义

```
┌─────────────────────┐
│    Application      │  用户空间应用
├─────────────────────┤
│    Services         │  系统服务
├─────────────────────┤
│    SysLib / POSIX   │  系统库 / 兼容层
├─────────────────────┤
│    Kernel           │  内核（调度/内存/FS/网络）
├─────────────────────┤
│    HAL              │  硬件抽象层
├─────────────────────┤
│    Hardware         │  硬件
└─────────────────────┘
```

### 12.2 依赖规则

- **Kernel** 可以依赖 **HAL**，但禁止反向依赖
- **HAL** 不得直接引用 `kernel/` 中的符号
- **SysLib** 只能通过系统调用接口访问 Kernel
- **POSIX** 层仅封装 SysLib，不直接调用 Kernel
- **Plugin** 系统通过注册接口与 Kernel 交互，不直接修改内核数据结构

### 12.3 跨层调用规范

- 上层调用下层必须通过明确定义的 trait 接口
- 禁止跨层直接访问内部数据结构
- HAL trait 定义在 `hal/` 中，Kernel 通过 `dyn trait` 使用

---

## 13. 性能考量

### 13.1 热路径

- 优化频繁执行的代码
- 对小型热函数使用内联

```rust
#[inline]
pub fn is_page_free(page: &Page) -> bool {
    page.ref_count.load(Ordering::Acquire) == 0
}
```

### 13.2 缓存效率

- 考虑缓存局部性
- 适当时使用 Per-CPU 数据结构

```rust
#[repr(C)]
pub struct Page {
    pub flags: AtomicU32,
    pub ref_count: AtomicU32,
    pub lru_next: *mut Page,
    pub lru_prev: *mut Page,
}
```

### 13.3 内存分配

- 小对象（<= 4096 字节）使用 Slab 分配器
- 大对象使用 Buddy 分配器

```rust
let task = slab_alloc::<Task>()?;

let pages = buddy_alloc(PAGE_SIZE * 10)?;
```

### 13.4 避免过早优化

- 先做性能分析再优化
- 关注实际瓶颈
- 保持可读性

---

## 14. 测试

### 14.1 单元测试

为模块编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_allocation() {
        let page = alloc_page(GFP_KERNEL);
        assert!(page.is_some());
    }

    #[test]
    fn test_page_free() {
        let page = alloc_page(GFP_KERNEL).unwrap();
        free_page(page);
    }
}
```

### 14.2 集成测试

为模块交互编写集成测试：

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_memory_mapping() {
        let vma = create_vma(0x1000, 0x2000)?;
        let page = alloc_page(GFP_KERNEL)?;
        map_page(&vma, page)?;
    }
}
```

### 14.3 Kernel 测试

使用 kernel 测试框架：

```rust
#[test_case]
fn test_scheduler() {
    let scheduler = Scheduler::new();
    assert_eq!(scheduler.cpu_count(), 8);
}
```

### 14.4 测试覆盖率

- 追求高测试覆盖率
- 测试边界情况和错误路径
- 保持测试可维护

---

## 15. Nuva 语言声明式编程范式

### 15.1 源文件扩展名

所有 Nuva 语言源文件必须使用 `.nv` 扩展名。这将 Nuva 源文件与项目中的 Rust（`.rs`）和其他语言文件区分开来。

```
ui/
├── main_screen.nv       # Nuva 源文件
├── components/
│   ├── button.nv
│   └── card.nv
└── styles/
    └── theme.nv
```

### 15.2 声明式 UI 组件

使用 `component` 关键字定义 UI 组件。组件是声明式的——它们描述 UI *应该* 是什么样，而不是*如何*构建它。

```nuva
component Button(text: String, onClick: () -> void) {
    Column {
        Text(text)
            .font_size(16)
            .padding(8)
    }
    .on_click(onClick)
}
```

规则：
- 组件必须使用 `component` 关键字声明
- 组件名使用 `PascalCase`
- Props 在参数列表中声明，带显式类型
- 子元素使用 `{ }` 代码块以树形结构声明
- 修饰符使用 `.`（点）语法链式调用
- 不允许命令式的 `new()` 或 `build()` 调用——框架负责对树进行协调更新

### 15.3 响应式数据（signal/effect）

使用 `signal` 声明响应式状态，使用 `effect` 声明副作用，当依赖发生变化时自动重新执行。

```nuva
signal count: Int = 0

effect {
    // 当 `count` 变化时自动重新运行
    console.log("Count is now: " + count)
}
```

规则：
- `signal` 声明一个具有自动变更传播的响应式变量
- `effect` 注册一个具有自动依赖跟踪的副作用
- 在 `effect` 体内读取 signal 会自动被跟踪为依赖
- 直接修改 signal 会触发所有依赖的 effect
- 避免在 effect 体中执行 I/O，除非 effect 显式标记为 `io`

### 15.4 声明式并发（async/await）

使用 `async`/`await` 进行声明式异步计算。编译器将异步函数转换为状态机。

```nuva
async fn fetch_data(url: String) -> Result<Data, Error> {
    let response = await http.get(url)
    let data = await response.json()
    return data
}
```

规则：
- `async` 将函数标记为返回 `Future<T>`
- `await` 挂起执行直到 `Future` 完成
- 异步函数被编译为状态机 IR
- 永远不要在异步上下文中阻塞响应式调度器——改用 `await`

### 15.5 声明式资源管理（resource/with）

使用 `resource` 进行声明式资源获取，使用 `with` 进行带自动清理的作用域资源管理。

```nuva
resource FileHandle(path: String) {
    acquire: fs.open(path, READ),
    release: handle.close()
}

with (handle = FileHandle("/data/config.json")) {
    let content = handle.read_all()
    process(content)
}
// handle.close() 在作用域退出时自动调用
```

规则：
- `resource` 声明一个带有 `acquire` 和 `release` 阶段的资源类型
- `with` 创建作用域绑定——资源在作用域退出时自动释放
- 即使发生异常，释放也能保证执行（RAII 语义）
- 资源不能泄漏到其 `with` 作用域之外

### 15.6 禁止的命令式模式

以下命令式模式在 `.nv` 文件中**禁止**使用：

| 禁止模式 | 声明式替代 |
|----------|-----------|
| `new Widget()`、`widget.build()` | `component` 声明 |
| `setState()`、`notifyStateChanged()` | 带自动传播的 `signal` |
| `addEventListener()`、`removeEventListener()` | Modifier 绑定的 `.on_click()`、`.on_change()` |

### 15.7 声明式驱动范式（kernel/driver/declarative）

声明式驱动模型将声明式范式扩展到内核驱动代码：

| 宏 | 用途 | 示例 |
|----|------|------|
| `declare_driver!` | 带元数据的静态驱动注册 | `declare_driver! { MY_DRV { name: "my_drv", compatible: &["v,d"], ... } }` |
| `declare_resource!` | 声明式资源获取/释放 | `declare_resource! { MY_RES { name: "irq", resource_type: Irq, optional: false } }` |
| `declare_pm!` | 声明式电源状态机 | `declare_pm! { MY_PM { On => Idle: 10us, ... } }` |

规则：
- 所有驱动注册必须使用 `declare_driver!`——禁止手动构造 `DriverDescriptor`
- 所有电源管理必须使用 `declare_pm!`——禁止手动构造状态机
- 所有资源绑定必须使用 `declare_resource!`——禁止手动 acquire/release 对
- 所有新驱动必须实现 `DeclarativeDriver` trait
- `CompatibleHashTable`（`kernel/driver/matching`）提供 O(1) 设备-驱动匹配

### 15.8 声明式调度器范式

调度器支持声明式策略配置：

```rust
pub struct SchedPolicyConfig {
    pub policy: SchedPolicy,
    pub min_granularity_ns: u64,
    pub latency_ns: u64,
    pub wakeup_granularity_ns: u64,
}
```

- 策略变更通过 `SchedPolicyConfig` 热更新以原子方式应用
- 每 CPU 运行队列（`PerCpuRunQueue`）按缓存行对齐，防止伪共享

---

## 16. 最佳实践

### 16.1 代码评审

- 所有代码在合并前都应经过评审
- 评审正确性、风格和设计
- 反馈应具有建设性

### 16.2 版本控制

- 频繁提交并使用清晰的提交信息
- 遵循约定式提交格式

```
feat(mm): add slab allocator

Implements slab allocator for small object allocation
with automatic reclamation and per-CPU caches.
```

### 16.3 文档更新

- 代码变更时更新文档
- 保持 README 和 API 文档同步
- 记录设计决策

### 16.4 向后兼容

- 避免破坏公共 API
- 对变更使用弃用警告
- 记录迁移路径

---

## 17. 工具

### 17.1 预提交钩子

使用预提交钩子强制执行规范：

```bash
cargo install cargo-husky
cargo husky install
```

### 17.2 CI/CD

配置 CI 运行检查：

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
          components: rust-src, clippy, rustfmt
      - run: rustup target add aarch64-unknown-none x86_64-unknown-none
      - run: cargo fmt -- --check
      - run: cargo clippy --target aarch64-unknown-none -- -D warnings
      - run: cargo test --target aarch64-unknown-none
```

---

## 总结

遵循这些编码规范可确保：

- **一致性**：代码在整个项目中看起来和感觉一致
- **质量**：更少的 Bug 和更好的可维护性
- **协作**：他人更容易理解和贡献
- **性能**：在关键处优化代码
- **安全**：`unsafe` 使用受到严格控制

请记住：这些是指导原则，而非绝对规则。运用判断力和常识。如有疑问，优先考虑可读性和安全性。

---

<!-- Translation Status: Chinese Translation | Last Updated: 2026-05-22 | Synchronized with English version -->

**最后更新**：2026 年 5 月 22 日
