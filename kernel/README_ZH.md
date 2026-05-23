# Kernel — 内核层 (L1)

## 概述

Kernel 层（Layer 1）是 Nuva OS 的核心，采用微内核架构设计。仅依赖 HAL (L0) 层，提供调度、内存管理、IPC、驱动框架、安全等核心子系统。

## 子模块

| 子模块 | 说明 |
|--------|------|
| arch/ | 架构相关代码（arm64, x64, loongarch64）：启动、上下文切换、MMU、中断、异常向量、链接脚本 |
| platform | 平台检测与引导信息（BootInfoType、PlatformInfo） |
| mm/ | 内存管理：Buddy 分配器、SLAB、页缓存、VMA、mmap/munmap/mprotect/msync、COW、NUMA、大页、OOM killer |
| mempool | 内存池管理 |
| cache | 内核缓存系统 |
| block | 块设备子系统 |
| sched/ | 调度器：CFS、EAS（能耗感知调度）、RT 实时调度、AI 调度器集成、红黑树、调度域、负载均衡 |
| process | 进程管理：fork、execve、wait4、信号处理、完整生命周期管理 |
| workqueue | 工作队列 |
| sync/ | 同步原语：自旋锁、互斥锁、原子操作 |
| irq | IRQ 中断请求管理 |
| interrupt/ | 中断管理：通用中断处理、GIC |
| trap | 陷阱/异常处理 |
| ipc/ | 进程间通信：NuvaIPC（快速路径、零拷贝）、共享内存、L4 IPC、量子安全 IPC |
| net/ | 网络协议栈：TCP/UDP/ICMP/IPv6/ARP/以太网、路由、防火墙、Socket、Netlink、NFS/SMB 网络客户端 |
| tcpip | TCP/IP 协议栈初始化 |
| socket | Socket API |
| fs/ | 内核文件系统：VFS（lookup/read/write/create/unlink/mkdir/rmdir）、缓冲区、目录缓存、页缓存、io_uring |
| journal | 日志系统 |
| driver/ | 驱动框架：设备模型、总线、IRQ（自动检测）、DMA、GPIO、I2C、SPI、时钟、调节器 |
| device_model | 设备模型 |
| driver_plugin | 驱动插件系统 |
| plugin/ | 插件系统：ELF 加载器完整实现、管理器、注册表、沙箱、SHA-256 校验、核心插件 |
| module | 内核模块加载器 |
| feature_plugin | 特性插件系统 |
| elf | ELF 解析器 |
| security/ | 安全子系统：LSM、ASLR、沙箱、栈金丝雀 |
| defense | 防御系统 |
| scanner | 病毒扫描器 |
| quantum/ | 量子计算支持：量子管理器、量子调度器 |
| debug/ | 内核调试：printk 宏（pr_err!/pr_info!/pr_warn! 等） |
| kdebug | 内核调试器 |
| log | 内核日志系统 |
| perf/ | 性能监控：事件计数、性能监视器 |
| perf_tune | 性能调优 |
| syscall/ | 系统调用接口 |
| timer/ | 定时器子系统 |
| time | 时间子系统 |
| cpu | CPU 管理 |
| hotplug | 热插拔 |
| power | 电源管理（ACPI） |
| pm | 电源管理器 |
| config | 内核配置 |
| cmdline | 内核命令行 |
| random | 随机数生成 |
| resource | 资源管理器 |
| signal | 信号处理 |
| stats | 统计信息 |
| notifier | 通知链 |
| apic_ops | APIC 操作 |
| vmx | 虚拟化支持 |
| posix | 内核 POSIX 兼容 |

## 依赖关系

- **下层依赖**：hal (L0)
- **上层被依赖**：syslib (L2)、services (L3)

## 构建配置

内核通过条件编译支持多架构：

- `--features arm64`：ARM64 架构（含麒麟/骁龙 SoC 支持）
- `--features x64`：x86_64 架构
- `--features loongarch64`：LoongArch64 架构
- `--features smp`：对称多处理器支持
- `--features debug`：调试模式

## 公开接口

内核通过系统调用（syscall）和内核 API 向上层暴露接口。主要入口点为 `kernel_main(boot_info: *const u8) -> !` 函数，接收引导信息并通过 `detect_platform_info()` 检测平台信息。
