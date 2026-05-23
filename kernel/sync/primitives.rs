use crate::{pr_info};
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


use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Spinlock
pub struct SpinLock {
 /// LockState
 locked: AtomicBool,
}

impl SpinLock {
 /// Create new Spinlock
 pub const fn new() -> Self {
 SpinLock {
 locked: AtomicBool::new(false),
 }
 }
 
 /// GetLock
 pub fn lock(&self) {
 while self.locked.compare_exchange_weak(
 false,
 true,
 Ordering::Acquire,
 Ordering::Relaxed,
 ).is_err() {
 // SpinWait
 while self.locked.load(Ordering::Relaxed) {
 core::hint::spin_loop();
 }
 }
 }
 
 /// tryGetLock
 pub fn try_lock(&self) -> bool {
 self.locked.compare_exchange_weak(
 false,
 true,
 Ordering::Acquire,
 Ordering::Relaxed,
 ).is_ok()
 }
 
 /// FreeLock
 pub fn unlock(&self) {
 self.locked.store(false, Ordering::Release);
 }
 
 /// ifLocked
 pub fn is_locked(&self) -> bool {
 self.locked.load(Ordering::Acquire)
 }
}

/// Spinlockguard
pub struct SpinLockGuard<'a> {
 lock: &'a SpinLock,
}

impl<'a> SpinLockGuard<'a> {
 pub fn new(lock: &'a SpinLock) -> Self {
 lock.lock();
 SpinLockGuard { lock }
 }
}

impl<'a> Drop for SpinLockGuard<'a> {
 fn drop(&mut self) {
 self.lock.unlock();
 }
}

/// MutexState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutexState {
 /// Lockfixed
 Unlocked = 0,
 /// Locked
 Locked = 1,
 /// finiteWaiter
 Contended = 2,
}

/// Mutex
pub struct Mutex {
 /// LockState
 state: AtomicU32,
 /// finiteer (Thread ID)
 owner: AtomicU32,
 /// Lockfixedtimenumber (useRecursionLock)
 count: AtomicU32,
}

impl Mutex {
 /// Create new Mutex
 pub const fn new() -> Self {
 Mutex {
 state: AtomicU32::new(MutexState::Unlocked as u32),
 owner: AtomicU32::new(0),
 count: AtomicU32::new(0),
 }
 }
 
 /// GetLock
 pub fn lock(&self, tid: u32) {
 // tryfastGet
 if self.try_lock(tid) {
 return;
 }
 
 // slowPath
 self.lock_slow(tid);
 }
 
 /// tryGetLock
 pub fn try_lock(&self, tid: u32) -> bool {
 // Checkifalreadyfinite (RecursionLock)
 if self.owner.load(Ordering::Acquire) == tid {
 self.count.fetch_add(1, Ordering::AcqRel);
 return true;
 }
 
 // tryGetLock
 if self.state.compare_exchange_weak(
 MutexState::Unlocked as u32,
 MutexState::Locked as u32,
 Ordering::Acquire,
 Ordering::Relaxed,
 ).is_ok() {
 self.owner.store(tid, Ordering::Release);
 self.count.store(1, Ordering::Release);
 return true;
 }
 
 false
 }
 
 /// slowGetLock
 fn lock_slow(&self, tid: u32) {
 // SetraceState
 self.state.store(MutexState::Contended as u32, Ordering::Release);
 
 // WaitLock
 loop {
 // WaitStatechangeasLockfixed
 while self.state.load(Ordering::Acquire) != MutexState::Unlocked as u32 {
 core::hint::spin_loop();
 }
 
 // tryGet
 if self.try_lock(tid) {
 return;
 }
 }
 }
 
 /// FreeLock
 pub fn unlock(&self, tid: u32) {
 // Checkfiniteer
 if self.owner.load(Ordering::Acquire) != tid {
 return; // notisfiniteer
 }
 
 // MinusfewCount
 let count = self.count.fetch_sub(1, Ordering::AcqRel);
 if count > 1 {
 return; // stillfiniteRecursionLockfixed
 }
 
 // clearDividefiniteer
 self.owner.store(0, Ordering::Release);
 
 // FreeLock
 self.state.store(MutexState::Unlocked as u32, Ordering::Release);
 }
 
 /// ifLocked
 pub fn is_locked(&self) -> bool {
 self.state.load(Ordering::Acquire) != MutexState::Unlocked as u32
 }
 
 /// Getfiniteer
 pub fn get_owner(&self) -> u32 {
 self.owner.load(Ordering::Acquire)
 }
}

/// Semaphore
pub struct Semaphore {
 /// Currentvalue
 value: AtomicU32,
 /// Maxvalue
 max: u32,
}

impl Semaphore {
 /// Create new Semaphore
 pub const fn new(initial: u32, max: u32) -> Self {
 Semaphore {
 value: AtomicU32::new(initial),
 max,
 }
 }
 
 /// CreatevalueSemaphore
 pub const fn binary() -> Self {
 Self::new(1, 1)
 }
 
 /// Wait (P Operation)
 pub fn wait(&self) -> bool {
 loop {
 let current = self.value.load(Ordering::Acquire);
 
 if current == 0 {
 // Wait
 core::hint::spin_loop();
 continue;
 }
 
 if self.value.compare_exchange_weak(
 current,
 current - 1,
 Ordering::AcqRel,
 Ordering::Relaxed,
 ).is_ok() {
 return true;
 }
 }
 }
 
 /// tryWait
 pub fn try_wait(&self) -> bool {
 loop {
 let current = self.value.load(Ordering::Acquire);
 
 if current == 0 {
 return false;
 }
 
 if self.value.compare_exchange_weak(
 current,
 current - 1,
 Ordering::AcqRel,
 Ordering::Relaxed,
 ).is_ok() {
 return true;
 }
 }
 }
 
 /// Free (V Operation)
 pub fn post(&self) -> bool {
 loop {
 let current = self.value.load(Ordering::Acquire);
 
 if current >= self.max {
 return false; // alreadyreachMaxvalue
 }
 
 if self.value.compare_exchange_weak(
 current,
 current + 1,
 Ordering::AcqRel,
 Ordering::Relaxed,
 ).is_ok() {
 return true;
 }
 }
 }
 
 /// GetCurrentvalue
 pub fn get_value(&self) -> u32 {
 self.value.load(Ordering::Acquire)
 }
}

/// Read-Write Lock
pub struct RwLock {
 /// State: high 32 BitaswriteLock, low 32 BitasreadLockCount
 state: AtomicU64,
 /// writeLockfiniteer
 writer: AtomicU32,
}

impl RwLock {
 /// Create new Read-Write Lock
 pub const fn new() -> Self {
 RwLock {
 state: AtomicU64::new(0),
 writer: AtomicU32::new(0),
 }
 }
 
 /// GetreadLock
 pub fn read_lock(&self) {
 loop {
 let state = self.state.load(Ordering::Acquire);
 let writer = (state >> 32) as u32;
 let readers = state as u32;
 
 // iffinitewriteLock,Wait
 if writer != 0 {
 core::hint::spin_loop();
 continue;
 }
 
 // tryincreasePlusreadCount
 let new_state = (readers + 1) as u64;
 if self.state.compare_exchange_weak(
 state,
 new_state,
 Ordering::AcqRel,
 Ordering::Relaxed,
 ).is_ok() {
 return;
 }
 }
 }
 
 /// tryGetreadLock
 pub fn try_read_lock(&self) -> bool {
 let state = self.state.load(Ordering::Acquire);
 let writer = (state >> 32) as u32;
 let readers = state as u32;
 
 if writer != 0 {
 return false;
 }
 
 let new_state = (readers + 1) as u64;
 self.state.compare_exchange_weak(
 state,
 new_state,
 Ordering::AcqRel,
 Ordering::Relaxed,
 ).is_ok()
 }
 
 /// FreereadLock
 pub fn read_unlock(&self) {
 loop {
 let state = self.state.load(Ordering::Acquire);
 let readers = state as u32;
 
 if readers == 0 {
 return; // finitereadLock
 }
 
 let new_state = ((state >> 32) << 32) | ((readers - 1) as u64);
 if self.state.compare_exchange_weak(
 state,
 new_state,
 Ordering::AcqRel,
 Ordering::Relaxed,
 ).is_ok() {
 return;
 }
 }
 }
 
 /// GetwriteLock
 pub fn write_lock(&self, tid: u32) {
 loop {
 let state = self.state.load(Ordering::Acquire);
 let writer = (state >> 32) as u32;
 let readers = state as u32;
 
 // iffiniteOtherLock,Wait
 if writer != 0 || readers != 0 {
 core::hint::spin_loop();
 continue;
 }
 
 // trySetwriteLock
 let new_state = (1u64 << 32);
 if self.state.compare_exchange_weak(
 state,
 new_state,
 Ordering::AcqRel,
 Ordering::Relaxed,
 ).is_ok() {
 self.writer.store(tid, Ordering::Release);
 return;
 }
 }
 }
 
 /// tryGetwriteLock
 pub fn try_write_lock(&self, tid: u32) -> bool {
 let state = self.state.load(Ordering::Acquire);
 let writer = (state >> 32) as u32;
 let readers = state as u32;
 
 if writer != 0 || readers != 0 {
 return false;
 }
 
 let new_state = (1u64 << 32);
 if self.state.compare_exchange_weak(
 state,
 new_state,
 Ordering::AcqRel,
 Ordering::Relaxed,
 ).is_ok() {
 self.writer.store(tid, Ordering::Release);
 return true;
 }
 
 false
 }
 
 /// FreewriteLock
 pub fn write_unlock(&self, tid: u32) {
 if self.writer.load(Ordering::Acquire) != tid {
 return; // notisfiniteer
 }
 
 self.writer.store(0, Ordering::Release);
 self.state.store(0, Ordering::Release);
 }
 
 /// iffinitereadLock
 pub fn has_readers(&self) -> bool {
 let state = self.state.load(Ordering::Acquire);
 (state as u32) != 0
 }
 
 /// iffinitewriteLock
 pub fn has_writer(&self) -> bool {
 let state = self.state.load(Ordering::Acquire);
 ((state >> 32) as u32) != 0
 }
 
 /// GetreadLockcount
 pub fn get_reader_count(&self) -> u32 {
 let state = self.state.load(Ordering::Acquire);
 state as u32
 }
}

/// InitializeSynchronoussourcelanguage
pub fn init_sync() {
 log_info!("Synchronization primitives initialized");
}