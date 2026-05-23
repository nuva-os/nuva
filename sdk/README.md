# SDK — Software Development Kit

## Overview

The SDK module provides the Nuva OS software development toolkit, including the command-line interface, build system, debugger, package manager, and profiler.

## Submodules

| Submodule | Description |
|-----------|-------------|
| cli/ | Command-line interface: argument parsing, commands (build/clean/debug/doc/fmt/init/lint/new/pkg/profile/run/test) |
| build/ | Build system: cache, configuration, cross-compilation, executor, scheduler, targets |
| debug/ | Debugger: breakpoints, execution control, memory inspection, stack traces, variable viewing, DAP protocol, ptrace backend, target process management |
| package/ | Package management: cache, dependency resolution, lock file, metadata, HTTP registry, resolver, validator |
| profiler/ | Profiler: CPU sampling, memory analysis, flame graph, sampler, I/O analysis, lock contention analysis |

## Dependencies

- **Peer collaboration**: tools (toolchain)
- **Lower dependency**: syslib (L2)

## Build Configuration

The SDK is compiled under the host toolchain (host target) and does not run on bare-metal targets.

## Public Interface

- `nuva build` — Build project
- `nuva run` — Run project
- `nuva test` — Run tests
- `nuva debug` — Start debugger
- `nuva pkg` — Package management
- `nuva profile` — Performance profiling

## Build Configuration File

- `sdk/build-config.toml` — Multi-architecture build target configuration
