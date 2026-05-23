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

/// WaitQueueNode
pub struct WaitQueueNode {
 /// Waiter ID (Thread/Process ID)
 pub waiter_id: u32,
 /// WaitFlag
 pub flags: AtomicU32,
 /// NextNode
 pub next: *mut WaitQueueNode,
}

/// WaitFlag
pub mod wait_flags {
 pub const EXCLUSIVE: u32 = 1 << 0; // exclusiveWait
 pub const WOKEN: u32 = 1 << 1; // alreadyWake
 pub const INTERRUPTIBLE: u32 = 1 << 2; // canInterrupt
}

impl WaitQueueNode {
 /// Create new WaitNode
 pub fn new(waiter_id: u32) -> Self {
 WaitQueueNode {
 waiter_id,
 flags: AtomicU32::new(wait_flags::EXCLUSIVE),
 next: core::ptr::null_mut(),
 }
 }
 
 /// ifexclusive
 pub fn is_exclusive(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & wait_flags::EXCLUSIVE) != 0
 }
 
 /// ifalreadyWake
 pub fn is_woken(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & wait_flags::WOKEN) != 0
 }
 
 /// SetWake
 pub fn set_woken(&self) {
 self.flags.fetch_or(wait_flags::WOKEN, Ordering::AcqRel);
 }
 
 /// ifcanInterrupt
 pub fn is_interruptible(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & wait_flags::INTERRUPTIBLE) != 0
 }
}

/// WaitQueue
pub struct WaitQueue {
 /// QueueHead
 head: *mut WaitQueueNode,
 /// QueueTail
 tail: *mut WaitQueueNode,
 /// Waitercount
 count: AtomicU32,
 /// Lock
 lock: AtomicU32,
}

impl WaitQueue {
 /// Create new WaitQueue
 pub const fn new() -> Self {
 WaitQueue {
 head: core::ptr::null_mut(),
 tail: core::ptr::null_mut(),
 count: AtomicU32::new(0),
 lock: AtomicU32::new(0),
 }
 }
 
 /// GetLock
 fn acquire(&self) {
 while self.lock.compare_exchange_weak(
 0, 1, Ordering::Acquire, Ordering::Relaxed
 ).is_err() {
 core::hint::spin_loop();
 }
 }
 
 /// FreeLock
 fn release(&self) {
 self.lock.store(0, Ordering::Release);
 }
 
 /// addPlusWaiter
 pub fn add(&mut self, node: &mut WaitQueueNode) {
 self.acquire();
 
 node.next = core::ptr::null_mut();
 
 if self.tail.is_null() {
 self.head = node;
 self.tail = node;
 } else {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 (*self.tail).next = node;
 }
 self.tail = node;
 }
 
 self.count.fetch_add(1, Ordering::AcqRel);
 self.release();
 }
 
 /// DivideWaiter
 pub fn remove(&mut self, node: &mut WaitQueueNode) {
 self.acquire();
 
 // FindparallelDivide
 let mut prev: *mut WaitQueueNode = core::ptr::null_mut();
 let mut current = self.head;
 
 while !current.is_null() {
 if current == node as *mut WaitQueueNode {
 // findto,Divide
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 if prev.is_null() {
 self.head = (*current).next;
 } else {
 (*prev).next = (*current).next;
 }
 
 if self.tail == current {
 self.tail = prev;
 }
 }
 
 self.count.fetch_sub(1, Ordering::AcqRel);
 break;
 }
 
 prev = current;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 current = (*current).next;
 }
 }
 
 self.release();
 }
 
 /// WakeaitemWaiter
 pub fn wake_one(&mut self) -> Option<u32> {
 self.acquire();
 
 if self.head.is_null() {
 self.release();
 return None;
 }
 
 let node = self.head;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 self.head = (*node).next;
 if self.head.is_null() {
 self.tail = core::ptr::null_mut();
 }
 
 (*node).set_woken();
 let waiter_id = (*node).waiter_id;
 
 self.count.fetch_sub(1, Ordering::AcqRel);
 self.release();
 
 return Some(waiter_id);
 }
 }
 
 /// WakeplacefiniteWaiter
 pub fn wake_all(&mut self) -> u32 {
 self.acquire();
 
 let mut count = 0u32;
 
 while !self.head.is_null() {
 let node = self.head;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 self.head = (*node).next;
 (*node).set_woken();
 count += 1;
 }
 }
 
 self.tail = core::ptr::null_mut();
 self.count.store(0, Ordering::Release);
 
 self.release();
 count
 }
 
 /// GetWaitercount
 pub fn get_count(&self) -> u32 {
 self.count.load(Ordering::Acquire)
 }
 
 /// ifasempty
 pub fn is_empty(&self) -> bool {
 self.count.load(Ordering::Acquire) == 0
 }
}

/// WaitQueueHead (Simplified)
pub struct WaitQueueHead {
 /// Waitercount
 pub count: AtomicU32,
 /// Waketimenumber
 pub wakeups: AtomicU64,
}

impl WaitQueueHead {
 pub const fn new() -> Self {
 WaitQueueHead {
 count: AtomicU32::new(0),
 wakeups: AtomicU64::new(0),
 }
 }
 
 /// Wait
 pub fn wait(&self) {
 self.count.fetch_add(1, Ordering::AcqRel);
 // TODO: realactual
 }
 
 /// Wake
 pub fn wake(&self) {
 if self.count.load(Ordering::Acquire) > 0 {
 self.count.fetch_sub(1, Ordering::AcqRel);
 self.wakeups.fetch_add(1, Ordering::AcqRel);
 }
 }
 
 /// Wakeall
 pub fn wake_all(&self) {
 let count = self.count.swap(0, Ordering::AcqRel);
 self.wakeups.fetch_add(count as u64, Ordering::AcqRel);
 }
}

/// Completion
pub struct Completion {
 /// completeFlag
 done: AtomicU32,
 /// WaitQueue
 wait: WaitQueueHead,
}

impl Completion {
 pub const fn new() -> Self {
 Completion {
 done: AtomicU32::new(0),
 wait: WaitQueueHead::new(),
 }
 }
 
 /// Waitcomplete
 pub fn wait(&self) {
 while self.done.load(Ordering::Acquire) == 0 {
 self.wait.wait();
 }
 }
 
 /// complete
 pub fn complete(&self) {
 self.done.store(1, Ordering::Release);
 self.wait.wake_all();
 }
 
 /// ifalreadyComplete
 pub fn is_done(&self) -> bool {
 self.done.load(Ordering::Acquire) != 0
 }
 
 /// Reset
 pub fn reset(&self) {
 self.done.store(0, Ordering::Release);
 }
}

/// InitializeWaitQueue
pub fn init_wait() {
 log_info!("Wait queue initialized");
}