# Nuva OS File System Module

## Overview

The file system module provides complete file management functionality, including VFS (Virtual File System), page cache, buffer cache, directory cache, NuvaFS native file system, ext4 and FAT32 compatible file systems, NFSv3 and SMB2/3 network file system clients, and io_uring asynchronous IO mechanism.

---

## Table of Contents

1. [Page Cache](#1-page-cache)
2. [Buffer Cache](#2-buffer-cache)
3. [Basic File System](#3-basic-file-system)
4. [VFS (Virtual File System)](#4-vfs-virtual-file-system)
5. [NuvaFS](#5-nuvafs)
6. [ext4 File System](#6-ext4-file-system)
7. [FAT32 File System](#7-fat32-file-system)
8. [Directory Cache](#8-directory-cache)
9. [NFSv3 Client](#9-nfsv3-client)
10. [SMB2/3 Client](#10-smb23-client)
11. [io_uring Asynchronous IO](#11-io_uring-asynchronous-io)
12. [File Structure](#12-file-structure)

---

## 1. Page Cache

### 1.1 Page Structure

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

### 1.2 Page Cache

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

### 1.3 Address Space

```rust
pub struct AddressSpace {
    pub ino: u64,
    pub dev: u64,
    pub page_count: AtomicU32,
    pub dirty_count: AtomicU32,
    pub writeback_index: AtomicU64,
}
```

### 1.4 Page Cache Features

1. **Page Management**: Page allocation and release
2. **Dirty Page Management**: Dirty page marking and writeback
3. **Address Space**: File mapping
4. **Statistics**: Hit rate statistics
5. **LRU Eviction**: Active/inactive dual list

### 1.5 Page Cache Operations

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

## 2. Buffer Cache

### 2.1 Buffer Head

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

### 2.2 Buffer Cache

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

### 2.3 Buffer Cache Features

1. **Buffer Management**: Buffer allocation and release
2. **Hash Lookup**: Fast lookup (device number + block number hash)
3. **LRU Eviction**: Buffer eviction
4. **Synchronization**: Dirty buffer writeback

---

## 3. Basic File System

### 3.1 Super Block

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

### 3.4 File

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

### 3.5 Basic File System Features

1. **Super Block**: File system metadata
2. **Inode**: File metadata
3. **Dentry**: Directory cache
4. **File**: Open file management

---

## 4. VFS (Virtual File System)

### 4.1 VFS Architecture

VFS provides a unified abstraction layer for different file systems:

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

### 4.2 File System Operations

```rust
pub trait FileSystem {
    fn mount(&mut self, device: &Device) -> Result<()>;
    fn unmount(&mut self) -> Result<()>;
    fn root(&self) -> &Inode;
    fn statfs(&self) -> Result<Statfs>;
}
```

### 4.3 Inode Operations

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

### 4.4 File Operations

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

### 4.5 VFS Features

1. **Unified Interface**: Single interface for different file systems
2. **Caching**: Page cache and buffer cache
3. **Namespace**: Mount points and path resolution
4. **Permissions**: File permission checks
5. **Network File Systems**: NFSv3 and SMB2/3 remote mount support

---

## 5. NuvaFS

### 5.1 NuvaFS Design

NuvaFS is the native file system of Nuva OS with the following features:

- **Log-structured**: Write operations append to log
- **Copy-on-write**: Data is never overwritten in place
- **Snapshots**: Point-in-time snapshots
- **Compression**: Data compression to save space

### 5.2 NuvaFS Structure

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

### 5.3 NuvaFS Features

1. **Journaling**: Metadata journal for crash recovery
2. **Copy-on-write**: Data integrity and snapshot support
3. **Compression**: ZSTD/LZ4 compression
4. **Encryption**: Optional file encryption
5. **Deduplication**: Block-level deduplication

### 5.4 Journaling Mechanism

NuvaFS uses Write-Ahead Logging (WAL) to ensure crash consistency:

1. **Transaction Begin**: Record transaction begin marker
2. **Journal Write**: Write metadata modifications to journal area
3. **Journal Commit**: Write commit record and force flush
4. **Data Write**: Write actual data blocks
5. **Checkpoint**: Periodically apply journal modifications to main area

**Journal Recovery**:
- Scan journal area on mount
- Replay committed but not checkpointed transactions
- Discard uncommitted transactions

### 5.5 Snapshot Mechanism

NuvaFS implements instantaneous snapshots based on COW:

1. **Create Snapshot**:
   - Copy root Inode pointer
   - Mark all data blocks as COW
   - Snapshot creation is O(1) operation
2. **COW Write**:
   - Copy block before modification
   - Execute write on new block
   - Update Inode to point to new block
3. **Delete Snapshot**:
   - Decrement reference count
   - Release blocks with zero references back to free pool

---

## 6. ext4 File System

### 6.1 ext4 Features

| Feature | Description |
|---------|-------------|
| Max file size | 16TB |
| Max filesystem size | 1EB |
| Block size | 1KB/2KB/4KB |
| Journal mode | journal/ordered/writeback |
| Extended attributes | Supported |
| Online resizing | Supported |

### 6.2 ext4 Journal Modes

| Mode | Description |
|------|-------------|
| `journal` | Both data and metadata written to journal, safest but slowest |
| `ordered` | Metadata written to journal, data written before journal commit (default) |
| `writeback` | Metadata written to journal, data ordering not guaranteed, fastest |

### 6.3 ext4 Extents

ext4 uses extents instead of traditional indirect block mapping:

```
extent: [logical_block, length, physical_block]
```

- Single extent can map up to 128MB contiguous blocks
- Extent tree up to 4 levels, supporting very large files

---

## 7. FAT32 File System

### 7.1 FAT32 Features

| Feature | Description |
|---------|-------------|
| Max file size | 4GB - 1 |
| Max filesystem size | 2TB |
| Cluster size | 512B - 64KB |
| Filename | 255 chars long (VFAT LFN) |

### 7.2 FAT32 Structure

```
+------------------+
|   Boot Sector    |  Contains BPB (BIOS Parameter Block)
+------------------+
|   FAT 1          |  File Allocation Table (primary)
+------------------+
|   FAT 2          |  File Allocation Table (backup)
+------------------+
|   Data Region    |  File data area
+------------------+
```

### 7.3 FAT Table Operations

- Free cluster: 0x00000000
- End of file: 0x0FFFFFF8 - 0x0FFFFFFF
- Bad cluster: 0x0FFFFFF7
- Next cluster: 0x00000002 - 0x0FFFFFF6

---

## 8. Directory Cache

### 8.1 Directory Cache Structure

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

### 8.2 Directory Cache Features

- **Hash Lookup**: Based on parent Inode + filename hash
- **LRU Eviction**: Unused dentries reclaimed by LRU
- **Negative Caching**: Cache lookup failures to avoid repeated lookups
- **Automatic Invalidation**: Invalidate related cache entries on file delete/rename

---

## 9. NFSv3 Client

### 9.1 Overview

The NFSv3 client (`kernel/net/nfs.rs`) implements the NFS version 3 protocol, supporting access to remote NFS servers via TCP/UDP socket transport.

### 9.2 Transport Layer

- **TCP/UDP Socket Transport**: Supports both TCP and UDP transport, with TCP mode providing reliable delivery
- **RPC Call**: The `rpc_call()` method implements complete RPC request sending, response receiving, and timeout retransmission logic

### 9.3 XDR Decoding

XDR (External Data Representation) response decoding implementation:

| Method | Description |
|--------|-------------|
| `parse_rpc_reply` | Parse RPC Reply message header, verify XID and accept status |
| `decode_status` | Decode NFS operation status code |
| `decode_fh` | Decode NFS file handle (fhandle3) |
| `decode_fattr` | Decode NFS file attributes (fattr3) |

### 9.4 NFS Operations

All NFS operation methods send requests and decode responses via `rpc_call()`:

- **GETATTR/SETATTR**: Get/set file attributes
- **LOOKUP**: Directory lookup
- **READ/READDIR**: Read file data / read directory contents
- **WRITE/CREATE/MKDIR**: Write file / create file / create directory
- **REMOVE/RMDIR**: Remove file / remove directory
- **RENAME**: Rename
- **SYMLINK/READLINK**: Create symbolic link / read symbolic link
- **MOUNT/UMOUNT**: Mount / unmount remote export

---

## 10. SMB2/3 Client

### 10.1 Overview

The SMB2/3 client (`kernel/net/smb.rs`) implements the SMB2 and SMB3 protocols, supporting access to remote SMB/CIFS servers via Direct TCP transport.

### 10.2 Transport Layer

- **TCP Socket Transport**: Uses TCP connection to communicate with SMB servers
- **Direct TCP Framing**: The `send_and_receive()` method implements Direct TCP protocol framing (0x00 prefix + NetBIOS length) and deframing logic
- **Reply Header Validation**: `parse_reply_header()` validates SMB2 reply header's protocol ID, structure size, and command fields

### 10.3 Connection Establishment

The `connect()` method performs the complete SMB2 connection establishment flow:

1. **TCP Connection**: Establish TCP socket connection to the target server
2. **Negotiate**: Send SMB2 negotiate request, parse server-supported dialect list and negotiate response
3. **Session Setup**: Establish SMB2 session (authentication)

### 10.4 SMB2/3 Operations

- **CREATE/CLOSE**: Open/close file
- **READ/WRITE**: Read/write file data
- **QUERY_DIRECTORY**: Directory listing
- **SET_INFO/QUERY_INFO**: Set/query file information
- **TREE_CONNECT/TREE_DISCONNECT**: Connect/disconnect share

---

## 11. io_uring Asynchronous IO

### 11.1 io_uring Architecture

io_uring provides a high-performance asynchronous IO interface through shared ring buffers for zero-copy communication between user space and kernel:

```rust
pub struct IoUring {
    pub sq_ring: IoSqRing,    // Submission Queue ring
    pub cq_ring: IoCqRing,    // Completion Queue ring
    pub sqes: *mut IoSqe,     // Submission Queue Entry array
}
```

### 11.2 io_uring Operations

```rust
impl IoUring {
    pub fn submit(&mut self, sqe: &IoSqe) -> i32;
    pub fn get_completion(&mut self) -> Option<IoCqe>;
}
```

**Workflow**:
1. **Submit**: Application writes IO request to SQE, updates SQ ring tail pointer
2. **Kernel Consumption**: Kernel reads requests from SQ and executes
3. **Complete**: Kernel writes results to CQE, updates CQ ring tail pointer
4. **Harvest**: Application reads completion events from CQ

### 11.3 IO Operation Codes

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

### 11.4 io_uring Features

- **Zero Copy**: User space and kernel share memory
- **Batch Submission**: Submit multiple IO requests in a single system call
- **Polling Mode**: `IORING_SETUP_IOPOLL` avoids interrupt overhead
- **Fixed Buffers**: `IORING_REGISTER_BUFFERS` avoids page table mapping
- **File Registration**: `IORING_REGISTER_FILES` avoids file descriptor lookup

---

## 12. File Structure

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

**Last Updated**: May 30, 2026
**License**: Apache-2.0
