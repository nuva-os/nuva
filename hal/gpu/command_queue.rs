/*
 * Nuva OS - HAL - Gpu
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
use super::{GpuCommand, GpuCommandType};

/// Command queue state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueState {
    /// Idle
    Idle = 0,
    /// Running
    Running = 1,
    /// Paused
    Paused = 2,
    /// Error
    Error = 3,
}

/// Command priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Realtime priority
    Realtime = 3,
}

/// Command queue
pub struct CommandQueue {
    /// Queue ID
    pub queue_id: u32,
    /// Queue name
    pub name: &'static str,
    /// Priority
    pub priority: CommandPriority,
    /// State
    pub state: AtomicU32,
    /// Command buffer
    pub commands: [Option<GpuCommand>; 64],
    /// Head pointer
    pub head: AtomicU32,
    /// Tail pointer
    pub tail: AtomicU32,
    /// Command count
    pub count: AtomicU32,
    /// Processed command count
    pub processed: AtomicU64,
}

impl CommandQueue {
    pub const fn new(queue_id: u32, name: &'static str, priority: CommandPriority) -> Self {
        CommandQueue {
            queue_id,
            name,
            priority,
            state: AtomicU32::new(QueueState::Idle as u32),
            commands: [None; 64],
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            count: AtomicU32::new(0),
            processed: AtomicU64::new(0),
        }
    }

    /// Submit command
    pub fn submit(&self, cmd: GpuCommand) -> i32 {
        // Check if queue is full
        if self.count.load(Ordering::Acquire) >= 64 {
            return -1;
        }

        // Get tail pointer
        let tail = self.tail.fetch_add(1, Ordering::AcqRel) % 64;

        // Store command at tail position
        // Safety: tail is always in range [0, 63] due to modulo
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let commands_ptr = self.commands.as_ptr() as *mut [Option<GpuCommand>; 64];
            (*commands_ptr)[tail as usize] = Some(cmd);
        }

        // Increase count
        self.count.fetch_add(1, Ordering::Release);

        log_debug!("Queue {}: Command submitted (count={})",
            self.queue_id, self.count.load(Ordering::Acquire));

        0
    }

    /// Get next command
    pub fn get_next(&self) -> Option<GpuCommand> {
        // Check if queue is empty
        if self.count.load(Ordering::Acquire) == 0 {
            return None;
        }

        // Get head pointer
        let head = self.head.fetch_add(1, Ordering::AcqRel) % 64;

        // Retrieve command from head position
        // Safety: head is always in range [0, 63] due to modulo
        // SAFETY: unsafe block required for low-level memory or hardware access
        let cmd = unsafe {
            let commands_ptr = self.commands.as_ptr() as *mut [Option<GpuCommand>; 64];
            (*commands_ptr)[head as usize].take()
        };

        // Decrease count
        self.count.fetch_sub(1, Ordering::Release);

        // Increase processed count
        self.processed.fetch_add(1, Ordering::Release);

        cmd
    }

    /// Get queue state
    pub fn get_state(&self) -> QueueState {
        match self.state.load(Ordering::Acquire) {
            0 => QueueState::Idle,
            1 => QueueState::Running,
            2 => QueueState::Paused,
            3 => QueueState::Error,
            _ => QueueState::Idle,
        }
    }

    /// Set queue state
    pub fn set_state(&self, state: QueueState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Get command count
    pub fn get_count(&self) -> u32 {
        self.count.load(Ordering::Acquire)
    }

    /// Clear queue
    pub fn clear(&self) {
        self.head.store(0, Ordering::Release);
        self.tail.store(0, Ordering::Release);
        self.count.store(0, Ordering::Release);
    }
}

/// Command queue manager
pub struct CommandQueueManager {
    /// Queue array
    pub queues: [Option<CommandQueue>; 4],
    /// Queue count
    pub num_queues: u32,
}

impl CommandQueueManager {
    pub const fn new() -> Self {
        CommandQueueManager {
            queues: [None, None, None, None],
            num_queues: 0,
        }
    }

    /// Create queue
    pub fn create_queue(&mut self, name: &'static str, priority: CommandPriority) -> Option<u32> {
        for (i, slot) in self.queues.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(CommandQueue::new(i as u32, name, priority));
                self.num_queues += 1;
                return Some(i as u32);
            }
        }
        None
    }

    /// Destroy queue
    pub fn destroy_queue(&mut self, queue_id: u32) -> i32 {
        if (queue_id as usize) >= self.queues.len() {
            return -1;
        }

        self.queues[queue_id as usize] = None;
        self.num_queues -= 1;
        0
    }

    /// Get queue
    pub fn get_queue(&self, queue_id: u32) -> Option<&CommandQueue> {
        if (queue_id as usize) >= self.queues.len() {
            return None;
        }

        self.queues[queue_id as usize].as_ref()
    }

    /// Submit command to highest priority queue
    pub fn submit_to_highest_priority(&self, cmd: GpuCommand) -> i32 {
        // Find queue from high to low priority
        for i in (0..4).rev() {
            if let Some(ref queue) = self.queues[i] {
                if queue.get_count() < 64 {
                    return queue.submit(cmd);
                }
            }
        }
        -1
    }
}

/// Global command queue manager
static QUEUE_MANAGER: core::sync::OnceLock<CommandQueueManager> = core::sync::OnceLock::new();

pub fn get_queue_manager() -> &'static mut CommandQueueManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut QUEUE_MANAGER }
}

pub fn init_command_queues() {
    let manager = get_queue_manager();

    // Create default queues
    manager.create_queue("low", CommandPriority::Low);
    manager.create_queue("normal", CommandPriority::Normal);
    manager.create_queue("high", CommandPriority::High);
    manager.create_queue("realtime", CommandPriority::Realtime);

    log_info!("GPU command queues initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_queue() {
        let mut manager = CommandQueueManager::new();
        let queue_id = manager.create_queue("test", CommandPriority::Normal);
        assert!(queue_id.is_some());
    }
}
