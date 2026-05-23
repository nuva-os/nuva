# HAL — 硬件抽象层 (L0)

## 概述

HAL（Hardware Abstraction Layer）是 Nuva OS 的最底层（Layer 0），提供统一的硬件访问接口。HAL 不依赖任何其他层，所有硬件相关操作均通过 HAL 抽象后供上层使用。

## 子模块

| 子模块 | 说明 |
|--------|------|
| cpu/ | CPU 抽象：DVFS、麒麟 SoC（PSCI SMC CPU_ON/CPU_OFF）、龙芯 SoC、热管理 |
| gpu/ | GPU 抽象：Maleoon GPU、命令队列 |
| npu/ | NPU 抽象：Da Vinci NPU HAL 桥接、ONNX 运行时、AI 调度器、推理预测器、设备管理 |
| quantum/ | 量子密码：PQC（Kyber KEM、Dilithium 签名）、QRNG 量子随机数、QKD 量子密钥分发 |
| power/ | 电源管理：PMIC、挂起/恢复、跨架构 C-state（MWAIT/WFI/idle） |
| ffi/ | 外部函数接口：C API（nuva_hal.h）、C++ API（nuva_hal.hpp）、ABI 稳定性 |
| input.rs | 输入设备 HAL |
| platform.rs | 平台检测和识别（架构、SoC、形态因子、BootInfoType） |
| dt.rs | 设备树解析器（ARM64 FDT/DTB） |
| acpi.rs | ACPI 表解析器（x86_64）、AcpiPowerDriver（Fadt、enter_sleep_state、S3/S5） |
| arm64/ | ARM64 架构特定 HAL 实现（FDT 引导、异常向量表） |
| x64/ | x86_64 架构特定 HAL（LAPIC/I/O APIC、GDT、IDT、CPU、MMU、定时器、电源、页表） |
| loongarch64/ | LoongArch64 架构特定 HAL（UEFI 引导、3级页表、Pte 结构体、LSX SIMD、LASX、LBT、LVZ） |
| snapdragon/ | 高通骁龙平台 HAL |

## 依赖关系

- **下层依赖**：无（最底层）
- **上层被依赖**：kernel (L1)、syslib (L2)

## 构建配置

| Feature | 条件 | 说明 |
|---------|------|------|
| `arm64` | arch = arm64 | 启用 ARM64 HAL |
| `x64` | arch = x86_64 | 启用 x86_64 HAL |
| `loongarch64` | arch = loongarch64 | 启用 LoongArch64 HAL |
| `snapdragon8gen4` | arm64 | 启用骁龙 8 Gen 4 特定实现 |
| `kirin9020` | arm64, kirin | 启用麒麟 9020 特定实现 |

## 公开接口

HAL 通过 trait 定义统一硬件接口：`CpuHal`、`GpuHal`、`NpuHal`、`PowerHal` 等。各架构通过条件编译提供具体实现。
