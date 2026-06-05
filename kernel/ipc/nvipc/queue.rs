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

//! Nuva IPC Message QueueImplementation
/*!*/
// ! SupportPriority Message Queue.

use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::MachMessage;

/// QueuePriority
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueuePriority {
 /// thenPriority
 Background = 0,
 /// lowPriority
 Low = 1,
 /// DefaultPriority
 Default = 2,
 /// highPriority
 High = 3,
}

impl Default for QueuePriority {
 fn default() -> Self {
 Self::Default
 }
}

/// PriorityMessage Queue
pub struct MessageQueue {
 /// highPriorityMessage Queue
 high: VecDeque<MachMessage>,
 /// DefaultPriorityMessage Queue
 default: VecDeque<MachMessage>,
 /// lowPriorityMessage Queue
 low: VecDeque<MachMessage>,
 /// thenPriorityMessage Queue
 background: VecDeque<MachMessage>,
 /// totalMessagenumber
 count: AtomicUsize,
 /// Maxquantification
 capacity: usize,
}

impl MessageQueue {
 /// CreatenewQueue
 pub fn new() -> Self {
 Self::with_capacity(1024)
 }

 /// Createbandquantification Queue
 pub fn with_capacity(capacity: usize) -> Self {
 Self {
 high: VecDeque::new(),
 default: VecDeque::new(),
 low: VecDeque::new(),
 background: VecDeque::new(),
 count: AtomicUsize::new(0),
 capacity,
 }
 }

 /// enterqueueMessage
 pub fn enqueue(&mut self, message: MachMessage, priority: QueuePriority) -> bool {
 if self.count.load(Ordering::Acquire) >= self.capacity {
 return false;
 }

 match priority {
 QueuePriority::High => self.high.push_back(message),
 QueuePriority::Default => self.default.push_back(message),
 QueuePriority::Low => self.low.push_back(message),
 QueuePriority::Background => self.background.push_back(message),
 }
 
 self.count.fetch_add(1, Ordering::AcqRel);
 true
 }

 /// exitqueueMessage (byPriority)
 pub fn dequeue(&mut self) -> Option<MachMessage> {
 // byPrioritySequentialexitqueue
 if let Some(msg) = self.high.pop_front() {
 self.count.fetch_sub(1, Ordering::AcqRel);
 return Some(msg);
 }
 
 if let Some(msg) = self.default.pop_front() {
 self.count.fetch_sub(1, Ordering::AcqRel);
 return Some(msg);
 }
 
 if let Some(msg) = self.low.pop_front() {
 self.count.fetch_sub(1, Ordering::AcqRel);
 return Some(msg);
 }
 
 if let Some(msg) = self.background.pop_front() {
 self.count.fetch_sub(1, Ordering::AcqRel);
 return Some(msg);
 }
 
 None
 }

 /// inspectionqueuefirstMessage (notDivide)
 pub fn peek(&self) -> Option<&MachMessage> {
 self.high.front()
 .or_else(|| self.default.front())
 .or_else(|| self.low.front())
 .or_else(|| self.background.front())
 }

 /// GetQueueLength
 pub fn len(&self) -> usize {
 self.count.load(Ordering::Acquire)
 }

 /// Check if empty
 pub fn is_empty(&self) -> bool {
 self.len() == 0
 }

 /// Checkifalreadysatisfy
 pub fn is_full(&self) -> bool {
 self.len() >= self.capacity
 }

 /// GetexpfixedPriority QueueLength
 pub fn len_at(&self, priority: QueuePriority) -> usize {
 match priority {
 QueuePriority::High => self.high.len(),
 QueuePriority::Default => self.default.len(),
 QueuePriority::Low => self.low.len(),
 QueuePriority::Background => self.background.len(),
 }
 }

 /// ClearQueue
 pub fn clear(&mut self) {
 self.high.clear();
 self.default.clear();
 self.low.clear();
 self.background.clear();
 self.count.store(0, Ordering::Release);
 }

 /// Getquantification
 pub fn capacity(&self) -> usize {
 self.capacity
 }

 /// Setquantification
 pub fn set_capacity(&mut self, capacity: usize) {
 self.capacity = capacity;
 }
}

impl Default for MessageQueue {
 fn default() -> Self {
 Self::new()
 }
}

/// QueuestatisticsInfo
#[derive(Debug, Clone, Copy, Default)]
pub struct QueueStats {
 /// highPriorityMessagenumber
 pub high_count: usize,
 /// DefaultPriorityMessagenumber
 pub default_count: usize,
 /// lowPriorityMessagenumber
 pub low_count: usize,
 /// thenPriorityMessagenumber
 pub background_count: usize,
 /// totalMessagenumber
 pub total: usize,
 /// quantification
 pub capacity: usize,
}

impl MessageQueue {
 /// Get statistics
 pub fn stats(&self) -> QueueStats {
 QueueStats {
 high_count: self.high.len(),
 default_count: self.default.len(),
 low_count: self.low.len(),
 background_count: self.background.len(),
 total: self.len(),
 capacity: self.capacity,
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_queue_basic() {
 let mut queue = MessageQueue::new();
 let msg = MachMessage::new_small(b"test");
 
 assert!(queue.enqueue(msg, QueuePriority::Default));
 assert_eq!(queue.len(), 1);
 
 let dequeued = queue.dequeue();
 assert!(dequeued.is_some());
 assert_eq!(queue.len(), 0);
 }

 #[test]
 fn test_queue_priority() {
 let mut queue = MessageQueue::new();
 
 let low_msg = MachMessage::new_small(b"low");
 let high_msg = MachMessage::new_small(b"high");
 let default_msg = MachMessage::new_small(b"default");
 
 queue.enqueue(low_msg, QueuePriority::Low);
 queue.enqueue(high_msg, QueuePriority::High);
 queue.enqueue(default_msg, QueuePriority::Default);
 
 // highPriorityshouldthefirstexitqueue
 let first = queue.dequeue().unwrap();
 assert_eq!(first.data(), b"high");
 
 let second = queue.dequeue().unwrap();
 assert_eq!(second.data(), b"default");
 
 let third = queue.dequeue().unwrap();
 assert_eq!(third.data(), b"low");
 }

 #[test]
 fn test_queue_capacity() {
 let mut queue = MessageQueue::with_capacity(2);
 
 assert!(queue.enqueue(MachMessage::new_small(b"1"), QueuePriority::Default));
 assert!(queue.enqueue(MachMessage::new_small(b"2"), QueuePriority::Default));
 assert!(!queue.enqueue(MachMessage::new_small(b"3"), QueuePriority::Default)); // shouldFailure
 }
}