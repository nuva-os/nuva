/*
 * Nuva OS - SystemLibrary - Lang
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


use core::sync::atomic::{AtomicU64, Ordering};

/// MemoryBlock
pub struct MemoryBlock {
 /// BlockAddress
 pub addr: u64,
 /// BlockSize
 pub size: usize,
 /// ifalreadyAllocate
 pub allocated: bool,
}

/// HeapManager
pub struct Heap {
 /// HeapstartbeginAddress
 pub start_addr: u64,
 /// HeapSize
 pub total_size: usize,
 /// alreadyuseSize
 pub used_size: AtomicU64,
 /// peakvaluemakeuse
 pub peak_usage: AtomicU64,
}

impl Heap {
 pub const fn new(start_addr: u64, total_size: usize) -> Self {
 Heap {
 start_addr,
 total_size,
 used_size: AtomicU64::new(0),
 peak_usage: AtomicU64::new(0),
 }
 }
 
 /// Initialize
 pub fn init(&mut self) {
 log_info!("Heap initialized");
 log_info!(" Start: {:#x}", self.start_addr);
 log_info!(" Size: {} MB", self.total_size / (1024 * 1024));
 }
 
 /// AllocateMemory
 pub fn alloc(&self, size: usize, alignment: usize) -> Option<u64> {
 // AlignmentSize
 let aligned_size = (size + alignment - 1) & !(alignment - 1);
 
 // Checkremainingremainderemptybetween
 let used = self.used_size.load(Ordering::Acquire) as usize;
 if used + aligned_size > self.total_size {
 return None;
 }
 
 // Allocate
 let addr = self.start_addr + used as u64;
 self.used_size.fetch_add(aligned_size as u64, Ordering::AcqRel);
 
 // Updatepeakvalue
 let new_used = self.used_size.load(Ordering::Acquire);
 let mut peak = self.peak_usage.load(Ordering::Acquire);
 while new_used > peak {
 match self.peak_usage.compare_exchange_weak(
 peak,
 new_used,
 Ordering::AcqRel,
 Ordering::Acquire,
 ) {
 Ok(_) => break,
 Err(current) => peak = current,
 }
 }
 
 Some(addr)
 }
 
 /// FreeMemory (simpleformImplementation,notSupportpartsplitFree)
 pub fn reset(&self) {
 self.used_size.store(0, Ordering::Release);
 }
 
 /// Getremainingremainderemptybetween
 pub fn available(&self) -> usize {
 let used = self.used_size.load(Ordering::Acquire) as usize;
 self.total_size - used
 }
 
 /// Getmakeuserate
 pub fn usage_ratio(&self) -> u32 {
 let used = self.used_size.load(Ordering::Acquire);
 (used * 100 / self.total_size as u64) as u32
 }
}

/// MemoryAllocatedevice
pub struct Allocator {
 /// Heap
 pub heap: Heap,
 /// Allocatetimenumber
 pub alloc_count: AtomicU64,
 /// Freetimenumber
 pub free_count: AtomicU64,
}

impl Allocator {
 pub const fn new(heap_start: u64, heap_size: usize) -> Self {
 Allocator {
 heap: Heap::new(heap_start, heap_size),
 alloc_count: AtomicU64::new(0),
 free_count: AtomicU64::new(0),
 }
 }
 
 /// Initialize
 pub fn init(&mut self) {
 self.heap.init();
 }
 
 /// Allocate
 pub fn alloc(&self, size: usize) -> Option<u64> {
 let addr = self.heap.alloc(size, 8)?; // 8 ByteAlignment
 self.alloc_count.fetch_add(1, Ordering::AcqRel);
 Some(addr)
 }
 
 /// Allocateparallelclear
 pub fn alloc_zeroed(&self, size: usize) -> Option<u64> {
 let addr = self.alloc(size)?;
 
 // clear
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let ptr = addr as *mut u8;
 for i in 0..size {
 *ptr.add(i) = 0;
 }
 }
 
 Some(addr)
 }
 
 /// repeatnewAllocate
 pub fn realloc(&self, _old_addr: u64, old_size: usize, new_size: usize) -> Option<u64> {
 // simpleformImplementation: AllocatenewBlockparallelCopy
 let new_addr = self.alloc(new_size)?;
 
 // CopyData
 let copy_size = old_size.min(new_size);
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let src = _old_addr as *const u8;
 let dst = new_addr as *mut u8;
 for i in 0..copy_size {
 *dst.add(i) = *src.add(i);
 }
 }
 
 Some(new_addr)
 }
 
 /// Free
 pub fn free(&self, _addr: u64) {
 // simpleformImplementation: notrealactualFree
 self.free_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// Get statistics
 pub fn get_stats(&self) -> (u64, u64, u64) {
 let alloc_count = self.alloc_count.load(Ordering::Acquire);
 let free_count = self.free_count.load(Ordering::Acquire);
 let used = self.heap.used_size.load(Ordering::Acquire);
 (alloc_count, free_count, used)
 }
}

/// GlobalAllocatedevice
static mut ALLOCATOR: Allocator = Allocator::new(0, 16 * 1024 * 1024); // 16 MB

pub fn get_allocator() -> &'static mut Allocator {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut ALLOCATOR }
}

pub fn init_allocator() {
 let allocator = get_allocator();
 allocator.init();
}