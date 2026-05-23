# Nuva OS API Reference

> **Document Scope**: This document provides an overview of all Nuva OS APIs across
> all layers (L0-L4). For detailed HAL trait signatures, C API bindings, key sizes,
> and performance characteristics, see [API_REFERENCE.md](api/API_REFERENCE.md).

## Overview

This document provides a comprehensive API reference for Nuva OS, covering Kernel API, File System API, IPC API, Network API, AI Engine API, Application Framework API, Nuva Language API, HAL Trait Interface, NPU API, Quantum-Safe API, Plugin API, C/C++ FFI API, Error Handling, Logging API, and Optimization Module API.

---

## Table of Contents

1. [Kernel API](#1-kernel-api)
2. [File System API](#2-file-system-api)
3. [IPC API](#3-ipc-api)
4. [Network API](#4-network-api)
5. [AI Engine API](#5-ai-engine-api)
6. [Application Framework API](#6-application-framework-api)
7. [Nuva Language API](#7-nuva-language-api)
8. [HAL Trait Interface](#8-hal-trait-interface)
9. [NPU API](#9-npu-api)
10. [Quantum-Safe API](#10-quantum-safe-api)
11. [Plugin API](#11-plugin-api)
12. [C/C++ FFI API](#12-cc-ffi-api)
13. [Error Handling](#13-error-handling)
14. [Logging API](#14-logging-api)
15. [Optimization Module API](#15-optimization-module-api)
16. [SDK API](#16-sdk-api)

---

## 1. Kernel API

### 1.1 Memory Management

```rust
pub fn alloc_pages(order: u32) -> Option<PhysAddr>;
pub fn free_pages(addr: PhysAddr, order: u32);
pub fn mmap(addr: VirtAddr, len: usize, prot: ProtFlags) -> Result<VirtAddr>;
pub fn munmap(addr: VirtAddr, len: usize) -> Result<()>;
```

### 1.2 Process Management

```rust
pub fn fork() -> Result<Pid>;
pub fn execve(path: &str, argv: &[&str], envp: &[&str]) -> Result<()>;
pub fn waitpid(pid: Pid, status: &mut i32, options: WaitFlags) -> Result<Pid>;
pub fn exit(status: i32) -> !;
```

### 1.3 Thread Management

```rust
pub fn thread_create(attr: &ThreadAttr, func: extern fn(*mut void) -> *mut void, arg: *mut void) -> Result<Tid>;
pub fn thread_exit(retval: *mut void) -> !;
pub fn thread_join(tid: Tid, retval: &mut *mut void) -> Result<()>;
```

### 1.4 Page Table Operations (LoongArch64)

```rust
pub trait PageTableOps {
    pub fn create() -> Result<Self>;
    pub fn destroy(&mut self) -> Result<()>;
    pub fn map(&mut self, vaddr: VirtAddr, paddr: PhysAddr, prot: ProtFlags) -> Result<()>;
    pub fn unmap(&mut self, vaddr: VirtAddr) -> Result<()>;
    pub fn translate(&self, vaddr: VirtAddr) -> Option<PhysAddr>;
    pub fn protect(&mut self, vaddr: VirtAddr, prot: ProtFlags) -> Result<()>;
}
```

### 1.5 Synchronization Primitives

```rust
pub struct SpinLock { /* ... */ }
impl SpinLock {
    pub fn new() -> Self;
    pub fn lock(&self);
    pub fn unlock(&self);
    pub fn try_lock(&self) -> bool;
}

pub struct Mutex { /* ... */ }
impl Mutex {
    pub fn new() -> Self;
    pub fn lock(&self);
    pub fn unlock(&self);
    pub fn try_lock(&self) -> bool;
}

pub struct RwLock { /* ... */ }
impl RwLock {
    pub fn new() -> Self;
    pub fn read(&self);
    pub fn write(&self);
    pub fn unlock(&self);
}

pub struct Condvar { /* ... */ }
impl Condvar {
    pub fn new() -> Self;
    pub fn wait(&self, mutex: &Mutex);
    pub fn signal(&self);
    pub fn broadcast(&self);
}
```

---

## 2. File System API

### 2.1 File Operations

```rust
pub fn open(path: &str, flags: OpenFlags, mode: Mode) -> Result<Fd>;
pub fn close(fd: Fd) -> Result<()>;
pub fn read(fd: Fd, buf: &mut [u8]) -> Result<usize>;
pub fn write(fd: Fd, buf: &[u8]) -> Result<usize>;
pub fn lseek(fd: Fd, offset: i64, whence: Whence) -> Result<i64>;
pub fn fstat(fd: Fd, stat: &mut Stat) -> Result<()>;
```

### 2.2 Directory Operations

```rust
pub fn mkdir(path: &str, mode: Mode) -> Result<()>;
pub fn rmdir(path: &str) -> Result<()>;
pub fn opendir(path: &str) -> Result<DirStream>;
pub fn readdir(dir: &mut DirStream) -> Result<Option<DirEntry>>;
pub fn closedir(dir: DirStream) -> Result<()>;
```

---

## 3. IPC API

### 3.1 Pipe

```rust
pub fn pipe(fds: &mut [Fd; 2]) -> Result<()>;
```

### 3.2 Shared Memory

```rust
pub fn shmget(key: i32, size: usize, flags: ShmFlags) -> Result<ShmId>;
pub fn shmat(shmid: ShmId, addr: *mut void, flags: ShmFlags) -> Result<*mut void>;
pub fn shmdt(addr: *mut void) -> Result<()>;
pub fn shmctl(shmid: ShmId, cmd: ShmCmd, buf: &mut ShmDs) -> Result<()>;
```

### 3.3 Semaphore

```rust
pub fn semget(key: i32, nsems: u32, flags: SemFlags) -> Result<SemId>;
pub fn semop(semid: SemId, ops: &[SemOp]) -> Result<()>;
pub fn semctl(semid: SemId, semnum: u32, cmd: SemCmd, arg: SemUnion) -> Result<i32>;
```

### 3.4 Message Queue

```rust
pub fn msgget(key: i32, flags: MsgFlags) -> Result<MsgId>;
pub fn msgsnd(msqid: MsgId, msg: &MsgBuf, flags: MsgFlags) -> Result<()>;
pub fn msgrcv(msqid: MsgId, msg: &mut MsgBuf, typ: i64, flags: MsgFlags) -> Result<usize>;
pub fn msgctl(msqid: MsgId, cmd: MsgCmd, buf: &mut MsqidDs) -> Result<()>;
```

---

## 4. Network API

### 4.1 Socket

```rust
pub fn socket(domain: Domain, sock_type: SockType, protocol: Protocol) -> Result<Fd>;
pub fn bind(sockfd: Fd, addr: &SockAddr) -> Result<()>;
pub fn listen(sockfd: Fd, backlog: i32) -> Result<()>;
pub fn accept(sockfd: Fd, addr: &mut SockAddr) -> Result<Fd>;
pub fn connect(sockfd: Fd, addr: &SockAddr) -> Result<()>;
pub fn send(sockfd: Fd, buf: &[u8], flags: MsgFlags) -> Result<usize>;
pub fn recv(sockfd: Fd, buf: &mut [u8], flags: MsgFlags) -> Result<usize>;
```

### 4.2 NFS (Network File System)

```rust
pub fn rpc_call(program: u32, version: u32, procedure: u32, args: &[u8]) -> Result<Vec<u8>>;
pub fn xdr_decode_int(data: &[u8]) -> Result<(i32, &[u8])>;
pub fn xdr_decode_uint(data: &[u8]) -> Result<(u32, &[u8])>;
pub fn xdr_decode_string(data: &[u8]) -> Result<(String, &[u8])>;
pub fn xdr_decode_opaque(data: &[u8], len: usize) -> Result<(&[u8], &[u8])>;
```

### 4.3 SMB (Server Message Block)

```rust
pub struct SmbClient { /* ... */ }

impl SmbClient {
    pub fn connect(server: &str, port: u16) -> Result<Self>;
    pub fn send_and_receive(&mut self, request: &SmbRequest) -> Result<SmbReply>;
    pub fn parse_reply_header(data: &[u8]) -> Result<SmbHeader>;
}
```

---

## 5. AI Engine API

### 5.1 Inference Engine

```rust
pub fn create_engine(config: &EngineConfig) -> Result<Engine>;
pub fn load_model(engine: &Engine, path: &str) -> Result<Model>;
pub fn create_context(model: &Model) -> Result<Context>;
pub fn execute(context: &mut Context, inputs: &[Tensor]) -> Result<Vec<Tensor>>;
```

### 5.2 Tensor Operations

```rust
pub fn create_tensor(shape: &[usize], dtype: DataType) -> Result<Tensor>;
pub fn get_tensor_data(tensor: &Tensor) -> &[u8];
pub fn set_tensor_data(tensor: &mut Tensor, data: &[u8]) -> Result<()>;
```

---

## 6. Application Framework API

### 6.1 UI Components

```rust
pub fn create_widget(parent: Option<&Widget>, props: WidgetProps) -> Result<Widget>;
pub fn set_prop(widget: &Widget, key: &str, value: &PropValue) -> Result<()>;
pub fn get_prop(widget: &Widget, key: &str) -> Result<PropValue>;
pub fn add_child(parent: &Widget, child: &Widget) -> Result<()>;
pub fn remove_child(parent: &Widget, child: &Widget) -> Result<()>;
```

### 6.2 Window Management

```rust
pub fn create_window(attrs: WindowAttrs) -> Result<Window>;
pub fn show_window(window: &Window) -> Result<()>;
pub fn hide_window(window: &Window) -> Result<()>;
pub fn destroy_window(window: Window) -> Result<()>;
```

### 6.3 Event Handling

```rust
pub fn register_handler(widget: &Widget, event_type: EventType, handler: EventHandler) -> Result<()>;
pub fn send_event(event: Event) -> Result<()>;
pub fn dispatch_event(event: Event) -> Result<()>;
```

---

## 7. Nuva Language API

### 7.1 Compiler

```rust
pub fn compile(source: &str, options: CompileOptions) -> Result<Module>;
pub fn run(module: &Module) -> Result<Value>;
```

### 7.2 Runtime

```rust
pub fn create_vm(config: VmConfig) -> Result<Vm>;
pub fn execute(vm: &mut Vm, bytecode: &[u8]) -> Result<Value>;
pub fn call(vm: &mut Vm, func: &str, args: &[Value]) -> Result<Value>;
```

---

## 8. HAL Trait Interface

HAL trait interfaces provide hardware abstraction. For complete trait signatures, C API bindings, key sizes, and performance characteristics, see [api/API_REFERENCE.md](api/API_REFERENCE.md).

### 8.1 Trait Overview

| Trait | Module | Description |
|-------|--------|-------------|
| `CpuHal` | `hal::cpu` | CPU frequency/voltage/temperature/idle state management |
| `GpuHal` | `hal::gpu` | GPU frame management, command queue |
| `NpuHal` | `hal::npu` | NPU initialization, model load/unload, buffer management, inference execution |
| `PowerHal` | `hal::power` | Power state management, suspend/resume |
| `InputHal` | `hal::input` | Input device event reading and polling |
| `PqcProvider` | `hal::quantum::pqc` | Post-quantum cryptography (Kyber/Dilithium) |
| `QrngProvider` | `hal::quantum::qrng` | Quantum random number generation |

---

## 9. NPU API

NPU device management and inference API. For detailed NPU HAL trait, C API bindings, and data structures, see [api/API_REFERENCE.md](api/API_REFERENCE.md#npu-hal).

### 9.1 NPU Device Management

```rust
pub fn init_npu() -> i32;
pub fn get_npu_manager() -> &'static mut NpuManager;

pub struct NpuDevice {
    pub info: NpuInfo,
    pub state: NpuState,
    pub features: NpuFeatures,
}
```

### 9.2 ONNX Runtime

```rust
pub fn init_onnx() -> i32;
pub fn get_onnx_runtime() -> &'static mut OnnxRuntime;
```

### 9.3 AI Scheduler

```rust
pub fn init_ai_scheduler() -> i32;
pub fn get_ai_scheduler() -> &'static mut AiScheduler;
pub fn notify_kernel_scheduler(task_id: u64, event: AiSchedulerEvent) -> Result<()>;
pub fn select_cpu_for_task(task: &AiTask) -> Result<u32>;
pub fn ai_wakeup_boost_external(task: &Task) -> Result<()>;
pub fn ai_latency_aware_pick_external(candidates: &[Task]) -> Result<Option<&Task>>;
```

### 9.4 DaVinci NPU Operations

```rust
pub fn davinci_npu_ops(op: NpuOpType, input: &[u8], output: &mut [u8]) -> Result<usize>;
```

---

## 10. Quantum-Safe API

Post-quantum cryptography API. For detailed trait signatures, key sizes, and C API bindings, see [api/API_REFERENCE.md](api/API_REFERENCE.md#quantum-hal).

### 10.1 Security Levels

| Level | KEM Variant | Signature Variant |
|-------|-------------|-------------------|
| NIST Level 1 | Kyber512 | Dilithium2 |
| NIST Level 3 | Kyber768 | Dilithium3 |
| NIST Level 5 | Kyber1024 | Dilithium5 |

### 10.2 Quantum-Safe Security Module

```rust
pub struct QuantumSafeSecurity { /* ... */ }

impl QuantumSafeSecurity {
    pub fn new(config: SecurityConfig) -> Result<Self, PqcError>;
    pub fn keygen(&self, algo: PqcAlgorithm) -> Result<(PublicKey, SecretKey), PqcError>;
    pub fn encapsulate(&self, pk: &PublicKey) -> Result<(SharedSecret, Ciphertext), PqcError>;
    pub fn decapsulate(&self, sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, PqcError>;
    pub fn sign(&self, sk: &SecretKey, msg: &[u8]) -> Result<Signature, PqcError>;
    pub fn verify(&self, pk: &PublicKey, msg: &[u8], sig: &Signature) -> Result<bool, PqcError>;
}
```

---

## 11. Plugin API

Dynamic plugin system. For detailed Plugin trait signature and PluginMeta structure, see [api/API_REFERENCE.md](api/API_REFERENCE.md#plugin-api).

### 11.1 Plugin Loader

```rust
pub struct PluginLoader { /* ... */ }

impl PluginLoader {
    pub fn new() -> Self;
    pub fn load(&mut self, path: &str) -> Result<Box<dyn Plugin>, PluginError>;
    pub fn load_from_elf(data: &[u8]) -> Result<Box<dyn Plugin>, PluginError>;
    pub fn unload(&mut self, handle: LibraryHandle) -> Result<(), PluginError>;
}

pub struct LoaderConfig {
    pub verify_signature: bool,
    pub enable_cache: bool,
    pub max_plugin_size: usize,
}
```

### 11.2 Plugin Registry

```rust
pub struct PluginRegistry { /* ... */ }

impl PluginRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, id: PluginId, meta: PluginMeta);
    pub fn unregister(&mut self, id: PluginId);
    pub fn get(&self, id: PluginId) -> Option<&PluginMeta>;
    pub fn get_by_name(&self, name: &str) -> Option<&PluginMeta>;
    pub fn get_by_type(&self, plugin_type: PluginType) -> Vec<&PluginMeta>;
}
```

### 11.3 Plugin Signature Verification

```rust
pub fn compute_plugin_hash(data: &[u8]) -> [u8; 32];
```

SHA-256 hash computation per FIPS 180-4 for plugin integrity verification.

### 11.4 Plugin Services

```rust
pub struct PluginServices { /* ... */ }

impl PluginServices {
    pub fn new() -> Self;
    pub fn kernel_privileges(&self) -> &KernelPrivileges;
    pub fn check_memory_limit(&self, requested: usize) -> bool;
    pub fn check_ipc_limit(&self, requested: usize) -> bool;
}
```

### 11.5 Plugin Manager Statistics

```rust
pub struct ManagerStats {
    pub total_plugins: u32,
    pub active_plugins: u32,
    pub failed_plugins: u32,
    pub total_load_time_ms: u64,
}
```

`failed_plugins` and `total_load_time_ms` track real runtime data.

### 11.6 Plugin Audit

```rust
pub fn current_timestamp() -> u64;
```

Returns `read_cycle_counter() / 1000` for cycle-accurate timing. Audit fingerprints use SHA-256.

### 11.7 Plugin Package Manager

```rust
pub fn fetch_package_info(name: &str) -> Result<PackageInfo>;
pub fn download_package(name: &str, version: &str) -> Result<Vec<u8>>;
pub fn pkg_verify(data: &[u8], expected_hash: &[u8; 32]) -> Result<()>;
```

`fetch_package_info` and `download_package` interact via TCP socket HTTP. `pkg_verify` uses SHA-256.

### 11.8 Plugin SDK

```rust
pub fn sdk_build(project_path: &str) -> Result<()>;
pub fn sdk_test(project_path: &str) -> Result<()>;
pub fn sdk_package(project_path: &str) -> Result<Vec<u8>>;
```

Each function checks source files via VFS and computes SHA-256 for integrity.

---

## 12. C/C++ FFI API

### 12.1 C API Bindings

```rust
pub mod c_api {
    pub mod bindings;
}
```

The C API provides C function interfaces corresponding to Rust HAL, enabling driver development in C:

```c
// CPU HAL C API
int32_t hal_cpu_init(void);
int32_t hal_cpu_boot(uint32_t cpu_id);
int32_t hal_cpu_set_frequency(uint32_t cpu_id, uint64_t freq);
uint64_t hal_cpu_get_frequency(uint32_t cpu_id);
int32_t hal_cpu_get_temperature(uint32_t cpu_id);
```

### 12.2 C++ API

```rust
pub mod cpp_api { /* ... */ }
```

The C++ API provides object-oriented wrappers supporting RAII resource management.

### 12.3 API Stability Framework

```rust
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub abi_version: u32,
}

impl ApiVersion {
    pub fn is_compatible(&self, other: &ApiVersion) -> bool;
}

pub struct ApiFunction {
    pub name: String,
    pub return_type: String,
    pub params: Vec<String>,
    pub version_added: ApiVersion,
    pub version_deprecated: Option<ApiVersion>,
    pub is_stable: bool,
}

pub struct ApiStruct {
    pub name: String,
    pub fields: Vec<ApiField>,
    pub size: usize,
    pub alignment: usize,
    pub version_added: ApiVersion,
}
```

**ABI Stability Checks**:
- Function signature compatibility verification
- Structure layout and size verification
- Version compatibility check (major must match, minor is backward compatible)

### 12.4 Initialization

```rust
pub fn init_hal_ffi() -> Result<(), &'static str>;
```

---

## 13. Error Handling

### 13.1 Error Codes

```rust
pub enum ErrorCode {
    Success = 0,
    Unknown = 1,
    InvalidParam = 2,
    OutOfMemory = 3,
    NotFound = 4,
    PermissionDenied = 5,
    Timeout = 6,
    // ...
}
```

### 13.2 Result Type

```rust
pub type Result<T> = core::result::Result<T, ErrorCode>;
```

---

## 14. Logging API

```rust
#[macro_export]
macro_rules! pr_info {
    ($($arg:tt)*) => { ... };
}

#[macro_export]
macro_rules! pr_debug {
    ($($arg:tt)*) => { ... };
}

#[macro_export]
macro_rules! pr_warn {
    ($($arg:tt)*) => { ... };
}

#[macro_export]
macro_rules! pr_error {
    ($($arg:tt)*) => { ... };
}
```

---

## 15. Optimization Module API

### 15.1 Per-CPU Page Cache

```rust
pub fn init_percpu_cache();

pub struct PerCpuPageCache {
    pub pages: [*mut Page; PCP_CACHE_SIZE],
    pub count: AtomicU32,
    pub high: u32,
    pub batch: u32,
}

impl PerCpuPageCache {
    pub fn alloc(&mut self) -> *mut Page;
    pub fn free(&mut self, page: *mut Page);
    pub fn bulk_alloc(&mut self, count: u32) -> u32;
    pub fn drain(&mut self, count: u32);
}
```

### 15.2 Huge Page Support

```rust
pub enum HugePageSize {
    Huge2MB = 21,
    Huge1GB = 30,
}

pub fn init_huge_pages();
pub fn alloc_huge_page(size: HugePageSize) -> Option<PhysAddr>;
pub fn free_huge_page(addr: PhysAddr, size: HugePageSize);
```

### 15.3 NUMA Support

```rust
pub fn init_numa();
pub fn cpu_to_node(cpu: u32) -> u32;
pub fn paddr_to_node(paddr: u64) -> u32;

pub struct NumaTopology {
    pub nodes: [NumaNode; MAX_NUMA_NODES],
    pub nr_nodes: u32,
}

impl NumaTopology {
    pub fn preferred_node(&self, current_node: u32, flags: u32) -> u32;
    pub fn find_nearest_node(&self, from: u32) -> u32;
}
```

### 15.4 Memory Compaction

```rust
pub fn init_memory_compaction();
pub fn compact_memory(order: u32) -> CompactResult;

pub enum CompactResult {
    Success = 0,
    Partial = 1,
    NoSuitablePages = 2,
    NotEnoughFree = 3,
    Skipped = 4,
}
```

### 15.5 Red-Black Tree

```rust
pub struct RbTree {
    pub root: *mut RbNode,
    pub count: u64,
    pub leftmost: *mut RbNode,
}

impl RbTree {
    pub fn insert(&mut self, node: *mut RbNode);
    pub fn remove(&mut self, node: *mut RbNode);
    pub fn first(&self) -> *mut RbNode;
    pub fn next(&self, node: *mut RbNode) -> *mut RbNode;
}
```

### 15.6 Scheduling Domains

```rust
pub struct SchedDomain {
    pub level: u32,
    pub span: CpuMask,
    pub groups: *mut SchedGroup,
    pub imbalance_pct: u32,
}

pub struct LoadBalancer {
    pub sd: *mut SchedDomain,
    pub busiest_cpu: u32,
    pub imbalance: u64,
}

impl LoadBalancer {
    pub fn find_busiest_group(&mut self) -> *mut SchedGroup;
    pub fn move_tasks(&mut self, count: u32) -> u32;
}
```

### 15.7 Energy Aware Scheduling (EAS)

```rust
pub struct EnergyModel {
    pub domains: [Option<PerfDomain>; MAX_NR_PERF_DOMAINS],
    pub nr_perf_domains: u32,
}

pub struct PerfDomain {
    pub cpus: CpuMask,
    pub perf_states: *mut PerfState,
    pub nr_perf_states: u32,
}

pub fn eas_select_task_rq(task_util: u32, prev_cpu: usize, sync: bool) -> usize;
```

### 15.8 File Page Cache

```rust
pub fn init_page_cache();

pub struct PageCache {
    pub hash_table: [*mut PageCacheEntry; HASH_SIZE],
    pub active_list: LruList,
    pub inactive_list: LruList,
}

impl PageCache {
    pub fn lookup(&mut self, key: &PageCacheKey) -> *mut PageCacheEntry;
    pub fn add(&mut self, entry: *mut PageCacheEntry) -> bool;
    pub fn read_page(&mut self, key: &PageCacheKey) -> *mut PageCacheEntry;
}
```

### 15.9 Directory Cache

```rust
pub fn init_dcache();

pub struct DentryCache {
    pub hash_table: [*mut Dentry; HASH_SIZE],
    pub lru_list: DentryLruList,
}

impl DentryCache {
    pub fn lookup(&mut self, key: &DentryKey, name: &[u8]) -> *mut Dentry;
    pub fn add(&mut self, dentry: *mut Dentry) -> bool;
}
```

### 15.10 io_uring

```rust
pub fn init_io_uring(ring_size: u32);

pub struct IoUring {
    pub sq_ring: IoSqRing,
    pub cq_ring: IoCqRing,
    pub sqes: *mut IoSqe,
}

impl IoUring {
    pub fn submit(&mut self, sqe: &IoSqe) -> i32;
    pub fn get_completion(&mut self) -> Option<IoCqe>;
}

pub enum IoOpCode {
    Nop = 0,
    Read = 1,
    Write = 2,
    Open = 5,
    Close = 6,
    // ...
}
```

### 15.11 TCP Fast Path

```rust
pub fn init_tcp_fast_path();

pub struct TcpConnection {
    pub state: AtomicU32,
    pub snd_una: AtomicU32,
    pub rcv_nxt: AtomicU32,
    pub cwnd: AtomicU32,
}

impl TcpConnection {
    pub fn fast_path_receive(&mut self, seq: u32, data_len: u32) -> bool;
    pub fn is_fast_path_eligible(&self) -> bool;
}
```

### 15.12 ASLR

```rust
pub fn init_aslr();
pub fn randomize_stack(base: u64, limit: u64) -> u64;
pub fn randomize_mmap(hint: u64, min_addr: u64, max_addr: u64) -> u64;
pub fn randomize_brk(base: u64, max_addr: u64) -> u64;
pub fn configure_aslr(enabled: bool, stack_bits: u32, mmap_bits: u32, brk_bits: u32);
```

### 15.13 Stack Canary

```rust
pub fn init_stack_canary();
pub fn get_global_canary() -> u64;
pub fn create_task_canary(task_id: u64, stack_base: *mut u8, stack_size: usize) -> TaskStackCanary;
pub fn verify_task_canary(canary: &TaskStackCanary) -> bool;
```

---

## 16. SDK API

### 16.1 Debug Target

```rust
pub fn launch(program: &str, args: &[&str]) -> Result<Pid>;
pub fn read_registers(pid: Pid, regs: &mut Registers) -> Result<()>;
pub fn read_memory(pid: Pid, addr: u64, buf: &mut [u8]) -> Result<usize>;
```

`launch` uses `fork` + `execv`. `read_registers` uses `ptrace`. `read_memory` uses `process_vm_readv`.

### 16.2 Profiler Sampler

```rust
pub fn capture_stack_trace(pid: Pid) -> Result<Vec<u64>>;
pub fn get_current_thread_id() -> Tid;
```

`capture_stack_trace` reads from `/proc/[pid]/maps` and stack. `get_current_thread_id` uses `gettid()`.

### 16.3 Package Resolver

```rust
pub struct PackageResolver { /* ... */ }

impl PackageResolver {
    pub fn with_registry(url: &str) -> Result<Self>;
    pub fn resolve(&self, spec: &PackageSpec) -> Result<ResolvedPackage>;
}
```

`with_registry` configures the registry URL. `resolve` handles transitive dependency resolution.

### 16.4 Package Registry

```rust
pub struct RegistryClient { /* ... */ }

impl RegistryClient {
    pub fn search(query: &str) -> Result<Vec<PackageInfo>>;
    pub fn publish(pkg: &Package) -> Result<()>;
    pub fn versions(name: &str) -> Result<Vec<String>>;
}
```

All methods perform real HTTP requests to the configured registry.

---

**Last Updated**: May 15, 2026
**License**: Apache-2.0
