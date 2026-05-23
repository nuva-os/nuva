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

//! Thread pool implementation for dispatch framework

use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use spin::Mutex as SpinLock;

use super::{Task, QueuePriority};

/// Worker thread
struct Worker {
    /// Thread ID
    id: u32,
    /// Whether the worker is active
    active: AtomicBool,
}

impl Worker {
    fn new(id: u32) -> Self {
        Self {
            id,
            active: AtomicBool::new(true),
        }
    }

    fn run(&self, pool: &ThreadPool) {
        while self.active.load(Ordering::Acquire) {
            // Try to dequeue a task
            if let Some(task) = pool.dequeue() {
                task.execute();
            } else {
                // No task available; spin-wait
                core::hint::spin_loop();
            }
        }
    }

    fn stop(&self) {
        self.active.store(false, Ordering::Release);
    }
}

/// Thread pool for concurrent task execution
pub struct ThreadPool {
    /// Task queue
    tasks: SpinLock<VecDeque<Box<dyn Task>>>,
    /// Worker thread list
    workers: SpinLock<Vec<Arc<Worker>>>,
    /// Maximum thread count
    max_threads: u32,
    /// Minimum thread count
    min_threads: u32,
    /// Active thread count
    active_count: AtomicU32,
    /// Whether the pool is running
    running: AtomicBool,
}

impl ThreadPool {
    /// Create a new thread pool
    pub fn new(min_threads: u32, max_threads: u32) -> Self {
        let pool = Self {
            tasks: SpinLock::new(VecDeque::new()),
            workers: SpinLock::new(Vec::new()),
            max_threads,
            min_threads,
            active_count: AtomicU32::new(0),
            running: AtomicBool::new(true),
        };

        // Create minimum count of worker threads
        for _ in 0..min_threads {
            pool.spawn_worker();
        }

        pool
    }

    /// Create a default thread pool (2-8 threads)
    pub fn default_pool() -> Self {
        Self::new(2, 8)
    }

    /// Submit a task to the pool
    pub fn submit(&self, task: Box<dyn Task>, _priority: QueuePriority) {
        if !self.running.load(Ordering::Acquire) {
            return;
        }

        // Enqueue task
        self.tasks.lock().push_back(task);

        // Check if more worker threads are needed
        if self.active_count.load(Ordering::Acquire) < self.max_threads {
            if self.tasks.lock().len() > self.active_count.load(Ordering::Acquire) as usize {
                self.spawn_worker();
            }
        }
    }

    /// Dequeue a task from the front of the queue
    fn dequeue(&self) -> Option<Box<dyn Task>> {
        self.tasks.lock().pop_front()
    }

    /// Spawn a new worker thread
    fn spawn_worker(&self) {
        let id = self.workers.lock().len() as u32;
        let worker = Arc::new(Worker::new(id));

        self.active_count.fetch_add(1, Ordering::AcqRel);
        self.workers.lock().push(worker.clone());

        // Actual implementation should create real kernel threads
        // This is a simplified placeholder
    }

    /// Shut down the thread pool
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Release);

        // Stop all worker threads
        for worker in self.workers.lock().iter() {
            worker.stop();
        }
    }

    /// Get the task queue length
    pub fn queue_len(&self) -> usize {
        self.tasks.lock().len()
    }

    /// Get the active worker thread count
    pub fn worker_count(&self) -> u32 {
        self.active_count.load(Ordering::Acquire)
    }

    /// Get the maximum thread count
    pub fn max_threads(&self) -> u32 {
        self.max_threads
    }

    /// Get the minimum thread count
    pub fn min_threads(&self) -> u32 {
        self.min_threads
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool_create() {
        let pool = ThreadPool::new(2, 4);
        assert_eq!(pool.min_threads(), 2);
        assert_eq!(pool.max_threads(), 4);
    }

    #[test]
    fn test_thread_pool_submit() {
        let pool = ThreadPool::default_pool();

        use super::super::ClosureTask;
        let task = Box::new(ClosureTask::new(|| {}));

        pool.submit(task, QueuePriority::Default);
        assert_eq!(pool.queue_len(), 1);
    }
}
