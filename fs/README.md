# FS — File System Implementations

## Overview

The FS module contains concrete file system implementations, providing ext4, FAT32, and the custom NuvaFS file system under the kernel VFS framework.

## Submodules

| Submodule | Description |
|-----------|-------------|
| ext4/ | ext4 file system implementation: journal, inode, extents, directory index |
| fat32/ | FAT32 file system implementation: FAT table, directory entries, long file names |
| nuvafs/ | NuvaFS custom file system: directory, file, inode, journal, POSIX compatibility, snapshot, superblock |

## Dependencies

- **Lower dependencies**: hal (L0 — block device driver), kernel (L1 — VFS framework)
- **Depended by**: None (file systems are dynamically mounted through VFS registration)

## Build Configuration

The file system module is compiled together with the kernel. Each file system is dynamically mounted through the VFS registration mechanism.

## Public Interface

Each file system implements the `FileSystem` trait and is exposed uniformly through VFS:

```rust
pub trait FileSystem {
    fn mount(&mut self, device: &Device) -> Result<()>;
    fn unmount(&mut self) -> Result<()>;
    fn root(&self) -> &Inode;
}
```

### NuvaFS Features

- Journaling: ensures crash consistency
- Copy-on-write (COW): supports snapshots
- POSIX compatibility: supports standard file operations
- Data compression: reduces storage space
