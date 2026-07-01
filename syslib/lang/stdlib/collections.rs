/*
 * Nuva OS - Syslib - Lang - Stdlib - Collections
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
 * Nuva OS - System Library - Lang - Collections
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Collection data structures for the Nuva language runtime.
 */

use alloc::alloc::{alloc, dealloc, Layout};
use alloc::boxed::Box;
use core::hash::Hash;
use alloc::vec::Vec;

/// Dynamic array
pub struct Vec<T> {
    /// Data pointer
    data: *mut T,
    /// Length
    len: usize,
    /// Capacity
    capacity: usize,
}

impl<T> Vec<T> {
    /// Create a new empty vector
    pub const fn new() -> Self {
        Vec {
            data: core::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }
    
    /// Create a vector with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let data = if capacity > 0 {
            let layout = Layout::array::<T>(capacity).unwrap_or_else(|_| Layout::new::<T>());
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { alloc(layout) as *mut T }
        } else {
            core::ptr::null_mut()
        };
        Vec {
            data,
            len: 0,
            capacity,
        }
    }
    
    /// Get the length
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Add an element to the end
    pub fn push(&mut self, value: T) {
        // 1. Check capacity and grow if needed
        if self.len >= self.capacity {
            let new_cap = if self.capacity == 0 { 4 } else { self.capacity * 2 };
            self.grow(new_cap);
        }
        
        // 2. Write the element
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write(self.data.add(self.len), value);
        }
        self.len += 1;
    }
    
    /// Remove and return the last element
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        
        self.len -= 1;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { Some(core::ptr::read(self.data.add(self.len))) }
    }
    
    /// Get an element by index
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len || self.data.is_null() {
            return None;
        }
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { Some(&*self.data.add(index)) }
    }
    
    /// Get a mutable element by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len || self.data.is_null() {
            return None;
        }
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { Some(&mut *self.data.add(index)) }
    }
    
    /// Clear all elements (does not free memory)
    pub fn clear(&mut self) {
        self.len = 0;
    }
    
    /// Grow the buffer to the new capacity
    fn grow(&mut self, new_cap: usize) {
        if new_cap <= self.capacity {
            return;
        }
        
        let new_layout = Layout::array::<T>(new_cap).unwrap_or_else(|_| Layout::new::<T>());
        // SAFETY: unsafe block required for low-level memory or hardware access
        let new_data = unsafe { alloc(new_layout) as *mut T };
        
        if new_data.is_null() {
            return; // Allocation failed
        }
        
        // Copy existing elements
        if !self.data.is_null() && self.len > 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::copy_nonoverlapping(self.data, new_data, self.len);
            }
            // Free old buffer
            if self.capacity > 0 {
                let old_layout = Layout::array::<T>(self.capacity).unwrap_or_else(|_| Layout::new::<T>());
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe { dealloc(self.data as *mut u8, old_layout); }
            }
        }
        
        self.data = new_data;
        self.capacity = new_cap;
    }
}

impl<T> Vec<T> {
    /// Iterate over elements by reference
    pub fn iter(&self) -> VecIter<'_, T> {
        VecIter {
            data: self.data,
            len: self.len,
            pos: 0,
            _marker: core::marker::PhantomData,
        }
    }

    /// Iterate over elements by mutable reference
    pub fn iter_mut(&mut self) -> VecIterMut<'_, T> {
        VecIterMut {
            data: self.data,
            len: self.len,
            pos: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

/// Iterator over Vec by reference
pub struct VecIter<'a, T> {
    data: *mut T,
    len: usize,
    pos: usize,
    _marker: core::marker::PhantomData<&'a T>,
}

impl<'a, T> core::iter::Iterator for VecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len || self.data.is_null() {
            return None;
        }
        // SAFETY: pos is within bounds [0, len)
        let item = unsafe { &*self.data.add(self.pos) };
        self.pos += 1;
        Some(item)
    }
}

/// Iterator over Vec by mutable reference
pub struct VecIterMut<'a, T> {
    data: *mut T,
    len: usize,
    pos: usize,
    _marker: core::marker::PhantomData<&'a mut T>,
}

impl<'a, T> core::iter::Iterator for VecIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len || self.data.is_null() {
            return None;
        }
        // SAFETY: pos is within bounds [0, len)
        let item = unsafe { &mut *self.data.add(self.pos) };
        self.pos += 1;
        Some(item)
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if !self.data.is_null() && self.capacity > 0 {
            // Drop all elements
            for i in 0..self.len {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe { core::ptr::drop_in_place(self.data.add(i)); }
            }
            // Free the buffer
            let layout = Layout::array::<T>(self.capacity).unwrap_or_else(|_| Layout::new::<T>());
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { dealloc(self.data as *mut u8, layout); }
        }
    }
}

/// String type
pub struct String {
    /// Byte array
    bytes: Vec<u8>,
}

impl String {
    /// Create a new empty string
    pub const fn new() -> Self {
        String { bytes: Vec::new() }
    }
    
    /// Create from a string literal
    pub fn from(s: &str) -> Self {
        let mut bytes = Vec::with_capacity(s.len());
        for &b in s.as_bytes() {
            bytes.push(b);
        }
        String { bytes }
    }
    
    /// Get the length
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    
    /// Add a character
    pub fn push(&mut self, ch: char) {
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        for &b in s.as_bytes() {
            self.bytes.push(b);
        }
    }
    
    /// Add a string slice
    pub fn push_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            self.bytes.push(b);
        }
    }
    
    /// Get as a string slice
    pub fn as_str(&self) -> &str {
        if self.bytes.data.is_null() || self.bytes.len == 0 {
            ""
        } else {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.bytes.data, self.bytes.len)) }
        }
    }
}

/// Hash map using separate chaining for collision resolution
pub struct HashMap<K, V> {
    /// Bucket array, each bucket holds a linked list of entries
    buckets: Vec<Option<Box<HashMapNode<K, V>>>>,
    /// Number of elements
    len: usize,
}

/// Linked list node for hash map chaining
struct HashMapNode<K, V> {
    /// Key
    key: K,
    /// Value
    value: V,
    /// Hash of the key
    hash: u64,
    /// Next node in the chain
    next: Option<Box<HashMapNode<K, V>>>,
}

impl<K: Hash + PartialEq, V> HashMap<K, V> {
    /// Create a new empty hash map
    pub const fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            len: 0,
        }
    }

    /// Compute the hash value for a key
    fn hash_key(key: &K) -> u64 {
        let mut hasher = core::hash::SipHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Find the bucket index for a hash value
    fn bucket_index(&self, hash: u64) -> usize {
        let num_buckets = self.buckets.len();
        if num_buckets == 0 {
            return 0;
        }
        (hash as usize) % num_buckets
    }

    /// Ensure buckets are allocated with minimum capacity
    fn ensure_capacity(&mut self) {
        if self.buckets.is_empty() {
            for _ in 0..16 {
                self.buckets.push(None);
            }
        } else if self.len * 3 > self.buckets.len() * 2 {
            // Load factor > 0.66, resize to double
            let new_cap = self.buckets.len() * 2;
            // Collect all entries from old buckets
            let mut all_entries: Vec<Option<Box<HashMapNode<K, V>>>> = Vec::with_capacity(self.len);
            for i in 0..self.buckets.len() {
                if let Some(entry) = self.buckets.get_mut(i) {
                    if let Some(node) = entry.take() {
                        all_entries.push(Some(node));
                    }
                }
            }
            // Resize bucket array
            self.buckets.clear();
            for _ in 0..new_cap {
                self.buckets.push(None);
            }
            // Rehash all entries
            for i in 0..all_entries.len() {
                if let Some(mut node) = all_entries.get_mut(i).and_then(|e| e.take()) {
                    let idx = (node.hash as usize) % new_cap;
                    node.next = self.buckets.get_mut(idx).and_then(|b| b.take());
                    if let Some(bucket) = self.buckets.get_mut(idx) {
                        *bucket = Some(node);
                    }
                }
            }
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Insert a key-value pair, returning the old value if present
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.ensure_capacity();
        let hash = Self::hash_key(&key);
        let idx = self.bucket_index(hash);

        // Search for existing key in the chain and replace if found
        if let Some(bucket) = self.buckets.get_mut(idx) {
            let mut current = bucket.as_mut();
            while let Some(node) = current {
                if node.hash == hash && node.key == key {
                    let old = core::mem::replace(&mut node.value, value);
                    return Some(old);
                }
                current = node.next.as_mut().map(|n| n.as_mut());
            }
        }

        // Key not found, insert new node at the head of the chain
        let new_node = Box::new(HashMapNode {
            key,
            value,
            hash,
            next: None,
        });

        if let Some(bucket) = self.buckets.get_mut(idx) {
            let mut node = new_node;
            node.next = bucket.take();
            *bucket = Some(node);
        }
        self.len += 1;
        None
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.buckets.is_empty() {
            return None;
        }
        let hash = Self::hash_key(key);
        let idx = self.bucket_index(hash);

        let mut current = self.buckets.get(idx).and_then(|opt| opt.as_ref());
        while let Some(node) = current {
            if node.hash == hash && &node.key == key {
                return Some(&node.value);
            }
            current = node.next.as_ref().map(|n| n.as_ref());
        }
        None
    }

    /// Remove a key-value pair, returning the value if present
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if self.buckets.is_empty() {
            return None;
        }
        let hash = Self::hash_key(key);
        let idx = self.bucket_index(hash);

        // Check head of chain
        let head = self.buckets.get_mut(idx).and_then(|opt| opt.take());
        if let Some(mut node) = head {
            if node.hash == hash && &node.key == key {
                let value = node.value;
                // Put the rest of the chain back
                if let Some(bucket) = self.buckets.get_mut(idx) {
                    *bucket = node.next.take();
                }
                self.len -= 1;
                return Some(value);
            }
            // Not at head, search the chain
            let mut prev: Option<&mut HashMapNode<K, V>> = Some(&mut *node);
            let mut current = node.next.take();
            while let Some(mut curr) = current {
                if curr.hash == hash && &curr.key == key {
                    // Found, unlink curr from chain
                    let value = curr.value;
                    let rest = curr.next.take();
                    if let Some(p) = prev {
                        p.next = rest;
                    }
                    // Restore head
                    if let Some(bucket) = self.buckets.get_mut(idx) {
                        *bucket = Some(node);
                    }
                    self.len -= 1;
                    return Some(value);
                }
                let rest = curr.next.take();
                prev = Some(&mut *curr);
                current = rest;
            }
            // Not found, restore head
            if let Some(bucket) = self.buckets.get_mut(idx) {
                *bucket = Some(node);
            }
        }
        None
    }
}

/// Linked list node
struct LinkedListNode<T> {
    value: T,
    next: Option<Box<LinkedListNode<T>>>,
}

/// Linked list
pub struct LinkedList<T> {
    head: Option<Box<LinkedListNode<T>>>,
    len: usize,
}

impl<T> LinkedList<T> {
    /// Create a new empty linked list
    pub const fn new() -> Self {
        LinkedList { head: None, len: 0 }
    }
    
    /// Get the length
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Add an element to the front
    pub fn push_front(&mut self, value: T) {
        let new_node = Box::new(LinkedListNode {
            value,
            next: self.head.take(),
        });
        self.head = Some(new_node);
        self.len += 1;
    }
    
    /// Remove and return the front element
    pub fn pop_front(&mut self) -> Option<T> {
        match self.head.take() {
            Some(node) => {
                self.head = node.next;
                self.len -= 1;
                Some(node.value)
            }
            None => None,
        }
    }
    
    /// Get a reference to the front element
    pub fn front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }
}