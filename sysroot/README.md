# Sysroot — System Root Directory

## Overview

The Sysroot module provides system root directory support for Nuva OS, containing C language header file definitions for use by C/C++ FFI and user-space programs.

## Submodules

| Submodule | Description |
|-----------|-------------|
| include/nuva/types.h | Nuva type definition header file |
| include/posix/posix.h | POSIX compatibility header file |

## Dependencies

- **Lower dependencies**: None (header-only, no runtime dependencies)
- **Depended by**: hal (L0), kernel (L1)

## Build Configuration

Header files in sysroot are integrated through the CMake build system:

- `CMakeLists.txt` includes the `sysroot/` directory
- Cross-compilation toolchain is configured via `toolchains/arm64-kirin.cmake`

## Public Interface

- `nuva/types.h` — Nuva system type definitions (`nuva_cpu_info_t`, etc.)
- `posix/posix.h` — POSIX standard type and constant definitions
