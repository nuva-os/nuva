/*
 * Nuva OS - System Library - Brain NPU Scheduler
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

/// NPU state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpuState {
    /// Idle
    Idle = 0,
    /// Busy
    Busy = 1,
    /// Suspended
    Suspended = 2,
    /// Error
    Error = 3,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Realtime priority
    Realtime = 3,
}

/// NPU task
pub struct NpuTask {
    /// Task ID
    pub task_id: u64,
    /// Model ID
    pub model_id: u64,
    /// Priority
    pub priority: TaskPriority,
    /// Input address
    pub input_addr: u64,
    /// Output address
    pub output_addr: u64,
    /// Input size
    pub input_size: usize,
    /// Output size
    pub output_size: usize,
    /// Submit time
    pub submit_time: u64,
    /// Timeout
    pub timeout: u64,
}

/// NPU scheduler
pub struct NpuScheduler {
    /// NPU state
    pub state: AtomicU32,
    /// Current task ID
    pub current_task: AtomicU64,
    /// Completed task count
    pub completed_tasks: AtomicU64,
    /// Total execution time
    pub total_time: AtomicU64,
    /// Queue length
    pub queue_length: AtomicU32,
}

impl NpuScheduler {
    pub const fn new() -> Self {
        NpuScheduler {
            state: AtomicU32::new(NpuState::Idle as u32),
            current_task: AtomicU64::new(0),
            completed_tasks: AtomicU64::new(0),
            total_time: AtomicU64::new(0),
            queue_length: AtomicU32::new(0),
        }
    }

    /// Initialize the NPU scheduler
    pub fn init(&mut self) -> i32 {
        log_info!("NPU scheduler initialized");
        0
    }

    /// Submit a task to the NPU
    pub fn submit(&mut self, task: NpuTask) -> Option<u64> {
        // Check NPU state
        if self.state.load(Ordering::Acquire) == NpuState::Error as u32 {
            return None;
        }

        log_debug!("NPU task submitted: id={}, model={}, priority={:?}",
            task.task_id, task.model_id, task.priority);

        // TODO: Add task to the queue
        // 1. Insert into queue based on priority
        // 2. Wake the scheduler

        self.queue_length.fetch_add(1, Ordering::AcqRel);

        Some(task.task_id)
    }

    /// Cancel a task
    pub fn cancel(&mut self, task_id: u64) -> i32 {
        // TODO: Remove task from the queue

        self.queue_length.fetch_sub(1, Ordering::AcqRel);

        log_debug!("NPU task cancelled: {}", task_id);
        0
    }

    /// Schedule the next task
    pub fn schedule(&mut self) -> Option<NpuTask> {
        // Check state
        if self.state.load(Ordering::Acquire) != NpuState::Idle as u32 {
            return None;
        }

        // TODO: Select the highest-priority task from the queue
        // 1. Traverse the queue
        // 2. Select the highest-priority task
        // 3. Remove it from the queue

        None
    }

    /// Execute a task on the NPU
    pub fn execute(&mut self, task: &NpuTask) -> i32 {
        // Set state to busy
        self.state.store(NpuState::Busy as u32, Ordering::Release);
        self.current_task.store(task.task_id, Ordering::Release);

        log_debug!("NPU executing task: {}", task.task_id);

        // TODO: Execute inference using NPU HAL
        // 1. Configure NPU
        // 2. Load model
        // 3. Set input/output buffers
        // 4. Start inference
        // 5. Wait for completion

        // Simulate execution
        let exec_time: u64 = 1000;  // 1ms

        // Update statistics
        self.completed_tasks.fetch_add(1, Ordering::AcqRel);
        self.total_time.fetch_add(exec_time, Ordering::AcqRel);
        self.queue_length.fetch_sub(1, Ordering::AcqRel);

        // Set state to idle
        self.state.store(NpuState::Idle as u32, Ordering::Release);
        self.current_task.store(0, Ordering::Release);

        0
    }

    /// Suspend the NPU
    pub fn suspend(&mut self) -> i32 {
        self.state.store(NpuState::Suspended as u32, Ordering::Release);
        log_info!("NPU suspended");
        0
    }

    /// Resume the NPU
    pub fn resume(&mut self) -> i32 {
        self.state.store(NpuState::Idle as u32, Ordering::Release);
        log_info!("NPU resumed");
        0
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64, u32) {
        let completed = self.completed_tasks.load(Ordering::Acquire);
        let total_time = self.total_time.load(Ordering::Acquire);
        let queue_len = self.queue_length.load(Ordering::Acquire);
        (completed, total_time, queue_len)
    }

    /// Get average execution time
    pub fn get_avg_time(&self) -> u64 {
        let completed = self.completed_tasks.load(Ordering::Acquire);
        if completed == 0 {
            return 0;
        }
        let total_time = self.total_time.load(Ordering::Acquire);
        total_time / completed
    }
}

/// Global NPU scheduler instance
static mut NPU_SCHEDULER: NpuScheduler = NpuScheduler::new();

/// Get the global NPU scheduler instance
pub fn get_npu_scheduler() -> &'static mut NpuScheduler {
    // SAFETY: Single-threaded access; synchronized externally.
    unsafe { &mut NPU_SCHEDULER }
}

/// Initialize the NPU scheduler
pub fn init_npu_scheduler() {
    let scheduler = get_npu_scheduler();
    scheduler.init();
}
