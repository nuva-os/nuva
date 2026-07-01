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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_info};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// BufferSize
pub const BUFFER_SIZE: usize = 4096;

/// MaxBuffernumber
pub const MAX_BUFFERS: usize = 1024;

/// BufferFlag
pub mod buffer_flags {
 pub const DIRTY: u32 = 1 << 0; // dirtyBuffer
 pub const VALID: u32 = 1 << 1; // Datavalid
 pub const LOCKED: u32 = 1 << 2; // Lockfixed
 pub const WRITEBACK: u32 = 1 << 3; // positiveinWriteback
 pub const ERROR: u32 = 1 << 4; // Error
 pub const UPTODATE: u32 = 1 << 5; // Datamostnew
 pub const SYNC: u32 = 1 << 6; // SynchronousWrite
}

/// BufferHead
pub struct BufferHead {
 /// Device ID
 pub dev: u64,
 /// Blocksignal
 pub block: u64,
 /// BlockSize
 pub size: u32,
 /// Flag
 pub flags: AtomicU32,
 /// referenceCount
 pub ref_count: AtomicU32,
 /// Data
 pub data: [u8; BUFFER_SIZE],
 /// NextBuffer
 pub next: *mut BufferHead,
 /// prefixaitemBuffer
 pub prev: *mut BufferHead,
 /// HashlinkformNext
 pub hash_next: *mut BufferHead,
 /// Hashlinkformprefixaitem
 pub hash_prev: *mut BufferHead,
 /// LRU linkformNext
 pub lru_next: *mut BufferHead,
 /// LRU linkformprefixaitem
 pub lru_prev: *mut BufferHead,
}

impl BufferHead {
 /// CreatenewBuffer
 pub fn new(dev: u64, block: u64, size: u32) -> Self {
 BufferHead {
 dev,
 block,
 size,
 flags: AtomicU32::new(0),
 ref_count: AtomicU32::new(0),
 data: [0; BUFFER_SIZE],
 next: core::ptr::null_mut(),
 prev: core::ptr::null_mut(),
 hash_next: core::ptr::null_mut(),
 hash_prev: core::ptr::null_mut(),
 lru_next: core::ptr::null_mut(),
 lru_prev: core::ptr::null_mut(),
 }
 }
 
 /// ifdirty
 pub fn is_dirty(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & buffer_flags::DIRTY) != 0
 }
 
 /// ifvalid
 pub fn is_valid(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & buffer_flags::VALID) != 0
 }
 
 /// ifLockfixed
 pub fn is_locked(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & buffer_flags::LOCKED) != 0
 }
 
 /// Setdirty
 pub fn set_dirty(&self) {
 self.flags.fetch_or(buffer_flags::DIRTY, Ordering::AcqRel);
 }
 
 /// clearDividedirty
 pub fn clear_dirty(&self) {
 self.flags.fetch_and(!buffer_flags::DIRTY, Ordering::AcqRel);
 }
 
 /// Setvalid
 pub fn set_valid(&self) {
 self.flags.fetch_or(buffer_flags::VALID, Ordering::AcqRel);
 }
 
 /// Lockfixed
 pub fn lock(&self) {
 while self.flags.compare_exchange_weak(
 0,
 buffer_flags::LOCKED,
 Ordering::Acquire,
 Ordering::Relaxed,
 ).is_err() {
 core::hint::spin_loop();
 }
 }
 
 /// Unlock
 pub fn unlock(&self) {
 self.flags.fetch_and(!buffer_flags::LOCKED, Ordering::AcqRel);
 }
 
 /// increasePlusreference
 pub fn get(&self) {
 self.ref_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// Minusfewreference
 pub fn put(&self) {
 self.ref_count.fetch_sub(1, Ordering::AcqRel);
 }
 
 /// GetreferenceCount
 pub fn get_ref_count(&self) -> u32 {
 self.ref_count.load(Ordering::Acquire)
 }
 
 /// ReadData
 pub fn read(&self, offset: usize, buf: &mut [u8]) -> usize {
 let start = offset.min(self.size as usize);
 let len = buf.len().min(self.size as usize - start);
 
 buf[..len].copy_from_slice(&self.data[start..start + len]);
 len
 }
 
 /// WriteData
 pub fn write(&mut self, offset: usize, buf: &[u8]) -> usize {
 let start = offset.min(self.size as usize);
 let len = buf.len().min(self.size as usize - start);
 
 self.data[start..start + len].copy_from_slice(&buf[..len]);
 self.set_dirty();
 len
 }
}

/// BufferCaching
pub struct BufferCache {
 /// Buffercount
 pub buffer_count: AtomicU32,
 /// dirtyBuffercount
 pub dirty_count: AtomicU32,
 /// infixtimenumber
 pub hits: AtomicU64,
 /// infixtimenumber
 pub misses: AtomicU64,
 /// Readtimenumber
 pub reads: AtomicU64,
 /// Writetimenumber
 pub writes: AtomicU64,
 /// MaxBuffernumber
 pub max_buffers: u32,
}

impl BufferCache {
 pub const fn new() -> Self {
 BufferCache {
 buffer_count: AtomicU32::new(0),
 dirty_count: AtomicU32::new(0),
 hits: AtomicU64::new(0),
 misses: AtomicU64::new(0),
 reads: AtomicU64::new(0),
 writes: AtomicU64::new(0),
 max_buffers: MAX_BUFFERS as u32,
 }
 }
 
 /// Initialize
 pub fn init(&mut self) {
 log_info!("Buffer cache initialized");
 log_info!(" Max buffers: {}", self.max_buffers);
 }
 
 /// FindBuffer
 pub fn find_buffer(&self, _dev: u64, _block: u64) -> Option<&BufferHead> {
 // TODO: ImplementationHashformFind
 self.misses.fetch_add(1, Ordering::AcqRel);
 None
 }
 
 /// GetBuffer
 pub fn get_buffer(&mut self, dev: u64, block: u64, size: u32) -> Option<&mut BufferHead> {
 // firstFind
 if self.find_buffer(dev, block).is_some() {
 self.hits.fetch_add(1, Ordering::AcqRel);
 // TODO: Returnfindto Buffer
 }
 
 // CreatenewBuffer
 if self.buffer_count.load(Ordering::Acquire) >= self.max_buffers {
 return None;
 }
 
 self.buffer_count.fetch_add(1, Ordering::AcqRel);
 
 // TODO: AllocateparallelReturnBuffer
 None
 }
 
 /// FreeBuffer
 pub fn release_buffer(&mut self, _bh: &mut BufferHead) {
 self.buffer_count.fetch_sub(1, Ordering::AcqRel);
 }
 
 /// ReadBlock
 pub fn read_block(&mut self, dev: u64, block: u64, buf: &mut [u8]) -> i64 {
 self.reads.fetch_add(1, Ordering::AcqRel);
 
 if let Some(bh) = self.get_buffer(dev, block, buf.len() as u32) {
 bh.read(0, buf) as i64
 } else {
 -1
 }
 }
 
 /// WriteBlock
 pub fn write_block(&mut self, dev: u64, block: u64, buf: &[u8]) -> i64 {
 self.writes.fetch_add(1, Ordering::AcqRel);
 
 if let Some(bh) = self.get_buffer(dev, block, buf.len() as u32) {
 bh.write(0, buf) as i64
 } else {
 -1
 }
 }
 
 /// SynchronousdirtyBuffer
 pub fn sync(&mut self) -> u32 {
 let count = self.dirty_count.swap(0, Ordering::AcqRel);
 // TODO: WritebackplacefinitedirtyBuffer
 count
 }
 
 /// GetBuffercount
 pub fn get_buffer_count(&self) -> u32 {
 self.buffer_count.load(Ordering::Acquire)
 }
 
 /// GetdirtyBuffercount
 pub fn get_dirty_count(&self) -> u32 {
 self.dirty_count.load(Ordering::Acquire)
 }
 
 /// Getinfixrate
 pub fn get_hit_rate(&self) -> u32 {
 let hits = self.hits.load(Ordering::Acquire);
 let misses = self.misses.load(Ordering::Acquire);
 let total = hits + misses;
 
 if total == 0 {
 return 0;
 }
 
 ((hits * 100) / total) as u32
 }
}

/// GlobalBufferCaching
static BUFFER_CACHE: crate::sync_oncelock::OnceLock<BufferCache> = crate::sync_oncelock::OnceLock::new();

pub fn buffer_cache() -> &'static BufferCache {
    BUFFER_CACHE.get_or_init(BufferCache::new)
}

pub fn init_buffer_cache() {
 let bc = buffer_cache();
 bc.init();
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_buffer_constants() {
 assert_eq!(BUFFER_SIZE, 4096);
 assert_eq!(MAX_BUFFERS, 1024);
 }

 #[test]
 fn test_buffer_flags() {
 assert_eq!(buffer_flags::DIRTY, 1 << 0);
 assert_eq!(buffer_flags::VALID, 1 << 1);
 assert_eq!(buffer_flags::LOCKED, 1 << 2);
 assert_eq!(buffer_flags::WRITEBACK, 1 << 3);
 assert_eq!(buffer_flags::ERROR, 1 << 4);
 assert_eq!(buffer_flags::UPTODATE, 1 << 5);
 assert_eq!(buffer_flags::SYNC, 1 << 6);
 }

 #[test]
 fn test_buffer_head_new() {
 let bh = BufferHead::new(1, 100, 4096);

 assert_eq!(bh.dev, 1);
 assert_eq!(bh.block, 100);
 assert_eq!(bh.size, 4096);
 assert_eq!(bh.get_ref_count(), 0);
 assert!(!bh.is_dirty());
 assert!(!bh.is_valid());
 assert!(!bh.is_locked());
 }

 #[test]
 fn test_buffer_head_dirty() {
 let bh = BufferHead::new(1, 0, 4096);

 assert!(!bh.is_dirty());

 bh.set_dirty();
 assert!(bh.is_dirty());

 bh.clear_dirty();
 assert!(!bh.is_dirty());
 }

 #[test]
 fn test_buffer_head_valid() {
 let bh = BufferHead::new(1, 0, 4096);

 assert!(!bh.is_valid());

 bh.set_valid();
 assert!(bh.is_valid());
 }

 #[test]
 fn test_buffer_head_lock() {
 let bh = BufferHead::new(1, 0, 4096);

 assert!(!bh.is_locked());

 bh.lock();
 assert!(bh.is_locked());

 bh.unlock();
 assert!(!bh.is_locked());
 }

 #[test]
 fn test_buffer_head_ref_count() {
 let bh = BufferHead::new(1, 0, 4096);

 assert_eq!(bh.get_ref_count(), 0);

 bh.get();
 assert_eq!(bh.get_ref_count(), 1);

 bh.get();
 assert_eq!(bh.get_ref_count(), 2);

 bh.put();
 assert_eq!(bh.get_ref_count(), 1);
 }

 #[test]
 fn test_buffer_head_read() {
 let mut bh = BufferHead::new(1, 0, 4096);

 // WriteasomeData
 bh.data[0] = 1;
 bh.data[1] = 2;
 bh.data[2] = 3;
 bh.data[3] = 4;

 let mut buf = [0u8; 4];
 let len = bh.read(0, &mut buf);

 assert_eq!(len, 4);
 assert_eq!(buf, [1, 2, 3, 4]);
 }

 #[test]
 fn test_buffer_head_read_with_offset() {
 let mut bh = BufferHead::new(1, 0, 4096);

 bh.data[10] = 100;
 bh.data[11] = 101;

 let mut buf = [0u8; 2];
 let len = bh.read(10, &mut buf);

 assert_eq!(len, 2);
 assert_eq!(buf, [100, 101]);
 }

 #[test]
 fn test_buffer_head_write() {
 let mut bh = BufferHead::new(1, 0, 4096);

 let data = [10, 20, 30, 40];
 let len = bh.write(0, &data);

 assert_eq!(len, 4);
 assert_eq!(bh.data[0], 10);
 assert_eq!(bh.data[1], 20);
 assert_eq!(bh.data[2], 30);
 assert_eq!(bh.data[3], 40);
 assert!(bh.is_dirty());
 }

 #[test]
 fn test_buffer_head_write_with_offset() {
 let mut bh = BufferHead::new(1, 0, 4096);

 let data = [50, 60];
 let len = bh.write(100, &data);

 assert_eq!(len, 2);
 assert_eq!(bh.data[100], 50);
 assert_eq!(bh.data[101], 60);
 }

 #[test]
 fn test_buffer_cache_new() {
 let bc = BufferCache::new();

 assert_eq!(bc.get_buffer_count(), 0);
 assert_eq!(bc.get_dirty_count(), 0);
 assert_eq!(bc.get_hit_rate(), 0);
 assert_eq!(bc.max_buffers, MAX_BUFFERS as u32);
 }

 #[test]
 fn test_buffer_cache_find_buffer() {
 let bc = BufferCache::new();

 // CurrentImplementationtotalisReturn None
 let result = bc.find_buffer(1, 0);
 assert!(result.is_none());
 assert_eq!(bc.misses.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_buffer_cache_sync() {
 let mut bc = BufferCache::new();

 // SetasomedirtyBuffer
 bc.dirty_count.store(5, Ordering::Relaxed);

 let count = bc.sync();
 assert_eq!(count, 5);
 assert_eq!(bc.get_dirty_count(), 0);
 }

 #[test]
 fn test_buffer_cache_read_block() {
 let mut bc = BufferCache::new();

 let mut buf = [0u8; 512];
 let result = bc.read_block(1, 0, &mut buf);

 // CurrentImplementationReturn -1(infinitelawAllocateBuffer)
 assert_eq!(result, -1);
 assert_eq!(bc.reads.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_buffer_cache_write_block() {
 let mut bc = BufferCache::new();

 let buf = [0u8; 512];
 let result = bc.write_block(1, 0, &buf);

 // CurrentImplementationreturn Errno::Eperm.to_ret_i32()
 assert_eq!(result, -1);
 assert_eq!(bc.writes.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_buffer_cache_hit_rate() {
 let bc = BufferCache::new();

 // infiniteaccesstimeinfixrateas 0
 assert_eq!(bc.get_hit_rate(), 0);

 // modelsimulatedasomeinfixsuminfix
 bc.hits.store(80, Ordering::Relaxed);
 bc.misses.store(20, Ordering::Relaxed);

 assert_eq!(bc.get_hit_rate(), 80);
 }

 #[test]
 fn test_buffer_cache_max_buffers() {
 let mut bc = BufferCache::new();

 // reachtoMaxBuffernumbertimeinfinitelawAllocate
 bc.buffer_count.store(bc.max_buffers, Ordering::Relaxed);

 let result = bc.get_buffer(1, 0, 4096);
 assert!(result.is_none());
 }

 #[test]
 fn test_buffer_head_data_size() {
 let bh = BufferHead::new(1, 0, 512);

 // size canwithLess Than BUFFER_SIZE
 assert_eq!(bh.size, 512);
 }
}