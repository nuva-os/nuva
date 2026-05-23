# Nuva HAL API 参考手册

> **文档范围**：本文档仅提供 HAL（L0）层 API 的详细参考，包括完整的 Rust trait 签名、C/C++ API 绑定、数据结构和性能特征。如需所有系统 API 的概览，请参阅 [API_zh.md](../API_zh.md)。

**版本**：1.2.0  
**日期**：2026-05-15

---

## 目录

1. [概述](#概述)
2. [错误处理](#错误处理)
3. [CPU HAL](#cpu-hal)
4. [GPU HAL](#gpu-hal)
5. [NPU HAL](#npu-hal)
6. [Quantum HAL](#quantum-hal)
7. [Power HAL](#power-hal)
8. [网络 API](#网络-api)
9. [插件 API](#插件-api)
10. [SDK API](#sdk-api)
11. [数据结构](#数据结构)

---

## 概述

Nuva HAL（硬件抽象层）提供跨不同平台访问硬件特性的统一接口。API 设计为：

- **平台无关**：可在 x86_64、ARM64 和 LoongArch64 上运行
- **C99 兼容**：可从 C 和 C++ 使用
- **类型安全**：强类型配合不透明句柄
- **错误显式**：所有函数返回错误码

### 头文件

```c
#include <nuva_hal.h>  // C API
```

```cpp
#include <nuva_hal.hpp>  // C++ API
```

### Rust trait 接口

HAL 层通过 Rust trait 提供抽象，位于 `hal/` 目录：

- `hal::cpu` — CPU 抽象 trait
- `hal::gpu` — GPU 抽象 trait
- `hal::npu::NpuHal` — NPU 抽象 trait
- `hal::quantum::pqc::PqcProvider` — 量子安全 PQC trait
- `hal::quantum::qrng::QrngProvider` — 量子随机数 trait
- `hal::power` — 电源管理 trait
- `hal::loongarch64` — LoongArch64 平台（MMU、中断、SIMD、LVZ、LBT）

---

## 错误处理

### 错误码

```c
typedef enum {
    NUVA_OK = 0,                    // 成功
    NUVA_ERROR_INVALID_PARAM = -1,  // 无效参数
    NUVA_ERROR_NOT_FOUND = -2,      // 未找到
    NUVA_ERROR_OUT_OF_MEMORY = -3,  // 内存不足
    NUVA_ERROR_NOT_SUPPORTED = -4,  // 不支持
    NUVA_ERROR_HARDWARE = -5,       // 硬件错误
    NUVA_ERROR_TIMEOUT = -6,        // 超时
    NUVA_ERROR_BUSY = -7,           // 忙
} nuva_result_t;
```

### C 错误检查模式

```c
nuva_result_t result = nuva_cpu_get_info(&info);
if (result != NUVA_OK) {
    printf("Error: %d\n", result);
    return result;
}
```

### C++ 异常处理

```cpp
try {
    auto info = nuva::Cpu::get_info();
} catch (const nuva::Exception& e) {
    std::cerr << "Error: " << e.what() << std::endl;
    std::cerr << "Code: " << e.result() << std::endl;
}
```

---

## CPU HAL

### C API

#### nuva_cpu_get_info

```c
nuva_result_t nuva_cpu_get_info(nuva_cpu_info_t* info);
```

- `info`：指向要填充的 `nuva_cpu_info_t` 结构的指针
- 返回 `NUVA_OK` 或 `NUVA_ERROR_INVALID_PARAM`

#### nuva_cpu_get_core_id

```c
uint32_t nuva_cpu_get_core_id(void);
```

返回当前核心 ID（0 到 core_count-1）。

#### nuva_cpu_enable_irq / nuva_cpu_disable_irq

```c
void nuva_cpu_enable_irq(void);
void nuva_cpu_disable_irq(void);
```

#### nuva_cpu_memory_barrier / read_barrier / write_barrier

```c
void nuva_cpu_memory_barrier(void);
void nuva_cpu_read_barrier(void);
void nuva_cpu_write_barrier(void);
```

### AI 调度器 API

AI 调度器提供内核态的 AI 任务调度与 CPU 亲和性选择：

```rust
pub fn notify_kernel_scheduler(event: AiSchedulerEvent) -> Result<(), SchedulerError>;
pub fn select_cpu_for_task(task: &AiTask) -> Result<CpuId, SchedulerError>;
```

- `notify_kernel_scheduler`：通知内核调度器 AI 工作负载变化（如新推理请求、模型加载完成等）
- `select_cpu_for_task`：根据 AI 任务特征（计算密集型/IO 密集型、NPU 亲和性等）选择最优 CPU 核心

C API：

```c
nuva_result_t nuva_ai_notify_scheduler(nuva_ai_scheduler_event_t event);
nuva_result_t nuva_ai_select_cpu_for_task(
    const nuva_ai_task_t* task,
    uint32_t* selected_cpu
);
```

---

## GPU HAL

### C API

#### nuva_gpu_init / nuva_gpu_shutdown

```c
nuva_result_t nuva_gpu_init(void);
nuva_result_t nuva_gpu_shutdown(void);
```

#### nuva_gpu_get_device_count

```c
nuva_result_t nuva_gpu_get_device_count(uint32_t* count);
```

#### nuva_gpu_get_device_info

```c
nuva_result_t nuva_gpu_get_device_info(uint32_t device_index, nuva_gpu_info_t* info);
```

#### nuva_gpu_create_buffer / nuva_gpu_destroy_buffer

```c
nuva_result_t nuva_gpu_create_buffer(nuva_gpu_device_t device, size_t size, nuva_gpu_buffer_t* buffer);
nuva_result_t nuva_gpu_destroy_buffer(nuva_gpu_buffer_t buffer);
```

---

## NPU HAL

### Rust Trait 完整签名

```rust
pub trait NpuHal: Send + Sync {
    fn initialize(&mut self) -> Result<(), NpuError>;
    fn load_model(&mut self, model: &ModelData) -> Result<ModelId, NpuError>;
    fn unload_model(&mut self, id: ModelId) -> Result<(), NpuError>;
    fn create_buffer(&mut self, size: usize) -> Result<BufferId, NpuError>;
    fn destroy_buffer(&mut self, id: BufferId) -> Result<(), NpuError>;
    fn write_buffer(&mut self, id: BufferId, data: &[u8]) -> Result<(), NpuError>;
    fn read_buffer(&mut self, id: BufferId) -> Result<Vec<u8>, NpuError>;
    fn execute(&mut self, request: InferenceRequest) -> Result<InferenceResult, NpuError>;
    fn execute_async(&mut self, request: InferenceRequest) -> Result<InferenceHandle, NpuError>;
    fn wait(&mut self, handle: InferenceHandle) -> Result<InferenceResult, NpuError>;
    fn capabilities(&self) -> NpuCapabilities;
    fn stats(&self) -> NpuStats;
    fn shutdown(&mut self) -> Result<(), NpuError>;
    fn name(&self) -> &str;
}
```

### NPU 推理 API 关键类型

#### ModelData

```rust
pub struct ModelData {
    pub format: ModelFormat,  // Onnx, TFLite, Custom
    pub data: Vec<u8>,
    pub name: String,
}
```

#### InferenceRequest

```rust
pub struct InferenceRequest {
    pub model_id: ModelId,
    pub input_buffers: Vec<BufferId>,
    pub output_buffers: Vec<BufferId>,
    pub priority: u32,        // 0 = 最高
    pub timeout_ms: u32,      // 0 = 无超时
}
```

#### InferenceResult

```rust
pub struct InferenceResult {
    pub output_buffers: Vec<BufferId>,
    pub inference_time_us: u64,
    pub success: bool,
}
```

#### NpuCapabilities

```rust
pub struct NpuCapabilities {
    pub max_model_size: usize,
    pub max_models: usize,
    pub max_buffer_size: usize,
    pub max_buffers: usize,
    pub supported_formats: Vec<ModelFormat>,
    pub async_execution: bool,
    pub quantization: bool,
    pub num_cores: u32,
    pub frequency_mhz: u32,
    pub total_memory: usize,
}
```

### C API

#### nuva_npu_init / nuva_npu_shutdown

```c
nuva_result_t nuva_npu_init(void);
nuva_result_t nuva_npu_shutdown(void);
```

#### nuva_npu_load_model / nuva_npu_unload_model

```c
nuva_result_t nuva_npu_load_model(
    nuva_npu_device_t device,
    const void* model_data,
    size_t model_size,
    nuva_npu_model_t* model
);
nuva_result_t nuva_npu_unload_model(nuva_npu_model_t model);
```

#### nuva_npu_execute

```c
nuva_result_t nuva_npu_execute(
    nuva_npu_model_t model,
    const nuva_npu_buffer_t* inputs,
    uint32_t input_count,
    nuva_npu_buffer_t* outputs,
    uint32_t output_count
);
```

#### nuva_npu_execute_async / nuva_npu_wait

```c
nuva_result_t nuva_npu_execute_async(
    nuva_npu_model_t model,
    const nuva_npu_buffer_t* inputs,
    uint32_t input_count,
    nuva_npu_buffer_t* outputs,
    uint32_t output_count,
    nuva_npu_handle_t* handle
);

nuva_result_t nuva_npu_wait(nuva_npu_handle_t handle, uint32_t timeout_ms);
```

#### DaVinci NPU 操作（davinci_npu_ops）

DaVinci NPU 通过 `davinci_npu_ops()` 获取 HAL 操作集合，提供昇腾 DaVinci 架构专用接口：

```rust
pub fn davinci_npu_ops() -> &'static DaVinciNpuOps {
    // Returns HAL ops table for HiSilicon DaVinci NPU
}

pub struct DaVinciNpuOps {
    pub aicore_execute: fn(task: &AicoreTask) -> Result<(), NpuError>,
    pub aicore_wait: fn(task_id: u32) -> Result<AicoreResult, NpuError>,
    pub tiling_update: fn(model: ModelId, tiling: &TilingData) -> Result<(), NpuError>,
    pub get_aicore_count: fn() -> u32,
    pub get_vector_core_count: fn() -> u32,
}
```

C API：

```c
nuva_result_t nuva_davinci_npu_aicore_execute(
    nuva_npu_device_t device,
    const nuva_aicore_task_t* task
);
nuva_result_t nuva_davinci_npu_tiling_update(
    nuva_npu_model_t model,
    const void* tiling_data,
    size_t tiling_size
);
```

---

## Quantum HAL

### Rust Trait 完整签名

#### PqcProvider (Kyber + Dilithium)

```rust
pub trait PqcProvider: Send + Sync {
    fn kyber_keygen(&self, variant: KyberVariant) -> Result<(PublicKey, SecretKey), PqcError>;
    fn kyber_encapsulate(&self, pk: &PublicKey) -> Result<(SharedSecret, Ciphertext), PqcError>;
    fn kyber_decapsulate(&self, sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, PqcError>;
    fn dilithium_keygen(&self, variant: DilithiumVariant) -> Result<(PublicKey, SecretKey), PqcError>;
    fn dilithium_sign(&self, sk: &SecretKey, msg: &[u8]) -> Result<Signature, PqcError>;
    fn dilithium_verify(&self, pk: &PublicKey, msg: &[u8], sig: &Signature) -> Result<bool, PqcError>;
    fn name(&self) -> &str;
    fn supported_algorithms(&self) -> Vec<PqcAlgorithm>;
    fn is_supported(&self, algo: PqcAlgorithm) -> bool;
}
```

#### QrngProvider (QRNG)

```rust
pub trait QrngProvider: Send + Sync {
    fn generate(&self, len: usize) -> Result<Vec<u8>, QrngError>;
    fn generate_u32(&self) -> Result<u32, QrngError>;
    fn generate_u64(&self) -> Result<u64, QrngError>;
    fn generate_range(&self, max: u64) -> Result<u64, QrngError>;
    fn verify_randomness(&self, data: &[u8]) -> Result<RandomnessQuality, QrngError>;
    fn entropy_level(&self) -> u8;
    fn name(&self) -> &str;
    fn is_quantum_source_available(&self) -> bool;
}
```

### Kyber 密钥大小

| 变体 | 公钥 (字节) | 私钥 (字节) | 密文 (字节) | 共享密钥 (字节) |
|------|-------------|-------------|-------------|-----------------|
| `Kyber512` | 800 | 1632 | 768 | 32 |
| `Kyber768` | 1184 | 2400 | 1088 | 32 |
| `Kyber1024` | 1568 | 3168 | 1568 | 32 |

### Dilithium 密钥大小

| 变体 | 公钥 (字节) | 私钥 (字节) | 签名 (字节) |
|------|-------------|-------------|-------------|
| `Dilithium2` | 1312 | 2560 | 2420 |
| `Dilithium3` | 1952 | 4032 | 3293 |
| `Dilithium5` | 2592 | 4864 | 4595 |

### QRNG 随机性质量检测 (NIST SP 800-22)

```rust
pub struct RandomnessQuality {
    pub monobit_test: f64,
    pub frequency_block_test: f64,
    pub runs_test: f64,
    pub longest_run_test: f64,
    pub serial_test: f64,
    pub approximate_entropy_test: f64,
    pub cumulative_sum_test: f64,
    pub overall_score: u8,
    pub is_random: bool,
}
```

### C API — Kyber

#### nuva_pqc_kyber_keygen

```c
nuva_result_t nuva_pqc_kyber_keygen(
    nuva_pqc_t pqc,
    nuva_kyber_variant_t variant,
    nuva_key_t* public_key,
    nuva_key_t* secret_key
);
```

变体：`NUVA_KYBER_512`、`NUVA_KYBER_768`（推荐）、`NUVA_KYBER_1024`

#### nuva_pqc_kyber_encapsulate

```c
nuva_result_t nuva_pqc_kyber_encapsulate(
    nuva_pqc_t pqc,
    nuva_key_t public_key,
    uint8_t* shared_secret,
    size_t* shared_secret_size,
    uint8_t* ciphertext,
    size_t* ciphertext_size
);
```

#### nuva_pqc_kyber_decapsulate

```c
nuva_result_t nuva_pqc_kyber_decapsulate(
    nuva_pqc_t pqc,
    nuva_key_t secret_key,
    const uint8_t* ciphertext,
    size_t ciphertext_size,
    uint8_t* shared_secret,
    size_t* shared_secret_size
);
```

### C API — Dilithium

#### nuva_pqc_dilithium_keygen

```c
nuva_result_t nuva_pqc_dilithium_keygen(
    nuva_pqc_t pqc,
    nuva_dilithium_variant_t variant,
    nuva_key_t* public_key,
    nuva_key_t* secret_key
);
```

变体：`NUVA_DILITHIUM_2`、`NUVA_DILITHIUM_3`（推荐）、`NUVA_DILITHIUM_5`

#### nuva_pqc_dilithium_sign

```c
nuva_result_t nuva_pqc_dilithium_sign(
    nuva_pqc_t pqc,
    nuva_key_t secret_key,
    const uint8_t* message,
    size_t message_size,
    uint8_t* signature,
    size_t* signature_size
);
```

#### nuva_pqc_dilithium_verify

```c
nuva_result_t nuva_pqc_dilithium_verify(
    nuva_pqc_t pqc,
    nuva_key_t public_key,
    const uint8_t* message,
    size_t message_size,
    const uint8_t* signature,
    size_t signature_size,
    bool* valid
);
```

### C API — QRNG

#### nuva_qrng_init / nuva_qrng_generate

```c
nuva_result_t nuva_qrng_init(nuva_qrng_t* qrng);
nuva_result_t nuva_qrng_generate(nuva_qrng_t qrng, uint8_t* buffer, size_t size);
```

### C API — QKD

QKD（量子密钥分发）模块 `hal/quantum/qkd/` 处于规划阶段，API 待定义。

---

## Power HAL

### nuva_power_set_state / nuva_power_get_state

```c
nuva_result_t nuva_power_set_state(nuva_handle_t device, nuva_power_state_t state);
nuva_result_t nuva_power_get_state(nuva_handle_t device, nuva_power_state_t* state);
```

电源状态：
- `NUVA_POWER_ON`：设备开启
- `NUVA_POWER_SLEEP`：设备休眠
- `NUVA_POWER_SUSPEND`：设备挂起
- `NUVA_POWER_OFF`：设备关闭

---

## 网络 API

### NFS RPC 调用与 XDR 解码

NFS 客户端通过 `rpc_call` 发起远程过程调用，通过 XDR 解码解析响应：

```rust
pub fn rpc_call(
    program: u32,
    version: u32,
    procedure: u32,
    args: &[u8],
    xdr_encode: fn(&[u8]) -> Vec<u8>,
) -> Result<RpcReply, RpcError>;

pub fn xdr_decode<T: XdrDecodable>(data: &[u8]) -> Result<T, XdrError>;
```

C API：

```c
nuva_result_t nuva_nfs_rpc_call(
    uint32_t program,
    uint32_t version,
    uint32_t procedure,
    const void* args,
    size_t args_size,
    void* reply,
    size_t* reply_size
);

nuva_result_t nuva_nfs_xdr_decode(
    const void* data,
    size_t data_size,
    void* decoded,
    size_t* decoded_size,
    nuva_xdr_type_t type
);
```

### SMB 请求/响应

SMB 客户端通过 `send_and_receive` 发送请求并接收响应，`parse_reply_header` 解析 SMB 响应头：

```rust
pub fn send_and_receive(
    session: &SmbSession,
    request: &SmbRequest,
) -> Result<SmbRawReply, SmbError>;

pub fn parse_reply_header(raw: &[u8]) -> Result<SmbReplyHeader, SmbError>;
```

C API：

```c
nuva_result_t nuva_smb_send_and_receive(
    nuva_smb_session_t session,
    const nuva_smb_request_t* request,
    nuva_smb_raw_reply_t* reply
);

nuva_result_t nuva_smb_parse_reply_header(
    const void* raw,
    size_t raw_size,
    nuva_smb_reply_header_t* header
);
```

---

## 插件 API

### Rust Plugin Trait 完整签名

```rust
pub trait Plugin: Send + Sync {
    fn meta(&self) -> &PluginMeta;
    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;
    fn activate(&mut self) -> Result<(), PluginError>;
    fn deactivate(&mut self) -> Result<(), PluginError>;
    fn unload(&mut self) -> Result<(), PluginError>;
}
```

### PluginMeta

```rust
pub struct PluginMeta {
    pub name: &'static str,
    pub version: Version,
    pub plugin_type: PluginType,
    pub dependencies: Vec<Dependency>,
    pub capabilities: Capabilities,
    pub author: &'static str,
    pub description: &'static str,
    pub priority: u32,
    pub flags: PluginFlags,
}
```

### PluginType

```rust
pub enum PluginType {
    Driver,
    FileSystem,
    Network,
    Security,
    // ...
}
```

### PluginServices 新方法

`PluginServices` 提供插件生命周期管理的服务接口：

```rust
pub trait PluginServices: Send + Sync {
    fn install(&self, source: &PluginSource) -> Result<PluginId, PluginError>;
    fn uninstall(&self, id: PluginId) -> Result<(), PluginError>;
    fn update(&self, id: PluginId) -> Result<Version, PluginError>;
    fn query(&self, filter: &PluginFilter) -> Vec<PluginMeta>;
    fn resolve_dependencies(&self, id: PluginId) -> Result<Vec<PluginId>, PluginError>;
}
```

### PluginLoader API 变更

`load_from_elf` 方法不再返回 `Err`，加载失败时返回 `Ok` 包含错误状态的插件实例：

```rust
impl PluginLoader {
    pub fn load_from_elf(&self, path: &str) -> Result<Arc<dyn Plugin>, PluginError>;
    pub fn load(&self, source: &PluginSource) -> Result<Arc<dyn Plugin>, PluginError>;
}
```

> **注意**：`load_from_elf` 的行为已变更 —— ELF 格式错误或签名验证失败时返回 `Ok(plugin)` 并在 `plugin.meta().flags` 中标记错误状态，而非返回 `Err`。新增 `load()` 方法支持从多种源（ELF、内存、网络）加载插件。

### 插件系统模块

| 模块 | 文件 | 职责 |
|------|------|------|
| core | `plugin/core/` | Plugin trait 定义和类型 |
| loader | `plugin/loader.rs` | 插件加载器 |
| manager | `plugin/manager.rs` | 插件管理器 |
| registry | `plugin/registry.rs` | 插件注册表 |
| sandbox | `plugin/sandbox.rs` | 插件沙箱 |
| services | `plugin/services.rs` | 插件服务接口 |
| legacy | `plugin/legacy.rs` | 遗留插件兼容 |

---

## SDK API

### Debug Target

调试目标提供进程级调试能力：

```rust
pub fn fork() -> Result<Pid, DebugError>;
pub fn execv(path: &str, args: &[&str]) -> Result<(), DebugError>;
pub fn ptrace(request: PtraceRequest, pid: Pid, addr: usize, data: usize) -> Result<usize, DebugError>;
pub fn process_vm_readv(pid: Pid, local_iov: &[IoVec], remote_iov: &[IoVec]) -> Result<usize, DebugError>;
```

C API：

```c
pid_t nuva_debug_fork(void);
nuva_result_t nuva_debug_execv(const char* path, char* const argv[]);
nuva_result_t nuva_debug_ptrace(
    nuva_ptrace_request_t request,
    pid_t pid,
    void* addr,
    void* data,
    uintptr_t* result
);
nuva_result_t nuva_debug_process_vm_readv(
    pid_t pid,
    const nuva_iovec_t* local_iov,
    size_t local_cnt,
    const nuva_iovec_t* remote_iov,
    size_t remote_cnt,
    size_t* bytes_read
);
```

### Profiler

性能分析器通过 `/proc` 文件系统和 `gettid` 提供采样能力：

```rust
pub fn proc_read(pid: Pid, entry: ProcEntry) -> Result<String, ProfilerError>;
pub fn gettid() -> Pid;
```

C API：

```c
nuva_result_t nuva_profiler_proc_read(
    pid_t pid,
    nuva_proc_entry_t entry,
    char* buffer,
    size_t buffer_size,
    size_t* bytes_written
);
pid_t nuva_profiler_gettid(void);
```

### Package

包管理器支持 HTTP 协议的远程包获取：

```rust
pub fn fetch_package_http(url: &str, dest: &mut [u8]) -> Result<usize, PackageError>;
```

C API：

```c
nuva_result_t nuva_package_fetch_http(
    const char* url,
    void* dest,
    size_t dest_size,
    size_t* bytes_written
);
```

---

## 数据结构

### nuva_cpu_info_t

```c
typedef struct {
    uint32_t core_count;        // CPU 核心数
    uint32_t frequency_mhz;     // CPU 频率 (MHz)
    uint32_t cache_line_size;   // 缓存行大小 (字节)
    uint64_t total_memory;      // 总内存 (字节)
    char vendor[32];            // CPU 厂商
    char model[64];             // CPU 型号
} nuva_cpu_info_t;
```

### nuva_gpu_info_t

```c
typedef struct {
    uint32_t device_id;         // 设备 ID
    uint32_t vendor_id;         // 厂商 ID
    uint64_t memory_size;       // 显存大小 (字节)
    uint32_t compute_units;     // 计算单元数
    char name[64];              // 设备名称
} nuva_gpu_info_t;
```

### nuva_npu_info_t

```c
typedef struct {
    uint32_t device_id;         // 设备 ID
    uint32_t vendor_id;         // 厂商 ID
    uint64_t memory_size;       // 内存大小 (字节)
    uint32_t num_cores;         // NPU 核心数
    uint32_t frequency_mhz;     // NPU 频率 (MHz)
    char name[64];              // 设备名称
} nuva_npu_info_t;
```

---

## 版本信息

```c
uint32_t nuva_hal_get_version(void);
// 版本格式: (major << 16) | (minor << 8) | patch

const char* nuva_hal_get_version_string(void);
// 返回类似 "1.0.0" 的字符串
```

---

## 线程安全

| 函数族 | 线程安全 | 备注 |
|--------|----------|------|
| `nuva_cpu_*` | 是 | 所有 CPU 函数线程安全 |
| `nuva_gpu_*` | 是 | 设备操作线程安全 |
| `nuva_npu_*` | 部分 | 模型执行可能需同步 |
| `nuva_qrng_*` | 是 | QRNG 线程安全 |
| `nuva_pqc_*` | 是 | PQC 操作线程安全 |

---

## 性能特征

| 操作 | 典型耗时 | 备注 |
|------|----------|------|
| `nuva_cpu_get_info` | <1μs | 缓存的信息 |
| `nuva_gpu_create_buffer` | ~1ms | 取决于大小 |
| `nuva_npu_load_model` | ~100ms | 取决于模型大小 |
| `nuva_npu_execute` | ~10ms | 取决于模型 |
| `nuva_pqc_kyber_keygen` | <1ms | 硬件加速 |
| `nuva_pqc_dilithium_sign` | <1ms | 硬件加速 |
| `nuva_qrng_generate` | ~10μs | 量子熵源 |

---

**文档版本**：1.2.0  
**最后更新**：2026-05-15
