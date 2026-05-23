# Tools — Toolchain Collection

## Overview

The Tools module contains the toolchains required for Nuva OS development, including the compiler, linker, language server (LSP), and dependency analyzer.

## Submodules

| Submodule | Description |
|-----------|-------------|
| compiler/ | Compiler tools: lexer, parser, semantic analysis, incremental compilation, parallel compilation, optimizer, diagnostics |
| linker/ | Linker tools: ELF, object files, linker scripts, symbol table, relocation |
| lsp/ | Language Server Protocol implementation: completion, diagnostics, hover, navigation, refactor, semantic analysis |
| toolchain/ | Toolchain management |
| dep_analyzer/ | Dependency analyzer: layer compliance checking (workspace members) |

## Dependencies

- **Lower dependencies**: None (host toolchain, independent of Nuva OS layers)
- **Depended by**: sdk (SDK CLI invokes toolchain as build/debug/profile backend)
- **Standalone**: dep_analyzer can be invoked independently in build.rs

## Build Configuration

- `dep_analyzer` is a Cargo workspace member, using the `walkdir` crate
- Other tools are compiled under the host toolchain

## Public Interface

- `dep_analyzer` — Automatically runs via `build.rs` during release builds, enforcing layered architecture boundary constraints
  - L0 (HAL): `allowed_deps = []`
  - L1 (Kernel): `allowed_deps = ["hal"]`
  - L2 (Lib): `allowed_deps = ["kernel", "hal"]`
