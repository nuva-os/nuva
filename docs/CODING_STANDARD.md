# Nuva OS Coding Standard

## Overview

This document defines the coding standards and conventions for Nuva OS development. Following these standards ensures code consistency, readability, and maintainability. Nuva OS is a `no_std` kernel project with additional coding constraints.

---

## Table of Contents

1. [General Principles](#1-general-principles)
2. [Rust Style Guide](#2-rust-style-guide)
3. [no_std Coding Conventions](#3-no_std-coding-conventions)
4. [Naming Conventions](#4-naming-conventions)
5. [Code Organization](#5-code-organization)
6. [Conditional Compilation Style](#6-conditional-compilation-style)
7. [Documentation and Comments](#7-documentation-and-comments)
8. [Error Handling](#8-error-handling)
9. [unsafe Usage Guidelines](#9-unsafe-usage-guidelines)
10. [Memory Safety](#10-memory-safety)
11. [Concurrency](#11-concurrency)
12. [Layered Architecture Coding Constraints](#12-layered-architecture-coding-constraints)
13. [Performance Considerations](#13-performance-considerations)
14. [Testing](#14-testing)
15. [Nuva Language Declarative Programming Paradigm](#15-nuva-language-declarative-programming-paradigm)
16. [Best Practices](#16-best-practices)
17. [Tools](#17-tools)

---

## 1. General Principles

### 1.1 Code Quality

- **Readability**: Code should be self-documenting and easy to understand
- **Simplicity**: Avoid over-engineering and unnecessary complexity
- **Consistency**: Follow established patterns and conventions
- **Correctness**: Ensure code is correct and handles edge cases

### 1.2 Design Philosophy

- **Unix Philosophy**: Small, focused modules that do one thing well
- **Separation of Policy and Mechanism**: The kernel provides mechanism; user space decides policy
- **Everything is a File**: Use unified interfaces whenever possible
- **Fail Fast**: Detect and report errors as early as possible

### 1.3 Prohibited Patterns — Android Compatibility Code

The following Android-style patterns are **prohibited** in new code and must be replaced with Nuva-native equivalents:

| Prohibited Pattern | Nuva-Native Replacement |
|-------------------|------------------------|
| `BinderService`, `BinderNode`, `BinderTransaction` | `NuvaIpcService` with `PortManager` |
| `ActivityState` (6-state lifecycle) | `NuvaLifecycleState` (4-state) |
| `ActivityManager`, `start_activity()` | `NuvaAppLifecycleManager`, `launch_app()` |
| `Permission` enum, `PermissionManagerService` | `NuvaCapabilityManager` with `CapSet` |
| `SecurityOps` C function pointer table | `SecurityHook` trait |
| `LegacySecurityModule` | `SecurityModule` (from `security_hook`) |
| `NetlinkProtocol::Selinux` | `NetlinkProtocol::NsmAudit` |
| `PolicyType::CpufreqLimit` etc. | `NuvaPolicyType::CpuFreqThrottle` etc. |
| `PackageFormat::AndroidApk` | NPK only (`.npk`) |
| `INSTALL_FLAG_FROM_ADB` | `INSTALL_FLAG_FROM_CLI` |
| `UnifiedKey::Back` (Android-style) | `UnifiedKey::NavigateBack` |
| `static mut` globals with `unsafe &mut` | `OnceLock` + interior mutability |
| Android DPI abbreviations (ldpi/mdpi/hdpi) | Nuva-native density tier descriptions |
| `View` trait, `BaseView`, `RenderContext` (UI) | `Component` trait + `Element` (declarative) |
| `Button` (imperative), `ButtonState` | `Button` component (declarative, ComponentProps) |
| `Widget`, `WidgetTree`, `WidgetId` | `Element` tree + `Reconciler` diff |
| `ActivityManager`, `ActivityState` | `ScreenLifecycleManager`, `NuvaScreenState` |
| `WindowManager`, `Window` (imperative) | `DeclarativeWindowManager`, `DeclarativeWindow` |
| `EventDispatcher`, `EventHandler` (imperative) | `DeclarativeEventDispatcher` (Modifier-bound) |
| `RenderContext`/`Painter`/`Compositor` (imperative) | `RenderPipeline` + `DeclarativeCompositor` |
| `ResourceManager`/`ResourceCache` (imperative) | `DeclarativeResourceManager` (declarative) |

---

## 2. Rust Style Guide

### 2.1 Formatting

Use `rustfmt` with default settings:

```bash
cargo fmt
```

### 2.2 Linting

Use `clippy` to catch common mistakes. For `no_std` projects, specify the target:

```bash
cargo clippy --target aarch64-unknown-none
```

### 2.3 Indentation

- Use 4 spaces for indentation
- Do not use tabs

### 2.4 Line Length

- Maximum line length: 100 characters
- Break lines at logical points

### 2.5 Braces

- Opening braces for functions, structs, and enums on the same line
- Opening braces for code blocks on a new line

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

## 3. no_std Coding Conventions

### 3.1 No Standard Library Dependencies

All kernel and HAL code must use `#![no_std]`. Importing crates that depend on `std` is prohibited.

```rust
#![no_std]
```

### 3.2 Use core and alloc Instead of std

| std Type | no_std Replacement |
|----------|-------------------|
| `std::sync::Arc` | `alloc::sync::Arc` |
| `std::vec::Vec` | `alloc::vec::Vec` |
| `std::string::String` | `alloc::string::String` |
| `std::boxed::Box` | `alloc::boxed::Box` |
| `std::sync::atomic` | `core::sync::atomic` |
| `std::mem` | `core::mem` |
| `std::ptr` | `core::ptr` |
| `std::fmt` | `core::fmt` |

### 3.3 Custom Global Allocator

Nuva OS defines `#[global_allocator]` in `kernel/main.rs` using a Slab + Buddy hybrid allocator. Implicit reliance on the `std` allocator (e.g., `Box::new([...])`) is prohibited.

### 3.4 Panic Handler

`no_std` projects must define `#[panic_handler]`:

```rust
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    log_error!("KERNEL PANIC!");
    loop {
        core::hint::spin_loop();
    }
}
```

### 3.5 Building the Standard Library

The project's `.cargo/config.toml` configures `build-std`:

```toml
[unstable]
build-std = ["core", "compiler_builtins", "alloc"]
build-std-features = ["compiler-builtins-mem"]
```

Modifying this configuration requires caution to ensure all target platforms can build.

---

## 4. Naming Conventions

### 4.1 General Rules

- Use descriptive names that convey intent
- Avoid abbreviations unless widely known
- Maintain consistent naming style

### 4.2 Variables and Functions

- Variables and functions use `snake_case`
- Use descriptive names

```rust
let page_count = calculate_page_count(size);

let pc = calc_pg_cnt(s);
```

### 4.3 Types

- Structs, enums, and type aliases use `PascalCase`

```rust
pub struct PageTable {}

pub enum PageFaultResult {
    Success,
    Failure,
}
```

### 4.4 Constants

- Constants use `SCREAMING_SNAKE_CASE`

```rust
pub const PAGE_SIZE: usize = 4096;
pub const MAX_CPUS: usize = 256;
```

### 4.5 Abbreviations

- Treat abbreviations as regular words (e.g., `Http` not `HTTP`)

```rust
pub struct HttpClient {}
pub fn parse_xml() {}

pub struct HTTPClient {}
pub fn parseXML() {}
```

### 4.6 Feature Flag Naming

Feature flags use `snake_case`, corresponding to platform/hardware names:

```toml
kirin9020 = ["arm64", "kirin"]
snapdragon8gen4 = ["arm64"]
intel_core = ["x64"]
amd_ryzen = ["x64"]
loongson3a6000 = ["loongarch64"]
```

---

## 5. Code Organization

### 5.1 Module Structure

Nuva OS uses a layered architecture organized as follows:

```
nuva/
├── kernel/          # Kernel core (no_std)
│   ├── arch/        # Architecture-specific code
│   │   ├── arm64/
│   │   ├── x64/
│   │   └── loongarch64/
│   ├── mm/          # Memory management
│   ├── sched/       # Process scheduling
│   ├── fs/          # File system
│   ├── net/         # Network stack
│   ├── sync/        # Synchronization primitives
│   ├── syscall/     # System calls
│   ├── security/    # Security module
│   ├── plugin/      # Plugin system
│   └── quantum/     # Quantum-safe scheduling
├── hal/             # Hardware Abstraction Layer
│   ├── arm64/
│   ├── x64/
│   ├── loongarch64/
│   ├── npu/         # NPU drivers
│   ├── gpu/         # GPU drivers
│   ├── quantum/     # Quantum security (PQC/QRNG)
│   └── power/       # Power management
├── syslib/          # System libraries
├── posix/           # POSIX compatibility layer
├── fs/              # File system implementations (NovaFS)
├── sdk/             # Development toolchain SDK
└── services/        # User-space services
```

### 5.2 Module Declaration

Use `mod` declarations in the parent module's `mod.rs`:

```rust
pub mod memory;
pub mod buddy;
pub mod slab;

pub use self::memory::*;
```

### 5.3 Visibility

- Use `pub` sparingly, only when necessary
- Prefer private by default
- Group related `pub` items together

### 5.4 Imports

- Group imports at the top of the file
- Use absolute paths for external crates
- Use relative paths for internal modules

```rust
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::mm::memory::*;
use super::scheduler::*;
```

---

## 6. Conditional Compilation Style

### 6.1 Architecture Conditional Compilation

Use `cfg(target_arch)` for architecture branching:

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

### 6.2 Feature Conditional Compilation

Use `cfg(feature = "...")` for platform/feature branching:

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

### 6.3 Conditional Compilation Rules

- **Prohibit** using `not(...)` for default branches (unless there is an explicit fallback)
- Every architecture branch must have a complete implementation; no empty branches
- Conditional compilation blocks should be short; extract complex logic into separate functions
- Prefer trait objects and generics over conditional compilation

---

## 7. Documentation and Comments

### 7.1 Comment Language: English

All code comments, documentation comments, and commit messages must be in **English**. This is a fundamental requirement for international collaboration.

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

### 7.2 Module Documentation

Use `//!` for module-level documentation:

```rust
//! # Memory Management Module
//!
//! This module provides physical and virtual memory management
//! functionality including page allocation, memory mapping, and
//! memory protection.
```

### 7.3 Function Documentation

Use `///` for public function documentation:

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

### 7.4 Struct Documentation

Document public structs:

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

### 7.5 Inline Comments

Use inline comments sparingly:

```rust
// Calculate virtual runtime for CFS
let vruntime = delta_exec * NICE_0_LOAD / se.load;
```

### 7.6 SAFETY Comments

All `unsafe` blocks must include a `// SAFETY:` comment explaining the safety invariants:

```rust
// SAFETY: The caller has verified that the address is mapped
// and properly aligned. No concurrent access is possible
// because interrupts are disabled.
unsafe {
    core::ptr::write_volatile(ptr, value);
}
```

---

## 8. Error Handling

### 8.1 Use Result Types

Fallible operations must use `Result<T, E>`:

```rust
pub fn allocate_page() -> Result<PhysAddr, AllocError> {
    // ...
}
```

### 8.2 Custom Error Types

Define custom error types with `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
    InvalidArgument,
    PermissionDenied,
}
```

### 8.3 Error Propagation

Use the `?` operator for error propagation:

```rust
pub fn allocate_pages(count: usize) -> Result<Vec<PhysAddr>, AllocError> {
    let mut pages = Vec::new();
    for _ in 0..count {
        pages.push(alloc_page()?);
    }
    Ok(pages)
}
```

### 8.4 Panic Usage

- **Prohibited** in production kernel code: `panic!`, `unwrap()`, `expect()`
- Only allowed in test code or when invariants are guaranteed
- Prefer graceful error handling

```rust
let page = alloc_page().ok_or(AllocError::OutOfMemory)?;

let page = alloc_page().unwrap();
```

#### 8.4.1 Panic Elimination Standard

All kernel production paths must be panic-free. The following replacements are mandatory:

| Prohibited | Replacement | Example |
|------------|-------------|---------|
| `unwrap()` | `ok_or()` + `?` | `x.ok_or(KernelError::InvalidState)?` |
| `expect(msg)` | `ok_or()` + `?` + log | `x.ok_or_else(|| { log_error!("..."); KernelError::InvalidState })?` |
| `panic!(msg)` | `return Err(...)` | `return Err(KernelError::InternalError)` |
| `assert!(cond)` | `if !cond { return Err }` | `if !valid { return Err(KernelError::InvalidArgument) }` |
| `unreachable!()` | `return Err(KernelError::InvalidState)` | Defensive error return |

**Enforcement**: Code review must reject any `panic!`/`unwrap()`/`expect()` in non-test kernel paths. The `#[panic_handler]` in `main.rs` is the only panic definition allowed.

### 8.5 Unified Kernel Error Type

Use the unified `KernelError` enum (defined in `kernel/error.rs`) for all kernel operations:

```rust
use crate::kernel::error::{KernelError, KernelResult};

/** Allocate pages with unified error handling */
pub fn alloc_pages(order: u32) -> KernelResult<*mut Page> {
    if order > MAX_ORDER {
        return Err(KernelError::InvalidArgument);
    }
    // ...
}
```

`KernelError` provides:
- 7 error categories: memory, scheduler, IPC, driver, filesystem, synchronization, security
- `to_errno()` method for POSIX errno mapping
- `is_recoverable()` and `is_user_error()` classification
- `KernelResult<T>` type alias for convenience

### 8.6 Allocation Constraint Check

Memory allocation is **forbidden** while a SpinLock is held or in IRQ context. Use the `kmalloc!` or `kmalloc_result!` macros:

```rust
// Returns None if allocation is forbidden
let ptr = kmalloc!(alloc::alloc::alloc(layout));

// Returns Err(KernelError::DeadlockDetected) if allocation is forbidden
let ptr = kmalloc_result!(alloc::alloc::alloc(layout))?;
```

---

## 9. unsafe Usage Guidelines

### 9.1 Minimize unsafe

- Minimize the scope of `unsafe` blocks
- Restrict `unsafe` blocks to the minimum necessary range
- Every `unsafe` block must have a `// SAFETY:` comment

### 9.2 unsafe Function Documentation

`unsafe` functions must document safety preconditions using the `# Safety` section:

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

### 9.3 Typical unsafe Use Cases

In Nuva OS, legitimate uses of `unsafe` include:

- Hardware MMIO register access (`core::ptr::read_volatile` / `write_volatile`)
- Inline assembly (`core::arch::asm!`)
- Page table operations
- Global static mutable state access
- FFI calls (e.g., PQC C library bindings)
- DMA buffer management

### 9.4 Prohibited unsafe Usage

- Prohibited: using `unsafe` to bypass the borrow checker (without thorough documentation)
- Prohibited: returning raw pointers from `unsafe` functions without wrapping in a safe interface
- Prohibited: performing unrelated operations within an `unsafe` block

### 9.5 SAFETY Annotation Standard

Every `unsafe` block **must** be preceded by a `// SAFETY:` comment that explains the specific safety invariants being upheld. Generic placeholder comments like "unsafe block required for low-level memory or hardware access" are **not acceptable**.

The SAFETY comment must address:
1. **Why** the unsafe operation is safe in this specific context
2. **Which invariants** are being maintained (pointer validity, no aliasing, no data race, etc.)
3. **What preconditions** must hold (e.g., "caller ensures ptr is non-null")

Additionally:
- Every `unsafe fn` must include a `# Safety` doc section explaining preconditions
- `unsafe impl` must include a `// SAFETY:` comment explaining how the trait contract is upheld
- SAFETY comments must reference specific invariants (e.g., "pointer is non-null", "no concurrent mutable access"), not vague justifications
- When an unsafe block depends on external state, the comment must identify which state and why it is in the expected configuration

**Acceptable example:**
```rust
// SAFETY: The free_list pointer was set during grow() or from a
// previous free() call. It points to a valid slab object. We read
// the first word of the object which stores the next free pointer
// (freelist linkage pattern). This is safe because:
// 1. obj is non-null (checked above)
// 2. obj points to a valid slab object (invariant of free_list)
// 3. The first pointer-sized word stores the next pointer
unsafe {
    self.free_list = *(obj as *const *mut u8);
}
```

**Unacceptable example:**
```rust
// SAFETY: unsafe block required for low-level memory or hardware access
unsafe {
    self.free_list = *(obj as *const *mut u8);
}
```

### 9.6 SAFETY Lint Tool

Run `tools/safety_lint.sh` to scan all `.rs` files in `kernel/` and `hal/` for missing SAFETY annotations. The script reports violations as `file:line` pairs and exits with non-zero status if any are found.

---

## 10. Memory Safety

### 10.1 Raw Pointers

- Prefer references over raw pointers
- When using raw pointers, document safety requirements

```rust
pub fn get_page(page_num: usize) -> Option<&'static Page> {
    // ...
}

pub unsafe fn get_page_raw(page_num: usize) -> *mut Page {
    // ...
}
```

### 10.2 Memory Allocation

- Use the kernel allocator, not the standard library
- Check allocation results

```rust
let page = alloc_page(GFP_KERNEL)?;
let buffer = kmalloc(size, GFP_KERNEL)?;

let buffer = Box::new([0u8; 4096]);
```

### 10.3 Volatile Access

MMIO registers must use `read_volatile` / `write_volatile`:

```rust
// SAFETY: MMIO register access, address is fixed by hardware spec
unsafe {
    core::ptr::write_volatile(uart_base.add(UART_CR as usize / 4), 0x301);
}
```

---

## 11. Concurrency

### 11.1 Lock Ordering

- Establish and document lock ordering
- Always acquire locks in the same order
- Avoid nested locks whenever possible

### 11.2 Atomic Operations

- Use atomic operations for simple counters
- Use correct memory ordering

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

### 11.3 Spinlocks

- Use spinlocks for short critical sections
- Sleeping or scheduling while holding a spinlock is prohibited

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

### 11.4 Interrupt Safety

- Disable interrupts when necessary
- Use interrupt-safe locks

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

## 12. Layered Architecture Coding Constraints

Nuva OS uses a layered architecture with strict dependency rules between layers.

### 12.1 Layer Definitions

```
┌─────────────────────┐
│    Application      │  User-space applications
├─────────────────────┤
│    Services         │  System services
├─────────────────────┤
│    SysLib / POSIX   │  System libraries / compatibility layer
├─────────────────────┤
│    Kernel           │  Kernel (scheduler/memory/FS/network)
├─────────────────────┤
│    HAL              │  Hardware Abstraction Layer
├─────────────────────┤
│    Hardware         │  Hardware
└─────────────────────┘
```

### 12.2 Dependency Rules

- **Kernel** may depend on **HAL**, but reverse dependency is prohibited
- **HAL** must not directly reference symbols from `kernel/`
- **SysLib** accesses Kernel only through system call interfaces
- **POSIX** layer only wraps SysLib; it must not call Kernel directly
- **Plugin** system interacts with Kernel through registration interfaces; it must not directly modify kernel data structures

### 12.3 Cross-Layer Call Conventions

- Upper layers calling lower layers must use well-defined trait interfaces
- Direct access to internal data structures across layers is prohibited
- HAL traits are defined in `hal/`; Kernel uses them through `dyn trait`

---

## 13. Performance Considerations

### 13.1 Hot Paths

- Optimize frequently executed code
- Use inlining for small hot functions

```rust
#[inline]
pub fn is_page_free(page: &Page) -> bool {
    page.ref_count.load(Ordering::Acquire) == 0
}
```

### 13.2 Cache Efficiency

- Consider cache locality
- Use Per-CPU data structures when appropriate

```rust
#[repr(C)]
pub struct Page {
    pub flags: AtomicU32,
    pub ref_count: AtomicU32,
    pub lru_next: *mut Page,
    pub lru_prev: *mut Page,
}
```

### 13.3 Memory Allocation

- Small objects (<= 4096 bytes) use the Slab allocator
- Large objects use the Buddy allocator

```rust
let task = slab_alloc::<Task>()?;

let pages = buddy_alloc(PAGE_SIZE * 10)?;
```

### 13.4 Avoid Premature Optimization

- Profile before optimizing
- Focus on actual bottlenecks
- Maintain readability

---

## 14. Testing

### 14.1 Unit Tests

Write unit tests for modules:

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

### 14.2 Integration Tests

Write integration tests for module interactions:

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

### 14.3 Kernel Tests

Use the kernel test framework:

```rust
#[test_case]
fn test_scheduler() {
    let scheduler = Scheduler::new();
    assert_eq!(scheduler.cpu_count(), 8);
}
```

### 14.4 Test Coverage

- Aim for high test coverage
- Test edge cases and error paths
- Keep tests maintainable

---

## 15. Nuva Language Declarative Programming Paradigm

### 15.1 Source File Extension

All Nuva language source files must use the `.nv` extension. This distinguishes Nuva source from Rust (`.rs`) and other language files in the project.

```
ui/
├── main_screen.nv       # Nuva source file
├── components/
│   ├── button.nv
│   └── card.nv
└── styles/
    └── theme.nv
```

### 15.2 Declarative UI Components

Use the `component` keyword to define UI components. Components are declarative — they describe *what* the UI should look like, not *how* to build it.

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

Rules:
- Components must be declared with the `component` keyword
- Component names use `PascalCase`
- Props are declared in the parameter list with explicit types
- Child elements are declared using a tree structure with `{ }` blocks
- Modifiers are chained with `.` (dot) syntax
- No imperative `new()` or `build()` calls — the framework reconciles the tree

### 15.3 Reactive Data (signal/effect)

Use `signal` for reactive state and `effect` for side effects that automatically re-execute when dependencies change.

```nuva
signal count: Int = 0

effect {
    // Automatically re-runs when `count` changes
    console.log("Count is now: " + count)
}
```

Rules:
- `signal` declares a reactive variable with automatic change propagation
- `effect` registers a side effect with automatic dependency tracking
- Signal reads within an `effect` body are automatically tracked as dependencies
- Direct signal mutation triggers all dependent effects
- Avoid performing I/O in effect bodies unless the effect is explicitly marked `io`

### 15.4 Declarative Concurrency (async/await)

Use `async`/`await` for declarative asynchronous computation. The compiler transforms async functions into state machines.

```nuva
async fn fetch_data(url: String) -> Result<Data, Error> {
    let response = await http.get(url)
    let data = await response.json()
    return data
}
```

Rules:
- `async` marks a function as returning `Future<T>`
- `await` suspends execution until the `Future` resolves
- Async functions are compiled to state machine IR
- Never block the reactive scheduler in an async context — use `await` instead

### 15.5 Declarative Resource Management (resource/with)

Use `resource` for declarative resource acquisition and `with` for scoped resource management with automatic cleanup.

```nuva
resource FileHandle(path: String) {
    acquire: fs.open(path, READ),
    release: handle.close()
}

with (handle = FileHandle("/data/config.json")) {
    let content = handle.read_all()
    process(content)
}
// handle.close() called automatically at scope exit
```

Rules:
- `resource` declares a resource type with `acquire` and `release` phases
- `with` creates a scoped binding — the resource is released when the scope exits
- Release is guaranteed even if an exception occurs (RAII semantics)
- Resources cannot be leaked outside their `with` scope

### 15.6 Prohibited Imperative Patterns

The following imperative patterns are **prohibited** in `.nv` files:

| Prohibited Pattern | Declarative Replacement |
|---------------------|------------------------|
| `new Widget()`, `widget.build()` | `component` declaration |
| `setState()`, `notifyStateChanged()` | `signal` with automatic propagation |
| `addEventListener()`, `removeEventListener()` | Modifier-bound `.on_click()`, `.on_change()` |

### 15.7 Declarative Driver Paradigm (kernel/driver/declarative)

The declarative driver model extends the declarative paradigm to kernel driver code:

| Macro | Purpose | Example |
|-------|---------|---------|
| `declare_driver!` | Static driver registration with metadata | `declare_driver! { MY_DRV { name: "my_drv", compatible: &["v,d"], ... } }` |
| `declare_resource!` | Declarative resource acquisition/release | `declare_resource! { MY_RES { name: "irq", resource_type: Irq, optional: false } }` |
| `declare_pm!` | Declarative power state machine | `declare_pm! { MY_PM { On => Idle: 10us, ... } }` |

Rules:
- All driver registration must use `declare_driver!` — no manual `DriverDescriptor` construction
- All power management must use `declare_pm!` — no manual state machine construction
- All resource bindings must use `declare_resource!` — no manual acquire/release pairs
- The `DeclarativeDriver` trait must be implemented for all new drivers
- `CompatibleHashTable` (in `kernel/driver/matching`) provides O(1) device-driver matching

### 15.8 Declarative Scheduler Paradigm

The scheduler supports declarative policy configuration:

```rust
pub struct SchedPolicyConfig {
    pub policy: SchedPolicy,
    pub min_granularity_ns: u64,
    pub latency_ns: u64,
    pub wakeup_granularity_ns: u64,
}
```

- Policy changes are applied atomically via `SchedPolicyConfig` hot-update
- Per-CPU run queues (`PerCpuRunQueue`) are cache-line aligned to prevent false sharing

---

## 16. Best Practices

### 16.1 Code Review

- All code should be reviewed before merging
- Review correctness, style, and design
- Feedback should be constructive

### 16.2 Version Control

- Commit frequently with clear commit messages
- Follow Conventional Commits format

```
feat(mm): add slab allocator

Implements slab allocator for small object allocation
with automatic reclamation and per-CPU caches.
```

### 16.3 Documentation Updates

- Update documentation when code changes
- Keep README and API docs in sync
- Document design decisions

### 16.4 Backward Compatibility

- Avoid breaking public APIs
- Use deprecation warnings for changes
- Document migration paths

---

## 17. Tools

### 17.1 Pre-commit Hooks

Use pre-commit hooks to enforce standards:

```bash
cargo install cargo-husky
cargo husky install
```

### 17.2 CI/CD

Configure CI to run checks:

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

## Summary

Following these coding standards ensures:

- **Consistency**: Code looks and feels the same across the project
- **Quality**: Fewer bugs and better maintainability
- **Collaboration**: Easier for others to understand and contribute
- **Performance**: Code is optimized where it matters
- **Safety**: `unsafe` usage is strictly controlled

Remember: these are guidelines, not absolute rules. Use judgment and common sense. When in doubt, prioritize readability and safety.

---

<!-- Translation Status: Source (English) | Last Updated: 2026-05-20 -->

**Last Updated**: 2026-05-20
