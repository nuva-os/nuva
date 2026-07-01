/*
 * Nuva OS - Tests - Perf - PerfTests
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
/*
 * Performance Tests
 *
 * Copyright (C) 2026 Nuva OS Team
 */

use crate::lib::core::*;
use crate::kernel::perf::*;
use alloc::vec;

#[test]
fn test_mpsc_queue() {
    let queue = MpscQueue::new();
    
    // Push elements
    queue.push(1);
    queue.push(2);
    queue.push(3);
    
    // Pop elements
    assert_eq!(queue.pop(), Some(1));
    assert_eq!(queue.pop(), Some(2));
    assert_eq!(queue.pop(), Some(3));
    assert_eq!(queue.pop(), None);
}

#[test]
fn test_spsc_queue() {
    let queue = SpscQueue::new(1024);
    
    // Push elements
    assert!(queue.push(1).is_ok());
    assert!(queue.push(2).is_ok());
    assert!(queue.push(3).is_ok());
    
    // Pop elements
    assert_eq!(queue.pop(), Some(1));
    assert_eq!(queue.pop(), Some(2));
    assert_eq!(queue.pop(), Some(3));
    assert_eq!(queue.pop(), None);
}

#[test]
fn test_lock_free_stack() {
    let stack = LockFreeStack::new();
    
    // Push elements
    stack.push(1);
    stack.push(2);
    stack.push(3);
    
    // Pop elements (LIFO order)
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), None);
}

#[test]
fn test_memory_pool() {
    let pool = MemoryPool::new(64, 16);
    
    // Allocate blocks
    let ptr1 = pool.alloc();
    let ptr2 = pool.alloc();
    let ptr3 = pool.alloc();
    
    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert!(!ptr3.is_null());
    assert_ne!(ptr1, ptr2);
    assert_ne!(ptr2, ptr3);
    
    // Check allocated count
    assert_eq!(pool.allocated(), 3);
    
    // Free blocks
    pool.free(ptr1);
    pool.free(ptr2);
    pool.free(ptr3);
    
    // Check allocated count
    assert_eq!(pool.allocated(), 0);
}

#[test]
fn test_pool_manager() {
    let config = PoolManagerConfig::default();
    let manager = PoolManager::new(config);
    
    // Allocate different sizes
    let ptr1 = manager.alloc(32);
    let ptr2 = manager.alloc(128);
    let ptr3 = manager.alloc(1024);
    
    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert!(!ptr3.is_null());
    
    // Free
    manager.free(ptr1, 32);
    manager.free(ptr2, 128);
    manager.free(ptr3, 1024);
}

#[test]
fn test_histogram() {
    let buckets = vec![1.0, 5.0, 10.0, 50.0, 100.0];
    let mut histogram = Histogram::new(buckets);
    
    // Observe values
    histogram.observe(0.5);
    histogram.observe(2.0);
    histogram.observe(7.0);
    histogram.observe(25.0);
    histogram.observe(75.0);
    histogram.observe(150.0);
    
    // Check statistics
    assert_eq!(histogram.count, 6);
    assert!(histogram.sum > 0.0);
}

#[test]
fn test_performance_monitor() {
    let config = MonitorConfig::default();
    let monitor = PerformanceMonitor::new(config);
    
    // Register collectors
    monitor.register_collector("cpu", Arc::new(CpuMetricsCollector));
    monitor.register_collector("memory", Arc::new(MemoryMetricsCollector));
    
    // Collect metrics
    let snapshot = monitor.collect().unwrap();
    
    // Verify snapshot
    assert!(snapshot.timestamp >= 0);
    assert!(!snapshot.metrics.is_empty());
}
