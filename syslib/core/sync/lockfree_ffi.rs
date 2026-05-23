/*
 * Nuva OS - Lock-Free Data Structures Rust FFI Bindings
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

//! Lock-Free Data Structures FFI Bindings
/*!*/
//! Safe Rust wrappers around the C lock-free data structures.

use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

// ============================================================================
// MPSC Queue FFI

/// C MPSC queue structure (opaque)
#[repr(C)]
pub struct CMpscQueue {
    _private: [u8; 0],
}

/// FFI declarations for MPSC queue
mod mpsc_ffi {
    use super::*;

    extern "C" {
        pub fn mpsc_queue_init(queue: *mut CMpscQueue) -> i32;
        pub fn mpsc_queue_push(queue: *mut CMpscQueue, data: *mut core::ffi::c_void) -> i32;
        pub fn mpsc_queue_pop(queue: *mut CMpscQueue) -> *mut core::ffi::c_void;
        pub fn mpsc_queue_is_empty(queue: *mut CMpscQueue) -> i32;
        pub fn mpsc_queue_length(queue: *mut CMpscQueue) -> usize;
        pub fn mpsc_queue_destroy(queue: *mut CMpscQueue);
    }
}

/// Safe wrapper for MPSC queue
pub struct MpscQueue {
    queue: *mut CMpscQueue,
}

impl MpscQueue {
    /// Create a new MPSC queue
    pub fn new() -> Result<Self, ()> {
        // Allocate queue structure
        // SAFETY: unsafe block required for low-level memory or hardware access
        let queue = unsafe {
            core::alloc::alloc(
                core::alloc::Layout::from_size_align(
                    core::mem::size_of::<CMpscQueue>(),
                    8,
                )
                .unwrap(),
            ) as *mut CMpscQueue
        };

        if queue.is_null() {
            return Err(());
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe { mpsc_ffi::mpsc_queue_init(queue) };
        if result != 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::alloc::dealloc(
                    queue as *mut u8,
                    core::alloc::Layout::from_size_align(
                        core::mem::size_of::<CMpscQueue>(),
                        8,
                    )
                    .unwrap(),
                );
            }
            return Err(());
        }

        Ok(Self { queue })
    }

    /// Push an item to the queue (producer)
    /// # Safety
    /// The caller must ensure the data pointer remains valid until popped.
    pub unsafe fn push(&self, data: *mut core::ffi::c_void) -> Result<(), ()> {
        let result = mpsc_ffi::mpsc_queue_push(self.queue, data);
        if result == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Pop an item from the queue (consumer)
    pub fn pop(&self) -> Option<*mut core::ffi::c_void> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { mpsc_ffi::mpsc_queue_pop(self.queue) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { mpsc_ffi::mpsc_queue_is_empty(self.queue) == 1 }
    }

    /// Get the queue length
    pub fn len(&self) -> usize {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { mpsc_ffi::mpsc_queue_length(self.queue) }
    }
}

impl Drop for MpscQueue {
    fn drop(&mut self) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            mpsc_ffi::mpsc_queue_destroy(self.queue);
            core::alloc::dealloc(
                self.queue as *mut u8,
                core::alloc::Layout::from_size_align(
                    core::mem::size_of::<CMpscQueue>(),
                    8,
                )
                .unwrap(),
            );
        }
    }
}

// ============================================================================
// SPSC Queue FFI

/// C SPSC queue structure (opaque)
#[repr(C)]
pub struct CSpscQueue {
    _private: [u8; 0],
}

mod spsc_ffi {
    use super::*;

    extern "C" {
        pub fn spsc_queue_init(queue: *mut CSpscQueue, capacity: usize) -> i32;
        pub fn spsc_queue_push(queue: *mut CSpscQueue, data: *mut core::ffi::c_void) -> i32;
        pub fn spsc_queue_pop(queue: *mut CSpscQueue) -> *mut core::ffi::c_void;
        pub fn spsc_queue_is_empty(queue: *mut CSpscQueue) -> i32;
        pub fn spsc_queue_is_full(queue: *mut CSpscQueue) -> i32;
        pub fn spsc_queue_length(queue: *mut CSpscQueue) -> usize;
        pub fn spsc_queue_destroy(queue: *mut CSpscQueue);
    }
}

/// Safe wrapper for SPSC queue
pub struct SpscQueue {
    queue: *mut CSpscQueue,
}

impl SpscQueue {
    /// Create a new SPSC queue with the given capacity
    pub fn new(capacity: usize) -> Result<Self, ()> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let queue = unsafe {
            core::alloc::alloc(
                core::alloc::Layout::from_size_align(
                    core::mem::size_of::<CSpscQueue>(),
                    8,
                )
                .unwrap(),
            ) as *mut CSpscQueue
        };

        if queue.is_null() {
            return Err(());
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe { spsc_ffi::spsc_queue_init(queue, capacity) };
        if result != 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::alloc::dealloc(
                    queue as *mut u8,
                    core::alloc::Layout::from_size_align(
                        core::mem::size_of::<CSpscQueue>(),
                        8,
                    )
                    .unwrap(),
                );
            }
            return Err(());
        }

        Ok(Self { queue })
    }

    /// Push an item to the queue (producer)
    pub unsafe fn push(&self, data: *mut core::ffi::c_void) -> Result<(), ()> {
        let result = spsc_ffi::spsc_queue_push(self.queue, data);
        if result == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Pop an item from the queue (consumer)
    pub fn pop(&self) -> Option<*mut core::ffi::c_void> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { spsc_ffi::spsc_queue_pop(self.queue) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { spsc_ffi::spsc_queue_is_empty(self.queue) == 1 }
    }

    /// Check if the queue is full
    pub fn is_full(&self) -> bool {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { spsc_ffi::spsc_queue_is_full(self.queue) == 1 }
    }

    /// Get the queue length
    pub fn len(&self) -> usize {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { spsc_ffi::spsc_queue_length(self.queue) }
    }
}

impl Drop for SpscQueue {
    fn drop(&mut self) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            spsc_ffi::spsc_queue_destroy(self.queue);
            core::alloc::dealloc(
                self.queue as *mut u8,
                core::alloc::Layout::from_size_align(
                    core::mem::size_of::<CSpscQueue>(),
                    8,
                )
                .unwrap(),
            );
        }
    }
}

// ============================================================================
// Lock-Free Stack FFI

/// C lock-free stack structure (opaque)
#[repr(C)]
pub struct CLfStack {
    _private: [u8; 0],
}

mod stack_ffi {
    use super::*;

    extern "C" {
        pub fn lf_stack_init(stack: *mut CLfStack) -> i32;
        pub fn lf_stack_push(stack: *mut CLfStack, data: *mut core::ffi::c_void) -> i32;
        pub fn lf_stack_pop(stack: *mut CLfStack) -> *mut core::ffi::c_void;
        pub fn lf_stack_is_empty(stack: *mut CLfStack) -> i32;
        pub fn lf_stack_length(stack: *mut CLfStack) -> usize;
        pub fn lf_stack_destroy(stack: *mut CLfStack);
    }
}

/// Safe wrapper for lock-free stack
pub struct LfStack {
    stack: *mut CLfStack,
}

impl LfStack {
    /// Create a new lock-free stack
    pub fn new() -> Result<Self, ()> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let stack = unsafe {
            core::alloc::alloc(
                core::alloc::Layout::from_size_align(
                    core::mem::size_of::<CLfStack>(),
                    8,
                )
                .unwrap(),
            ) as *mut CLfStack
        };

        if stack.is_null() {
            return Err(());
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe { stack_ffi::lf_stack_init(stack) };
        if result != 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::alloc::dealloc(
                    stack as *mut u8,
                    core::alloc::Layout::from_size_align(
                        core::mem::size_of::<CLfStack>(),
                        8,
                    )
                    .unwrap(),
                );
            }
            return Err(());
        }

        Ok(Self { stack })
    }

    /// Push an item onto the stack
    pub unsafe fn push(&self, data: *mut core::ffi::c_void) -> Result<(), ()> {
        let result = stack_ffi::lf_stack_push(self.stack, data);
        if result == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Pop an item from the stack
    pub fn pop(&self) -> Option<*mut core::ffi::c_void> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { stack_ffi::lf_stack_pop(self.stack) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { stack_ffi::lf_stack_is_empty(self.stack) == 1 }
    }

    /// Get the stack length
    pub fn len(&self) -> usize {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { stack_ffi::lf_stack_length(self.stack) }
    }
}

impl Drop for LfStack {
    fn drop(&mut self) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            stack_ffi::lf_stack_destroy(self.stack);
            core::alloc::dealloc(
                self.stack as *mut u8,
                core::alloc::Layout::from_size_align(
                    core::mem::size_of::<CLfStack>(),
                    8,
                )
                .unwrap(),
            );
        }
    }
}

// ============================================================================
// MPMC Queue FFI

/// C MPMC queue structure (opaque)
#[repr(C)]
pub struct CMpmcQueue {
    _private: [u8; 0],
}

mod mpmc_ffi {
    use super::*;

    extern "C" {
        pub fn mpmc_queue_init(queue: *mut CMpmcQueue, capacity: usize) -> i32;
        pub fn mpmc_queue_push(queue: *mut CMpmcQueue, data: *mut core::ffi::c_void) -> i32;
        pub fn mpmc_queue_pop(queue: *mut CMpmcQueue) -> *mut core::ffi::c_void;
        pub fn mpmc_queue_is_empty(queue: *mut CMpmcQueue) -> i32;
        pub fn mpmc_queue_length(queue: *mut CMpmcQueue) -> usize;
        pub fn mpmc_queue_destroy(queue: *mut CMpmcQueue);
    }
}

/// Safe wrapper for MPMC queue
pub struct MpmcQueue {
    queue: *mut CMpmcQueue,
}

impl MpmcQueue {
    /// Create a new MPMC queue with the given capacity
    pub fn new(capacity: usize) -> Result<Self, ()> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let queue = unsafe {
            core::alloc::alloc(
                core::alloc::Layout::from_size_align(
                    core::mem::size_of::<CMpmcQueue>(),
                    8,
                )
                .unwrap(),
            ) as *mut CMpmcQueue
        };

        if queue.is_null() {
            return Err(());
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe { mpmc_ffi::mpmc_queue_init(queue, capacity) };
        if result != 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::alloc::dealloc(
                    queue as *mut u8,
                    core::alloc::Layout::from_size_align(
                        core::mem::size_of::<CMpmcQueue>(),
                        8,
                    )
                    .unwrap(),
                );
            }
            return Err(());
        }

        Ok(Self { queue })
    }

    /// Push an item to the queue
    pub unsafe fn push(&self, data: *mut core::ffi::c_void) -> Result<(), ()> {
        let result = mpmc_ffi::mpmc_queue_push(self.queue, data);
        if result == 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Pop an item from the queue
    pub fn pop(&self) -> Option<*mut core::ffi::c_void> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { mpmc_ffi::mpmc_queue_pop(self.queue) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { mpmc_ffi::mpmc_queue_is_empty(self.queue) == 1 }
    }

    /// Get the queue length
    pub fn len(&self) -> usize {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { mpmc_ffi::mpmc_queue_length(self.queue) }
    }
}

impl Drop for MpmcQueue {
    fn drop(&mut self) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            mpmc_ffi::mpmc_queue_destroy(self.queue);
            core::alloc::dealloc(
                self.queue as *mut u8,
                core::alloc::Layout::from_size_align(
                    core::mem::size_of::<CMpmcQueue>(),
                    8,
                )
                .unwrap(),
            );
        }
    }
}

// ============================================================================
// Typed Wrappers

/// Typed MPSC queue wrapper
pub struct TypedMpscQueue<T> {
    queue: MpscQueue,
    _marker: core::marker::PhantomData<T>,
}

impl<T> TypedMpscQueue<T> {
    /// Create a new typed MPSC queue
    pub fn new() -> Result<Self, ()> {
        MpscQueue::new().map(|queue| Self {
            queue,
            _marker: core::marker::PhantomData,
        })
    }

    /// Push an item
    pub fn push(&self, item: T) -> Result<(), ()> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let boxed = unsafe { core::alloc::alloc(core::alloc::Layout::new::<T>()) as *mut T };
        if boxed.is_null() {
            return Err(());
        }
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write(boxed, item);
            self.queue.push(boxed as *mut core::ffi::c_void)
        }
    }

    /// Pop an item
    pub fn pop(&self) -> Option<T> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        self.queue.pop().map(|ptr| unsafe {
            let typed_ptr = ptr as *mut T;
            let item = core::ptr::read(typed_ptr);
            core::alloc::dealloc(
                typed_ptr as *mut u8,
                core::alloc::Layout::new::<T>(),
            );
            item
        })
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Typed SPSC queue wrapper
pub struct TypedSpscQueue<T> {
    queue: SpscQueue,
    _marker: core::marker::PhantomData<T>,
}

impl<T> TypedSpscQueue<T> {
    /// Create a new typed SPSC queue
    pub fn new(capacity: usize) -> Result<Self, ()> {
        SpscQueue::new(capacity).map(|queue| Self {
            queue,
            _marker: core::marker::PhantomData,
        })
    }

    /// Push an item
    pub fn push(&self, item: T) -> Result<(), ()> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let boxed = unsafe { core::alloc::alloc(core::alloc::Layout::new::<T>()) as *mut T };
        if boxed.is_null() {
            return Err(());
        }
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write(boxed, item);
            self.queue.push(boxed as *mut core::ffi::c_void)
        }
    }

    /// Pop an item
    pub fn pop(&self) -> Option<T> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        self.queue.pop().map(|ptr| unsafe {
            let typed_ptr = ptr as *mut T;
            let item = core::ptr::read(typed_ptr);
            core::alloc::dealloc(
                typed_ptr as *mut u8,
                core::alloc::Layout::new::<T>(),
            );
            item
        })
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Check if full
    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Typed lock-free stack wrapper
pub struct TypedLfStack<T> {
    stack: LfStack,
    _marker: core::marker::PhantomData<T>,
}

impl<T> TypedLfStack<T> {
    /// Create a new typed stack
    pub fn new() -> Result<Self, ()> {
        LfStack::new().map(|stack| Self {
            stack,
            _marker: core::marker::PhantomData,
        })
    }

    /// Push an item
    pub fn push(&self, item: T) -> Result<(), ()> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let boxed = unsafe { core::alloc::alloc(core::alloc::Layout::new::<T>()) as *mut T };
        if boxed.is_null() {
            return Err(());
        }
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write(boxed, item);
            self.stack.push(boxed as *mut core::ffi::c_void)
        }
    }

    /// Pop an item
    pub fn pop(&self) -> Option<T> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        self.stack.pop().map(|ptr| unsafe {
            let typed_ptr = ptr as *mut T;
            let item = core::ptr::read(typed_ptr);
            core::alloc::dealloc(
                typed_ptr as *mut u8,
                core::alloc::Layout::new::<T>(),
            );
            item
        })
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpsc_queue_new() {
        let queue = MpscQueue::new();
        assert!(queue.is_ok());
        let queue = queue.unwrap();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_spsc_queue_new() {
        let queue = SpscQueue::new(16);
        assert!(queue.is_ok());
        let queue = queue.unwrap();
        assert!(queue.is_empty());
        assert!(!queue.is_full());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_lf_stack_new() {
        let stack = LfStack::new();
        assert!(stack.is_ok());
        let stack = stack.unwrap();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_mpmc_queue_new() {
        let queue = MpmcQueue::new(16);
        assert!(queue.is_ok());
        let queue = queue.unwrap();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }
}
