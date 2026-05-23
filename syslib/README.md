# Syslib — System Library Layer (L2)

## Overview

The Syslib layer (Layer 2) provides a collection of system libraries for applications and services. It can depend on Kernel API and HAL traits, offering high-level abstractions and functional libraries to upper layers.

## Submodules

| Submodule | Description |
|-----------|-------------|
| core/ | Core library: allocator, synchronization primitives (lock-free data structures) |
| brain/ | Nuva Brain AI engine: learning, prediction, NPU scheduling, inference engine, operators (conv/activation/pooling) |
| ai/ | AI library: model manager, optimizer, scheduler |
| lang/ | NuvaLang compiler and runtime: lexer, parser, semantic analysis, type inference, codegen, optimizer, standard library, binary formats (Native/NEX), GC, VM |
| ml/ | Machine learning library: tensors, models, inference engine |
| net/ | Network library: HTTP, WebSocket, JSON, TCP/UDP protocol stack, ARP, Ethernet |
| data/ | Data structure library: database, key-value store |
| gfx/ | Graphics library: FPS monitoring |
| ui/ | UI library: layout, views, windows |
| std/ | Standard library: collections, basic types, IO |
| runtime/ | Runtime library: Arc, metadata, protocols |
| dispatch/ | Concurrency framework (GCD-style): thread pool, semaphore, queue, group |
| posix/ | POSIX compatibility layer: system call wrappers |

## Dependencies

- **Lower dependencies**: hal (L0), kernel (L1)
- **Depended by**: services (L3), application (L4)

## Build Configuration

Syslib libraries are compiled together with the kernel and require no additional feature flags.

## Public Interface

Each submodule exposes its public API through `pub mod`, including:
- `brain` — AI inference and training interface
- `ml` — Tensor operations and model management
- `net` — Network protocols and HTTP client
- `dispatch` — Asynchronous task dispatch
