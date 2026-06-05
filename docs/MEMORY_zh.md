# Nuva OS 内存管理模块

## 概述

内存管理模块提供完整的内存管理功能，包括物理内存管理、虚拟内存管理和高级内存特性。核心分配器采用 Buddy+SLAB 二级架构，支持 NUMA 多节点、大页机制、内存热插拔和写时复制。

---

## 目录

1. [内存布局](#1-内存布局)
2. [物理内存管理](#2-物理内存管理)
3. [虚拟内存管理](#3-虚拟内存管理)
4. [高级内存管理](#4-高级内存管理)
5. [页分配器](#5-页分配器)
6. [COW 机制](#6-cow-机制)
7. [NUMA 支持](#7-numa-支持)
8. [内存热插拔](#8-内存热插拔)
9. [页迁移](#9-页迁移)
10. [OOM Killer](#10-oom-killer)
11. [内存规整](#11-内存规整)
12. [大页机制](#12-大页机制)
13. [统计](#13-统计)
14. [文件结构](#14-文件结构)

---

## 1. 内存布局

### 1.1 虚拟地址空间

ARM64 使用 48 位虚拟地址空间，总大小 256TB：

```
+------------------+ 0xFFFFFFFF_FFFFFFFF
|   Kernel Space   | 128TB (High 128TB)
+------------------+ 0x00008000_00000000
|   User Space     | 128TB (Low 128TB)
+------------------+ 0x00000000_00000000
```

### 1.2 Kernel 空间布局

```
+------------------+ 0xFFFFFF80_00000000 (Kernel virtual base)
|   Kernel Text    | .text (Read-Execute)
+------------------+
|   Kernel RO Data | .rodata (Read-Only)
+------------------+
|   Kernel Data    | .data (Read-Write)
+------------------+
|   Kernel BSS     | .bss (Read-Write)
+------------------+
|   Kernel Heap    | 1GB (Dynamic allocation)
+------------------+ 0xFFFFFF80_08000000
```

### 1.3 用户空间布局

```
+------------------+ 0x00007FFF_FFFFFFFF
|   User Stack     | 8MB (Grows down)
+------------------+
|   mmap Region    | (Dynamic mapping)
+------------------+ 0x00004000_00000000
|   User Heap      | (Grows up)
+------------------+
|   User Data      | .data
+------------------+
|   User Text      | .text
+------------------+
|   VVAR/VDSO      | 8MB
+------------------+ 0x00000000_00000000
```

### 1.4 物理内存布局

```
+------------------+ 0x00000000
|   DMA Zone       | 16MB
+------------------+ 0x01000000
|   Normal Zone    | ~4GB
+------------------+ 0x100000000 (4GB)
|   HighMem Zone   | >4GB
+------------------+
```

### 1.5 页表级别

ARM64/x86-64 使用 4 级页表，LoongArch64 使用 3 级页表（4KB 页）：

**ARM64/x86-64（4 级）**：

| 级别 | 名称 | 每项覆盖范围 |
|-------|------|-------------------|
| Level 0 | PGD | 512GB |
| Level 1 | PUD | 1GB |
| Level 2 | PMD | 2MB |
| Level 3 | PTE | 4KB |

**LoongArch64（3 级）**：

| 级别 | 名称 | 每项覆盖范围 |
|-------|------|-------------------|
| Level 0 | PGD | 256GB |
| Level 1 | PMD | 512MB |
| Level 2 | PTE | 4KB |

### 1.6 LoongArch64 内存布局

LoongArch64 使用与 x86-64 兼容的虚拟地址空间布局：

```
+------------------+ 0xFFFF_FFFF_FFFF_FFFF
|   Kernel Space   | 128TB
+------------------+ 0xFFFF_8000_0000_0000
|   Non-Canonical  | 128TB (Non-addressable)
+------------------+ 0x0000_8000_0000_0000
|   User Space     | 128TB
+------------------+ 0x0000_0000_0000_0000
```

LoongArch64 MMU 特性：
- 3 级页表，4KB 基本页大小
- 支持大页（2MB、1GB）
- TLB 硬件管理，使用 `invtlb` 指令刷新
- `kernel/arch/loongarch64/mod.rs` 提供 `PageTableOps` 完整实现

LoongArch64 PageTableOps 实现（`LoongArch64PageTable`）：
- `create()`：通过 buddy allocator FFI 分配 PGD 页
- `destroy(pgd)`：递归释放 3 级页表所有中间页和叶子页
- `map(pgd, vaddr, paddr, prot)`：3 级页表遍历，按需分配中间级页表页
- `unmap(pgd, vaddr)`：清除叶子 PTE 并刷新 TLB
- `translate(pgd, vaddr)`：3 级页表地址翻译
- `protect(pgd, vaddr, prot)`：修改叶子 PTE 权限位
- 页表页分配通过 `buddy_alloc_page` / `buddy_free_page` FFI 接口

---

## 2. 物理内存管理

### 2.1 页结构

```rust
#[repr(C)]
pub struct Page {
    pub flags: AtomicU32,        // Flags
    pub ref_count: AtomicU32,    // Reference count
    pub phys_addr: PhysAddr,     // Physical address
    pub map_count: AtomicU32,    // Map count
    pub mm: u64,                 // Address space
    pub private: u64,            // Private data
    pub lru_next: *mut Page,     // LRU list
    pub lru_prev: *mut Page,
}
```

### 2.2 页标志

```rust
pub mod page_flags {
    pub const PG_LOCKED: u32     = 0x00000001;  // Locked
    pub const PG_DIRTY: u32      = 0x00000002;  // Modified
    pub const PG_UPTODATE: u32   = 0x00000004;  // Data valid
    pub const PG_COW: u32        = 0x00000200;  // COW page
    pub const PG_ANON: u32       = 0x00000400;  // Anonymous page
    pub const PG_REFERENCED: u32 = 0x00002000;  // Referenced
    pub const PG_LRU: u32        = 0x00004000;  // In LRU list
    pub const PG_ACTIVE: u32     = 0x00008000;  // Active
}
```

### 2.3 页分配函数

| 函数 | 描述 |
|----------|-------------|
| `alloc_page()` | 分配单个页（4KB） |
| `alloc_pages(order)` | 分配 2^order 个连续页 |
| `alloc_zeroed_page()` | 分配并清零页 |
| `free_page(phys)` | 释放单个页 |
| `free_pages(phys, order)` | 释放多个连续页 |

### 2.4 引用计数管理

| 函数 | 描述 |
|----------|-------------|
| `inc_page_ref(phys)` | 增加引用计数 |
| `dec_page_ref(phys)` | 减少引用计数（为 0 时自动释放） |
| `get_page_ref(phys)` | 获取引用计数 |

---

## 3. 虚拟内存管理

### 3.1 虚拟内存区域 (VMA)

```rust
pub struct Vma {
    pub start: u64,              // Start virtual address
    pub end: u64,                // End virtual address
    pub flags: AtomicU32,        // Flags
    pub prot: u32,               // Protection attributes
    pub pgoff: u64,              // File offset
    pub file: u64,               // Associated file
    pub next: *mut Vma,          // Next VMA
    pub prev: *mut Vma,          // Previous VMA
    pub ref_count: AtomicU32,    // Reference count
}
```

### 3.2 内存描述符 (MmStruct)

```rust
pub struct MmStruct {
    pub start_code: u64,         // Code segment start
    pub end_code: u64,           // Code segment end
    pub start_data: u64,         // Data segment start
    pub end_data: u64,           // Data segment end
    pub start_brk: u64,          // Heap start
    pub brk: AtomicU64,          // Current heap position
    pub start_stack: u64,        // Stack start
    pub mmap: *mut Vma,          // VMA list
    pub map_count: AtomicU32,    // VMA count
    pub total_vm: AtomicU64,     // Total virtual memory
    pub pgd: u64,                // Page global directory
    pub ref_count: AtomicU32,    // Reference count
}
```

### 3.3 内存映射操作

| 操作 | 描述 |
|-----------|-------------|
| `do_mmap()` | 创建内存映射 |
| `do_munmap()` | 删除内存映射 |
| `find_vma()` | 查找 VMA |
| `merge_vma()` | 合并相邻 VMA |

---

## 4. 高级内存管理

### 4.1 动态 mem_map 分配

```rust
pub struct DynamicMemMap {
    pub mem_map: *mut Page,        // mem_map array pointer
    pub size: u64,                 // Array size
    pub start_pfn: u64,            // Start page frame number
    pub end_pfn: u64,              // End page frame number
    pub is_dynamic: bool,          // Is dynamically allocated
    pub initialized: AtomicBool,   // Initialized flag
}
```

**特性**：
- 运行时动态分配支持
- 静态初始化支持
- 扩展和释放支持

### 4.2 缺页处理程序

```rust
pub struct PageFaultHandler {
    pub fault_count: AtomicU64,    // Fault count
    pub user_faults: AtomicU64,    // User-space faults
    pub kernel_faults: AtomicU64,  // Kernel-space faults
    pub write_faults: AtomicU64,   // Write faults
    pub read_faults: AtomicU64,    // Read faults
    pub cow_count: AtomicU64,      // COW count
    pub swapin_count: AtomicU64,   // Swap-in count
}
```

**缺页结果**：
- `Success`：处理成功
- `Retry`：需要重试
- `WriteProtect`：写保护
- `Segfault`：段错误
- `BusError`：总线错误
- `Oom`：内存不足

---

## 5. 页分配器

### 5.1 Buddy 分配器

Buddy 分配器用于页级分配（4KB - 4MB），采用二级伙伴系统：

| 阶 | 大小 |
|-------|------|
| 0 | 4KB（1 页） |
| 1 | 8KB（2 页） |
| 2 | 16KB（4 页） |
| 9 | 2MB（512 页） |
| 10 | 4MB（1024 页） |

**Buddy 分配器技术细节**：

- **数据结构**：每个阶维护一个空闲块链表（`free_list[order]`），以及空闲块计数（`nr_free[order]`）
- **分配算法**：
  1. 在目标阶的空闲链表中查找
  2. 若无空闲块，从更高阶分裂（buddy splitting）
  3. 分裂时将伙伴块加入低一阶的空闲链表
- **释放算法**：
  1. 计算伙伴块地址（`buddy = addr ^ (1 << (order + PAGE_SHIFT))`）
  2. 若伙伴块空闲，合并为更高阶块
  3. 递归合并直到伙伴块不可合并
- **特性**：
  - O(1) 时间复杂度分配和释放
  - 减少外部碎片
  - 支持大页分配
  - Per-CPU 页缓存加速

### 5.2 Slab 分配器

Slab 分配器用于小对象分配，构建在 Buddy 分配器之上：

```rust
pub struct SlabAllocator {
    pub caches: [Option<KmemCache>; 32],  // Cache array
    pub total_allocated: AtomicU64,        // Total allocated
    pub total_freed: AtomicU64,            // Total freed
}
```

**SLAB 分配器技术细节**：

- **三级结构**：Cache → Slab → Object
  - **Cache**（`KmemCache`）：管理同类型对象的缓存，包含对象大小、构造/析构函数
  - **Slab**：由一个或多个连续物理页组成，分为 full/partial/free 三个链表
  - **Object**：Slab 内的空闲对象通过内嵌空闲链表指针管理
- **Slab 着色（Coloring）**：通过偏移起始地址减少缓存行冲突
  - `color = offset % cache_line_size`
  - 不同 Slab 使用不同颜色，提高 CPU 缓存利用率
- **Per-CPU 缓存**：每个 CPU 维护本地对象缓存，避免全局锁
  - 分配时优先从 Per-CPU 缓存获取
  - 缓存满/空时批量转移（batch）到/从全局 Slab 链表
- **特性**：
  - 减少内部碎片
  - 对象缓存复用
  - 支持构造/析构函数
  - 硬件缓存对齐

### 5.3 Per-CPU 页缓存

```rust
pub struct PerCpuPageCache {
    pub pages: [*mut Page; PCP_CACHE_SIZE],
    pub count: AtomicU32,
    pub high: u32,    // 高水位线
    pub batch: u32,   // 批量转移大小
}
```

Per-CPU 页缓存提供无锁的快速分配路径，避免 Buddy 分配器的全局锁竞争。

---

## 6. COW 机制

### 6.1 COW 流程

**fork 时**：
1. 标记页为 COW（设置 `PG_COW` 标志）
2. 增加引用计数
3. 将页表项设为只读

**COW 缺页时**：
1. 分配新页
2. 复制页内容
3. 减少原页引用
4. 将页表项更新为可写

### 6.2 COW 实现细节

COW 机制基于页错误处理实现：

1. **写保护触发**：进程写入只读页时触发页错误
2. **判断 COW**：检查 `PG_COW` 标志和引用计数
3. **复制页**：
   - 若 `ref_count == 1`：直接移除 COW 标记，设为可写
   - 若 `ref_count > 1`：分配新页，`copy_page(dst, src)`，更新页表
4. **TLB 刷新**：更新后刷新对应 TLB 条目

### 6.3 COW 相关函数

| 函数 | 描述 |
|----------|-------------|
| `mark_page_cow(phys)` | 标记为 COW 页 |
| `is_page_cow(phys)` | 检查是否为 COW 页 |
| `copy_page(dst, src)` | 复制页内容 |

---

## 7. NUMA 支持

### 7.1 NUMA 节点结构

```rust
pub struct NumaNode {
    pub node_id: u32,              // Node ID
    pub name: &'static str,        // Node name
    pub start_pfn: u64,            // Start page frame number
    pub end_pfn: u64,              // End page frame number
    pub total_pages: AtomicU64,    // Total pages
    pub free_pages: AtomicU64,     // Free pages
    pub mem_map: *mut Page,        // mem_map array
    pub zones: [Option<Zone>; 4],  // Memory zones
    pub distances: [u32; 16],      // Distance matrix
    pub cpus: [u32; 64],           // CPU list
}
```

### 7.2 NUMA 分配策略

1. 优先在提示节点分配（MPOL_PREFERRED）
2. 否则在当前节点分配（MPOL_LOCAL）
3. 最后尝试其他节点，按距离矩阵排序（MPOL_INTERLEAVE）

### 7.3 距离矩阵

用于优化跨节点访问，选择最近的节点。距离值 10 表示本地访问，大于 10 表示远程访问延迟。

### 7.4 NUMA 自动均衡

- 周期性扫描进程内存访问模式
- 将频繁访问的页迁移到进程运行的节点
- 减少跨节点内存访问延迟

---

## 8. 内存热插拔

### 8.1 内存区域结构

```rust
pub struct MemoryRegion {
    pub region_id: u32,            // Region ID
    pub start_phys: PhysAddr,      // Start physical address
    pub size: u64,                 // Size
    pub start_pfn: u64,            // Start page frame number
    pub end_pfn: u64,              // End page frame number
    pub state: MemoryRegionState,  // State
    pub node_id: u32,              // NUMA node ID
}
```

### 8.2 内存区域状态

| 状态 | 描述 |
|-------|-------------|
| `Offline` | 离线 |
| `GoingOnline` | 正在上线 |
| `Online` | 在线 |
| `GoingOffline` | 正在下线 |

### 8.3 热插拔操作

| 操作 | 描述 |
|-----------|-------------|
| `add_region()` | 添加内存区域 |
| `online_region()` | 上线内存区域 |
| `offline_region()` | 下线内存区域 |

---

## 9. 页迁移

### 9.1 迁移原因

| 原因 | 描述 |
|--------|-------------|
| `Compaction` | 内存规整 |
| `Hotplug` | 内存热插拔 |
| `NumaBalance` | NUMA 均衡 |
| `MemoryPolicy` | 内存策略 |
| `CopyOnWrite` | COW |

### 9.2 迁移函数

| 函数 | 描述 |
|----------|-------------|
| `migrate_page()` | 迁移单个页 |
| `migrate_range()` | 迁移页范围 |
| `compact_zone()` | 内存规整 |

---

## 10. OOM Killer

当系统内存耗尽且无法通过回收释放足够内存时，OOM Killer 选择终止进程以释放内存：

### 10.1 OOM 评分计算

```
oom_score = (process_memory / total_memory) * 1000 + oom_score_adj
```

- `process_memory`：进程使用的总内存（RSS + swap）
- `total_memory`：系统可用总内存
- `oom_score_adj`：用户可调的 OOM 评分调整值（-1000 到 1000）

### 10.2 OOM Killer 流程

1. 检查内存不足条件
2. 对所有进程计算 OOM 评分
3. 选择评分最高的进程
4. 发送 SIGKILL 信号终止进程
5. 等待内存释放
6. 若仍不足，选择下一个进程

### 10.3 OOM 策略配置

- `oom_score_adj`：设置进程 OOM 评分调整（-1000 = 永不杀死，1000 = 优先杀死）
- `oom_kill_disable`：禁用 OOM Killer（需要 cgroup 支持）

---

## 11. 内存规整

内存规整通过迁移可移动页来减少外部碎片，为大块连续分配腾出空间：

### 11.1 规整流程

1. 扫描内存区域，从低端开始寻找空闲页
2. 从高端扫描可迁移页
3. 将可迁移页移到低端已释放的位置
4. 在高端形成连续空闲块
5. 检查是否满足分配需求

### 11.2 规整结果

```rust
pub enum CompactResult {
    Success = 0,         // 规整成功，满足分配需求
    Partial = 1,         // 部分规整，未完全满足
    NoSuitablePages = 2, // 无可迁移页
    NotEnoughFree = 3,   // 空闲页不足
    Skipped = 4,         // 跳过规整
}
```

### 11.3 规整触发条件

- 直接规整：高阶分配失败时同步触发
- 后台规整：kcompactd 内核线程周期性规整
- 手动规整：通过 `/proc/sys/vm/compact_memory` 触发

---

## 12. 大页机制

### 12.1 大页类型

| 类型 | 大小 | 页表阶 |
|------|------|--------|
| 标准页 | 4KB | 0 |
| 透明大页 (THP) | 2MB | 9 |
| 巨页 | 1GB | 18 |

### 12.2 大页 API

```rust
pub enum HugePageSize {
    Huge2MB = 21,
    Huge1GB = 30,
}

pub fn init_huge_pages();
pub fn alloc_huge_page(size: HugePageSize) -> Option<PhysAddr>;
pub fn free_huge_page(addr: PhysAddr, size: HugePageSize);
```

### 12.3 透明大页 (THP)

- 自动将连续 4KB 页合并为 2MB 大页
- 减少页表层级和 TLB 失效
- 通过 `khugepaged` 内核线程后台扫描合并
- 可通过 `/sys/kernel/mm/transparent_hugepage/enabled` 配置

---

## 13. 统计

### 13.1 页分配统计

```rust
pub struct PageAllocStats {
    pub total_allocs: AtomicU64,   // Total allocations
    pub total_frees: AtomicU64,    // Total frees
    pub alloc_fails: AtomicU64,    // Allocation failures
    pub current_pages: AtomicU64,  // Current allocated pages
    pub cow_pages: AtomicU64,      // COW pages
    pub anon_pages: AtomicU64,     // Anonymous pages
}
```

---

## 14. 文件结构

```
kernel/mm/
├── memory.rs           # Physical memory management
├── buddy.rs            # Buddy allocator
├── slab.rs             # Slab allocator
├── allocator.rs        # Unified allocator interface
├── percpu_cache.rs     # Per-CPU page cache
├── page_alloc.rs       # Page allocation
├── mmap.rs             # Memory mapping
├── vma.rs              # VMA management
├── address_space.rs    # Address space management
├── fault.rs            # Page fault handling
├── numa.rs             # NUMA support
├── hotplug.rs          # Memory hotplug
├── migrate.rs          # Page migration
├── mem_map.rs          # mem_map implementation
├── complete_mem_map.rs # Complete mem_map
├── advanced_memory.rs  # Advanced memory features
├── advanced_features.rs # Advanced features
├── complete_features.rs # Complete features
└── mem_pool_ffi.rs     # Memory pool FFI
```

---

**最后更新**：2026 年 5 月 30 日
**许可证**：Apache-2.0
