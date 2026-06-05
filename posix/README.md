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

The POSIX layer is compiled as an **optional feature gate**. Enable it with:

```toml
[features]
default = ["nuva_native"]
posix = []
bsd_compat = ["posix"]
```

When the `posix` feature is disabled:
- `posix/` directory is not compiled
- `kernel/process/fork.rs`, `signal.rs`, `execve.rs`, `wait4.rs` are not compiled
- `kernel/ipc/shm.rs`, `shm_ipc.rs` (System V compat) are not compiled
- Kernel core paths have zero POSIX dependency

When enabled, it provides POSIX-standard interfaces via adapter patterns that map to Nuva native kernel primitives.

## Public Interface

Provides standard POSIX system call interfaces with error codes following POSIX semantics. All function signatures are compatible with the POSIX standard.
