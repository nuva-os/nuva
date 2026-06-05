# Nuva OS Memory Management Module

## Overview

The memory management module provides complete memory management functionality, including physical memory management, virtual memory management, and advanced memory features. The core allocator uses a Buddy+SLAB two-level architecture, with NUMA multi-node support, huge page mechanism, memory hotplug, and copy-on-write.

---

## Table of Contents

1. [Memory Layout](#1-memory-layout)
2. [Physical Memory Management](#2-physical-memory-management)
3. [Virtual Memory Management](#3-virtual-memory-management)
4. [Advanced Memory Management](#4-advanced-memory-management)
5. [Page Allocator](#5-page-allocator)
6. [COW Mechanism](#6-cow-mechanism)
7. [NUMA Support](#7-numa-support)
8. [Memory Hotplug](#8-memory-hotplug)
9. [Page Migration](#9-page-migration)
10. [OOM Killer](#10-oom-killer)
11. [Memory Compaction](#11-memory-compaction)
12. [Huge Pages](#12-huge-pages)
13. [Statistics](#13-statistics)
14. [File Structure](#14-file-structure)

---

## 1. Memory Layout

### 1.1 Virtual Address Space

ARM64 uses 48-bit virtual address space, total size 256TB:

```
+------------------+ 0xFFFFFFFF_FFFFFFFF
|   Kernel Space   | 128TB (High 128TB)
+------------------+ 0x00008000_00000000
|   User Space     | 128TB (Low 128TB)
+------------------+ 0x00000000_00000000
```

### 1.2 Kernel Space Layout

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

### 1.3 User Space Layout

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

### 1.4 Physical Memory Layout

```
+------------------+ 0x00000000
|   DMA Zone       | 16MB
+------------------+ 0x01000000
|   Normal Zone    | ~4GB
+------------------+ 0x100000000 (4GB)
|   HighMem Zone   | >4GB
+------------------+
```

### 1.5 Page Table Levels

ARM64/x86-64 use 4-level page tables, LoongArch64 uses 3-level page tables (4KB pages):

**ARM64/x86-64 (4-level)**:

| Level | Name | Coverage per Entry |
|-------|------|-------------------|
| Level 0 | PGD | 512GB |
| Level 1 | PUD | 1GB |
| Level 2 | PMD | 2MB |
| Level 3 | PTE | 4KB |

**LoongArch64 (3-level)**:

| Level | Name | Coverage per Entry |
|-------|------|-------------------|
| Level 0 | PGD | 256GB |
| Level 1 | PMD | 512MB |
| Level 2 | PTE | 4KB |

### 1.6 LoongArch64 Memory Layout

LoongArch64 uses a virtual address space layout compatible with x86-64:

```
+------------------+ 0xFFFF_FFFF_FFFF_FFFF
|   Kernel Space   | 128TB
+------------------+ 0xFFFF_8000_0000_0000
|   Non-Canonical  | 128TB (Non-addressable)
+------------------+ 0x0000_8000_0000_0000
|   User Space     | 128TB
+------------------+ 0x0000_0000_0000_0000
```

LoongArch64 MMU features:
- 3-level page tables, 4KB base page size
- Huge page support (2MB, 1GB)
- Hardware-managed TLB, flushed via `invtlb` instruction
- `kernel/arch/loongarch64/mod.rs` provides complete `PageTableOps` implementation

LoongArch64 PageTableOps implementation (`LoongArch64PageTable`):
- `create()`: Allocate PGD page via buddy allocator FFI
- `destroy(pgd)`: Recursively free all intermediate and leaf pages of 3-level page table
- `map(pgd, vaddr, paddr, prot)`: 3-level page table walk, allocate intermediate page table pages on demand
- `unmap(pgd, vaddr)`: Clear leaf PTE and flush TLB
- `translate(pgd, vaddr)`: 3-level page table address translation
- `protect(pgd, vaddr, prot)`: Modify leaf PTE permission bits
- Page table page allocation via `buddy_alloc_page` / `buddy_free_page` FFI interface

---

## 2. Physical Memory Management

### 2.1 Page Structure

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

### 2.2 Page Flags

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

### 2.3 Page Allocation Functions

| Function | Description |
|----------|-------------|
| `alloc_page()` | Allocate single page (4KB) |
| `alloc_pages(order)` | Allocate 2^order contiguous pages |
| `alloc_zeroed_page()` | Allocate and zero a page |
| `free_page(phys)` | Free single page |
| `free_pages(phys, order)` | Free multiple contiguous pages |

### 2.4 Reference Count Management

| Function | Description |
|----------|-------------|
| `inc_page_ref(phys)` | Increment reference count |
| `dec_page_ref(phys)` | Decrement reference count (auto-free when 0) |
| `get_page_ref(phys)` | Get reference count |

---

## 3. Virtual Memory Management

### 3.1 Virtual Memory Area (VMA)

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

### 3.2 Memory Descriptor (MmStruct)

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

### 3.3 Memory Mapping Operations

| Operation | Description |
|-----------|-------------|
| `do_mmap()` | Create memory mapping |
| `do_munmap()` | Delete memory mapping |
| `find_vma()` | Find VMA |
| `merge_vma()` | Merge adjacent VMAs |

---

## 4. Advanced Memory Management

### 4.1 Dynamic mem_map Allocation

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

**Features**:
- Runtime dynamic allocation support
- Static initialization support
- Expansion and release support

### 4.2 Page Fault Handler

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

**Page Fault Results**:
- `Success`: Handled successfully
- `Retry`: Retry needed
- `WriteProtect`: Write protection
- `Segfault`: Segmentation fault
- `BusError`: Bus error
- `Oom`: Out of memory

---

## 5. Page Allocator

### 5.1 Buddy Allocator

The Buddy allocator is used for page-level allocation (4KB - 4MB), using a two-level buddy system:

| Order | Size |
|-------|------|
| 0 | 4KB (1 page) |
| 1 | 8KB (2 pages) |
| 2 | 16KB (4 pages) |
| 9 | 2MB (512 pages) |
| 10 | 4MB (1024 pages) |

**Buddy Allocator Technical Details**:

- **Data Structure**: Each order maintains a free block list (`free_list[order]`) and free block count (`nr_free[order]`)
- **Allocation Algorithm**:
  1. Search the free list for the target order
  2. If no free block exists, split from a higher order (buddy splitting)
  3. When splitting, add the buddy block to the next lower order's free list
- **Free Algorithm**:
  1. Calculate buddy block address (`buddy = addr ^ (1 << (order + PAGE_SHIFT))`)
  2. If buddy block is free, merge into a higher order block
  3. Recursively merge until buddy block cannot be merged
- **Features**:
  - O(1) time complexity for allocation and free
  - Reduces external fragmentation
  - Supports huge page allocation
  - Per-CPU page cache acceleration

### 5.2 Slab Allocator

The Slab allocator is used for small object allocation, built on top of the Buddy allocator:

```rust
pub struct SlabAllocator {
    pub caches: [Option<KmemCache>; 32],  // Cache array
    pub total_allocated: AtomicU64,        // Total allocated
    pub total_freed: AtomicU64,            // Total freed
}
```

**SLAB Allocator Technical Details**:

- **Three-Level Structure**: Cache -> Slab -> Object
  - **Cache** (`KmemCache`): Manages object caches of the same type, containing object size, constructor/destructor
  - **Slab**: Composed of one or more contiguous physical pages, divided into full/partial/free lists
  - **Object**: Free objects within a slab managed via embedded free list pointers
- **Slab Coloring**: Reduces cache line conflicts by offsetting the start address
  - `color = offset % cache_line_size`
  - Different slabs use different colors, improving CPU cache utilization
- **Per-CPU Cache**: Each CPU maintains a local object cache to avoid global lock contention
  - Prioritizes Per-CPU cache for allocation
  - Batch transfer (batch) to/from global slab lists when cache is full/empty
- **Features**:
  - Reduces internal fragmentation
  - Object cache reuse
  - Supports constructor/destructor
  - Hardware cache alignment

### 5.3 Per-CPU Page Cache

```rust
pub struct PerCpuPageCache {
    pub pages: [*mut Page; PCP_CACHE_SIZE],
    pub count: AtomicU32,
    pub high: u32,    // High watermark
    pub batch: u32,   // Batch transfer size
}
```

The Per-CPU page cache provides a lock-free fast allocation path, avoiding global lock contention in the Buddy allocator.

---

## 6. COW Mechanism

### 6.1 COW Flow

**During fork**:
1. Mark page as COW (set `PG_COW` flag)
2. Increment reference count
3. Set page table entry to read-only

**During COW page fault**:
1. Allocate new page
2. Copy page content
3. Decrement original page reference
4. Update page table entry to writable

### 6.2 COW Implementation Details

The COW mechanism is implemented based on page fault handling:

1. **Write Protection Trigger**: A page fault is triggered when a process writes to a read-only page
2. **COW Detection**: Check `PG_COW` flag and reference count
3. **Page Copy**:
   - If `ref_count == 1`: Remove COW mark directly, set writable
   - If `ref_count > 1`: Allocate new page, `copy_page(dst, src)`, update page table
4. **TLB Flush**: Flush corresponding TLB entry after update

### 6.3 COW Related Functions

| Function | Description |
|----------|-------------|
| `mark_page_cow(phys)` | Mark as COW page |
| `is_page_cow(phys)` | Check if COW page |
| `copy_page(dst, src)` | Copy page content |

---

## 7. NUMA Support

### 7.1 NUMA Node Structure

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

### 7.2 NUMA Allocation Policy

1. Prefer allocation on hinted node (MPOL_PREFERRED)
2. Otherwise allocate on current node (MPOL_LOCAL)
3. Finally try other nodes, sorted by distance matrix (MPOL_INTERLEAVE)

### 7.3 Distance Matrix

Used to optimize cross-node access by selecting the nearest node. Distance value 10 indicates local access, values greater than 10 indicate remote access latency.

### 7.4 NUMA Auto-Balancing

- Periodically scan process memory access patterns
- Migrate frequently accessed pages to the node where the process runs
- Reduce cross-node memory access latency

---

## 8. Memory Hotplug

### 8.1 Memory Region Structure

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

### 8.2 Memory Region States

| State | Description |
|-------|-------------|
| `Offline` | Offline |
| `GoingOnline` | Going online |
| `Online` | Online |
| `GoingOffline` | Going offline |

### 8.3 Hotplug Operations

| Operation | Description |
|-----------|-------------|
| `add_region()` | Add memory region |
| `online_region()` | Bring memory region online |
| `offline_region()` | Bring memory region offline |

---

## 9. Page Migration

### 9.1 Migration Reasons

| Reason | Description |
|--------|-------------|
| `Compaction` | Memory compaction |
| `Hotplug` | Memory hotplug |
| `NumaBalance` | NUMA balancing |
| `MemoryPolicy` | Memory policy |
| `CopyOnWrite` | COW |

### 9.2 Migration Functions

| Function | Description |
|----------|-------------|
| `migrate_page()` | Migrate single page |
| `migrate_range()` | Migrate page range |
| `compact_zone()` | Memory compaction |

---

## 10. OOM Killer

When system memory is exhausted and sufficient memory cannot be freed through reclamation, the OOM Killer selects processes to terminate in order to free memory:

### 10.1 OOM Score Calculation

```
oom_score = (process_memory / total_memory) * 1000 + oom_score_adj
```

- `process_memory`: Total memory used by the process (RSS + swap)
- `total_memory`: Total available system memory
- `oom_score_adj`: User-adjustable OOM score adjustment value (-1000 to 1000)

### 10.2 OOM Killer Flow

1. Check out-of-memory condition
2. Calculate OOM score for all processes
3. Select process with highest score
4. Send SIGKILL signal to terminate process
5. Wait for memory to be freed
6. If still insufficient, select next process

### 10.3 OOM Policy Configuration

- `oom_score_adj`: Set process OOM score adjustment (-1000 = never kill, 1000 = kill first)
- `oom_kill_disable`: Disable OOM Killer (requires cgroup support)

---

## 11. Memory Compaction

Memory compaction reduces external fragmentation by migrating movable pages to make room for large contiguous allocations:

### 11.1 Compaction Flow

1. Scan memory zone, find free pages from low end
2. Scan migratable pages from high end
3. Move migratable pages to freed positions at low end
4. Form contiguous free blocks at high end
5. Check if allocation requirements are met

### 11.2 Compaction Results

```rust
pub enum CompactResult {
    Success = 0,         // Compaction successful, meets allocation requirements
    Partial = 1,         // Partial compaction, not fully met
    NoSuitablePages = 2, // No migratable pages
    NotEnoughFree = 3,   // Not enough free pages
    Skipped = 4,         // Compaction skipped
}
```

### 11.3 Compaction Trigger Conditions

- Direct compaction: Triggered synchronously when high-order allocation fails
- Background compaction: Periodic compaction by kcompactd kernel thread
- Manual compaction: Triggered via `/proc/sys/vm/compact_memory`

---

## 12. Huge Pages

### 12.1 Huge Page Types

| Type | Size | Page Table Order |
|------|------|-----------------|
| Standard Page | 4KB | 0 |
| Transparent Huge Page (THP) | 2MB | 9 |
| Giant Page | 1GB | 18 |

### 12.2 Huge Page API

```rust
pub enum HugePageSize {
    Huge2MB = 21,
    Huge1GB = 30,
}

pub fn init_huge_pages();
pub fn alloc_huge_page(size: HugePageSize) -> Option<PhysAddr>;
pub fn free_huge_page(addr: PhysAddr, size: HugePageSize);
```

### 12.3 Transparent Huge Pages (THP)

- Automatically merge contiguous 4KB pages into 2MB huge pages
- Reduces page table levels and TLB misses
- Background scanning and merging via `khugepaged` kernel thread
- Configurable via `/sys/kernel/mm/transparent_hugepage/enabled`

---

## 13. Statistics

### 13.1 Page Allocation Statistics

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

## 14. File Structure

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

**Last Updated**: May 30, 2026
**License**: Apache-2.0
