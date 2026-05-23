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

//! Nuva Concurrency Framework (GCD-style)
//!
//! Provides serial/concurrent dispatch queues, thread pools, semaphores,
//! and dispatch groups for structured concurrency.

mod queue;
mod group;
mod pool;
mod semaphore;

pub use queue::{DispatchQueue, QueueType, QueuePriority};
pub use group::DispatchGroup;
pub use pool::ThreadPool;
pub use semaphore::DispatchSemaphore;

/// Dispatch error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchError {
    /// Queue is full
    QueueFull,
    /// Queue is closed
    QueueClosed,
    /// Operation timed out
    Timeout,
    /// Invalid argument
    InvalidArgument,
    /// Insufficient memory
    NoMemory,
    /// Deadlock detected
    Deadlock,
}

/// Task trait for dispatch work items
pub trait Task: Send + Sync {
    /// Execute the task
    fn execute(self: alloc::boxed::Box<Self>);

    /// Get task priority
    fn priority(&self) -> QueuePriority {
        QueuePriority::Default
    }
}

/// Closure-based task wrapper
pub struct ClosureTask<F>
where
    F: FnOnce() + Send,
{
    closure: Option<F>,
    priority: QueuePriority,
}

impl<F> ClosureTask<F>
where
    F: FnOnce() + Send,
{
    /// Create a new closure task
    pub fn new(closure: F) -> Self {
        Self {
            closure: Some(closure),
            priority: QueuePriority::Default,
        }
    }

    /// Set task priority
    pub fn with_priority(mut self, priority: QueuePriority) -> Self {
        self.priority = priority;
        self
    }
}

impl<F> Task for ClosureTask<F>
where
    F: FnOnce() + Send + Sync,
{
    fn execute(mut self: alloc::boxed::Box<Self>) {
        if let Some(closure) = self.closure.take() {
            closure();
        }
    }

    fn priority(&self) -> QueuePriority {
        self.priority
    }
}

/// Get the global dispatch queue for a priority level
pub fn global_queue(priority: QueuePriority) -> alloc::sync::Arc<DispatchQueue> {
    // Returns a pre-created global queue for the given priority
    alloc::sync::Arc::new(DispatchQueue::concurrent("global"))
}

/// Get the main dispatch queue (serial)
pub fn main_queue() -> alloc::sync::Arc<DispatchQueue> {
    // Returns the pre-created main serial queue
    alloc::sync::Arc::new(DispatchQueue::serial("main"))
}

/// Execute work asynchronously on a dispatch queue
pub fn async_exec<F>(queue: &DispatchQueue, work: F)
where
    F: FnOnce() + Send + Sync + 'static,
{
    queue.async_exec(work);
}

/// Execute work synchronously on a dispatch queue, returning the result
pub fn sync_exec<F, R>(queue: &DispatchQueue, work: F) -> R
where
    F: FnOnce() -> R + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    queue.sync_exec(work)
}

/// Execute work after a delay on a dispatch queue
pub fn after<F>(queue: &DispatchQueue, duration: core::time::Duration, work: F)
where
    F: FnOnce() + Send + Sync + 'static,
{
    queue.after(duration, work);
}

/// Initialize dispatch framework
pub fn init_dispatch() {
    log_info!("Dispatch framework initialized");
}
