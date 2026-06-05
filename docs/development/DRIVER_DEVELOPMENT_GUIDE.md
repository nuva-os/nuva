# C/C++ Driver Development Guide

**Version**: 1.2.0  
**Date**: 2026-05-15  
**Status**: Released

---

## 1. Overview

This guide provides the complete C/C++ driver development workflow for Nuva OS, including:

- HAL C API usage
- C++ RAII wrappers
- Driver Framework (Device Model/Bus/IRQ/DMA/GPIO/I2C/SPI)
- Device Tree matching and parsing
- Power Management integration
- DMA-BUF shared buffer framework
- Driver lifecycle management
- Best practices and examples

### 1.1 Target Audience

- Driver development engineers
- System integration engineers
- Hardware vendor development teams

### 1.2 Prerequisites

- C/C++ programming fundamentals
- Hardware Abstraction Layer concepts
- Operating system driver model
- Device Tree basics

---

## 2. Driver Framework Overview

The Nuva OS driver framework is located at `kernel/driver/` with a layered design:

```
kernel/driver/
├── framework/          # Core driver framework
│   ├── display.rs      #   Display framework
│   └── input.rs        #   Input framework
├── class/              # Device classes
│   ├── audio.rs        #   Audio device
│   ├── backlight.rs    #   Backlight device
│   ├── bluetooth.rs    #   Bluetooth device
│   ├── camera.rs       #   Camera
│   ├── display.rs      #   Display device
│   ├── input.rs        #   Input device
│   ├── led.rs          #   LED
│   ├── nfc.rs          #   NFC
│   ├── power.rs        #   Power device
│   ├── sensor.rs       #   Sensor
│   ├── storage.rs      #   Storage device
│   ├── touch.rs        #   Touchscreen
│   ├── usb.rs          #   USB
│   ├── vibrator.rs     #   Vibrator
│   ├── wifi.rs         #   WiFi
│   └── ...
├── impl/               # Driver implementations
│   └── irqchip/        #   Interrupt controller implementations
├── device.rs           # Device trait
├── dma.rs              # DMA channel abstraction
├── dmabuf.rs           # DMA-BUF shared buffer
├── gpio.rs             # GPIO subsystem
├── i2c.rs              # I2C bus
├── spi.rs              # SPI bus
├── irq.rs              # IRQ management
├── pm.rs               # Driver power management
├── dt.rs               # Device tree matching
├── block.rs            # Block device
├── char.rs             # Character device
├── clk.rs              # Clock framework
├── freq.rs             # Frequency scaling
├── opp.rs              # Operating Performance Points
├── phy.rs              # PHY abstraction
├── pinctrl.rs          # Pin control
├── pwm.rs              # PWM
├── regulator.rs        # Voltage regulator
├── reset.rs            # Reset control
├── rtc.rs              # Real-time clock
├── thermal.rs          # Thermal management
├── watchdog.rs         # Watchdog
├── event.rs            # Event notification
├── mfd.rs              # Multi-function device
├── icc.rs              # Inter-core connectivity
└── adapter.rs          # Bus adapter
```

### 2.1 Device Model

All devices must implement the `Device` trait (`device.rs`):

```rust
pub trait Device: Send + Sync {
    fn name(&self) -> &str;
    fn device_id(&self) -> DeviceId;
    fn bus_type(&self) -> BusType;
    fn power_state(&self) -> PowerState;
    fn suspend(&mut self) -> Result<(), DriverError>;
    fn resume(&mut self) -> Result<(), DriverError>;
}
```

### 2.2 Bus Types

Nuva OS supports the following bus types:

| Bus | File | Description |
|-----|------|-------------|
| I2C | `i2c.rs` | Two-wire serial bus for low-speed peripherals |
| SPI | `spi.rs` | Serial Peripheral Interface for mid-speed peripherals |
| IRQ | `irq.rs` | Interrupt request bus |
| Platform | implicit | Platform devices (device tree matching) |

### 2.3 IRQ Management

Interrupt management is implemented through `irq.rs` and `impl/irqchip/`:

- IRQ allocation and release
- Interrupt handler registration
- Interrupt affinity setting (multi-core)
- Interrupt priority management
- GIC (Generic Interrupt Controller) driver implementation

### 2.4 DMA Framework

The DMA subsystem consists of two core components:

**DMA Channel** (`dma.rs`):
- DMA channel request and release
- Single/cyclic transfer configuration
- Transfer completion callbacks

**DMA-BUF Shared Buffer** (`dmabuf.rs`):
- Zero-copy buffer sharing mechanism
- Cross-device buffer passing
- Memory mapping and synchronization
- Fence synchronization primitives

### 2.5 GPIO Subsystem

`gpio.rs` provides generic GPIO control:

- GPIO request and release
- Input/output direction setting
- Level read/write
- Interrupt trigger mode configuration (edge/level)
- GPIO controller driver registration

### 2.6 I2C Bus Driver

`i2c.rs` provides I2C bus abstraction:

- I2C adapter registration
- I2C message transfer (`i2c_transfer`)
- SMBus protocol support
- I2C device probing and registration
- Device tree I2C node matching

### 2.7 SPI Bus Driver

`spi.rs` provides SPI bus abstraction:

- SPI master controller registration
- SPI message transfer (`spi_transfer`)
- SPI mode configuration (CPOL/CPHA)
- Chip select management
- SPI device registration

---

## 3. Device Tree Matching

### 3.1 Device Tree Parsing

Nuva OS provides device tree support through `kernel/driver/dt.rs` and `hal/dt.rs`:

- ARM64: Parse FDT (Flattened Device Tree) via `hal/dt.rs`
- x86_64: Parse ACPI tables via `hal/acpi.rs`
- LoongArch64: Device tree support

### 3.2 Driver Device Tree Matching

Drivers match device tree nodes via compatible strings:

```c
static const char* compatible_strings[] = {
    "vendor,device-name",
    NULL
};

nuva_driver_info_t my_driver = {
    .name = "my_device",
    .abi_version = DDF_ABI_VERSION,
    .compatible = compatible_strings,
    .init = my_driver_init,
    .cleanup = my_driver_cleanup,
    .probe = my_driver_probe,
    .remove = my_driver_remove,
    .suspend = my_driver_suspend,
    .resume = my_driver_resume,
};
```

### 3.3 Device Tree Node Example

```dts
my_device: device@10000000 {
    compatible = "vendor,device-name";
    reg = <0x10000000 0x1000>;
    interrupts = <0 32 4>;
    clocks = <&clk_device>;
    resets = <&reset_device>;
    vdd-supply = <&regulator_1v8>;
};
```

---

## 4. Power Management Integration

### 4.1 Driver Power Management

Driver power management is integrated through `kernel/driver/pm.rs`:

- Runtime power management (runtime PM)
- System suspend/resume callbacks
- Device idle state management
- Wake source configuration

### 4.2 Power States

```c
typedef enum {
    NUVA_POWER_ON = 0,       // Device powered on
    NUVA_POWER_SLEEP = 1,    // Device sleeping
    NUVA_POWER_SUSPEND = 2,  // Device suspended
    NUVA_POWER_OFF = 3,      // Device powered off
} nuva_power_state_t;
```

### 4.3 Driver PM Callbacks

```c
typedef struct {
    nuva_result_t (*suspend)(nuva_handle_t device, nuva_power_state_t state);
    nuva_result_t (*resume)(nuva_handle_t device);
    nuva_result_t (*runtime_suspend)(nuva_handle_t device);
    nuva_result_t (*runtime_resume)(nuva_handle_t device);
    nuva_result_t (*idle)(nuva_handle_t device);
} nuva_driver_pm_ops_t;
```

### 4.4 HAL Power Management

HAL layer power management (`hal/power/`) provides:

- PMIC control (`hal/power/pmic.rs`)
- System suspend/resume flow (`hal/power/suspend.rs`)

---

## 5. DMA-BUF Framework

### 5.1 Overview

The DMA-BUF framework (`kernel/driver/dmabuf.rs`) provides a zero-copy buffer sharing mechanism that allows different devices to share the same memory region without data copying.

### 5.2 Core API

```c
// Export buffer as dma-buf
nuva_dmabuf_t dmabuf;
nuva_result_t result = nuva_dmabuf_export(device, buffer, size, &dmabuf);

// Get dma-buf fd (for cross-process sharing)
int fd = nuva_dmabuf_get_fd(dmabuf);

// Get dma-buf from fd
nuva_dmabuf_t imported;
result = nuva_dmabuf_get_from_fd(fd, &imported);

// Map dma-buf for device access
void* vaddr;
result = nuva_dmabuf_map(dmabuf, NUVA_DMA_BIDIRECTIONAL, &vaddr);

// Sync operation
nuva_dmabuf_sync(dmabuf, offset, length, NUVA_DMA_SYNC_FOR_DEVICE);

// Unmap
nuva_dmabuf_unmap(dmabuf, vaddr);

// Release
nuva_dmabuf_put(dmabuf);
```

### 5.3 Fence Synchronization

DMA-BUF supports fence synchronization primitives to ensure safe buffer passing between devices:

```c
// Create fence
nuva_fence_t fence;
nuva_fence_create(&fence);

// Wait for fence completion
result = nuva_fence_wait(fence, timeout_ms);

// Signal fence after device operation completes
nuva_fence_signal(fence);
```

---

## 6. Quick Start

### 6.1 Environment Setup

#### Install Development Tools

```bash
# Install Nuva SDK
sudo apt install nuva-sdk

# Install build toolchain
sudo apt install gcc g++ cmake

# Install debug tools
sudo apt install gdb nuva-debug
```

#### Configure Environment Variables

```bash
export NUVA_SDK=/opt/nuva/sdk
export NUVA_HAL_INCLUDE=$NUVA_SDK/include
export NUVA_HAL_LIB=$NUVA_SDK/lib
```

### 6.2 First Driver

#### C Example

```c
#include <nuva_hal.h>
#include <stdio.h>

int main() {
    uint32_t version = nuva_hal_get_version();
    printf("HAL Version: %s\n", nuva_hal_get_version_string());

    nuva_cpu_info_t cpu_info;
    nuva_result_t result = nuva_cpu_get_info(&cpu_info);
    
    if (result == NUVA_OK) {
        printf("CPU Cores: %u\n", cpu_info.core_count);
        printf("CPU Frequency: %u MHz\n", cpu_info.frequency_mhz);
        printf("Total Memory: %lu MB\n", 
               cpu_info.total_memory / (1024 * 1024));
    } else {
        printf("Failed to get CPU info: %d\n", result);
        return -1;
    }

    return 0;
}
```

#### C++ Example

```cpp
#include <nuva_hal.hpp>
#include <iostream>

int main() {
    try {
        std::cout << "HAL Version: " 
                  << nuva::get_version_string() 
                  << std::endl;

        auto cpu_info = nuva::Cpu::get_info();
        std::cout << "CPU Cores: " << cpu_info.core_count << std::endl;
        std::cout << "CPU Frequency: " << cpu_info.frequency_mhz << " MHz" << std::endl;
        std::cout << "Total Memory: " 
                  << cpu_info.total_memory / (1024 * 1024) 
                  << " MB" << std::endl;

    } catch (const nuva::Exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return -1;
    }

    return 0;
}
```

### 6.3 Build and Run

#### CMake Configuration

```cmake
cmake_minimum_required(VERSION 3.10)
project(nuva_driver_example)

find_package(NuvaHAL REQUIRED)

add_executable(driver_c main.c)
target_link_libraries(driver_c NuvaHAL::C)

add_executable(driver_cpp main.cpp)
target_link_libraries(driver_cpp NuvaHAL::Cpp)
```

#### Build Commands

```bash
mkdir build && cd build
cmake ..
make
```

---

## 7. HAL API Reference

### 7.1 CPU HAL

#### Get CPU Info

```c
nuva_cpu_info_t info;
nuva_result_t result = nuva_cpu_get_info(&info);
if (result == NUVA_OK) {
    // Use info
}
```

#### CPU Info Structure

```c
typedef struct {
    uint32_t core_count;        // Number of CPU cores
    uint32_t frequency_mhz;     // CPU frequency (MHz)
    uint32_t cache_line_size;   // Cache line size
    uint64_t total_memory;      // Total memory (bytes)
    char vendor[32];            // CPU vendor
    char model[64];             // CPU model
} nuva_cpu_info_t;
```

#### Interrupt Control

```c
nuva_cpu_disable_irq();
// Critical section
nuva_cpu_enable_irq();
```

#### Memory Barriers

```c
nuva_cpu_memory_barrier();   // Full memory barrier
nuva_cpu_read_barrier();     // Read barrier
nuva_cpu_write_barrier();    // Write barrier
```

### 7.2 GPU HAL

#### Initialize GPU

```c
nuva_result_t result = nuva_gpu_init();
if (result != NUVA_OK) {
    // Handle error
}

uint32_t device_count;
result = nuva_gpu_get_device_count(&device_count);

for (uint32_t i = 0; i < device_count; i++) {
    nuva_gpu_info_t info;
    result = nuva_gpu_get_device_info(i, &info);
    if (result == NUVA_OK) {
        printf("GPU %u: %s\n", i, info.name);
    }
}
```

#### Create GPU Buffer

```c
nuva_gpu_buffer_t buffer;
result = nuva_gpu_create_buffer(device, size, &buffer);
if (result == NUVA_OK) {
    // Use buffer
    nuva_gpu_destroy_buffer(buffer);
}
```

### 7.3 NPU HAL

#### Initialize NPU

```c
nuva_result_t result = nuva_npu_init();

uint32_t device_count;
result = nuva_npu_get_device_count(&device_count);

for (uint32_t i = 0; i < device_count; i++) {
    nuva_npu_info_t info;
    result = nuva_npu_get_device_info(i, &info);
    if (result == NUVA_OK) {
        printf("NPU %u: %s (%u cores)\n", 
               i, info.name, info.num_cores);
    }
}
```

#### Load and Execute Model

```c
nuva_npu_model_t model;
result = nuva_npu_load_model(device, model_data, model_size, &model);

nuva_npu_buffer_t input_buffer;
result = nuva_npu_create_buffer(device, input_size, &input_buffer);
result = nuva_npu_write_buffer(input_buffer, input_data, input_size);

nuva_npu_buffer_t output_buffer;
result = nuva_npu_create_buffer(device, output_size, &output_buffer);

result = nuva_npu_execute(model, &input_buffer, 1, &output_buffer, 1);
result = nuva_npu_read_buffer(output_buffer, output_data, output_size);

nuva_npu_destroy_buffer(input_buffer);
nuva_npu_destroy_buffer(output_buffer);
nuva_npu_unload_model(model);
```

### 7.4 DaVinci NPU HAL Bridging

DaVinci NPU (Ascend architecture) obtains the HAL operation set via `davinci_npu_ops()`. Driver developers must implement the following bridging steps:

#### 7.4.1 Get DaVinci NPU Operations

```c
const nuva_davinci_npu_ops_t* ops = nuva_davinci_npu_get_ops();
if (ops == NULL) {
    printf("DaVinci NPU not available\n");
    return -1;
}

uint32_t aicore_count = ops->get_aicore_count();
uint32_t vector_core_count = ops->get_vector_core_count();
printf("AiCore: %u, VectorCore: %u\n", aicore_count, vector_core_count);
```

#### 7.4.2 AiCore Task Execution

```c
nuva_aicore_task_t task;
task.model = model;
task.input_buffers = input_buffers;
task.output_buffers = output_buffers;

result = ops->aicore_execute(&task);
if (result != NUVA_OK) {
    return result;
}

nuva_aicore_result_t aicore_result;
result = ops->aicore_wait(task.id, &aicore_result);
```

#### 7.4.3 Tiling Data Update

```c
result = ops->tiling_update(model, tiling_data, tiling_size);
```

> **Note**: DaVinci NPU driver implementation is located at `hal/npu/davinci/` and must bridge to the generic NPU HAL trait via `davinci_npu_ops()`. The device tree compatible string is `"hisilicon,davinci-npu"`.

### 7.5 AI Scheduler Integration

The AI scheduler allows driver developers to coordinate NPU/GPU workloads with the kernel scheduler for optimal CPU affinity selection.

#### 7.5.1 Notify Kernel Scheduler

When the driver detects AI workload changes, it should notify the kernel scheduler:

```c
nuva_ai_scheduler_event_t event = NUVA_AI_SCHED_NEW_INFERENCE_REQUEST;
result = nuva_ai_notify_scheduler(event);
```

Event types:
- `NUVA_AI_SCHED_NEW_INFERENCE_REQUEST`: New inference request arrived
- `NUVA_AI_SCHED_MODEL_LOADED`: Model loading complete
- `NUVA_AI_SCHED_WORKLOAD_COMPLETE`: Workload complete
- `NUVA_AI_SCHED_NPU_BUSY`: NPU resource busy
- `NUVA_AI_SCHED_NPU_IDLE`: NPU resource idle

#### 7.5.2 Select Optimal CPU Core

```c
nuva_ai_task_t ai_task;
ai_task.type = NUVA_AI_TASK_INFERENCE;
ai_task.npu_affinity = npu_device_id;
ai_task.priority = 0;

uint32_t selected_cpu;
result = nuva_ai_select_cpu_for_task(&ai_task, &selected_cpu);
if (result == NUVA_OK) {
    printf("Optimal CPU core: %u\n", selected_cpu);
}
```

> **Note**: The AI scheduler API resides in the L1 kernel layer. Driver developers access it through the system call interface. In the EAS (Energy-Aware Scheduling) scheduler, AI scheduler integration provides energy-aware NPU task scheduling.

### 7.6 Quantum HAL

#### QRNG Usage

```c
nuva_qrng_t qrng;
nuva_result_t result = nuva_qrng_init(&qrng);

uint8_t random_bytes[32];
result = nuva_qrng_generate(qrng, random_bytes, 32);
```

#### PQC Key Generation

```c
nuva_pqc_t pqc;
result = nuva_pqc_init(&pqc);

nuva_key_t public_key, secret_key;
result = nuva_pqc_kyber_keygen(pqc, NUVA_KYBER_768, 
                                &public_key, &secret_key);

uint8_t shared_secret[32];
size_t shared_secret_size = 32;
uint8_t ciphertext[NUVA_KYBER_CIPHERTEXT_SIZE];
size_t ciphertext_size = NUVA_KYBER_CIPHERTEXT_SIZE;

result = nuva_pqc_kyber_encapsulate(pqc, public_key,
                                     shared_secret, &shared_secret_size,
                                     ciphertext, &ciphertext_size);

uint8_t decrypted_secret[32];
size_t decrypted_size = 32;

result = nuva_pqc_kyber_decapsulate(pqc, secret_key,
                                     ciphertext, ciphertext_size,
                                     decrypted_secret, &decrypted_size);

nuva_key_free(public_key);
nuva_key_free(secret_key);
```

---

## 8. C++ Advanced Features

### 8.1 RAII Resource Management

```cpp
{
    nuva::Gpu::Buffer buffer(device, 1024);
    // Use buffer, auto-destroyed on scope exit
}

{
    nuva::Npu::Model model(device, model_data, model_size);
    // Use model, auto-unloaded on scope exit
}
```

### 8.2 Exception Handling

```cpp
try {
    auto cpu_info = nuva::Cpu::get_info();
    // Use cpu_info
} catch (const nuva::Exception& e) {
    std::cerr << "Error: " << e.what() << std::endl;
    std::cerr << "Result code: " << e.result() << std::endl;
}
```

### 8.3 Move Semantics

```cpp
nuva::Npu::Buffer buffer1(device, 1024);
nuva::Npu::Buffer buffer2 = std::move(buffer1);
// buffer1 is now empty
// buffer2 owns the resource
```

---

## 9. Driver Lifecycle

### 9.1 Driver Registration

```c
typedef struct {
    const char* name;
    uint32_t abi_version;
    const char** compatible;
    nuva_result_t (*init)(void);
    void (*cleanup)(void);
    nuva_result_t (*probe)(nuva_handle_t device);
    void (*remove)(nuva_handle_t device);
} nuva_driver_info_t;

static nuva_result_t my_driver_init(void) {
    return NUVA_OK;
}

static void my_driver_cleanup(void) {
    // Cleanup
}

static nuva_result_t my_driver_probe(nuva_handle_t device) {
    // Device probe and initialization
    return NUVA_OK;
}

static void my_driver_remove(nuva_handle_t device) {
    // Device removal
}

static const char* my_compatibles[] = {
    "vendor,my-device",
    NULL
};

nuva_driver_info_t my_driver = {
    .name = "my_driver",
    .abi_version = DDF_ABI_VERSION,
    .compatible = my_compatibles,
    .init = my_driver_init,
    .cleanup = my_driver_cleanup,
    .probe = my_driver_probe,
    .remove = my_driver_remove,
};

nuva_driver_register(&my_driver);
```

### 9.2 Power Management Callbacks

```c
nuva_result_t result = nuva_power_set_state(device, NUVA_POWER_SUSPEND);

nuva_power_state_t state;
result = nuva_power_get_state(device, &state);
```

---

## 10. Best Practices

### 10.1 Error Handling

```c
nuva_result_t result = nuva_cpu_get_info(&info);
if (result != NUVA_OK) {
    switch (result) {
        case NUVA_ERROR_INVALID_PARAM:
            break;
        case NUVA_ERROR_NOT_SUPPORTED:
            break;
        default:
            break;
    }
    return result;
}
```

### 10.2 Resource Management

```c
nuva_npu_buffer_t buffer = NUVA_INVALID_HANDLE;
nuva_result_t result = nuva_npu_create_buffer(device, size, &buffer);
if (result != NUVA_OK) {
    return result;
}

// Use buffer

if (buffer != NUVA_INVALID_HANDLE) {
    nuva_npu_destroy_buffer(buffer);
}
```

### 10.3 Thread Safety

```c
nuva_cpu_write_barrier();
shared_data = value;
nuva_cpu_write_barrier();

// Another thread
nuva_cpu_read_barrier();
value = shared_data;
nuva_cpu_read_barrier();
```

### 10.4 Performance Optimization

```c
for (int i = 0; i < count; i++) {
    nuva_npu_execute(model, &inputs[i], 1, &outputs[i], 1);
}
```

---

## 11. Debugging Tips

### 11.1 Log Output

```c
#define DEBUG_LOG(fmt, ...) \
    printf("[DEBUG] " fmt "\n", ##__VA_ARGS__)

DEBUG_LOG("CPU cores: %u", info.core_count);
```

### 11.2 Error Tracing

```c
#define CHECK_RESULT(call) \
    do { \
        nuva_result_t _result = (call); \
        if (_result != NUVA_OK) { \
            printf("Error at %s:%d: %d\n", \
                   __FILE__, __LINE__, _result); \
            return _result; \
        } \
    } while (0)

CHECK_RESULT(nuva_cpu_get_info(&info));
```

### 11.3 Performance Profiling

```c
#include <time.h>

struct timespec start, end;
clock_gettime(CLOCK_MONOTONIC, &start);

nuva_npu_execute(model, inputs, 1, outputs, 1);

clock_gettime(CLOCK_MONOTONIC, &end);
double elapsed = (end.tv_sec - start.tv_sec) * 1000.0 +
                 (end.tv_nsec - start.tv_nsec) / 1000000.0;
printf("Execution time: %.2f ms\n", elapsed);
```

---

## 12. Common Issues

### 12.1 Compilation Error

**Problem**: Header file not found

```bash
fatal error: nuva_hal.h: No such file or directory
```

**Solution**: Check environment variables

```bash
export C_INCLUDE_PATH=$NUVA_HAL_INCLUDE:$C_INCLUDE_PATH
export CPLUS_INCLUDE_PATH=$NUVA_HAL_INCLUDE:$CPLUS_INCLUDE_PATH
```

### 12.2 Link Error

**Problem**: Undefined reference

```bash
undefined reference to `nuva_cpu_get_info'
```

**Solution**: Add library link

```cmake
target_link_libraries(your_target NuvaHAL::C)
```

### 12.3 Runtime Error

**Problem**: API returns `NUVA_ERROR_NOT_SUPPORTED`

**Solution**: Check hardware support

```c
uint32_t count;
result = nuva_npu_get_device_count(&count);
if (result != NUVA_OK || count == 0) {
    printf("NPU not available\n");
}
```

---

## 13. API Quick Reference

### 13.1 Error Codes

| Error Code | Value | Description |
|------------|-------|-------------|
| `NUVA_OK` | 0 | Success |
| `NUVA_ERROR_INVALID_PARAM` | -1 | Invalid parameter |
| `NUVA_ERROR_NOT_FOUND` | -2 | Not found |
| `NUVA_ERROR_OUT_OF_MEMORY` | -3 | Out of memory |
| `NUVA_ERROR_NOT_SUPPORTED` | -4 | Not supported |
| `NUVA_ERROR_HARDWARE` | -5 | Hardware error |
| `NUVA_ERROR_TIMEOUT` | -6 | Timeout |
| `NUVA_ERROR_BUSY` | -7 | Busy |

### 13.2 Version Information

```c
uint32_t version = nuva_hal_get_version();
// version = (major << 16) | (minor << 8) | patch

const char* version_str = nuva_hal_get_version_string();
// "1.0.0"
```

---

## 14. Appendix

### 14.1 Complete Examples

See the `examples/` directory:

- `cpu_info.c`: CPU info query
- `gpu_compute.cpp`: GPU compute example
- `npu_inference.cpp`: NPU inference example
- `quantum_crypto.c`: Quantum cryptography example

### 14.2 API Header Files

- `nuva_hal.h`: C API header
- `nuva_hal.hpp`: C++ API header

### 14.3 Related Documentation

- [HAL API Reference](../api/API_REFERENCE.md)
- [Layer Architecture Rules](../architecture/LAYER_RULES.md)
- [Documentation Standard](../standards/DOCUMENTATION_STANDARD.md)

---

**Document Version**: 1.2.0  
**Last Updated**: 2026-05-30  
**Maintainer**: Nuva OS Team
