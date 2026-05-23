# 虚拟化子系统

## 概述

`kernel/virt/` 模块提供内核级虚拟化支持，覆盖所有支持的架构：VMX（x86-64 Intel VT-x / AMD-V）、VHE（ARM64 虚拟化主机扩展）和 LVZ（龙架构虚拟化扩展，包含 LBT 二进制翻译）。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口 |
| `vmx.rs` | VMX 虚拟化支持（x86-64 VMX、ARM64 VHE、LoongArch64 LVZ） |

## 架构支持

| 架构 | 虚拟化技术 |
|------|-----------|
| x86-64 | VMX（Intel VT-x / AMD-V） |
| ARM64 | VHE（虚拟化主机扩展） |
| LoongArch64 | LVZ（龙架构虚拟化扩展）+ LBT（二进制翻译） |

## 初始化顺序

虚拟化在阶段 8（平台与诊断）中初始化，在 APIC 操作配置完成后进行：

1. APIC 操作（`apic_ops::init_apic_ops`）— 阶段 8 前置条件
2. VMX/VHE/LVZ 初始化（`vmx::init_vmx`）— 阶段 8

## 依赖关系

- **内部依赖**：`kernel/core`（CPU）、`kernel/irq_mgmt`（APIC ops）
- **上层被依赖**：虚拟机监视器、Hypervisor 服务（L3）

## 公开接口

- `vmx` 模块：统一虚拟化 API，抽象了 x86-64 VMX、ARM64 VHE 和 LoongArch64 LVZ
  - `init_vmx()`：初始化当前架构的硬件虚拟化扩展
  - VM 进入/退出控制、EPT/NPT 页表管理、VMCS/VMCB 状态配置

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `vmx`

---

*最后更新：2026-05-20 | Nuva OS v1.0.0*
