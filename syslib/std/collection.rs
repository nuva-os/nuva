/*
 * Nuva OS - SystemLibrary - Std
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

//! SetType

use alloc::boxed::Box;
use alloc::alloc::{alloc, dealloc, Layout};


/// DynamicArray
#[derive(Debug)]
pub struct Vec<T> {
 data: *mut T,
 len: usize,
 capacity: usize,
}

impl<T> Vec<T> {
 pub fn new() -> Self {
 Self {
 data: core::ptr::null_mut(),
 len: 0,
 capacity: 0,
 }
 }

 pub fn with_capacity(capacity: usize) -> Self {
 let layout = Layout::array::<T>(capacity).unwrap();
 // SAFETY: unsafe block required for low-level memory or hardware access
 let data = unsafe { alloc(layout) as *mut T };
 
 Self {
 data,
 len: 0,
 capacity,
 }
 }

 pub fn push(&mut self, value: T) {
 if self.len == self.capacity {
 self.grow();
 }
 
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 self.data.add(self.len).write(value);
 }
 self.len += 1;
 }

 pub fn pop(&mut self) -> Option<T> {
 if self.len > 0 {
 self.len -= 1;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { Some(self.data.add(self.len).read()) }
 } else {
 None
 }
 }

 pub fn get(&self, index: usize) -> Option<&T> {
 if index < self.len {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { Some(&*self.data.add(index)) }
 } else {
 None
 }
 }

 pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
 if index < self.len {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { Some(&mut *self.data.add(index)) }
 } else {
 None
 }
 }

 pub fn len(&self) -> usize {
 self.len
 }

 pub fn is_empty(&self) -> bool {
 self.len == 0
 }

 pub fn capacity(&self) -> usize {
 self.capacity
 }

 fn grow(&mut self) {
 let new_capacity = if self.capacity == 0 { 4 } else { self.capacity * 2 };
 
 let new_layout = Layout::array::<T>(new_capacity).unwrap();
 // SAFETY: unsafe block required for low-level memory or hardware access
 let new_data = unsafe { alloc(new_layout) as *mut T };
 
 if !self.data.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 for i in 0..self.len {
 new_data.add(i).write(self.data.add(i).read());
 }
 
 let old_layout = Layout::array::<T>(self.capacity).unwrap();
 dealloc(self.data as *mut u8, old_layout);
 }
 }
 
 self.data = new_data;
 self.capacity = new_capacity;
 }

 pub fn as_slice(&self) -> &[T] {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { core::slice::from_raw_parts(self.data, self.len) }
 }

 pub fn as_slice_mut(&mut self) -> &mut [T] {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { core::slice::from_raw_parts_mut(self.data, self.len) }
 }
}

impl<T> Drop for Vec<T> {
 fn drop(&mut self) {
 if !self.data.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 for i in 0..self.len {
 self.data.add(i).drop_in_place();
 }
 
 let layout = Layout::array::<T>(self.capacity).unwrap();
 dealloc(self.data as *mut u8, layout);
 }
 }
 }
}

/// linkformNode
struct ListNode<T> {
 value: T,
 next: *mut ListNode<T>,
 prev: *mut ListNode<T>,
}

/// Two-waylinkform
pub struct LinkedList<T> {
 head: *mut ListNode<T>,
 tail: *mut ListNode<T>,
 len: usize,
}

impl<T> LinkedList<T> {
 pub fn new() -> Self {
 Self {
 head: core::ptr::null_mut(),
 tail: core::ptr::null_mut(),
 len: 0,
 }
 }

 pub fn push_front(&mut self, value: T) {
 let node = Box::into_raw(Box::new(ListNode {
 value,
 next: self.head,
 prev: core::ptr::null_mut(),
 }));
 
 if !self.head.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { (*self.head).prev = node; }
 } else {
 self.tail = node;
 }
 
 self.head = node;
 self.len += 1;
 }

 pub fn push_back(&mut self, value: T) {
 let node = Box::into_raw(Box::new(ListNode {
 value,
 next: core::ptr::null_mut(),
 prev: self.tail,
 }));
 
 if !self.tail.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { (*self.tail).next = node; }
 } else {
 self.head = node;
 }
 
 self.tail = node;
 self.len += 1;
 }

 pub fn pop_front(&mut self) -> Option<T> {
 if self.head.is_null() {
 return None;
 }
 
 let node = self.head;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 self.head = (*node).next;
 if !self.head.is_null() {
 (*self.head).prev = core::ptr::null_mut();
 } else {
 self.tail = core::ptr::null_mut();
 }

        let value = core::ptr::read(&(*node).value);
        Box::from_raw(node);
 self.len -= 1;
 Some(value)
 }
 }

 pub fn pop_back(&mut self) -> Option<T> {
 if self.tail.is_null() {
 return None;
 }
 
 let node = self.tail;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 self.tail = (*node).prev;
 if !self.tail.is_null() {
 (*self.tail).next = core::ptr::null_mut();
 } else {
 self.head = core::ptr::null_mut();
 }

        let value = core::ptr::read(&(*node).value);
        Box::from_raw(node);
 self.len -= 1;
 Some(value)
 }
 }

 pub fn len(&self) -> usize {
 self.len
 }

 pub fn is_empty(&self) -> bool {
 self.len == 0
 }
}

impl<T> Drop for LinkedList<T> {
 fn drop(&mut self) {
 while self.pop_front().is_some() {}
 }
}

/// Hashform
pub struct HashMap<K, V> {
 buckets: Vec<Option<(K, V)>>,
 size: usize,
}

impl<K: PartialEq, V> HashMap<K, V> {
 pub fn new() -> Self {
 Self {
 buckets: Vec::new(),
 size: 0,
 }
 }

 pub fn with_capacity(capacity: usize) -> Self {
 let mut buckets = Vec::with_capacity(capacity);
 for _ in 0..capacity {
 buckets.push(None);
 }
 
 Self {
 buckets,
 size: 0,
 }
 }

 pub fn insert(&mut self, key: K, value: V) -> Option<V> {
 if self.buckets.is_empty() {
 self.buckets = Vec::with_capacity(16);
 for _ in 0..16 {
 self.buckets.push(None);
 }
 }
 
 let hash = self.hash(&key);
 let index = hash % self.buckets.len();
 
 if let Some(ref mut slot) = self.buckets.as_slice_mut()[index] {
 if slot.0 == key {
 let old = core::mem::replace(&mut slot.1, value);
 return Some(old);
 }
 }
 
 self.buckets.as_slice_mut()[index] = Some((key, value));
 self.size += 1;
 None
 }

 pub fn get(&self, key: &K) -> Option<&V> {
 if self.buckets.is_empty() {
 return None;
 }
 
 let hash = self.hash(key);
 let index = hash % self.buckets.len();
 
 if let Some(ref slot) = self.buckets.as_slice()[index] {
 if &slot.0 == key {
 return Some(&slot.1);
 }
 }
 None
 }

 pub fn remove(&mut self, key: &K) -> Option<V> {
 if self.buckets.is_empty() {
 return None;
 }
 
 let hash = self.hash(key);
 let index = hash % self.buckets.len();
 
 if let Some(slot) = self.buckets.as_slice_mut()[index].take() {
 if &slot.0 == key {
 self.size -= 1;
 return Some(slot.1);
 } else {
 self.buckets.as_slice_mut()[index] = Some(slot);
 }
 }
 None
 }

 pub fn len(&self) -> usize {
 self.size
 }

 pub fn is_empty(&self) -> bool {
 self.size == 0
 }

 fn hash(&self, key: &K) -> usize {
 let ptr = key as *const K as usize;
 ptr.wrapping_mul(31)
 }
}

/// Queue
pub struct Queue<T> {
 data: Vec<T>,
}

impl<T> Queue<T> {
 pub fn new() -> Self {
 Self { data: Vec::new() }
 }

 pub fn enqueue(&mut self, value: T) {
 self.data.push(value);
 }

 pub fn dequeue(&mut self) -> Option<T> {
 if self.data.is_empty() {
 return None;
 }
 
 // SAFETY: unsafe block required for low-level memory or hardware access
 let value = unsafe { core::ptr::read(self.data.as_slice().as_ptr()) };
 // Moveprime
 for i in 1..self.data.len() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let src = core::ptr::read(self.data.as_slice().as_ptr().add(i));
 core::ptr::write(self.data.as_slice_mut().as_mut_ptr().add(i - 1), src);
 }
 }
 self.data.pop();
 Some(value)
 }

 pub fn peek(&self) -> Option<&T> {
 self.data.get(0)
 }

 pub fn len(&self) -> usize {
 self.data.len()
 }

 pub fn is_empty(&self) -> bool {
 self.data.is_empty()
 }
}

/// Stack
pub struct Stack<T> {
 data: Vec<T>,
}

impl<T> Stack<T> {
 pub fn new() -> Self {
 Self { data: Vec::new() }
 }

 pub fn push(&mut self, value: T) {
 self.data.push(value);
 }

 pub fn pop(&mut self) -> Option<T> {
 self.data.pop()
 }

 pub fn peek(&self) -> Option<&T> {
 if self.data.is_empty() {
 None
 } else {
 self.data.get(self.data.len() - 1)
 }
 }

 pub fn len(&self) -> usize {
 self.data.len()
 }

 pub fn is_empty(&self) -> bool {
 self.data.is_empty()
 }
}