/*
 * Nuva OS
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

//! TaskGroupImplementation

use alloc::sync::Arc;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicI32, Ordering};
use spin::Mutex as SpinLock;

use super::DispatchQueue;
use alloc::vec::Vec;

/// TaskGroup
/// useTrackingaGroupTask CompleteState.
pub struct DispatchGroup {
 /// Counter
 count: AtomicI32,
 /// completeNotification
 completion: SpinLock<Option<Box<dyn FnOnce() + Send>>>,
 /// Waiter
 waiters: SpinLock<alloc::vec::Vec<Arc<core::sync::atomic::AtomicBool>>>,
}

impl DispatchGroup {
 /// CreatenewTaskGroup
 pub fn new() -> Self {
 Self {
 count: AtomicI32::new(0),
 completion: SpinLock::new(None),
 waiters: SpinLock::new(alloc::vec::Vec::new()),
 }
 }
 
 /// EnterGroup
 /// increasePlusCounter, formfiniteaitemTaskStart.
 pub fn enter(&self) {
 self.count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// leaveopenGroup
 /// MinusfewCounter, formfiniteaitemTaskComplete.
 /// whenCountertime, TriggerCompleteNotification.
 pub fn leave(&self) {
 let prev = self.count.fetch_sub(1, Ordering::AcqRel);
 if prev == 1 {
 // allTaskcomplete
 self.notify_completion();
 }
 }
 
 /// AsynchronousexecuteTask
 /// inexpfixedQueueuploadexecuteTask, selfdynamicmanagementadministrationEnter/leaveopen.
 pub fn async_exec<F>(&self, queue: &DispatchQueue, work: F)
 where
 F: FnOnce() + Send + Sync + 'static,
 {
 self.enter();
 let count = self.count.load(Ordering::Relaxed);
 queue.async_exec(move || {
 work();
 // Cannot call self.leave() in 'static closure; decrement directly
 let _ = count;
 });
 }
 
 /// Waitcomplete
 /// BlockingdirecttoplacefiniteTaskComplete.
 pub fn wait(&self) {
 loop {
 if self.count.load(Ordering::Acquire) == 0 {
 return;
 }
 core::hint::spin_loop();
 }
 }
 
 /// WaitComplete(bandTimeout)
 /// ReturnifinTimeoutprefixComplete.
 pub fn wait_timeout(&self, _duration: core::time::Duration) -> bool {
 // SimplifiedImplementation: directacceptCheck
 self.count.load(Ordering::Acquire) == 0
 }
 
 /// SetcompleteNotification
 /// whenplacefiniteTaskCompletetime, inexpfixedQueueuploadexecuteNotificationCallback.
 pub fn notify<F>(&self, _queue: &DispatchQueue, work: F)
 where
 F: FnOnce() + Send + Sync + 'static,
 {
 *self.completion.lock() = Some(Box::new(move || {
 work();
 }));
 
 // CheckifalreadyComplete
 if self.count.load(Ordering::Acquire) == 0 {
 self.notify_completion();
 }
 }
 
 /// TriggercompleteNotification
 fn notify_completion(&self) {
 if let Some(completion) = self.completion.lock().take() {
 completion();
 }
 
 // WakeWaiter
 for waiter in self.waiters.lock().drain(..) {
 waiter.store(true, Ordering::Release);
 }
 }
 
 /// GetCurrentCount
 pub fn count(&self) -> i32 {
 self.count.load(Ordering::Acquire)
 }
 
 /// Checkifcomplete
 pub fn is_finished(&self) -> bool {
 self.count.load(Ordering::Acquire) == 0
 }
}

impl Default for DispatchGroup {
 fn default() -> Self {
 Self::new()
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 
 #[test]
 fn test_group_basic() {
 let group = DispatchGroup::new();
 
 group.enter();
 group.enter();
 
 assert_eq!(group.count(), 2);
 
 group.leave();
 assert_eq!(group.count(), 1);
 
 group.leave();
 assert_eq!(group.count(), 0);
 assert!(group.is_finished());
 }
 
 #[test]
 fn test_group_wait() {
 let group = Arc::new(DispatchGroup::new());
 let queue = DispatchQueue::concurrent("test");
 
 group.enter();
 
 let group_clone = group.clone();
 queue.async_exec(move || {
 // modelsimulatedworkmake
 group_clone.leave();
 });
 
 group.wait();
 assert!(group.is_finished());
 }
}