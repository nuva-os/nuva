# C/C++ 驱动开发指南

**版本**: 1.2.0  
**日期**: 2026-05-15  
**状态**: 正式发布

---

## 一、概述

本指南为 Nuva OS 提供完整的 C/C++ 驱动开发流程，包括：

- HAL C API 使用方法
- C++ RAII 包装
- 驱动框架（设备模型/总线/IRQ/DMA/GPIO/I2C/SPI）
- 设备树匹配与解析
- 电源管理集成
- DMA-BUF 共享缓冲区框架
- 驱动生命周期管理
- 最佳实践和示例

### 1.1 目标读者

- 驱动开发工程师
- 系统集成工程师
- 硬件厂商开发团队

### 1.2 前置知识

- C/C++ 编程基础
- 硬件抽象层概念
- 操作系统驱动模型
- 设备树（Device Tree）基础知识

---

## 二、驱动框架概述

Nuva OS 驱动框架位于 `kernel/driver/`，采用分层设计：

```
kernel/driver/
├── framework/          # 驱动核心框架
│   ├── display.rs      #   显示框架
│   └── input.rs        #   输入框架
├── class/              # 设备类
│   ├── audio.rs        #   音频设备
│   ├── backlight.rs    #   背光设备
│   ├── bluetooth.rs    #   蓝牙设备
│   ├── camera.rs       #   摄像头
│   ├── display.rs      #   显示设备
│   ├── input.rs        #   输入设备
│   ├── led.rs          #   LED
│   ├── nfc.rs          #   NFC
│   ├── power.rs        #   电源设备
│   ├── sensor.rs       #   传感器
│   ├── storage.rs      #   存储设备
│   ├── touch.rs        #   触摸屏
│   ├── usb.rs          #   USB
│   ├── vibrator.rs     #   振动器
│   ├── wifi.rs         #   WiFi
│   └── ...
├── impl/               # 驱动实现
│   └── irqchip/        #   中断控制器实现
├── device.rs           # Device trait
├── dma.rs              # DMA 通道抽象
├── dmabuf.rs           # DMA-BUF 共享缓冲区
├── gpio.rs             # GPIO 子系统
├── i2c.rs              # I2C 总线
├── spi.rs              # SPI 总线
├── irq.rs              # 中断管理
├── pm.rs               # 驱动电源管理
├── dt.rs               # 设备树匹配
├── block.rs            # 块设备
├── char.rs             # 字符设备
├── clk.rs              # 时钟框架
├── freq.rs             # 频率调节
├── opp.rs              # 操作性能点
├── phy.rs              # PHY 抽象
├── pinctrl.rs          # 引脚控制
├── pwm.rs              # PWM
├── regulator.rs        # 电压调节器
├── reset.rs            # 复位控制
├── rtc.rs              # 实时时钟
├── thermal.rs          # 热管理
├── watchdog.rs         # 看门狗
├── event.rs            # 事件通知
├── mfd.rs              # 多功能设备
├── icc.rs              # 跨核心互联
└── adapter.rs          # 总线适配器
```

### 2.1 设备模型

所有设备必须实现 `Device` trait（`device.rs`）：

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

### 2.2 总线类型

Nuva OS 支持以下总线类型：

| 总线 | 文件 | 说明 |
|------|------|------|
| I2C | `i2c.rs` | 两线串行总线，适用于低速外设 |
| SPI | `spi.rs` | 串行外设接口，适用于中速外设 |
| IRQ | `irq.rs` | 中断请求总线 |
| Platform | 隐含 | 平台设备（设备树匹配） |

### 2.3 IRQ 管理

中断管理通过 `irq.rs` 和 `impl/irqchip/` 实现：

- IRQ 分配与释放
- 中断处理程序注册
- 中断亲和性设置（多核）
- 中断优先级管理
- GIC（通用中断控制器）驱动实现

### 2.4 DMA 框架

DMA 子系统包含两个核心组件：

**DMA 通道** (`dma.rs`)：
- DMA 通道申请与释放
- 单次/循环传输配置
- 传输完成回调

**DMA-BUF 共享缓冲区** (`dmabuf.rs`)：
- 零拷贝缓冲区共享机制
- 跨设备缓冲区传递
- 内存映射与同步
- fence 同步原语

### 2.5 GPIO 子系统

`gpio.rs` 提供通用 GPIO 控制：

- GPIO 申请与释放
- 输入/输出方向设置
- 电平读写
- 中断触发模式配置（边沿/电平）
- GPIO 控制器驱动注册

### 2.6 I2C 总线驱动

`i2c.rs` 提供 I2C 总线抽象：

- I2C 适配器注册
- I2C 消息传输（`i2c_transfer`）
- SMBus 协议支持
- I2C 设备探测与注册
- 设备树 I2C 节点匹配

### 2.7 SPI 总线驱动

`spi.rs` 提供 SPI 总线抽象：

- SPI 主控制器注册
- SPI 消息传输（`spi_transfer`）
- SPI 模式配置（CPOL/CPHA）
- 片选管理
- SPI 设备注册

---

## 三、设备树匹配

### 3.1 设备树解析

Nuva OS 通过 `kernel/driver/dt.rs` 和 `hal/dt.rs` 提供设备树支持：

- ARM64：通过 `hal/dt.rs` 解析 FDT（扁平设备树）
- x86_64：通过 `hal/acpi.rs` 解析 ACPI 表
- LoongArch64：通过设备树支持

### 3.2 驱动设备树匹配

驱动通过 compatible 字符串与设备树节点匹配：

```c
// 驱动声明 compatible 字符串
static const char* compatible_strings[] = {
    "vendor,device-name",
    NULL
};

// 驱动信息结构
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

### 3.3 设备树节点示例

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

## 四、电源管理集成

### 4.1 驱动电源管理

驱动电源管理通过 `kernel/driver/pm.rs` 集成：

- 运行时电源管理（runtime PM）
- 系统挂起/恢复回调
- 设备空闲状态管理
- 唤醒源配置

### 4.2 电源状态

```c
typedef enum {
    NUVA_POWER_ON = 0,       // 设备开启
    NUVA_POWER_SLEEP = 1,    // 设备休眠
    NUVA_POWER_SUSPEND = 2,  // 设备挂起
    NUVA_POWER_OFF = 3,      // 设备关闭
} nuva_power_state_t;
```

### 4.3 驱动 PM 回调

```c
typedef struct {
    nuva_result_t (*suspend)(nuva_handle_t device, nuva_power_state_t state);
    nuva_result_t (*resume)(nuva_handle_t device);
    nuva_result_t (*runtime_suspend)(nuva_handle_t device);
    nuva_result_t (*runtime_resume)(nuva_handle_t device);
    nuva_result_t (*idle)(nuva_handle_t device);
} nuva_driver_pm_ops_t;
```

### 4.4 HAL 电源管理

HAL 层电源管理（`hal/power/`）提供：

- PMIC 控制（`hal/power/pmic.rs`）
- 系统挂起/恢复流程（`hal/power/suspend.rs`）

---

## 五、DMA-BUF 框架

### 5.1 概述

DMA-BUF 框架（`kernel/driver/dmabuf.rs`）提供零拷贝缓冲区共享机制，允许不同设备共享同一块内存，避免数据拷贝。

### 5.2 核心 API

```c
// 导出缓冲区为 dma-buf
nuva_dmabuf_t dmabuf;
nuva_result_t result = nuva_dmabuf_export(device, buffer, size, &dmabuf);

// 获取 dma-buf fd（用于跨进程共享）
int fd = nuva_dmabuf_get_fd(dmabuf);

// 从 fd 获取 dma-buf
nuva_dmabuf_t imported;
result = nuva_dmabuf_get_from_fd(fd, &imported);

// 映射 dma-buf 用于设备访问
void* vaddr;
result = nuva_dmabuf_map(dmabuf, NUVA_DMA_BIDIRECTIONAL, &vaddr);

// 同步操作
nuva_dmabuf_sync(dmabuf, offset, length, NUVA_DMA_SYNC_FOR_DEVICE);

// 解映射
nuva_dmabuf_unmap(dmabuf, vaddr);

// 释放
nuva_dmabuf_put(dmabuf);
```

### 5.3 fence 同步

DMA-BUF 支持 fence 同步原语，确保缓冲区在设备间安全传递：

```c
// 创建 fence
nuva_fence_t fence;
nuva_fence_create(&fence);

// 等待 fence 完成
result = nuva_fence_wait(fence, timeout_ms);

// 在设备操作完成后 signal fence
nuva_fence_signal(fence);
```

---

## 六、快速开始

### 6.1 环境准备

#### 安装开发工具

```bash
# 安装 Nuva SDK
sudo apt install nuva-sdk

# 安装编译工具链
sudo apt install gcc g++ cmake

# 安装调试工具
sudo apt install gdb nuva-debug
```

#### 配置环境变量

```bash
export NUVA_SDK=/opt/nuva/sdk
export NUVA_HAL_INCLUDE=$NUVA_SDK/include
export NUVA_HAL_LIB=$NUVA_SDK/lib
```

### 6.2 第一个驱动

#### C 语言示例

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

#### C++ 语言示例

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

### 6.3 编译和运行

#### CMake 配置

```cmake
cmake_minimum_required(VERSION 3.10)
project(nuva_driver_example)

find_package(NuvaHAL REQUIRED)

add_executable(driver_c main.c)
target_link_libraries(driver_c NuvaHAL::C)

add_executable(driver_cpp main.cpp)
target_link_libraries(driver_cpp NuvaHAL::Cpp)
```

#### 编译命令

```bash
mkdir build && cd build
cmake ..
make
```

---

## 七、HAL API 详解

### 7.1 CPU HAL

#### 获取 CPU 信息

```c
nuva_cpu_info_t info;
nuva_result_t result = nuva_cpu_get_info(&info);
if (result == NUVA_OK) {
    // 使用 info
}
```

#### CPU 信息结构

```c
typedef struct {
    uint32_t core_count;        // CPU 核心数
    uint32_t frequency_mhz;     // CPU 频率 (MHz)
    uint32_t cache_line_size;   // 缓存行大小
    uint64_t total_memory;      // 总内存 (字节)
    char vendor[32];            // CPU 厂商
    char model[64];             // CPU 型号
} nuva_cpu_info_t;
```

#### 中断控制

```c
nuva_cpu_disable_irq();
// 临界区代码
nuva_cpu_enable_irq();
```

#### 内存屏障

```c
nuva_cpu_memory_barrier();   // 完整内存屏障
nuva_cpu_read_barrier();     // 读屏障
nuva_cpu_write_barrier();    // 写屏障
```

### 7.2 GPU HAL

#### 初始化 GPU

```c
nuva_result_t result = nuva_gpu_init();
if (result != NUVA_OK) {
    // 处理错误
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

#### 创建 GPU 缓冲区

```c
nuva_gpu_buffer_t buffer;
result = nuva_gpu_create_buffer(device, size, &buffer);
if (result == NUVA_OK) {
    // 使用 buffer
    nuva_gpu_destroy_buffer(buffer);
}
```

### 7.3 NPU HAL

#### 初始化 NPU

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

#### 加载和执行模型

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

### 7.4 DaVinci NPU HAL 桥接

DaVinci NPU（昇腾架构）通过 `davinci_npu_ops()` 获取 HAL 操作集，驱动开发者需实现以下桥接步骤：

#### 7.4.1 获取 DaVinci NPU 操作集

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

#### 7.4.2 AiCore 任务执行

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

#### 7.4.3 Tiling 数据更新

```c
result = ops->tiling_update(model, tiling_data, tiling_size);
```

> **注意**：DaVinci NPU 驱动实现位于 `hal/npu/davinci/`，需通过 `davinci_npu_ops()` 桥接到通用 NPU HAL trait。设备树 compatible 字符串为 `"hisilicon,davinci-npu"`。

### 7.5 AI 调度器集成

AI 调度器允许驱动开发者将 NPU/GPU 工作负载与内核调度器协作，实现最优 CPU 亲和性选择。

#### 7.5.1 通知内核调度器

当驱动检测到 AI 工作负载变化时，应通知内核调度器：

```c
nuva_ai_scheduler_event_t event = NUVA_AI_SCHED_NEW_INFERENCE_REQUEST;
result = nuva_ai_notify_scheduler(event);
```

事件类型：
- `NUVA_AI_SCHED_NEW_INFERENCE_REQUEST`：新推理请求到达
- `NUVA_AI_SCHED_MODEL_LOADED`：模型加载完成
- `NUVA_AI_SCHED_WORKLOAD_COMPLETE`：工作负载完成
- `NUVA_AI_SCHED_NPU_BUSY`：NPU 资源繁忙
- `NUVA_AI_SCHED_NPU_IDLE`：NPU 资源空闲

#### 7.5.2 选择最优 CPU 核心

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

> **注意**：AI 调度器 API 位于 L1 内核层，驱动开发者通过系统调用接口访问。在 EAS（Energy-Aware Scheduling）调度器中，AI 调度器集成提供能耗感知的 NPU 任务调度。

### 7.6 Quantum HAL

#### QRNG 使用

```c
nuva_qrng_t qrng;
nuva_result_t result = nuva_qrng_init(&qrng);

uint8_t random_bytes[32];
result = nuva_qrng_generate(qrng, random_bytes, 32);
```

#### PQC 密钥生成

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

## 八、C++ 高级特性

### 8.1 RAII 资源管理

```cpp
{
    nuva::Gpu::Buffer buffer(device, 1024);
    // 使用 buffer，出作用域自动销毁
}

{
    nuva::Npu::Model model(device, model_data, model_size);
    // 使用 model，出作用域自动卸载
}
```

### 8.2 异常处理

```cpp
try {
    auto cpu_info = nuva::Cpu::get_info();
    // 使用 cpu_info
} catch (const nuva::Exception& e) {
    std::cerr << "Error: " << e.what() << std::endl;
    std::cerr << "Result code: " << e.result() << std::endl;
}
```

### 8.3 移动语义

```cpp
nuva::Npu::Buffer buffer1(device, 1024);
nuva::Npu::Buffer buffer2 = std::move(buffer1);
// buffer1 现在为空
// buffer2 拥有资源
```

---

## 九、驱动生命周期

### 9.1 驱动注册

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
    // 清理代码
}

static nuva_result_t my_driver_probe(nuva_handle_t device) {
    // 设备探测与初始化
    return NUVA_OK;
}

static void my_driver_remove(nuva_handle_t device) {
    // 设备移除
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

### 9.2 电源管理回调

```c
nuva_result_t result = nuva_power_set_state(device, NUVA_POWER_SUSPEND);

nuva_power_state_t state;
result = nuva_power_get_state(device, &state);
```

---

## 十、最佳实践

### 10.1 错误处理

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

### 10.2 资源管理

```c
nuva_npu_buffer_t buffer = NUVA_INVALID_HANDLE;
nuva_result_t result = nuva_npu_create_buffer(device, size, &buffer);
if (result != NUVA_OK) {
    return result;
}

// 使用 buffer

if (buffer != NUVA_INVALID_HANDLE) {
    nuva_npu_destroy_buffer(buffer);
}
```

### 10.3 线程安全

```c
nuva_cpu_write_barrier();
shared_data = value;
nuva_cpu_write_barrier();

// 另一个线程
nuva_cpu_read_barrier();
value = shared_data;
nuva_cpu_read_barrier();
```

### 10.4 性能优化

```c
for (int i = 0; i < count; i++) {
    nuva_npu_execute(model, &inputs[i], 1, &outputs[i], 1);
}
```

---

## 十一、调试技巧

### 11.1 日志输出

```c
#define DEBUG_LOG(fmt, ...) \
    printf("[DEBUG] " fmt "\n", ##__VA_ARGS__)

DEBUG_LOG("CPU cores: %u", info.core_count);
```

### 11.2 错误追踪

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

### 11.3 性能分析

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

## 十二、常见问题

### 12.1 编译错误

**问题**: 找不到头文件

```bash
fatal error: nuva_hal.h: No such file or directory
```

**解决**: 检查环境变量

```bash
export C_INCLUDE_PATH=$NUVA_HAL_INCLUDE:$C_INCLUDE_PATH
export CPLUS_INCLUDE_PATH=$NUVA_HAL_INCLUDE:$CPLUS_INCLUDE_PATH
```

### 12.2 链接错误

**问题**: 未定义的引用

```bash
undefined reference to `nuva_cpu_get_info'
```

**解决**: 添加库链接

```cmake
target_link_libraries(your_target NuvaHAL::C)
```

### 12.3 运行时错误

**问题**: API 返回 `NUVA_ERROR_NOT_SUPPORTED`

**解决**: 检查硬件支持

```c
uint32_t count;
result = nuva_npu_get_device_count(&count);
if (result != NUVA_OK || count == 0) {
    printf("NPU not available\n");
}
```

---

## 十三、API 参考

### 13.1 错误码

| 错误码 | 值 | 说明 |
|--------|-----|------|
| `NUVA_OK` | 0 | 成功 |
| `NUVA_ERROR_INVALID_PARAM` | -1 | 无效参数 |
| `NUVA_ERROR_NOT_FOUND` | -2 | 未找到 |
| `NUVA_ERROR_OUT_OF_MEMORY` | -3 | 内存不足 |
| `NUVA_ERROR_NOT_SUPPORTED` | -4 | 不支持 |
| `NUVA_ERROR_HARDWARE` | -5 | 硬件错误 |
| `NUVA_ERROR_TIMEOUT` | -6 | 超时 |
| `NUVA_ERROR_BUSY` | -7 | 忙 |

### 13.2 版本信息

```c
uint32_t version = nuva_hal_get_version();
// version = (major << 16) | (minor << 8) | patch

const char* version_str = nuva_hal_get_version_string();
// "1.0.0"
```

---

## 十四、附录

### 14.1 完整示例

参见 `examples/` 目录

- `cpu_info.c`: CPU 信息查询
- `gpu_compute.cpp`: GPU 计算示例
- `npu_inference.cpp`: NPU 推理示例
- `quantum_crypto.c`: 量子密码示例

### 14.2 API 头文件

- `nuva_hal.h`: C API 头文件
- `nuva_hal.hpp`: C++ API 头文件

### 14.3 相关文档

- [HAL API 参考](../api/API_REFERENCE_zh.md)
- [分层架构规则](../architecture/LAYER_RULES_zh.md)
- [文档编写标准](../standards/DOCUMENTATION_STANDARD_zh.md)

---

**文档版本**: 1.2.0  
**最后更新**: 2026-05-30  
**维护者**: Nuva OS Team
