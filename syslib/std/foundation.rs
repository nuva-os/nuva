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

//! Foundation Module
//!
//! Provides basic types and functionality.

/// Integer types
pub type Int = i64;
pub type UInt = u64;
pub type Int32 = i32;
pub type UInt32 = u32;
pub type Int8 = i8;
pub type UInt8 = u8;

/// Floating point types
pub type Float = f64;
pub type Float32 = f32;

/// Boolean type
pub type Bool = bool;

/// Character type
pub type Char = u32;

/// Optional type
#[derive(Debug, Clone, Copy)]
pub enum Optional<T> {
    Some(T),
    None,
}

impl<T> Optional<T> {
    pub fn is_some(&self) -> bool {
        matches!(self, Optional::Some(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Optional::None)
    }

    pub fn unwrap(self) -> T {
        match self {
            Optional::Some(v) => v,
            Optional::None => panic!("unwrap on None"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Optional::Some(v) => v,
            Optional::None => default,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Optional<U> {
        match self {
            Optional::Some(v) => Optional::Some(f(v)),
            Optional::None => Optional::None,
        }
    }
}

/// Result type
#[derive(Debug, Clone, Copy)]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Result<T, E> {
    pub fn is_ok(&self) -> bool {
        matches!(self, Result::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, Result::Err(_))
    }

    pub fn unwrap(self) -> T {
        match self {
            Result::Ok(v) => v,
            Result::Err(_) => panic!("unwrap on Err"),
        }
    }

    pub fn unwrap_err(self) -> E {
        match self {
            Result::Err(e) => e,
            Result::Ok(_) => panic!("unwrap_err on Ok"),
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result<U, E> {
        match self {
            Result::Ok(v) => Result::Ok(f(v)),
            Result::Err(e) => Result::Err(e),
        }
    }

    pub fn map_err<F, F2: FnOnce(E) -> F>(self, f: F2) -> Result<T, F> {
        match self {
            Result::Ok(v) => Result::Ok(v),
            Result::Err(e) => Result::Err(f(e)),
        }
    }
}

/// String type
#[derive(Debug, Clone)]
pub struct String {
    data: [u8; 256],
    len: u8,
}

impl String {
    pub fn new() -> Self {
        Self {
            data: [0; 256],
            len: 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut data = [0u8; 256];
        let len = bytes.len().min(255);
        data[..len].copy_from_slice(&bytes[..len]);

        Self {
            data,
            len: len as u8,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, byte: u8) {
        if self.len < 255 {
            self.data[self.len as usize] = byte;
            self.len += 1;
        }
    }

    pub fn push_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            self.push(b);
        }
    }
}

impl Default for String {
    fn default() -> Self {
        Self::new()
    }
}

/// Array type
#[derive(Debug, Clone)]
pub struct Array<T, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T: Default + Copy, const N: usize> Array<T, N> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> bool {
        if self.len < N {
            self.data[self.len] = value;
            self.len += 1;
            return true;
        }
        false
    }

    pub fn pop(&mut self) -> Optional<T> {
        if self.len > 0 {
            self.len -= 1;
            Optional::Some(self.data[self.len])
        } else {
            Optional::None
        }
    }

    pub fn get(&self, index: usize) -> Optional<&T> {
        if index < self.len {
            Optional::Some(&self.data[index])
        } else {
            Optional::None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }
}

/// Dictionary type
#[derive(Debug, Clone)]
pub struct Dictionary<K, V, const N: usize> {
    keys: [K; N],
    values: [V; N],
    len: usize,
}

impl<K: Default + Copy + PartialEq, V: Default + Copy, const N: usize> Dictionary<K, V, N> {
    pub fn new() -> Self {
        Self {
            keys: [K::default(); N],
            values: [V::default(); N],
            len: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> bool {
        // Check if already exists
        for i in 0..self.len {
            if self.keys[i] == key {
                self.values[i] = value;
                return true;
            }
        }

        // Add new entry
        if self.len < N {
            self.keys[self.len] = key;
            self.values[self.len] = value;
            self.len += 1;
            return true;
        }
        false
    }

    pub fn get(&self, key: &K) -> Optional<&V> {
        for i in 0..self.len {
            if &self.keys[i] == key {
                return Optional::Some(&self.values[i]);
            }
        }
        Optional::None
    }

    pub fn remove(&mut self, key: &K) -> Optional<V> {
        for i in 0..self.len {
            if &self.keys[i] == key {
                let value = self.values[i];
                // Move following elements
                for j in i..self.len - 1 {
                    self.keys[j] = self.keys[j + 1];
                    self.values[j] = self.values[j + 1];
                }
                self.len -= 1;
                return Optional::Some(value);
            }
        }
        Optional::None
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Range type
#[derive(Debug, Clone, Copy)]
pub struct Range<T> {
    pub start: T,
    pub end: T,
}

impl<T> Range<T> {
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
}

impl Range<usize> {
    pub fn iter(&self) -> RangeIterator {
        RangeIterator {
            current: self.start,
            end: self.end,
        }
    }
}

/// Range iterator
pub struct RangeIterator {
    current: usize,
    end: usize,
}

impl Iterator for RangeIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let value = self.current;
            self.current += 1;
            Some(value)
        } else {
            None
        }
    }
}

/// Timestamp
#[derive(Debug, Clone, Copy)]
pub struct Timestamp {
    pub seconds: u64,
    pub nanos: u32,
}

impl Timestamp {
    /// Get current system time
    /// Use architecture-specific time counters (e.g., x86 TSC or ARM CNTVCT)
    pub fn now() -> Self {
        // SAFETY: Reading time counter is a secure read-only operation
        let (seconds, nanos) = unsafe { Self::get_system_time_raw() };
        Self { seconds, nanos }
    }

    /// Get raw time value from hardware time counter
    /// # Safety
    /// This function performs architecture-specific hardware counter reading:
    /// - x86-64: Read TSC (Time Stamp Counter)
    /// - ARM64: Read CNTVCT (Virtual Timer Count register)
    #[inline]
    unsafe fn get_system_time_raw() -> (u64, u32) {
        #[cfg(target_arch = "x86_64")]
        {
            // Read TSC counter
            let tsc: u64;
            core::arch::asm!(
                "rdtsc",
                out("rax") tsc,
                out("rdx") _,
                options(nomem, nostack),
            );
            // Assume TSC frequency is 1GHz, convert to seconds and nanoseconds
            // Actual frequency should be calibrated via CPUID at boot time
            (tsc / 1_000_000_000, (tsc % 1_000_000_000) as u32)
        }

        #[cfg(target_arch = "aarch64")]
        {
            // Read ARM64 virtual timer counter
            let cntvct: u64;
            core::arch::asm!(
                "mrs {}, cntvct_el0",
                out(reg) cntvct,
                options(nomem, nostack),
            );
            // Assume counter frequency is 1GHz
            (cntvct / 1_000_000_000, (cntvct % 1_000_000_000) as u32)
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            // Unsupported architecture returns zero
            (0, 0)
        }
    }

    pub fn from_seconds(seconds: u64) -> Self {
        Self { seconds, nanos: 0 }
    }
}

/// UUID
#[derive(Debug, Clone, Copy)]
pub struct UUID {
    pub bytes: [u8; 16],
}

impl UUID {
    pub fn new() -> Self {
        Self {
            bytes: [0; 16],
        }
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }
}

/// Error protocol
pub trait Error {
    fn message(&self) -> &[u8];
}

/// Comparable protocol
pub trait Comparable {
    fn compare(&self, other: &Self) -> Ordering;
}

/// Ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    Less,
    Equal,
    Greater,
}

/// Hashable protocol
pub trait Hashable {
    fn hash(&self) -> u64;
}

/// Copyable protocol
pub trait Copyable {}

/// Cloneable protocol
pub trait Cloneable {
    fn clone(&self) -> Self;
}

/// Debuggable protocol
pub trait Debuggable {
    fn debug_string(&self) -> String;
}

/// Serializable protocol
pub trait Serializable {
    fn serialize(&self) -> Result<[u8; 256], ()>;
    fn deserialize(data: &[u8]) -> Result<Self, ()> where Self: Sized;
}
