# Nuva HAL API Reference

> **Document Scope**: This document provides a detailed reference for the HAL (L0)
> layer APIs only, including complete Rust trait signatures, C/C++ API bindings,
> data structures, and performance characteristics. For a broader overview of all
> system APIs, see [API.md](../API.md).

**Version**: 1.2.0  
**Date**: 2026-05-15

---

## Table of Contents

1. [Overview](#overview)
2. [Error Handling](#error-handling)
3. [CPU HAL](#cpu-hal)
4. [GPU HAL](#gpu-hal)
5. [NPU HAL](#npu-hal)
6. [Quantum HAL](#quantum-hal)
7. [Power HAL](#power-hal)
8. [Network API](#network-api)
9. [Plugin API](#plugin-api)
10. [SDK API](#sdk-api)
11. [Data Structures](#data-structures)

---

## Overview

Nuva HAL (Hardware Abstraction Layer) provides a unified interface for accessing hardware features across different platforms. The API is designed to be:

- **Platform-independent**: Runs on x86_64, ARM64, and LoongArch64
- **C99 compatible**: Usable from both C and C++
- **Type-safe**: Strong types with opaque handles
- **Explicit errors**: All functions return error codes

### Header Files

```c
#include <nuva_hal.h>  // C API
```

```cpp
#include <nuva_hal.hpp>  // C++ API
```

### Rust Trait Interfaces

The HAL layer provides abstractions through Rust traits in the `hal/` directory:

- `hal::cpu` — CPU abstraction trait
- `hal::gpu` — GPU abstraction trait
- `hal::npu::NpuHal` — NPU abstraction trait
- `hal::quantum::pqc::PqcProvider` — Post-quantum crypto trait
- `hal::quantum::qrng::QrngProvider` — Quantum RNG trait
- `hal::power` — Power management trait
- `hal::loongarch64` — LoongArch64 platform (MMU, interrupts, SIMD, LVZ, LBT)

---

## Error Handling

### Error Codes

```c
typedef enum {
    NUVA_OK = 0,                    // Success
    NUVA_ERROR_INVALID_PARAM = -1,  // Invalid parameter
    NUVA_ERROR_NOT_FOUND = -2,      // Not found
    NUVA_ERROR_OUT_OF_MEMORY = -3,  // Out of memory
    NUVA_ERROR_NOT_SUPPORTED = -4,  // Not supported
    NUVA_ERROR_HARDWARE = -5,       // Hardware error
    NUVA_ERROR_TIMEOUT = -6,        // Timeout
    NUVA_ERROR_BUSY = -7,           // Busy
} nuva_result_t;
```

### C Error Checking Pattern

```c
nuva_result_t result = nuva_cpu_get_info(&info);
if (result != NUVA_OK) {
    printf("Error: %d\n", result);
    return result;
}
```

### C++ Exception Handling

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

- `info`: Pointer to `nuva_cpu_info_t` structure to fill
- Returns `NUVA_OK` or `NUVA_ERROR_INVALID_PARAM`

#### nuva_cpu_get_core_id

```c
uint32_t nuva_cpu_get_core_id(void);
```

Returns current core ID (0 to core_count-1).

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

### AI Scheduler API

The AI scheduler provides kernel-level AI task scheduling and CPU affinity selection:

```rust
pub fn notify_kernel_scheduler(event: AiSchedulerEvent) -> Result<(), SchedulerError>;
pub fn select_cpu_for_task(task: &AiTask) -> Result<CpuId, SchedulerError>;
```

- `notify_kernel_scheduler`: Notifies the kernel scheduler of AI workload changes (e.g., new inference request, model loading complete)
- `select_cpu_for_task`: Selects the optimal CPU core based on AI task characteristics (compute-bound / IO-bound, NPU affinity, etc.)

C API:

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

### Rust Trait Complete Signature

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

### NPU Inference API Key Types

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
    pub priority: u32,        // 0 = highest
    pub timeout_ms: u32,      // 0 = no timeout
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

#### DaVinci NPU Operations (davinci_npu_ops)

DaVinci NPU obtains the HAL operation set via `davinci_npu_ops()`, providing HiSilicon DaVinci architecture-specific interfaces:

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

C API:

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

### Rust Trait Complete Signatures

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

### Kyber Key Sizes

| Variant | Public Key (bytes) | Secret Key (bytes) | Ciphertext (bytes) | Shared Secret (bytes) |
|---------|-------------------|-------------------|-------------------|----------------------|
| `Kyber512` | 800 | 1632 | 768 | 32 |
| `Kyber768` | 1184 | 2400 | 1088 | 32 |
| `Kyber1024` | 1568 | 3168 | 1568 | 32 |

### Dilithium Key Sizes

| Variant | Public Key (bytes) | Secret Key (bytes) | Signature (bytes) |
|---------|-------------------|-------------------|------------------|
| `Dilithium2` | 1312 | 2560 | 2420 |
| `Dilithium3` | 1952 | 4032 | 3293 |
| `Dilithium5` | 2592 | 4864 | 4595 |

### QRNG Randomness Quality Tests (NIST SP 800-22)

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

Variants: `NUVA_KYBER_512`, `NUVA_KYBER_768` (recommended), `NUVA_KYBER_1024`

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

Variants: `NUVA_DILITHIUM_2`, `NUVA_DILITHIUM_3` (recommended), `NUVA_DILITHIUM_5`

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

The QKD (Quantum Key Distribution) module at `hal/quantum/qkd/` is in the planning phase; API is TBD.

---

## Power HAL

### nuva_power_set_state / nuva_power_get_state

```c
nuva_result_t nuva_power_set_state(nuva_handle_t device, nuva_power_state_t state);
nuva_result_t nuva_power_get_state(nuva_handle_t device, nuva_power_state_t* state);
```

Power states:
- `NUVA_POWER_ON`: Device powered on
- `NUVA_POWER_SLEEP`: Device sleeping
- `NUVA_POWER_SUSPEND`: Device suspended
- `NUVA_POWER_OFF`: Device powered off

---

## Network API

### NFS RPC Call and XDR Decoding

The NFS client initiates remote procedure calls via `rpc_call` and parses responses via XDR decoding:

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

C API:

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

### SMB Request/Response

The SMB client sends requests and receives responses via `send_and_receive`, and parses SMB reply headers via `parse_reply_header`:

```rust
pub fn send_and_receive(
    session: &SmbSession,
    request: &SmbRequest,
) -> Result<SmbRawReply, SmbError>;

pub fn parse_reply_header(raw: &[u8]) -> Result<SmbReplyHeader, SmbError>;
```

C API:

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

## Plugin API

### Rust Plugin Trait Complete Signature

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

### Plugin System Modules

| Module | File | Responsibility |
|--------|------|---------------|
| core | `plugin/core/` | Plugin trait definitions and types |
| loader | `plugin/loader.rs` | Plugin loader |
| manager | `plugin/manager.rs` | Plugin manager |
| registry | `plugin/registry.rs` | Plugin registry |
| sandbox | `plugin/sandbox.rs` | Plugin sandbox |
| services | `plugin/services.rs` | Plugin services interface |
| legacy | `plugin/legacy.rs` | Legacy plugin compatibility |

### PluginServices New Methods

`PluginServices` provides a service interface for plugin lifecycle management:

```rust
pub trait PluginServices: Send + Sync {
    fn install(&self, source: &PluginSource) -> Result<PluginId, PluginError>;
    fn uninstall(&self, id: PluginId) -> Result<(), PluginError>;
    fn update(&self, id: PluginId) -> Result<Version, PluginError>;
    fn query(&self, filter: &PluginFilter) -> Vec<PluginMeta>;
    fn resolve_dependencies(&self, id: PluginId) -> Result<Vec<PluginId>, PluginError>;
}
```

### PluginLoader API Changes

The `load_from_elf` method no longer returns `Err`; on load failure, it returns `Ok` containing a plugin instance in an error state:

```rust
impl PluginLoader {
    pub fn load_from_elf(&self, path: &str) -> Result<Arc<dyn Plugin>, PluginError>;
    pub fn load(&self, source: &PluginSource) -> Result<Arc<dyn Plugin>, PluginError>;
}
```

> **Note**: `load_from_elf` behavior has changed — ELF format errors or signature verification failures now return `Ok(plugin)` with the error state flagged in `plugin.meta().flags`, instead of returning `Err`. The new `load()` method supports loading plugins from multiple sources (ELF, memory, network).

---

## SDK API

### Debug Target

The debug target provides process-level debugging capabilities:

```rust
pub fn fork() -> Result<Pid, DebugError>;
pub fn execv(path: &str, args: &[&str]) -> Result<(), DebugError>;
pub fn ptrace(request: PtraceRequest, pid: Pid, addr: usize, data: usize) -> Result<usize, DebugError>;
pub fn process_vm_readv(pid: Pid, local_iov: &[IoVec], remote_iov: &[IoVec]) -> Result<usize, DebugError>;
```

C API:

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

The profiler provides sampling capabilities via the `/proc` filesystem and `gettid`:

```rust
pub fn proc_read(pid: Pid, entry: ProcEntry) -> Result<String, ProfilerError>;
pub fn gettid() -> Pid;
```

C API:

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

The package manager supports HTTP-based remote package fetching:

```rust
pub fn fetch_package_http(url: &str, dest: &mut [u8]) -> Result<usize, PackageError>;
```

C API:

```c
nuva_result_t nuva_package_fetch_http(
    const char* url,
    void* dest,
    size_t dest_size,
    size_t* bytes_written
);
```

---

## Data Structures

### nuva_cpu_info_t

```c
typedef struct {
    uint32_t core_count;        // Number of CPU cores
    uint32_t frequency_mhz;     // CPU frequency (MHz)
    uint32_t cache_line_size;   // Cache line size (bytes)
    uint64_t total_memory;      // Total memory (bytes)
    char vendor[32];            // CPU vendor
    char model[64];             // CPU model
} nuva_cpu_info_t;
```

### nuva_gpu_info_t

```c
typedef struct {
    uint32_t device_id;         // Device ID
    uint32_t vendor_id;         // Vendor ID
    uint64_t memory_size;       // VRAM size (bytes)
    uint32_t compute_units;     // Number of compute units
    char name[64];              // Device name
} nuva_gpu_info_t;
```

### nuva_npu_info_t

```c
typedef struct {
    uint32_t device_id;         // Device ID
    uint32_t vendor_id;         // Vendor ID
    uint64_t memory_size;       // Memory size (bytes)
    uint32_t num_cores;         // Number of NPU cores
    uint32_t frequency_mhz;     // NPU frequency (MHz)
    char name[64];              // Device name
} nuva_npu_info_t;
```

---

## Version Information

```c
uint32_t nuva_hal_get_version(void);
// Version format: (major << 16) | (minor << 8) | patch

const char* nuva_hal_get_version_string(void);
// Returns string like "1.0.0"
```

---

## Thread Safety

| Function Family | Thread Safe | Notes |
|-----------------|-------------|-------|
| `nuva_cpu_*` | Yes | All CPU functions are thread-safe |
| `nuva_gpu_*` | Yes | Device operations are thread-safe |
| `nuva_npu_*` | Partial | Model execution may require synchronization |
| `nuva_qrng_*` | Yes | QRNG is thread-safe |
| `nuva_pqc_*` | Yes | PQC operations are thread-safe |

---

## Performance Characteristics

| Operation | Typical Time | Notes |
|-----------|-------------|-------|
| `nuva_cpu_get_info` | <1μs | Cached information |
| `nuva_gpu_create_buffer` | ~1ms | Depends on size |
| `nuva_npu_load_model` | ~100ms | Depends on model size |
| `nuva_npu_execute` | ~10ms | Depends on model |
| `nuva_pqc_kyber_keygen` | <1ms | Hardware accelerated |
| `nuva_pqc_dilithium_sign` | <1ms | Hardware accelerated |
| `nuva_qrng_generate` | ~10μs | Quantum entropy source |

---

**Document Version**: 1.2.0  
**Last Updated**: 2026-05-15
