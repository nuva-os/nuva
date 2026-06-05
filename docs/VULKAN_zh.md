# Nuva OS Vulkan 集成

## 概述

Nuva OS 将 Vulkan 作为原生 GPU/计算 API 集成，提供零拷贝 GPU 直通架构。此架构优于 Android 和苹果平台：

- **优于 Android**：Android 通过 Gralloc+HAL 中间层访问 Vulkan。Nuva OS 完全消除 HAL 层——内核直接暴露 Vulkan 能力的 GPU 设备。
- **优于苹果**：苹果使用私有 Metal API，锁定开发者生态。Nuva 使用开放 Vulkan 标准配合基于能力的 GPU 访问。

**设计哲学**：nuva is not unix, nuva is not linux。Vulkan 集成遵循 Nuva 原生设计：基于能力的安全模型、零拷贝内存共享、设备直通。

---

## 架构

### GPU 访问路径对比

| 方面 | Android | 苹果 (Metal) | Nuva OS (NvVulkan) |
|------|---------|-------------|---------------------|
| API | Vulkan via HAL .so | Metal（私有） | Vulkan（开放标准） |
| GPU 访问 | App→Gralloc→HAL→Driver→GPU | App→Metal→IOKit→GPU | App→NvVulkan syscall→GPU |
| 内存 | Gralloc 缓冲中介 | IOSurface 中介 | 零拷贝共享页面 |
| 安全 | UID/GID + SELinux | 权限声明 | NvGpuCapability 令牌 |
| 中间层数 | 3+ | 2 | **1（内核系统调用）** |

### 零拷贝 GPU 内存

Nuva OS 在内核级别建立共享 CPU-GPU 页表项：

```
CPU 虚拟地址 ──┐
               ├── 相同物理页面 ──→ GPU 硬件
GPU 虚拟地址 ──┘
```

- **HOST_VISIBLE + HOST_COHERENT**：CPU 和 GPU 映射相同物理页面
- **命令缓冲区**：CPU 写入命令，GPU 直接读取——无需拷贝
- **Doorbell 提交**：写入 GPU doorbell 寄存器触发执行

### GPU 能力安全模型

```rust
NvGpuPermission::GPU_COMPUTE   // 计算着色器访问
NvGpuPermission::GPU_RENDER    // 图形管线访问
NvGpuPermission::GPU_MEMORY    // GPU 内存分配
NvGpuPermission::GPU_PRESENT   // 显示呈现
NvGpuPermission::GPU_VIDEO     // 视频编解码加速
```

- 每个 Vulkan Instance 需要有效的 NvGpuCapability
- NvGpuCapability 强制执行 GPU 内存配额（max_memory_bytes）
- 能力撤销级联：绑定的 Vulkan Instance 被标记为无效

---

## 系统调用 (0x70-0x8F)

| 调用号 | 名称 | 描述 |
|-------|------|------|
| 0x70 | `NV_VULKAN_INSTANCE_CREATE` | 创建 Vulkan Instance（需 GPU_RENDER 能力） |
| 0x71 | `NV_VULKAN_INSTANCE_DESTROY` | 销毁 Vulkan Instance |
| 0x72 | `NV_VULKAN_DEVICE_ENUMERATE` | 枚举物理 GPU 设备 |
| 0x73 | `NV_VULKAN_DEVICE_CREATE` | 创建逻辑 Device |
| 0x74 | `NV_VULKAN_DEVICE_DESTROY` | 销毁逻辑 Device |
| 0x75 | `NV_VULKAN_MEMORY_ALLOCATE` | 分配 GPU 内存（零拷贝路径） |
| 0x76 | `NV_VULKAN_MEMORY_FREE` | 释放 GPU 内存 |
| 0x77 | `NV_VULKAN_QUEUE_SUBMIT` | 提交命令缓冲区（零拷贝） |
| 0x78 | `NV_VULKAN_QUEUE_WAIT` | 等待队列空闲 |
| 0x79 | `NV_VULKAN_FENCE_CREATE` | 创建栅栏 |
| 0x7A | `NV_VULKAN_FENCE_WAIT` | 等待栅栏 |
| 0x7B | `NV_VULKAN_SEMAPHORE_CREATE` | 创建信号量 |
| 0x7C | `NV_VULKAN_SEMAPHORE_WAIT` | 等待信号量 |
| 0x7D | `NV_VULKAN_SWAPCHAIN_CREATE` | 创建交换链 |
| 0x7E | `NV_VULKAN_SWAPCHAIN_PRESENT` | 呈现帧 |
| 0x7F | `NV_VULKAN_DESCRIPTOR_UPDATE` | 更新描述符集合 |
| 0x80 | `NV_VULKAN_PIPELINE_CREATE` | 创建管线 |
| 0x81 | `NV_VULKAN_PIPELINE_DESTROY` | 销毁管线 |
| 0x82 | `NV_VULKAN_SHADER_LOAD` | 加载着色器模块 |
| 0x83 | `NV_VULKAN_BATCH_SUBMIT` | 批量命令提交 |

所有 Vulkan 系统调用首参为 `capability_id`。

---

## 构建配置

Vulkan 为可选 feature，默认不启用：

```bash
# 默认构建（无 Vulkan）
cargo build

# 启用 Vulkan 支持
cargo build --features vulkan

# 同时启用 POSIX 和 Vulkan
cargo build --features "posix,vulkan"
```

---

## 文件结构

```
kernel/vulkan/           # 内核 Vulkan 子系统
├── mod.rs               # 模块入口
├── gpu_capability.rs    # NvGpuCapability 安全模型
├── gpu_memory.rs        # 零拷贝 GPU 内存管理
└── instance.rs          # Instance/Device 生命周期管理

kernel/syscall/
└── nv_vulkan_syscall.rs # Vulkan 系统调用分发 (0x70-0x8F)

syslib/nv_vulkan/        # 用户空间 Vulkan API 桥接
└── mod.rs               # Vulkan API → NvVulkan syscall 桥接

syslib/gfx/              # 图形框架
└── mod.rs               # GraphicsBackend (Vulkan/Software) 选择

hal/gpu/                 # GPU HAL（遗留兼容回退）
└── mod.rs               # 标记为遗留兼容层
```

---

## 性能目标

| 指标 | 目标 |
|------|------|
| Vulkan 命令提交 | ≤ 5 μs |
| 零拷贝 GPU 内存分配 | 无拷贝开销 |
| GPU 内存带宽（HOST_VISIBLE） | ≥ 设备本地带宽的 80% |
| 批量提交（N 个缓冲区） | 1 次系统调用提交 N 个缓冲区 |
| 内核大小增加（vulkan feature） | ≤ 5% |

---

**最后更新**：2026 年 5 月 30 日
