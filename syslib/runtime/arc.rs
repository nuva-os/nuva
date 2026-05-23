/*
 * Nuva OS - SystemLibrary - Runtime
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

//! ARC (automatic reference counting) runtime

use core::sync::atomic::{AtomicI32, AtomicU32, AtomicPtr, Ordering};
use alloc::alloc::{alloc, dealloc, handle_alloc_error, Layout};

/// Reference Count Header
#[derive(Debug)]
#[repr(C)]
pub struct RefCountHeader {
    /// Strong reference count
    strong_count: AtomicI32,

    /// Weak reference count
    weak_count: AtomicI32,

    /// Type metadata pointer
    metadata: AtomicPtr<u8>,

    /// Flags
    flags: AtomicU32,
}

/// Flag bits
pub const FLAG_DEALLOCATING: u32 = 1 << 0;
pub const FLAG_DEALLOCATED: u32 = 1 << 1;

impl RefCountHeader {
    pub fn new(metadata: *const u8) -> Self {
        Self {
            strong_count: AtomicI32::new(1),
            weak_count: AtomicI32::new(1),
            metadata: AtomicPtr::new(metadata as *mut u8),
            flags: AtomicU32::new(0),
        }
    }

    /// Increment strong reference count
    pub fn retain(&self) -> bool {
        loop {
            let count = self.strong_count.load(Ordering::Acquire);
            if count <= 0 {
                return false;
            }
            if self.strong_count.compare_exchange_weak(
                count,
                count + 1,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                return true;
            }
        }
    }

    /// Decrement strong reference count
    pub fn release(&self) -> bool {
        loop {
            let count = self.strong_count.load(Ordering::Acquire);
            if count <= 0 {
                return false;
            }
            if self.strong_count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                return count == 1;
            }
        }
    }

    /// Increment weak reference count
    pub fn retain_weak(&self) -> bool {
        loop {
            let count = self.weak_count.load(Ordering::Acquire);
            if count <= 0 {
                return false;
            }
            if self.weak_count.compare_exchange_weak(
                count,
                count + 1,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                return true;
            }
        }
    }

    /// Decrement weak reference count
    pub fn release_weak(&self) -> bool {
        loop {
            let count = self.weak_count.load(Ordering::Acquire);
            if count <= 0 {
                return false;
            }
            if self.weak_count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                return count == 1;
            }
        }
    }

    /// Get strong reference count
    pub fn strong_count(&self) -> i32 {
        self.strong_count.load(Ordering::Acquire)
    }

    /// Get weak reference count
    pub fn weak_count(&self) -> i32 {
        self.weak_count.load(Ordering::Acquire)
    }

    /// Mark as deallocating
    pub fn set_deallocating(&self) {
        self.flags.fetch_or(FLAG_DEALLOCATING, Ordering::Release);
    }

    /// Check if currently deallocating
    pub fn is_deallocating(&self) -> bool {
        self.flags.load(Ordering::Acquire) & FLAG_DEALLOCATING != 0
    }

    /// Mark as already deallocated
    pub fn set_deallocated(&self) {
        self.flags.fetch_or(FLAG_DEALLOCATED, Ordering::Release);
    }

    /// Check if already deallocated
    pub fn is_deallocated(&self) -> bool {
        self.flags.load(Ordering::Acquire) & FLAG_DEALLOCATED != 0
    }
}

/// ARC pointer
#[derive(Debug)]
#[repr(transparent)]
pub struct ArcPtr<T> {
    ptr: *mut T,
}

impl<T> ArcPtr<T> {
    /// Create from a raw pointer
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self { ptr }
    }

    /// Get the reference count header
    fn header(&self) -> &RefCountHeader {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            &*(self.ptr as *const RefCountHeader).sub(1)
        }
    }

    /// Increment reference count
    pub fn retain(&self) -> Self {
        self.header().retain();
        Self { ptr: self.ptr }
    }

    /// Decrement reference count
    pub fn release(&self) {
        if self.header().release() {
            self.deallocate();
        }
    }

    /// Free memory
    fn deallocate(&self) {
        self.header().set_deallocating();

        // Call destructor
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::drop_in_place(self.ptr);
        }

        // Free weak reference
        if self.header().release_weak() {
            // Free memory
            let layout = Layout::new::<RefCountHeader>();
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                dealloc(
                    self.header() as *const RefCountHeader as *mut u8,
                    layout,
                );
            }
        } else {
            self.header().set_deallocated();
        }
    }

    /// Get a reference to the value
    pub fn get(&self) -> &T {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { &*self.ptr }
    }

    /// Get a mutable reference to the value
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.header().strong_count() == 1 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { Some(&mut *self.ptr) }
        } else {
            None
        }
    }

    /// Convert to raw pointer
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }
}

impl<T> Clone for ArcPtr<T> {
    fn clone(&self) -> Self {
        self.retain()
    }
}

impl<T> Drop for ArcPtr<T> {
    fn drop(&mut self) {
        self.release();
    }
}

/// Weak reference pointer
#[derive(Debug)]
#[repr(transparent)]
pub struct WeakPtr<T> {
    ptr: *mut T,
}

impl<T> WeakPtr<T> {
    /// Create from an ArcPtr
    pub fn from_arc(arc: &ArcPtr<T>) -> Self {
        arc.header().retain_weak();
        Self { ptr: arc.ptr }
    }

    /// Get the reference count header
    fn header(&self) -> &RefCountHeader {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            &*(self.ptr as *const RefCountHeader).sub(1)
        }
    }

    /// Try to upgrade to a strong reference
    pub fn upgrade(&self) -> Option<ArcPtr<T>> {
        if self.header().is_deallocated() {
            return None;
        }

        if self.header().retain() {
            Some(ArcPtr { ptr: self.ptr })
        } else {
            None
        }
    }

    /// Release weak reference
    pub fn release(&self) {
        if self.header().release_weak() {
            // Free memory
            let layout = Layout::new::<RefCountHeader>();
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                dealloc(
                    self.header() as *const RefCountHeader as *mut u8,
                    layout,
                );
            }
        }
    }
}

impl<T> Clone for WeakPtr<T> {
    fn clone(&self) -> Self {
        self.header().retain_weak();
        Self { ptr: self.ptr }
    }
}

impl<T> Drop for WeakPtr<T> {
    fn drop(&mut self) {
        self.release();
    }
}

/// ARC Memory Allocator
pub struct ArcAllocator;

impl ArcAllocator {
    /// Allocate ARC managed memory
    pub fn alloc<T>(value: T, metadata: *const u8) -> ArcPtr<T> {
        let header_layout = Layout::new::<RefCountHeader>();
        let value_layout = Layout::new::<T>();

        let (layout, _offset) = header_layout.extend(value_layout).unwrap();

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                handle_alloc_error(layout);
            }

            // Initialize reference count header
            let header = ptr as *mut RefCountHeader;
            core::ptr::write(header, RefCountHeader::new(metadata));

            // Write value
            let value_ptr = header.add(1) as *mut T;
            core::ptr::write(value_ptr, value);

            ArcPtr::from_raw(value_ptr)
        }
    }
}

/// ARC Statistics
pub struct ArcStats {
    pub total_allocations: AtomicU32,
    pub total_deallocations: AtomicU32,
    pub current_live: AtomicU32,
    pub total_retains: AtomicU32,
    pub total_releases: AtomicU32,
}

impl ArcStats {
    pub const fn new() -> Self {
        Self {
            total_allocations: AtomicU32::new(0),
            total_deallocations: AtomicU32::new(0),
            current_live: AtomicU32::new(0),
            total_retains: AtomicU32::new(0),
            total_releases: AtomicU32::new(0),
        }
    }

    pub fn record_alloc(&self) {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.current_live.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dealloc(&self) {
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.current_live.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_retain(&self) {
        self.total_retains.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_release(&self) {
        self.total_releases.fetch_add(1, Ordering::Relaxed);
    }
}

/// Global ARC Statistics
pub static ARC_STATS: ArcStats = ArcStats::new();
