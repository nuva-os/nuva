# FS — 文件系统实现

## 概述

FS 模块包含具体文件系统实现，在内核 VFS 框架之下提供 ext4、FAT32 和自研 NuvaFS 文件系统。

## 子模块

| 子模块 | 说明 |
|--------|------|
| ext4/ | ext4 文件系统实现：日志、inode、扩展区、目录索引 |
| fat32/ | FAT32 文件系统实现：FAT 表、目录项、长文件名 |
| nuvafs/ | NuvaFS 自研文件系统：目录、文件、inode、日志、POSIX 兼容、快照、超级块 |

## 依赖关系

- **下层依赖**：hal (L0 — 块设备驱动)、kernel (L1 — VFS 框架)
- **上层被依赖**：无（文件系统通过 VFS 注册机制动态挂载）

## 构建配置

文件系统模块随内核一起编译。各文件系统通过 VFS 注册机制动态挂载。

## 公开接口

各文件系统实现 `FileSystem` trait，通过 VFS 统一暴露：

```rust
pub trait FileSystem {
    fn mount(&mut self, device: &Device) -> Result<()>;
    fn unmount(&mut self) -> Result<()>;
    fn root(&self) -> &Inode;
}
```

### NuvaFS 特性

- 日志结构：保证崩溃一致性
- 写时复制（COW）：支持快照
- POSIX 兼容：支持标准文件操作
- 数据压缩：减少存储空间
