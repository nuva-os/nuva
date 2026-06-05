# 变更日志

本文件记录 Nuva OS 的所有重要更改。

格式基于 [Keep a Changelog](https://keepachangelog.com/)，
版本号遵循 [Semantic Versioning](https://semver.org/)。

## [Unreleased]

### 新增

- NvScheduler AI智能调度器：NPU推理驱动、四级调度类别(AI_REALTIME/AI_NORMAL/AI_BATCH/AI_IDLE)、三级降级(AI→声明式→CFS+RT)、12维特征向量
- NvBalancer异构硬件均衡器：设备拓扑管理、负载采集、均衡优化、迁移执行、震荡检测(32项环形缓冲区)、热插拔支持
- NvPowerMgr AI驱动功耗优化：功耗预算管理(5%超限容许)、DVFS控制器(安全切换序列)、设备功耗控制(关键设备保护)、温度监控(85°C主动降功耗)、绿色计算指标(PUE/碳排放/效率)、AI功耗优化器
- 三方协同机制：NvScheduler↔NvBalancer↔NvPowerMgr运行时不变量验证
- 调度-功耗协同：调度决策评估功耗影响
- 调度-均衡协同：NvScheduler驱动NvBalancer负载均衡
- 均衡-功耗协同：均衡决策考虑设备功耗效率
- 功耗-调度协同：NvPowerMgr不休眠有活跃高优先级任务的设备
- 声明式策略引擎增强：新增ai_confidence_threshold/inference_budget_us/power_aware_enabled/balancer_driven字段
- RISC-V 64 (RV64G) 架构支持（SBI 启动、页表、PLIC、trap 处理、定时器）
- `riscv64` 和 `qemu_virt` Feature Flags
- `skip_dep_check` Feature Flags
- HAL RISC-V 64 平台模块（CPU、MMU、中断控制器）
- Kernel RISC-V 64 架构模块（boot/SBI、trap、MMU、PLIC、定时器、上下文）
- RISC-V Sv39 三级页表遍历(map/unmap/translate/protect)，含页表分配/释放和超级页支持
- Maleoon GPU中断处理(fence/GART故障/挂起/命令完成)和VRAM分配器(最佳适配+合并)
- Da Vinci NPU中断处理(推理完成/错误/模型加载/挂起)和可回收模型内存管理器(最佳适配+按模型释放+合并)
- DVFS硬件接口(dvfs_set_frequency/dvfs_set_voltage)寄存器级实现
- 热管理：85°C被动降频节流、105°C紧急关机

### 变更

- 更新项目结构，包含 RISC-V 64 架构目录
- 更新支持平台，包含 RISC-V 64 (QEMU virt)
- 更新构建系统，添加 RISC-V 64 构建和运行目标
- MALEOON_GPU_OPS桥接到实际MaleoonGpuHal方法（原为占位桩）
- PMIC_POWER_OPS桥接到实际PmicDriver方法（原为占位桩）
- CpuFreqInfo::set_freq()调用DVFS硬件(dvfs_set_frequency)
- PowerManager::power_off()/reboot()遍历电源域并调用平台操作
- PowerManager::register_default_domains()正确注册3个默认域

### 修复

- 修复层级依赖分析器在 FFI 边界模块的误报问题
- 修复 POSIX 兼容性 feature gate 在各构建目标间的一致性问题

## [1.0.0] - 2026-05-27

### 新增

- 完整多架构支持（ARM64、x86-64、LoongArch64、RISC-V 64）
- 抗量子安全（CRYSTALS-Kyber、CRYSTALS-Dilithium、SHA-256 FIPS 180-4）
- AI 原生设计（达芬奇 NPU HAL、AI 驱动调度器 EAS）
- 零拷贝 IPC（NuvaIPC 小消息延迟 <100ns）
- 插件架构（ELF 动态加载器、沙箱隔离、热插拔）
- 完整 SDK（调试器 DAP、性能分析器 /proc、包管理器 HTTP、CLI、构建系统）
- LoongArch64 页表、中断、SIMD 支持
- Nuva 编程语言（.nv）声明式范式

## [0.1.0] - 2026-01-01

### 新增

- 初始项目脚手架，基于 `#![no_std]` Rust 裸机内核
- ARM64 (AArch64) 架构支持（启动、GIC、MMU、异常向量）
- x86-64 架构支持（启动、GDT、IDT、APIC、异常处理）
- 基础内存管理（Buddy 分配器、SLAB 分配器、页表）
- 基础进程管理（进程创建、O(1) 调度器、上下文切换）
- 基础系统调用接口与处理框架
- 基础 VFS 层及文件操作（open/close/read/write）
- HAL 层，包含 CPU、GPU 及中断抽象
- 基于 Cargo 和 Makefile 的构建系统

## 版本说明

### 版本号格式

- **主版本号**: 不兼容的 API 更改
- **次版本号**: 向后兼容的功能新增
- **修订号**: 向后兼容的问题修复

### 更改类型

- **新增**: 新增功能
- **变更**: 现有功能的更改
- **弃用**: 即将移除的功能
- **移除**: 已移除的功能
- **修复**: Bug 修复
- **安全**: 安全相关修复
