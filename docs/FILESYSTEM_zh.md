# Nuva OS 文件系统模块

## 概述

文件系统模块提供完整的文件管理功能，包括 VFS（虚拟文件系统）、页缓存、缓冲区缓存、目录缓存、NuvaFS 原生文件系统、ext4 和 FAT32 兼容文件系统、NFSv3 和 SMB2/3 网络文件系统客户端，以及 io_uring 异步 IO 机制。

---

## 目录

1. [页缓存](#1-页缓存)
2. [缓冲区缓存](#2-缓冲区缓存)
3. [基本文件系统](#3-基本文件系统)
4. [VFS（虚拟文件系统）](#4-vfs虚拟文件系统)
5. [NuvaFS](#5-nuvafs)
6. [ext4 文件系统](#6-ext4-文件系统)
7. [FAT32 文件系统](#7-fat32-文件系统)
8. [目录缓存](#8-目录缓存)
9. [NFSv3 客户端](#9-nfsv3-客户端)
10. [SMB2/3 客户端](#10-smb23-客户端)
11. [io_uring 异步 IO](#11-io_uring-异步-io)
12. [文件结构](#12-文件结构)

---

## 1. 页缓存

### 1.1 页结构

```rust
pub struct Page {
    pub index: u64,
    pub flags: AtomicU32,
    pub ref_count: AtomicU32,
    pub map_count: AtomicU32,
    pub data: [u8; PAGE_SIZE],
    pub next: *mut Page,
    pub prev: *mut Page,
}
```

### 1.2 页缓存

```rust
pub struct PageCache {
    pub page_count: AtomicU32,
    pub dirty_count: AtomicU32,
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub max_pages: u32,
}
```

### 1.3 地址空间

```rust
pub struct AddressSpace {
    pub ino: u64,
    pub dev: u64,
    pub page_count: AtomicU32,
    pub dirty_count: AtomicU32,
    pub writeback_index: AtomicU64,
}
```

### 1.4 页缓存特性

1. **页管理**：页分配和释放
2. **脏页管理**：脏页标记和回写
3. **地址空间**：文件映射
4. **统计**：命中率统计
5. **LRU 淘汰**：活跃/非活跃双链表

### 1.5 页缓存操作

```rust
pub struct PageCache {
    pub hash_table: [*mut PageCacheEntry; HASH_SIZE],
    pub active_list: LruList,
    pub inactive_list: LruList,
}

impl PageCache {
    pub fn lookup(&mut self, key: &PageCacheKey) -> *mut PageCacheEntry;
    pub fn add(&mut self, entry: *mut PageCacheEntry) -> bool;
    pub fn read_page(&mut self, key: &PageCacheKey) -> *mut PageCacheEntry;
}
```

---

## 2. 缓冲区缓存

### 2.1 缓冲区头

```rust
pub struct BufferHead {
    pub dev: u64,
    pub block: u64,
    pub size: u32,
    pub flags: AtomicU32,
    pub ref_count: AtomicU32,
    pub data: [u8; BUFFER_SIZE],
    pub next: *mut BufferHead,
    pub prev: *mut BufferHead,
    pub hash_next: *mut BufferHead,
    pub hash_prev: *mut BufferHead,
    pub lru_next: *mut BufferHead,
    pub lru_prev: *mut BufferHead,
}
```

### 2.2 缓冲区缓存

```rust
pub struct BufferCache {
    pub buffer_count: AtomicU32,
    pub dirty_count: AtomicU32,
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub max_buffers: u32,
}
```

### 2.3 缓冲区缓存特性

1. **缓冲区管理**：缓冲区分配和释放
2. **哈希查找**：快速查找（设备号+块号哈希）
3. **LRU 淘汰**：缓冲区淘汰
4. **同步**：脏缓冲区回写

---

## 3. 基本文件系统

### 3.1 超级块

```rust
pub struct SuperBlock {
    pub dev: u64,
    pub block_size: u32,
    pub block_count: u64,
    pub free_blocks: AtomicU64,
    pub inode_count: u64,
    pub free_inodes: AtomicU64,
    pub fs_type: [u8; 16],
    pub flags: AtomicU32,
    pub mount_point: [u8; 256],
    pub root_ino: Ino,
}
```

### 3.2 Inode

```rust
pub struct Inode {
    pub ino: Ino,
    pub sb: *mut SuperBlock,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub nlink: AtomicU32,
    pub size: AtomicU64,
    pub blocks: AtomicU64,
    pub atime: AtomicU64,
    pub mtime: AtomicU64,
    pub ctime: AtomicU64,
    pub flags: AtomicU32,
    pub ref_count: AtomicU32,
    pub private: u64,
}
```

### 3.3 Dentry

```rust
pub struct Dentry {
    pub name: [u8; 256],
    pub name_len: u32,
    pub inode: *mut Inode,
    pub parent: *mut Dentry,
    pub child: *mut Dentry,
    pub sibling: *mut Dentry,
    pub ref_count: AtomicU32,
    pub flags: AtomicU32,
}
```

### 3.4 文件

```rust
pub struct File {
    pub dentry: *mut Dentry,
    pub inode: *mut Inode,
    pub flags: u32,
    pub pos: AtomicU64,
    pub ref_count: AtomicU32,
    pub private: u64,
}
```

### 3.5 基本文件系统特性

1. **超级块**：文件系统元数据
2. **Inode**：文件元数据
3. **Dentry**：目录缓存
4. **文件**：打开文件管理

---

## 4. VFS（虚拟文件系统）

### 4.1 VFS 架构

VFS 为不同文件系统提供统一的抽象层：

```
+------------------+
|   Applications   |
+------------------+
         |
+------------------+
|   System Calls   |
+------------------+
         |
+------------------+
|      VFS         |
+------------------+
         |
    +----+----+----+
    |         |    |
+-------+ +----+ +------+
|NuvaFS | |Ext4| |FAT32 |
+-------+ +----+ +------+
    |         |
+-------+ +-------+
| NFSv3 | | SMB2/3|
+-------+ +-------+
```

### 4.2 文件系统操作

```rust
pub trait FileSystem {
    fn mount(&mut self, device: &Device) -> Result<()>;
    fn unmount(&mut self) -> Result<()>;
    fn root(&self) -> &Inode;
    fn statfs(&self) -> Result<Statfs>;
}
```

### 4.3 Inode 操作

```rust
pub trait InodeOps {
    fn lookup(&self, name: &str) -> Result<Arc<Inode>>;
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, offset: u64, buf: &[u8]) -> Result<usize>;
    fn create(&self, name: &str, mode: u32) -> Result<Arc<Inode>>;
    fn unlink(&self, name: &str) -> Result<()>;
    fn mkdir(&self, name: &str, mode: u32) -> Result<Arc<Inode>>;
    fn rmdir(&self, name: &str) -> Result<()>;
}
```

### 4.4 文件操作

```rust
pub trait FileOps {
    fn read(&self, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, buf: &[u8]) -> Result<usize>;
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
    fn ioctl(&self, cmd: u32, arg: u64) -> Result<u64>;
    fn mmap(&self, addr: u64, len: u64, prot: u32) -> Result<u64>;
    fn close(&mut self) -> Result<()>;
}
```

### 4.5 VFS 特性

1. **统一接口**：不同文件系统的单一接口
2. **缓存**：页缓存和缓冲区缓存
3. **命名空间**：挂载点和路径解析
4. **权限**：文件权限检查
5. **网络文件系统**：NFSv3 和 SMB2/3 远程挂载支持

---

## 5. NuvaFS

### 5.1 NuvaFS 设计

NuvaFS 是 Nuva OS 的原生文件系统，具有以下特性：

- **日志结构**：写操作追加到日志
- **写时复制**：数据永远不会原地覆盖
- **快照**：时间点快照
- **压缩**：数据压缩以节省空间

### 5.2 NuvaFS 结构

```
+------------------+
|   Super Block    |
+------------------+
|   Log Header     |
+------------------+
|   Log Entries    |
+------------------+
|   Inode Table    |
+------------------+
|   Data Blocks    |
+------------------+
```

### 5.3 NuvaFS 特性

1. **日志**：元数据日志，用于崩溃恢复
2. **写时复制**：数据完整性和快照支持
3. **压缩**：ZSTD/LZ4 压缩
4. **加密**：可选文件加密
5. **去重**：块级去重

### 5.4 日志机制

NuvaFS 采用写前日志（WAL）保证崩溃一致性：

1. **事务开始**：记录事务开始标记
2. **日志写入**：将元数据修改写入日志区域
3. **日志提交**：写入提交记录并强制刷盘
4. **数据写入**：写入实际数据块
5. **检查点**：周期性将日志中的修改应用到主区域

**日志恢复**：
- 挂载时扫描日志区域
- 重放已提交但未检查点的事务
- 丢弃未提交的事务

### 5.5 快照机制

NuvaFS 基于 COW 实现瞬间快照：

1. **创建快照**：
   - 复制根 Inode 指针
   - 标记所有数据块为 COW
   - 快照创建是 O(1) 操作
2. **COW 写入**：
   - 修改数据时先复制块
   - 在新块上执行写入
   - 更新 Inode 指向新块
3. **快照删除**：
   - 引用计数减 1
   - 引用为 0 的块释放回空闲池

---

## 6. ext4 文件系统

### 6.1 ext4 特性

| 特性 | 说明 |
|------|------|
| 最大文件大小 | 16TB |
| 最大文件系统大小 | 1EB |
| 块大小 | 1KB/2KB/4KB |
| 日志模式 | journal/ordered/writeback |
| 扩展属性 | 支持 |
| 在线扩容 | 支持 |

### 6.2 ext4 日志模式

| 模式 | 说明 |
|------|------|
| `journal` | 数据和元数据都写入日志，最安全但最慢 |
| `ordered` | 元数据写入日志，数据在日志提交前写入（默认） |
| `writeback` | 元数据写入日志，数据不保证顺序，最快 |

### 6.3 ext4 extent

ext4 使用 extent 替代传统的间接块映射：

```
extent: [逻辑块号, 长度, 物理块号]
```

- 单个 extent 可映射多达 128MB 连续块
- extent 树最多 4 层，支持超大文件

---

## 7. FAT32 文件系统

### 7.1 FAT32 特性

| 特性 | 说明 |
|------|------|
| 最大文件大小 | 4GB - 1 |
| 最大文件系统大小 | 2TB |
| 簇大小 | 512B - 64KB |
| 文件名 | 长 255 字符（VFAT LFN） |

### 7.2 FAT32 结构

```
+------------------+
|   Boot Sector    |  包含 BPB（BIOS Parameter Block）
+------------------+
|   FAT 1          |  文件分配表（主）
+------------------+
|   FAT 2          |  文件分配表（备份）
+------------------+
|   Data Region    |  文件数据区
+------------------+
```

### 7.3 FAT 表操作

- 空簇：0x00000000
- 文件结束：0x0FFFFFF8 - 0x0FFFFFFF
- 坏簇：0x0FFFFFF7
- 下一簇：0x00000002 - 0x0FFFFFF6

---

## 8. 目录缓存

### 8.1 目录缓存结构

```rust
pub struct DentryCache {
    pub hash_table: [*mut Dentry; HASH_SIZE],
    pub lru_list: DentryLruList,
}

impl DentryCache {
    pub fn lookup(&mut self, key: &DentryKey, name: &[u8]) -> *mut Dentry;
    pub fn add(&mut self, dentry: *mut Dentry) -> bool;
}
```

### 8.2 目录缓存特性

- **哈希查找**：基于父目录 Inode + 文件名哈希
- **LRU 淘汰**：未使用目录项按 LRU 回收
- **负缓存**：缓存查找失败结果，避免重复查找
- **自动失效**：文件删除/重命名时自动失效相关缓存项

---

## 9. NFSv3 客户端

### 9.1 概述

NFSv3 客户端（`kernel/net/nfs.rs`）实现了 NFS 版本 3 协议，支持通过 TCP/UDP socket 传输访问远程 NFS 服务器。

### 9.2 传输层

- **TCP/UDP Socket 传输**：支持 TCP 和 UDP 两种传输方式，TCP 模式提供可靠传输
- **RPC 调用**：`rpc_call()` 方法实现完整的 RPC 请求发送、响应接收和超时重传逻辑

### 9.3 XDR 解码

XDR（外部数据表示）响应解码实现：

| 方法 | 说明 |
|------|------|
| `parse_rpc_reply` | 解析 RPC Reply 消息头，验证 XID 和接受状态 |
| `decode_status` | 解码 NFS 操作状态码 |
| `decode_fh` | 解码 NFS 文件句柄（fhandle3） |
| `decode_fattr` | 解码 NFS 文件属性（fattr3） |

### 9.4 NFS 操作

所有 NFS 操作方法通过 `rpc_call()` 发送请求并解码响应：

- **GETATTR/SETATTR**：获取/设置文件属性
- **LOOKUP**：目录查找
- **READ/READDIR**：读取文件数据/目录内容
- **WRITE/CREATE/MKDIR**：写入文件/创建文件/创建目录
- **REMOVE/RMDIR**：删除文件/删除目录
- **RENAME**：重命名
- **SYMLINK/READLINK**：符号链接创建/读取
- **MOUNT/UMOUNT**：挂载/卸载远程导出

---

## 10. SMB2/3 客户端

### 10.1 概述

SMB2/3 客户端（`kernel/net/smb.rs`）实现了 SMB2 和 SMB3 协议，支持通过 Direct TCP 传输访问远程 SMB/CIFS 服务器。

### 10.2 传输层

- **TCP Socket 传输**：使用 TCP 连接与 SMB 服务器通信
- **Direct TCP 封包**：`send_and_receive()` 方法实现 Direct TCP 协议的封包（0x00 前缀 + NetBIOS 长度）和解包逻辑
- **响应头验证**：`parse_reply_header()` 验证 SMB2 响应头的协议 ID、结构大小和命令字段

### 10.3 连接建立

`connect()` 方法执行完整的 SMB2 连接建立流程：

1. **TCP 连接**：建立与目标服务器的 TCP socket 连接
2. **Negotiate**：发送 SMB2 negotiate 请求，解析服务器支持的方言列表和协商响应
3. **Session Setup**：建立 SMB2 会话（认证）

### 10.4 SMB2/3 操作

- **CREATE/CLOSE**：打开/关闭文件
- **READ/WRITE**：读取/写入文件数据
- **QUERY_DIRECTORY**：目录列举
- **SET_INFO/QUERY_INFO**：设置/查询文件信息
- **TREE_CONNECT/TREE_DISCONNECT**：连接/断开共享

---

## 11. io_uring 异步 IO

### 11.1 io_uring 架构

io_uring 提供高性能异步 IO 接口，通过共享环形缓冲区实现用户态和内核态零拷贝通信：

```rust
pub struct IoUring {
    pub sq_ring: IoSqRing,    // 提交队列环（Submission Queue）
    pub cq_ring: IoCqRing,    // 完成队列环（Completion Queue）
    pub sqes: *mut IoSqe,     // 提交队列条目数组
}
```

### 11.2 io_uring 操作

```rust
impl IoUring {
    pub fn submit(&mut self, sqe: &IoSqe) -> i32;
    pub fn get_completion(&mut self) -> Option<IoCqe>;
}
```

**工作流程**：
1. **提交**：应用将 IO 请求写入 SQE，更新 SQ 环尾指针
2. **内核消费**：内核从 SQ 读取请求并执行
3. **完成**：内核将结果写入 CQE，更新 CQ 环尾指针
4. **收割**：应用从 CQ 读取完成事件

### 11.3 IO 操作码

```rust
pub enum IoOpCode {
    Nop = 0,
    Read = 1,
    Write = 2,
    Open = 5,
    Close = 6,
    // ...
}
```

### 11.4 io_uring 特性

- **零拷贝**：用户态和内核态共享内存
- **批量提交**：一次系统调用提交多个 IO 请求
- **轮询模式**：`IORING_SETUP_IOPOLL` 避免中断开销
- **固定缓冲区**：`IORING_REGISTER_BUFFERS` 避免页表映射
- **文件注册**：`IORING_REGISTER_FILES` 避免文件描述符查找

---

## 12. 文件结构

```
kernel/fs/
├── vfs/                # VFS implementation
│   └── ...
├── page_cache.rs       # Page cache
├── buffer.rs           # Buffer cache
├── filesystem.rs       # Basic file system
├── nuvafs.rs           # NuvaFS implementation
├── inode.rs            # Inode management
├── dentry.rs           # Dentry cache
└── file.rs             # File management

kernel/net/
├── nfs.rs              # NFSv3 client
└── smb.rs              # SMB2/3 client
```

---

**最后更新**：2026 年 5 月 30 日
**许可证**：Apache-2.0
