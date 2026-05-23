# Tools — 工具链集合

## 概述

Tools 模块包含 Nuva OS 开发所需的工具链，包括编译器、链接器、语言服务器（LSP）和依赖分析器。

## 子模块

| 子模块 | 说明 |
|--------|------|
| compiler/ | 编译器工具：词法分析、语法分析、语义分析、增量编译、并行编译、优化器、诊断 |
| linker/ | 链接器工具：ELF、目标文件、链接脚本、符号表、重定位 |
| lsp/ | 语言服务器协议实现：补全、诊断、悬停、导航、重构、语义分析 |
| toolchain/ | 工具链管理 |
| dep_analyzer/ | 依赖分析器：层级合规检查（workspace 成员） |

## 依赖关系

- **下层依赖**：无（主机工具链，独立于 Nuva OS 层）
- **上层被依赖**：sdk（SDK CLI 调用工具链作为构建/调试/性能分析后端）
- **独立运行**：dep_analyzer 可在 build.rs 中独立调用

## 构建配置

- `dep_analyzer` 是 Cargo workspace 成员，使用 `walkdir` crate
- 其他工具在本机工具链下编译

## 公开接口

- `dep_analyzer` — 在 release 构建时通过 `build.rs` 自动运行，强制执行分层架构边界约束
  - L0 (HAL): `allowed_deps = []`
  - L1 (Kernel): `allowed_deps = ["hal"]`
  - L2 (Lib): `allowed_deps = ["kernel", "hal"]`
