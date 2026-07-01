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

//! Dispatch queue implementation (serial and concurrent)

use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use spin::Mutex as SpinLock;

use super::{Task, ClosureTask, DispatchError};

/// Queue execution type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    /// Serial queue: tasks execute one at a time
    Serial = 0,
    /// Concurrent queue: tasks may execute in parallel
    Concurrent = 1,
}

impl Default for QueueType {
    fn default() -> Self {
        Self::Serial
    }
}

/// Queue priority level
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueuePriority {
    /// Background priority
    Background = 0,
    /// Low priority
    Low = 1,
    /// Default priority
    Default = 2,
    /// High priority
    High = 3,
}

impl Default for QueuePriority {
    fn default() -> Self {
        Self::Default
    }
}

/// Dispatch queue
pub struct DispatchQueue {
    /// Queue label for identification
    label: Option<String>,
    /// Queue execution type (serial or concurrent)
    queue_type: QueueType,
    /// Queue priority level
    priority: AtomicU8,
    /// Pending task queue
    tasks: SpinLock<VecDeque<Box<dyn Task>>>,
    /// Active task count
    active_count: AtomicU32,
    /// Suspend count (queue is suspended when > 0)
    suspend_count: AtomicU32,
    /// Target queue for redirection
    target_queue: Option<Arc<DispatchQueue>>,
}

impl DispatchQueue {
    /// Create a new serial dispatch queue
    pub fn serial(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            queue_type: QueueType::Serial,
            priority: AtomicU8::new(QueuePriority::Default as u8),
            tasks: SpinLock::new(VecDeque::new()),
            active_count: AtomicU32::new(0),
            suspend_count: AtomicU32::new(0),
            target_queue: None,
        }
    }

    /// Create a new concurrent dispatch queue
    pub fn concurrent(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            queue_type: QueueType::Concurrent,
            priority: AtomicU8::new(QueuePriority::Default as u8),
            tasks: SpinLock::new(VecDeque::new()),
            active_count: AtomicU32::new(0),
            suspend_count: AtomicU32::new(0),
            target_queue: None,
        }
    }

    /// Get the queue label
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Get the queue type
    pub fn queue_type(&self) -> QueueType {
        self.queue_type
    }

    /// Get the current priority
    pub fn priority(&self) -> QueuePriority {
        match self.priority.load(Ordering::Acquire) {
            0 => QueuePriority::Background,
            1 => QueuePriority::Low,
            2 => QueuePriority::Default,
            _ => QueuePriority::High,
        }
    }

    /// Set the queue priority
    pub fn set_priority(&self, priority: QueuePriority) {
        self.priority.store(priority as u8, Ordering::Release);
    }

    /// Execute work asynchronously on this queue
    pub fn async_exec<F>(&self, work: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let task = Box::new(ClosureTask::new(work));
        self.enqueue(task);
    }

    /// Execute work synchronously on this queue, returning the result
    pub fn sync_exec<F, R>(&self, work: F) -> R
    where
        F: FnOnce() -> R + Send + Sync + 'static,
        R: Send + 'static,
    {
        if self.is_current_queue() {
            // Already on this queue, execute directly to avoid deadlock
            work()
        } else {
            // Submit and wait for completion
            use core::cell::OnceCell;
            let result = Arc::new(SpinLock::new(OnceCell::new()));
            let result_clone = result.clone();

            self.async_exec(move || {
                let r = work();
                *result_clone.lock() = OnceCell::from(r);
            });

            // Spin-wait for completion
            loop {
                if let Some(r) = result.lock().take() {
                    return r;
                }
                core::hint::spin_loop();
            }
        }
    }

    /// Execute work after a delay
    pub fn after<F>(&self, duration: core::time::Duration, work: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        // Simplified: enqueue immediately; full implementation uses timer
        let _ = duration;
        self.async_exec(work);
    }

    /// Enqueue a task onto this queue
    pub fn enqueue(&self, task: Box<dyn Task>) {
        match self.queue_type {
            QueueType::Serial => {
                self.tasks.lock().push_back(task);
                self.schedule_next();
            }
            QueueType::Concurrent => {
                // Concurrent queue: execute immediately
                self.execute_task(task);
            }
        }
    }

    /// Schedule the next task from the queue
    fn schedule_next(&self) {
        if self.suspend_count.load(Ordering::Acquire) > 0 {
            return;
        }

        if let Some(task) = self.tasks.lock().pop_front() {
            self.execute_task(task);
        }
    }

    /// Execute a single task
    fn execute_task(&self, task: Box<dyn Task>) {
        self.active_count.fetch_add(1, Ordering::AcqRel);

        task.execute();

        self.active_count.fetch_sub(1, Ordering::AcqRel);

        // Serial queue: schedule next task after completion
        if self.queue_type == QueueType::Serial {
            self.schedule_next();
        }
    }

    /// Check if currently executing on this queue
    fn is_current_queue(&self) -> bool {
        // Full implementation checks current thread's associated queue
        false
    }

    /// Suspend the queue (increment suspend count)
    pub fn suspend(&self) {
        self.suspend_count.fetch_add(1, Ordering::AcqRel);
    }

    /// Resume the queue (decrement suspend count)
    pub fn resume(&self) {
        let prev = self.suspend_count.fetch_sub(1, Ordering::AcqRel);
        if prev == 1 {
            // Last suspend released; resume execution
            self.schedule_next();
        }
    }

    /// Get the number of pending tasks
    pub fn len(&self) -> usize {
        self.tasks.lock().len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.lock().is_empty()
    }

    /// Get the number of currently executing tasks
    pub fn active_count(&self) -> u32 {
        self.active_count.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use core::sync::atomic::AtomicU8;

    #[test]
    fn test_serial_queue() {
        let queue = DispatchQueue::serial("test");
        assert_eq!(queue.queue_type(), QueueType::Serial);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_concurrent_queue() {
        let queue = DispatchQueue::concurrent("test");
        assert_eq!(queue.queue_type(), QueueType::Concurrent);
    }

    #[test]
    fn test_async_exec() {
        let queue = DispatchQueue::serial("test");
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        queue.async_exec(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Wait for execution to complete
        while counter.load(Ordering::SeqCst) == 0 {
            core::hint::spin_loop();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
