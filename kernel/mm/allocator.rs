/*
 * Nuva OS - Kernel - Kernel
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.

 */

// ! DynamicMemoryAllocatedeviceImplementation
/*!*/
// ! theModuleImplementationKernel DynamicMemoryAllocateWorkcan, Package:
// ! - HeapMemorymanagementadministration
// ! - Slab Allocatedevice
// ! - Memorypoolmanagementadministration
// ! - largeBlockMemoryAllocate

use core::ptr;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use core::alloc::Layout;
use crate::mm::buddy::{get_buddy, MAX_ORDER};
use crate::mm::memory::{PhysAddr, VirtAddr, PAGE_SIZE, phys_to_virt, virt_to_phys};
use crate::mm::page_alloc::{alloc_pages, free_pages};

/// Error code
pub mod errno {
 pub const ENOMEM: i64 = -12;
 pub const EINVAL: i64 = -22;
}

/// HeapMemoryManager
pub struct HeapAllocator {
 /// HeapstartbeginAddress
 pub heap_start: VirtAddr,
 /// HeapEndAddress
 pub heap_end: VirtAddr,
 /// HeapCurrentpointer
 pub heap_current: AtomicU64,
 /// HeapSize
 pub heap_size: u64,
 /// alreadyAllocateBytenumber
 pub allocated_bytes: AtomicU64,
 /// Initialized flag
 pub initialized: AtomicBool,
}

impl HeapAllocator {
 pub const fn new() -> Self {
 HeapAllocator {
 heap_start: 0,
 heap_end: 0,
 heap_current: AtomicU64::new(0),
 heap_size: 0,
 allocated_bytes: AtomicU64::new(0),
 initialized: AtomicBool::new(false),
 }
 }

 /// InitializeHeap
 pub fn init(&mut self, start: VirtAddr, size: u64) {
 if self.initialized.load(Ordering::Acquire) {
 log_warn!("Heap already initialized");
 return;
 }

 self.heap_start = start;
 self.heap_end = start + size;
 self.heap_current.store(start, Ordering::Release);
 self.heap_size = size;
 self.allocated_bytes.store(0, Ordering::Release);
 self.initialized.store(true, Ordering::Release);

 log_info!("Heap allocator initialized:");
 log_info!(" Start: {:#x}", start);
 log_info!(" End: {:#x}", self.heap_end);
 log_info!(" Size: {} bytes ({} MB)", size, size / 1024 / 1024);
 }

 /// AllocateMemory(simpleformLinearAllocate)
 pub fn alloc(&self, size: usize, align: usize) -> *mut u8 {
 if !self.initialized.load(Ordering::Acquire) {
 log_error!("Heap not initialized");
 return ptr::null_mut();
 }

 // AlignmentSize
 let size = (size + align - 1) & !(align - 1);

 // AtomicAllocate
 loop {
 let current = self.heap_current.load(Ordering::Acquire);
 let aligned = (current + align as u64 - 1) & !(align as u64 - 1);
 let new_current = aligned + size as u64;

 // CheckifexceedexitHeapRange
 if new_current > self.heap_end {
 log_error!("Heap exhausted: requested {} bytes", size);
 return ptr::null_mut();
 }

 // CAS Operation
 if self.heap_current.compare_exchange(
 current,
 new_current,
 Ordering::AcqRel,
 Ordering::Acquire,
 ).is_ok() {
 self.allocated_bytes.fetch_add(size as u64, Ordering::AcqRel);
 return aligned as *mut u8;
 }
 }
 }

 /// GetHeapmakeuseStatistics
 pub fn get_stats(&self) -> HeapStats {
 HeapStats {
 total_size: self.heap_size,
 used_size: self.allocated_bytes.load(Ordering::Acquire),
 free_size: self.heap_end - self.heap_current.load(Ordering::Acquire),
 }
 }

 /// CheckifalreadyInitialize
 pub fn is_initialized(&self) -> bool {
 self.initialized.load(Ordering::Acquire)
 }
}

/// HeapStatisticsInfo
#[derive(Debug, Clone, Copy)]
pub struct HeapStats {
 pub total_size: u64,
 pub used_size: u64,
 pub free_size: u64,
}

/// GlobalHeapAllocatedevice
static HEAP_ALLOCATOR: core::sync::OnceLock<HeapAllocator> = core::sync::OnceLock::new();

/// GetHeapAllocatedevice
pub fn heap() -> &'static HeapAllocator {
    HEAP_ALLOCATOR.get_or_init(HeapAllocator::new)
}

/// InitializeHeap
pub fn init_heap(start: VirtAddr, size: u64) {
 let heap = get_heap();
 heap.init(start, size);
}

/// Slab Allocatedevice
pub struct SlabAllocator {
 /// Slab CachingArray
 pub caches: [SlabCache; 16],
 /// Initialized flag
 pub initialized: AtomicBool,
}

/// Slab Caching
pub struct SlabCache {
 /// CachingName
 pub name: &'static str,
 /// ObjectSize
 pub object_size: usize,
 /// emptyidlelinkform
 pub free_list: *mut SlabObject,
 /// alreadyAllocateObjectnumber
 pub active: AtomicU64,
 /// totalObjectnumber
 pub total: AtomicU64,
 /// Slab pageFacenumber
 pub slab_pages: AtomicU64,
}

/// Slab Object
#[repr(C)]
pub struct SlabObject {
 /// NextemptyidleObject
 pub next: *mut SlabObject,
 /// Data
 pub data: [u8; 0],
}

impl SlabCache {
 pub const fn new(name: &'static str, object_size: usize) -> Self {
 SlabCache {
 name,
 object_size,
 free_list: ptr::null_mut(),
 active: AtomicU64::new(0),
 total: AtomicU64::new(0),
 slab_pages: AtomicU64::new(0),
 }
 }

 /// AllocateObject
 pub fn alloc(&mut self) -> *mut u8 {
 // Checkemptyidlelinkform
 if self.free_list.is_null() {
 // needwantAllocatenew slab
 if !self.grow() {
 return ptr::null_mut();
 }
 }

 // secondaryemptyidlelinkformtakeexit
 let obj = self.free_list;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 self.free_list = (*obj).next;
 }

 self.active.fetch_add(1, Ordering::AcqRel);
 obj as *mut u8
 }

 /// FreeObject
 pub fn free(&mut self, obj: *mut u8) {
 if obj.is_null() {
 return;
 }

 // addPlustoemptyidlelinkform
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let slab_obj = obj as *mut SlabObject;
 (*slab_obj).next = self.free_list;
 self.free_list = slab_obj;
 }

 self.active.fetch_sub(1, Ordering::AcqRel);
 }

 /// ScalingCaching
 fn grow(&mut self) -> bool {
 // AllocateaitempageFace
 let phys = alloc_pages(0);
 if phys == 0 {
 log_error!("SlabCache::grow: failed to allocate page");
 return false;
 }

 let virt = phys_to_virt(phys);
 let num_objects = PAGE_SIZE as usize / self.object_size;

 // Initializeemptyidlelinkform
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 for i in 0..num_objects {
 let obj = (virt as usize + i * self.object_size) as *mut SlabObject;
 (*obj).next = self.free_list;
 self.free_list = obj;
 }
 }

 self.total.fetch_add(num_objects as u64, Ordering::AcqRel);
 self.slab_pages.fetch_add(1, Ordering::AcqRel);

 log_debug!("SlabCache::grow: added {} objects for cache '{}'", num_objects, self.name);
 true
 }

 /// Get statistics
 pub fn get_stats(&self) -> SlabCacheStats {
 SlabCacheStats {
 name: self.name,
 object_size: self.object_size,
 active: self.active.load(Ordering::Acquire),
 total: self.total.load(Ordering::Acquire),
 slab_pages: self.slab_pages.load(Ordering::Acquire),
 }
 }
}

/// Slab CachingstatisticsInfo
#[derive(Debug, Clone, Copy)]
pub struct SlabCacheStats {
 pub name: &'static str,
 pub object_size: usize,
 pub active: u64,
 pub total: u64,
 pub slab_pages: u64,
}

impl SlabAllocator {
 pub const fn new() -> Self {
 SlabAllocator {
 caches: [
 SlabCache::new("size-8", 8),
 SlabCache::new("size-16", 16),
 SlabCache::new("size-32", 32),
 SlabCache::new("size-64", 64),
 SlabCache::new("size-128", 128),
 SlabCache::new("size-256", 256),
 SlabCache::new("size-512", 512),
 SlabCache::new("size-1024", 1024),
 SlabCache::new("size-2048", 2048),
 SlabCache::new("size-4096", 4096),
 SlabCache::new("size-8192", 8192),
 SlabCache::new("size-16384", 16384),
 SlabCache::new("size-32768", 32768),
 SlabCache::new("size-65536", 65536),
 SlabCache::new("size-131072", 131072),
 SlabCache::new("size-262144", 262144),
 ],
 initialized: AtomicBool::new(false),
 }
 }

 /// Initialize Slab Allocatedevice
 pub fn init(&mut self) {
 if self.initialized.load(Ordering::Acquire) {
 log_warn!("Slab allocator already initialized");
 return;
 }

 log_info!("Slab allocator initialized with {} caches", self.caches.len());
 self.initialized.store(true, Ordering::Release);
 }

 /// AllocateMemory
 pub fn alloc(&mut self, size: usize) -> *mut u8 {
 if size == 0 {
 return ptr::null_mut();
 }

 // findtocombinefit Caching
 for cache in &mut self.caches {
 if size <= cache.object_size {
 return cache.alloc();
 }
 }

 // toolarge, makeusepageFaceAllocatedevice
 let order = calculate_order(size);
 if order > MAX_ORDER {
 log_error!("SlabAllocator::alloc: size too large: {}", size);
 return ptr::null_mut();
 }

 let phys = alloc_pages(order);
 if phys == 0 {
 return ptr::null_mut();
 }

 phys_to_virt(phys) as *mut u8
 }

 /// FreeMemory
 pub fn free(&mut self, ptr: *mut u8, size: usize) {
 if ptr.is_null() {
 return;
 }

 // findtocombinefit Caching
 for cache in &mut self.caches {
 if size <= cache.object_size {
 cache.free(ptr);
 return;
 }
 }

 // makeusepageFaceAllocatedeviceFree
 let order = calculate_order(size);
 if order <= MAX_ORDER {
 let virt = ptr as VirtAddr;
 let phys = virt_to_phys(virt);
 free_pages(phys, order);
 }
 }

 /// Get statistics
 pub fn get_stats(&self) -> &[SlabCache; 16] {
 &self.caches
 }

 /// CheckifalreadyInitialize
 pub fn is_initialized(&self) -> bool {
 self.initialized.load(Ordering::Acquire)
 }
}

/// Global Slab Allocatedevice
static SLAB_ALLOCATOR: core::sync::OnceLock<SlabAllocator> = core::sync::OnceLock::new();

/// Get Slab Allocatedevice
pub fn slab() -> &'static SlabAllocator {
    SLAB_ALLOCATOR.get_or_init(SlabAllocator::new)
}

/// Initialize Slab Allocatedevice
pub fn init_slab() {
 let slab = get_slab();
 slab.init();
}

/// Memorypool
pub struct MemoryPool {
 /// poolName
 pub name: &'static str,
 /// poolstartbeginAddress
 pub start: VirtAddr,
 /// poolSize
 pub size: u64,
 /// ObjectSize
 pub object_size: usize,
 /// emptyidlelinkform
 pub free_list: *mut u8,
 /// alreadyAllocateObjectnumber
 pub active: AtomicU64,
 /// totalObjectnumber
 pub total: AtomicU64,
}

impl MemoryPool {
 pub const fn new(name: &'static str, object_size: usize) -> Self {
 MemoryPool {
 name,
 start: 0,
 size: 0,
 object_size,
 free_list: ptr::null_mut(),
 active: AtomicU64::new(0),
 total: AtomicU64::new(0),
 }
 }

 /// InitializeMemorypool
 pub fn init(&mut self, start: VirtAddr, size: u64) {
 self.start = start;
 self.size = size;
 self.free_list = ptr::null_mut();

 // Initializeemptyidlelinkform
 let num_objects = size as usize / self.object_size;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 for i in 0..num_objects {
 let obj = (start as usize + i * self.object_size) as *mut u8;
 *(obj as *mut *mut u8) = self.free_list;
 self.free_list = obj;
 }
 }

 self.total.store(num_objects as u64, Ordering::Release);
 self.active.store(0, Ordering::Release);

 log_info!("Memory pool '{}' initialized:", self.name);
 log_info!(" Start: {:#x}", start);
 log_info!(" Size: {} bytes", size);
 log_info!(" Object size: {} bytes", self.object_size);
 log_info!(" Total objects: {}", num_objects);
 }

 /// AllocateObject
 pub fn alloc(&mut self) -> *mut u8 {
 if self.free_list.is_null() {
 log_error!("Memory pool '{}' exhausted", self.name);
 return ptr::null_mut();
 }

 let obj = self.free_list;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 self.free_list = *(obj as *const *mut u8);
 }

 self.active.fetch_add(1, Ordering::AcqRel);
 obj
 }

 /// FreeObject
 pub fn free(&mut self, obj: *mut u8) {
 if obj.is_null() {
 return;
 }

 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 *(obj as *mut *mut u8) = self.free_list;
 self.free_list = obj;
 }

 self.active.fetch_sub(1, Ordering::AcqRel);
 }

 /// Get statistics
 pub fn get_stats(&self) -> MemoryPoolStats {
 MemoryPoolStats {
 name: self.name,
 object_size: self.object_size,
 active: self.active.load(Ordering::Acquire),
 total: self.total.load(Ordering::Acquire),
 }
 }
}

/// MemorypoolStatisticsInfo
#[derive(Debug, Clone, Copy)]
pub struct MemoryPoolStats {
 pub name: &'static str,
 pub object_size: usize,
 pub active: u64,
 pub total: u64,
}

/// largeBlockMemoryAllocatedevice
pub struct LargeAllocator {
 /// Initialized flag
 pub initialized: AtomicBool,
 /// totalAllocatetimenumber
 pub total_allocs: AtomicU64,
 /// totalFreetimenumber
 pub total_frees: AtomicU64,
 /// CurrentAllocateBytenumber
 pub current_bytes: AtomicU64,
}

impl LargeAllocator {
 pub const fn new() -> Self {
 LargeAllocator {
 initialized: AtomicBool::new(false),
 total_allocs: AtomicU64::new(0),
 total_frees: AtomicU64::new(0),
 current_bytes: AtomicU64::new(0),
 }
 }

 /// Initialize
 pub fn init(&mut self) {
 if self.initialized.load(Ordering::Acquire) {
 return;
 }

 log_info!("Large allocator initialized");
 self.initialized.store(true, Ordering::Release);
 }

 /// AllocatelargeBlockMemory
 pub fn alloc(&mut self, size: usize) -> *mut u8 {
 if size == 0 {
 return ptr::null_mut();
 }

 // Computeneedwant pagenumber
 let pages_needed = (size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
 let order = calculate_order(pages_needed * PAGE_SIZE as usize);

 if order > MAX_ORDER {
 log_error!("LargeAllocator::alloc: size too large: {}", size);
 return ptr::null_mut();
 }

 let phys = alloc_pages(order);
 if phys == 0 {
 log_error!("LargeAllocator::alloc: failed to allocate {} bytes", size);
 return ptr::null_mut();
 }

 self.total_allocs.fetch_add(1, Ordering::AcqRel);
 self.current_bytes.fetch_add((1 << order) * PAGE_SIZE, Ordering::AcqRel);

 phys_to_virt(phys) as *mut u8
 }

 /// FreelargeBlockMemory
 pub fn free(&mut self, ptr: *mut u8, size: usize) {
 if ptr.is_null() {
 return;
 }

 let pages_needed = (size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
 let order = calculate_order(pages_needed * PAGE_SIZE as usize);

 if order <= MAX_ORDER {
 let virt = ptr as VirtAddr;
 let phys = virt_to_phys(virt);
 free_pages(phys, order);

 self.total_frees.fetch_add(1, Ordering::AcqRel);
 self.current_bytes.fetch_sub((1 << order) * PAGE_SIZE, Ordering::AcqRel);
 }
 }

 /// Get statistics
 pub fn get_stats(&self) -> LargeAllocatorStats {
 LargeAllocatorStats {
 total_allocs: self.total_allocs.load(Ordering::Acquire),
 total_frees: self.total_frees.load(Ordering::Acquire),
 current_bytes: self.current_bytes.load(Ordering::Acquire),
 }
 }
}

/// largeBlockMemoryAllocatedeviceStatisticsInfo
#[derive(Debug, Clone, Copy)]
pub struct LargeAllocatorStats {
 pub total_allocs: u64,
 pub total_frees: u64,
 pub current_bytes: u64,
}

/// GloballargeBlockMemoryAllocatedevice
static LARGE_ALLOCATOR: core::sync::OnceLock<LargeAllocator> = core::sync::OnceLock::new();

/// GetlargeBlockMemoryAllocatedevice
pub fn large() -> &'static LargeAllocator {
    LARGE_ALLOCATOR.get_or_init(LargeAllocator::new)
}

/// InitializelargeBlockMemoryAllocatedevice
pub fn init_large() {
 let large = get_large();
 large.init();
}

/// Computestepnumber
fn calculate_order(size: usize) -> usize {
 if size == 0 {
 return 0;
 }

 let pages = (size + PAGE_SIZE as usize - 1) / PAGE_SIZE as usize;
 let mut order = 0;
 let mut tmp = 1;

 while tmp < pages {
 order += 1;
 tmp *= 2;
 }

 order
}

/// systema MemoryAllocateInterface
pub fn kmalloc(size: usize) -> *mut u8 {
 if size == 0 {
 return ptr::null_mut();
 }

 // smallObjectmakeuse Slab Allocatedevice
 if size <= 262144 {
 let slab = get_slab();
 if slab.is_initialized() {
 return slab.alloc(size);
 }
 }

 // largeObjectmakeuselargeBlockMemoryAllocatedevice
 let large = get_large();
 if large.initialized.load(Ordering::Acquire) {
 return large.alloc(size);
 }

 // mostthenmakeuseHeapAllocatedevice
 let heap = get_heap();
 if heap.is_initialized() {
 return heap.alloc(size, 8);
 }

 ptr::null_mut()
}

/// systema MemoryFreeInterface
pub fn kfree(ptr: *mut u8, size: usize) {
 if ptr.is_null() {
 return;
 }

 // smallObjectmakeuse Slab Allocatedevice
 if size <= 262144 {
 let slab = get_slab();
 if slab.is_initialized() {
 slab.free(ptr, size);
 return;
 }
 }

 // largeObjectmakeuselargeBlockMemoryAllocatedevice
 let large = get_large();
 if large.initialized.load(Ordering::Acquire) {
 large.free(ptr, size);
 return;
 }

 // HeapAllocatedevicenotSupportFree
 log_warn!("kfree: cannot free memory from heap allocator");
}

/// AllocateAlignmentMemory
pub fn kmalloc_aligned(size: usize, align: usize) -> *mut u8 {
 if size == 0 {
 return ptr::null_mut();
 }

 let heap = get_heap();
 if heap.is_initialized() {
 return heap.alloc(size, align);
 }

 // OtherAllocatedevicetempnotSupportAlignmentAllocate
 kmalloc(size)
}

/// InitializeplacefiniteAllocatedevice
pub fn init_allocators(heap_start: VirtAddr, heap_size: u64) {
 log_info!("Initializing memory allocators");

 // InitializeHeapAllocatedevice
 init_heap(heap_start, heap_size);

 // Initialize Slab Allocatedevice
 init_slab();

 // InitializelargeBlockMemoryAllocatedevice
 init_large();

 log_info!("All memory allocators initialized");
}

/// printstampAllocatedeviceStatisticsInfo
pub fn print_allocator_stats() {
 log_info!("Memory Allocator Statistics:");

 // HeapStatistics
 let heap = get_heap();
 if heap.is_initialized() {
 let stats = heap.get_stats();
 log_info!(" Heap:");
 log_info!(" Total: {} bytes", stats.total_size);
 log_info!(" Used: {} bytes", stats.used_size);
 log_info!(" Free: {} bytes", stats.free_size);
 }

 // Slab statistics
 let slab = get_slab();
 if slab.is_initialized() {
 log_info!(" Slab:");
 for cache in slab.get_stats() {
 if cache.total.load(Ordering::Acquire) > 0 {
 let stats = cache.get_stats();
 log_info!(" {}: {}/{} objects, {} pages",
 stats.name, stats.active, stats.total, stats.slab_pages);
 }
 }
 }

 // largeBlockMemoryStatistics
 let large = get_large();
 if large.initialized.load(Ordering::Acquire) {
 let stats = large.get_stats();
 log_info!(" Large:");
 log_info!(" Total allocs: {}", stats.total_allocs);
 log_info!(" Total frees: {}", stats.total_frees);
 log_info!(" Current bytes: {}", stats.current_bytes);
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_heap_new() {
 let heap = HeapAllocator::new();
 assert!(!heap.is_initialized());
 }

 #[test]
 fn test_slab_new() {
 let slab = SlabAllocator::new();
 assert!(!slab.is_initialized());
 }

 #[test]
 fn test_calculate_order() {
 assert_eq!(calculate_order(0), 0);
 assert_eq!(calculate_order(1), 0);
 assert_eq!(calculate_order(4096), 0);
 assert_eq!(calculate_order(4097), 1);
 assert_eq!(calculate_order(8192), 1);
 assert_eq!(calculate_order(8193), 2);
 }
}