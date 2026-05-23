# POSIX — POSIX Compatibility Layer

## Overview

The POSIX compatibility layer provides POSIX-standard compliant interfaces, enabling Nuva OS to run applications that conform to the POSIX standard. It resides at the same level as syslib (L2) and depends on kernel (L1) for underlying implementations.

## Submodules

| Submodule | Description |
|-----------|-------------|
| unistd.rs | POSIX process and file operations (read, write, close, fork, exec, pipe, etc.) |
| fcntl.rs | File control (open, creat, fcntl, dup, etc.) |
| signal.rs | Signal handling (sigaction, kill, raise, sigprocmask, etc.) |
| errno.rs | Error code definitions (EPERM, ENOENT, ESRCH, etc.) |

## Dependencies

- **Lower dependencies**: kernel (L1)
- **Peer collaboration**: syslib (L2)
- **Depended by**: services (L3)

## Build Configuration

The POSIX layer is compiled together with the kernel and requires no additional feature flags.

## Public Interface

Provides standard POSIX system call interfaces with error codes following POSIX semantics. All function signatures are compatible with the POSIX standard.
