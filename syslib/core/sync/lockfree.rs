/*
 * Lock-Free Data Structures
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module implements high-performance lock-free data structures
 * for concurrent access without mutex overhead.
 */

use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use core::ptr;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Lock-free MPSC (Multi-Producer Single-Consumer) queue
/// High-performance queue for multiple producers and single consumer.
/// Based on Michael-Scott queue algorithm.
pub struct MpscQueue<T> {
    /// Head pointer (consumer side)
    head: AtomicPtr<Node<T>>,

    /// Tail pointer (producer side)
    tail: AtomicPtr<Node<T>>,

    /// Queue length
    length: AtomicUsize,
}

impl<T> MpscQueue<T> {
    /// Create new MPSC queue
    pub fn new() -> Self {
        // Create sentinel node
        let sentinel = Box::into_raw(Box::new(Node {
            data: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        Self {
            head: AtomicPtr::new(sentinel),
            tail: AtomicPtr::new(sentinel),
            length: AtomicUsize::new(0),
        }
    }

    /// Push item to queue (producer)
    /// @param item: Item to push
    pub fn push(&self, item: T) {
        // Create new node
        let new_node = Box::into_raw(Box::new(Node {
            data: Some(item),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        // Add to tail
        loop {
            let tail = self.tail.load(Ordering::Acquire);
            // SAFETY: atomic memory operation on shared state
            let next = unsafe { (*tail).next.load(Ordering::Acquire) };

            // Check if tail is still the tail
            if tail == self.tail.load(Ordering::Acquire) {
                if next.is_null() {
                    // Try to link new node
                    // SAFETY: unsafe block required for low-level memory or hardware access
                    if unsafe { (*tail).next.compare_exchange_weak(
                        ptr::null_mut(),
                        new_node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    ).is_ok() } {
                        // Successfully linked, advance tail
                        self.tail.compare_exchange(
                            tail,
                            new_node,
                            Ordering::Release,
                            Ordering::Relaxed,
                        );
                        self.length.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                } else {
                    // Tail is lagging, advance it
                    self.tail.compare_exchange(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                }
            }
        }
    }

    /// Pop item from queue (consumer)
    /// @return: Item if available
    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            // SAFETY: atomic memory operation on shared state
            let next = unsafe { (*head).next.load(Ordering::Acquire) };

            // Check if head is still the head
            if head == self.head.load(Ordering::Acquire) {
                if head == tail {
                    if next.is_null() {
                        // Queue is empty
                        return None;
                    }
                    // Tail is lagging, advance it
                    self.tail.compare_exchange(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                } else {
                    // Read value
                    // SAFETY: unsafe block required for low-level memory or hardware access
                    if let Some(data) = unsafe { (*next).data.take() } {
                        // Advance head
                        if self.head.compare_exchange_weak(
                            head,
                            next,
                            Ordering::Release,
                            Ordering::Relaxed,
                        ).is_ok() {
                            // Free old head
                            // SAFETY: unsafe block required for low-level memory or hardware access
                            unsafe { drop(Box::from_raw(head)); }
                            self.length.fetch_sub(1, Ordering::Relaxed);
                            return Some(data);
                        }
                        // Put data back
                        // SAFETY: unsafe block required for low-level memory or hardware access
                        unsafe { (*next).data = Some(data); }
                    }
                }
            }
        }
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.length.load(Ordering::Relaxed) == 0
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.length.load(Ordering::Relaxed)
    }
}

impl<T> Drop for MpscQueue<T> {
    fn drop(&mut self) {
        // Pop all remaining items
        while self.pop().is_some() {}

        // Free sentinel node
        let head = self.head.load(Ordering::Relaxed);
        if !head.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { drop(Box::from_raw(head)); }
        }
    }
}

/// Lock-free SPSC (Single-Producer Single-Consumer) queue
/// Optimized for single producer and single consumer scenario.
/// Based on bounded ring buffer.
pub struct SpscQueue<T> {
    /// Buffer
    buffer: Box<[AtomicPtr<T>]>,

    /// Capacity
    capacity: usize,

    /// Head index (consumer)
    head: AtomicUsize,

    /// Tail index (producer)
    tail: AtomicUsize,
}

impl<T> SpscQueue<T> {
    /// Create new SPSC queue
    /// @param capacity: Queue capacity (must be power of 2)
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        let buffer: Vec<AtomicPtr<T>> = (0..capacity)
            .map(|_| AtomicPtr::new(ptr::null_mut()))
            .collect();

        Self {
            buffer: buffer.into_boxed_slice(),
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Push item to queue (producer)
    /// @param item: Item to push
    /// @return: true if successful
    pub fn push(&self, item: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % self.capacity;

        // Check if full
        if next_tail == self.head.load(Ordering::Acquire) {
            return false;
        }

        // Store item
        let ptr = Box::into_raw(Box::new(item));
        self.buffer[tail].store(ptr, Ordering::Release);
        self.tail.store(next_tail, Ordering::Release);

        true
    }

    /// Pop item from queue (consumer)
    /// @return: Item if available
    pub fn pop(&self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        // Check if empty
        if head == self.tail.load(Ordering::Acquire) {
            return None;
        }

        // Load item
        let ptr = self.buffer[head].load(Ordering::Acquire);
        let next_head = (head + 1) % self.capacity;
        self.head.store(next_head, Ordering::Release);

        if ptr.is_null() {
            None
        } else {
            // SAFETY: unsafe block required for low-level memory or hardware access
            Some(*unsafe { Box::from_raw(ptr) })
        }
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        let next_tail = (self.tail.load(Ordering::Relaxed) + 1) % self.capacity;
        next_tail == self.head.load(Ordering::Relaxed)
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        (tail + self.capacity - head) % self.capacity
    }

    /// Get queue capacity
    pub fn capacity(&self) -> usize {
        self.capacity - 1 // One slot is always empty
    }
}

impl<T> Drop for SpscQueue<T> {
    fn drop(&mut self) {
        // Pop all remaining items
        while self.pop().is_some() {}
    }
}

/// Lock-free stack (MP)
/// Multiple producers can push, single consumer can pop.
/// Based on Treiber stack.
pub struct LockFreeStack<T> {
    /// Head pointer
    head: AtomicPtr<StackNode<T>>,

    /// Stack length
    length: AtomicUsize,
}

impl<T> LockFreeStack<T> {
    /// Create new lock-free stack
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            length: AtomicUsize::new(0),
        }
    }

    /// Push item to stack
    /// @param item: Item to push
    pub fn push(&self, item: T) {
        let new_node = Box::into_raw(Box::new(StackNode {
            data: item,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        loop {
            let head = self.head.load(Ordering::Acquire);
            // SAFETY: atomic memory operation on shared state
            unsafe { (*new_node).next.store(head, Ordering::Release); }

            if self.head.compare_exchange_weak(
                head,
                new_node,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                self.length.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
    }

    /// Pop item from stack
    /// @return: Item if available
    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);

            if head.is_null() {
                return None;
            }

            // SAFETY: atomic memory operation on shared state
            let next = unsafe { (*head).next.load(Ordering::Acquire) };

            if self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                // SAFETY: unsafe block required for low-level memory or hardware access
                let node = unsafe { Box::from_raw(head) };
                self.length.fetch_sub(1, Ordering::Relaxed);
                return Some(node.data);
            }
        }
    }

    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed).is_null()
    }

    /// Get stack length
    pub fn len(&self) -> usize {
        self.length.load(Ordering::Relaxed)
    }
}

impl<T> Drop for LockFreeStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

/// Node for MPSC queue
struct Node<T> {
    data: Option<T>,
    next: AtomicPtr<Node<T>>,
}

/// Node for lock-free stack
struct StackNode<T> {
    data: T,
    next: AtomicPtr<StackNode<T>>,
}

impl<T> Default for MpscQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Default for LockFreeStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpsc_queue() {
        let queue = MpscQueue::new();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.push(1);
        queue.push(2);
        queue.push(3);

        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 3);

        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);

        assert!(queue.is_empty());
    }

    #[test]
    fn test_spsc_queue() {
        let queue = SpscQueue::new(4);

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        assert!(queue.push(1));
        assert!(queue.push(2));
        assert!(queue.push(3));

        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 3);

        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);

        assert!(queue.is_empty());
    }

    #[test]
    fn test_lock_free_stack() {
        let stack = LockFreeStack::new();

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);

        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 3);

        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);

        assert!(stack.is_empty());
    }
}
