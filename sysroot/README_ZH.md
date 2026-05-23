# Sysroot — 系统根目录

## 概述

Sysroot 模块提供 Nuva OS 的系统根目录支持，包含 C 语言头文件定义，供 C/C++ FFI 和用户空间程序使用。

## 子模块

| 子模块 | 说明 |
|--------|------|
| include/nuva/types.h | Nuva 类型定义头文件 |
| include/posix/posix.h | POSIX 兼容头文件 |

## 依赖关系

- **下层依赖**：无（纯头文件，无运行时依赖）
- **上层被依赖**：hal (L0)、kernel (L1)

## 构建配置

Sysroot 中的头文件通过 CMake 构建系统集成：

- `CMakeLists.txt` 中包含 `sysroot/` 目录
- 交叉编译工具链通过 `toolchains/arm64-kirin.cmake` 配置

## 公开接口

- `nuva/types.h` — Nuva 系统类型定义（`nuva_cpu_info_t` 等）
- `posix/posix.h` — POSIX 标准类型和常量定义
