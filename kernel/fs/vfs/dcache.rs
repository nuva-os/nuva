/*
 * Nuva OS - Kernel - Fs
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

use super::inode::Inode;
use core::ptr;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

// Directory entry state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DentryState {
    // Unused
    Unused = 0,
    // In use
    InUse = 1,
    // Negative caching (file does not exist)
    Negative = 2,
}

// Directory entry flags
pub mod dentry_flags {
    pub const DCACHE_UNHASHED: u32 = 0x0001;
    pub const DCACHE_REFERENCED: u32 = 0x0002;
    pub const DCACHE_DIRTY: u32 = 0x0004;
    pub const DCACHE_OP_INVALIDATE: u32 = 0x0008;
    pub const DCACHE_OP_PRUNE: u32 = 0x0010;
    pub const DCACHE_OP_REVALIDATE: u32 = 0x0020;
    pub const DCACHE_OP_DELETE: u32 = 0x0040;
    pub const DCACHE_MOUNTPOINT: u32 = 0x0080;
    pub const DCACHE_NEED_AUTOMOUNT: u32 = 0x0100;
    pub const DCACHE_MANAGE_TRANSIT: u32 = 0x0200;
    pub const DCACHE_MANAGED_DENTRY: u32 =
        DCACHE_MOUNTPOINT | DCACHE_NEED_AUTOMOUNT | DCACHE_MANAGE_TRANSIT;
    pub const DCACHE_LRU_LIST: u32 = 0x0400;
    pub const DCACHE_ENTRY_TYPE: u32 = 0x0800;
    pub const DCACHE_FALLTHRU: u32 = 0x1000;
    pub const DCACHE_ENCRYPTED_NAME: u32 = 0x2000;
    pub const DCACHE_MAY_FREE: u32 = 0x4000;
}

// Directory entry operations
pub struct DentryOperations {
    // Revalidate
    pub d_revalidate: fn(dentry: &Dentry, flags: u32) -> i32,
    // Hash
    pub d_hash: fn(dentry: &Dentry, name: &str) -> u64,
    /// Compare
    pub d_compare: fn(dentry: &Dentry, name1: &str, name2: &str) -> i32,
    /// Delete
    pub d_delete: fn(dentry: &Dentry) -> i32,
    /// Free
    pub d_release: fn(dentry: &Dentry),
    // Invalidate
    pub d_invalidate: fn(dentry: &Dentry) -> i32,
    /// Initialize
    pub d_init: fn(dentry: &Dentry),
}

// Directory entry
pub struct Dentry {
    // Reference count
    pub d_count: AtomicU32,
    /// Flag
    pub d_flags: AtomicU32,
    /// Lock
    pub d_lock: u64,
    /// Inode
    pub d_inode: *mut Inode,
    /// ParentDirectory
    pub d_parent: *mut Dentry,
    /// Name
    pub d_name: DentryName,
    // Hash value
    pub d_hash: u64,
    /// File SystemData
    pub d_fsdata: u64,
    // Directory entry operations
    pub d_op: &'static DentryOperations,
    // File system
    pub d_sb: u64,
    /// Timestamp
    pub d_time: u64,
    // LRU list
    pub d_lru: *mut Dentry,
    // Hash list
    pub d_hash_next: *mut Dentry,
    pub d_hash_prev: *mut Dentry,
    // Child directory list
    pub d_subdirs: *mut Dentry,
    // Sibling list
    pub d_child: *mut Dentry,
}

// Directory entryName
pub struct DentryName {
    /// NameLength
    pub len: u32,
    // Name hash
    pub hash: u32,
    /// NameData
    pub name: [u8; 256],
}

impl DentryName {
    pub const fn new() -> Self {
        DentryName {
            len: 0,
            hash: 0,
            name: [0; 256],
        }
    }

    /// SetName
    pub fn set_name(&mut self, name: &str) {
        let bytes = name.as_bytes();
        let len = bytes.len().min(self.name.len());
        self.name[..len].copy_from_slice(&bytes[..len]);
        self.len = len as u32;
        self.hash = self.calc_hash();
    }

    /// GetName
    pub fn get_name(&self) -> &str {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::str::from_utf8_unchecked(&self.name[..self.len as usize]) }
    }

    // Calculate hash
    fn calc_hash(&self) -> u32 {
        let mut hash: u32 = 0;
        for i in 0..self.len as usize {
            hash = hash.wrapping_mul(31).wrapping_add(self.name[i] as u32);
        }
        hash
    }
}

impl Dentry {
    pub const fn new() -> Self {
        Dentry {
            d_count: AtomicU32::new(0),
            d_flags: AtomicU32::new(0),
            d_lock: 0,
            d_inode: ptr::null_mut(),
            d_parent: ptr::null_mut(),
            d_name: DentryName::new(),
            d_hash: 0,
            d_fsdata: 0,
            d_op: &DENTRY_OPS_NONE,
            d_sb: 0,
            d_time: 0,
            d_lru: ptr::null_mut(),
            d_hash_next: ptr::null_mut(),
            d_hash_prev: ptr::null_mut(),
            d_subdirs: ptr::null_mut(),
            d_child: ptr::null_mut(),
        }
    }

    // Increase reference count
    pub fn inc_count(&self) {
        self.d_count.fetch_add(1, Ordering::Relaxed);
    }

    // Decrease reference count
    pub fn dec_count(&self) -> u32 {
        self.d_count.fetch_sub(1, Ordering::Relaxed)
    }

    // Check if negative caching
    pub fn is_negative(&self) -> bool {
        self.d_inode.is_null()
    }

    // Check if mount point
    pub fn is_mountpoint(&self) -> bool {
        (self.d_flags.load(Ordering::Acquire) & dentry_flags::DCACHE_MOUNTPOINT) != 0
    }
}

// Empty directory entry operations
static DENTRY_OPS_NONE: DentryOperations = DentryOperations {
    d_revalidate: |_dentry, _flags| 0,
    d_hash: |_dentry, _name| 0,
    d_compare: |_dentry, _name1, _name2| 0,
    d_delete: |_dentry| 0,
    d_release: |_dentry| {},
    d_invalidate: |_dentry| 0,
    d_init: |_dentry| {},
};

// Directory entryCaching
pub struct Dcache {
    // Hash table
    hash_table: [*mut Dentry; 1024],
    // LRU list
    lru_list: *mut Dentry,
    // Directory entry count
    count: AtomicU64,
}

impl Dcache {
    pub const fn new() -> Self {
        Dcache {
            hash_table: [ptr::null_mut(); 1024],
            lru_list: ptr::null_mut(),
            count: AtomicU64::new(0),
        }
    }

    // Find directory entry
    pub fn lookup(&self, parent: &Dentry, name: &str) -> Option<&Dentry> {
        let hash = self.calc_hash(parent, name);
        let idx = (hash % 1024) as usize;

        let mut dentry = self.hash_table[idx];
        while !dentry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*dentry).d_parent == parent as *const Dentry as *mut Dentry {
                    if (*dentry).d_name.get_name() == name {
                        return Some(&*dentry);
                    }
                }
                dentry = (*dentry).d_hash_next;
            }
        }

        None
    }

    // Insert directory entry
    pub fn insert(&mut self, dentry: *mut Dentry) {
        if dentry.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let hash = (*dentry).d_hash;
            let idx = (hash % 1024) as usize;

            (*dentry).d_hash_next = self.hash_table[idx];
            (*dentry).d_hash_prev = ptr::null_mut();

            if !self.hash_table[idx].is_null() {
                (*self.hash_table[idx]).d_hash_prev = dentry;
            }

            self.hash_table[idx] = dentry;
        }

        self.count.fetch_add(1, Ordering::Relaxed);
    }

    // Delete directory entry
    pub fn remove(&mut self, dentry: *mut Dentry) {
        if dentry.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if !(*dentry).d_hash_prev.is_null() {
                (*(*dentry).d_hash_prev).d_hash_next = (*dentry).d_hash_next;
            } else {
                let hash = (*dentry).d_hash;
                let idx = (hash % 1024) as usize;
                self.hash_table[idx] = (*dentry).d_hash_next;
            }

            if !(*dentry).d_hash_next.is_null() {
                (*(*dentry).d_hash_next).d_hash_prev = (*dentry).d_hash_prev;
            }
        }

        self.count.fetch_sub(1, Ordering::Relaxed);
    }

    // Calculate hash
    fn calc_hash(&self, parent: &Dentry, name: &str) -> u64 {
        let mut hash: u64 = parent as *const Dentry as u64;
        for byte in name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }
}

// Global directory entry cache
static DCACHE: core::sync::OnceLock<Dcache> = core::sync::OnceLock::new();

pub fn dcache() -> &'static Dcache {
    DCACHE.get_or_init(Dcache::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dentry() {
        let mut dentry = Dentry::new();
        dentry.d_name.set_name("test");

        assert_eq!(dentry.d_name.get_name(), "test");
    }
}
