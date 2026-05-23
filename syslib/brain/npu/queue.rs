/*
 * Nuva OS - SystemLibrary - Brain
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


use super::scheduler::{NpuTask, TaskPriority};

/// Task QueueNode
pub struct TaskNode {
 /// Task
 pub task: Option<NpuTask>,
 /// NextNode
 pub next: u32,
}

/// PriorityQueue
pub struct PriorityQueue {
 /// Nodepool
 nodes: [TaskNode; 128],
 /// emptyidlelinkformHead
 free_head: u32,
 /// PriorityQueueHead
 queue_heads: [u32; 4],
 /// PriorityQueueTail
 queue_tails: [u32; 4],
 /// QueueLength
 lengths: [u32; 4],
}

impl PriorityQueue {
 pub const fn new() -> Self {
 PriorityQueue {
 nodes: [TaskNode { task: None, next: 0 }; 128],
 free_head: 0,
 queue_heads: [0; 4],
 queue_tails: [0; 4],
 lengths: [0; 4],
 }
 }
 
 /// Initialize
 pub fn init(&mut self) {
 // Initializeemptyidlelinkform
 for i in 0..127 {
 self.nodes[i].next = (i + 1) as u32;
 }
 self.nodes[127].next = 0;
 self.free_head = 1; // secondary 1 Start,0 formempty
 }
 
 /// AllocateNode
 fn alloc_node(&mut self) -> Option<u32> {
 if self.free_head == 0 {
 return None;
 }
 
 let node_id = self.free_head;
 self.free_head = self.nodes[node_id as usize].next;
 
 Some(node_id)
 }
 
 /// FreeNode
 fn free_node(&mut self, node_id: u32) {
 self.nodes[node_id as usize].next = self.free_head;
 self.free_head = node_id;
 }
 
 /// enterqueue
 pub fn enqueue(&mut self, task: NpuTask) -> bool {
 // AllocateNode
 let node_id = match self.alloc_node() {
 Some(id) => id,
 None => return false,
 };
 
 // SetTask
 self.nodes[node_id as usize].task = Some(task);
 self.nodes[node_id as usize].next = 0;
 
 // GetPriorityIndex
 let priority_idx = task.priority as usize;
 
 // PlusenterQueueTailpart
 if self.queue_tails[priority_idx] == 0 {
 self.queue_heads[priority_idx] = node_id;
 } else {
 self.nodes[self.queue_tails[priority_idx] as usize].next = node_id;
 }
 self.queue_tails[priority_idx] = node_id;
 
 self.lengths[priority_idx] += 1;
 
 true
 }
 
 /// exitqueue (mosthighPriority)
 pub fn dequeue(&mut self) -> Option<NpuTask> {
 // secondaryhightolowCheckPriorityQueue
 for priority_idx in (0..4).rev() {
 if self.queue_heads[priority_idx] != 0 {
 let node_id = self.queue_heads[priority_idx];
 let task = self.nodes[node_id as usize].task.take();
 
 // UpdateQueueHead
 self.queue_heads[priority_idx] = self.nodes[node_id as usize].next;
 if self.queue_heads[priority_idx] == 0 {
 self.queue_tails[priority_idx] = 0;
 }
 
 // FreeNode
 self.free_node(node_id);
 
 self.lengths[priority_idx] -= 1;
 
 return task;
 }
 }
 
 None
 }
 
 /// inspectionqueuefirst (notDivide)
 pub fn peek(&self) -> Option<&NpuTask> {
 for priority_idx in (0..4).rev() {
 if self.queue_heads[priority_idx] != 0 {
 let node_id = self.queue_heads[priority_idx];
 return self.nodes[node_id as usize].task.as_ref();
 }
 }
 None
 }
 
 /// GetQueueLength
 pub fn len(&self) -> u32 {
 self.lengths.iter().sum()
 }
 
 /// Check if empty
 pub fn is_empty(&self) -> bool {
 self.len() == 0
 }
 
 /// GetexpfixedPriority QueueLength
 pub fn len_by_priority(&self, priority: TaskPriority) -> u32 {
 self.lengths[priority as usize]
 }
}

/// GlobalPriorityQueue
static mut PRIORITY_QUEUE: PriorityQueue = PriorityQueue::new();

pub fn get_priority_queue() -> &'static mut PriorityQueue {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut PRIORITY_QUEUE }
}

pub fn init_priority_queue() {
 let queue = get_priority_queue();
 queue.init();
}