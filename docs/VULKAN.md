# Nuva OS Vulkan Integration

## Overview

Nuva OS integrates Vulkan as its native GPU/compute API, providing zero-copy direct passthrough to GPU hardware. This architecture is superior to both Android and Apple platforms:

- **Superior to Android**: Android accesses Vulkan through Gralloc+HAL intermediate layers. Nuva OS eliminates the HAL layer entirely — the kernel directly exposes Vulkan-capable GPU devices.
- **Superior to Apple**: Apple uses the proprietary Metal API, locking developers into their ecosystem. Nuva uses the open Vulkan standard with capability-based GPU access.

**Design Philosophy**: nuva is not unix, nuva is not linux. Vulkan integration follows Nuva's native design: capability-based security, zero-copy memory sharing, direct device passthrough.

---

## Architecture

### GPU Access Path Comparison

| Aspect | Android | Apple (Metal) | Nuva OS (NvVulkan) |
|--------|---------|---------------|---------------------|
| API | Vulkan via HAL .so | Metal (proprietary) | Vulkan (open standard) |
| GPU Access | App → Gralloc → HAL → Driver → GPU | App → Metal → IOKit → GPU | App → NvVulkan syscall → GPU |
| Memory | Gralloc buffer mediation | IOSurface mediation | Zero-copy shared pages |
| Security | UID/GID + SELinux | Entitlements | NvGpuCapability tokens |
| Layers | 3+ intermediate | 2 intermediate | **1 (kernel syscall)** |

### Zero-Copy GPU Memory

Nuva OS establishes shared CPU-GPU page table entries at the kernel level:

```
CPU Virtual Address ──┐
                      ├── Same Physical Page ──→ GPU Hardware
GPU Virtual Address ──┘
```

- **HOST_VISIBLE + HOST_COHERENT**: CPU and GPU map identical physical pages
- **Command Buffers**: CPU writes commands, GPU reads directly — no copy
- **Doorbell Submit**: Write to GPU doorbell register to trigger execution

### GPU Capability Security

```rust
NvGpuPermission::GPU_COMPUTE   // Compute shader access
NvGpuPermission::GPU_RENDER    // Graphics pipeline access
NvGpuPermission::GPU_MEMORY    // GPU memory allocation
NvGpuPermission::GPU_PRESENT   // Display presentation
NvGpuPermission::GPU_VIDEO     // Video decode/encode acceleration
```

- Each Vulkan Instance requires a valid NvGpuCapability
- NvGpuCapability enforces GPU memory quotas (max_memory_bytes)
- Capability revocation cascades: bound Vulkan Instances are invalidated

---

## System Calls (0x70-0x8F)

| Call Number | Name | Description |
|-------------|------|-------------|
| 0x70 | `NV_VULKAN_INSTANCE_CREATE` | Create Vulkan Instance (requires GPU_RENDER cap) |
| 0x71 | `NV_VULKAN_INSTANCE_DESTROY` | Destroy Vulkan Instance |
| 0x72 | `NV_VULKAN_DEVICE_ENUMERATE` | Enumerate physical GPU devices |
| 0x73 | `NV_VULKAN_DEVICE_CREATE` | Create logical Device |
| 0x74 | `NV_VULKAN_DEVICE_DESTROY` | Destroy logical Device |
| 0x75 | `NV_VULKAN_MEMORY_ALLOCATE` | Allocate GPU memory (zero-copy path) |
| 0x76 | `NV_VULKAN_MEMORY_FREE` | Free GPU memory |
| 0x77 | `NV_VULKAN_QUEUE_SUBMIT` | Submit command buffers (zero-copy) |
| 0x78 | `NV_VULKAN_QUEUE_WAIT` | Wait for queue idle |
| 0x79 | `NV_VULKAN_FENCE_CREATE` | Create fence |
| 0x7A | `NV_VULKAN_FENCE_WAIT` | Wait on fence |
| 0x7B | `NV_VULKAN_SEMAPHORE_CREATE` | Create semaphore |
| 0x7C | `NV_VULKAN_SEMAPHORE_WAIT` | Wait on semaphore |
| 0x7D | `NV_VULKAN_SWAPCHAIN_CREATE` | Create swapchain |
| 0x7E | `NV_VULKAN_SWAPCHAIN_PRESENT` | Present frame |
| 0x7F | `NV_VULKAN_DESCRIPTOR_UPDATE` | Update descriptor set |
| 0x80 | `NV_VULKAN_PIPELINE_CREATE` | Create pipeline |
| 0x81 | `NV_VULKAN_PIPELINE_DESTROY` | Destroy pipeline |
| 0x82 | `NV_VULKAN_SHADER_LOAD` | Load shader module |
| 0x83 | `NV_VULKAN_BATCH_SUBMIT` | Batch command submit |

All Vulkan syscalls require `capability_id` as the first argument.

---

## Build Configuration

Vulkan is an optional feature, disabled by default:

```bash
# Default build (no Vulkan)
cargo build

# Enable Vulkan support
cargo build --features vulkan

# Enable both POSIX and Vulkan
cargo build --features "posix,vulkan"
```

---

## File Structure

```
kernel/vulkan/           # Kernel Vulkan subsystem
├── mod.rs               # Module entry
├── gpu_capability.rs    # NvGpuCapability security model
├── gpu_memory.rs        # Zero-copy GPU memory management
└── instance.rs          # Instance/Device lifecycle management

kernel/syscall/
└── nv_vulkan_syscall.rs # Vulkan system call dispatch (0x70-0x8F)

syslib/nv_vulkan/        # User-space Vulkan API bridge
└── mod.rs               # Vulkan API → NvVulkan syscall bridge

syslib/gfx/              # Graphics framework
└── mod.rs               # GraphicsBackend (Vulkan/Software) selection

hal/gpu/                 # GPU HAL (legacy fallback)
└── mod.rs               # Marked as Legacy Compatibility Layer
```

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Vulkan command submit | ≤ 5 μs |
| Zero-copy GPU memory alloc | No copy overhead |
| GPU memory bandwidth (HOST_VISIBLE) | ≥ 80% of device-local |
| Batch submit (N buffers) | 1 syscall for N buffers |
| Kernel size increase (vulkan feature) | ≤ 5% |

---

**Last Updated**: May 30, 2026
