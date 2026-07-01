/*
 * Nuva OS - Kernel - Directory Cache (dcache)
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

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::ptr;

/// Dcache configuration
pub mod dcache_config {
    /// Hash table size (must be power of 2)
    pub const HASH_SIZE: usize = 8192;
    
    /// Maximum dentry count
    pub const MAX_DENTRY_COUNT: u32 = 65536;
    
    /// Name maximum length
    pub const NAME_MAX: usize = 255;
}

/// Dentry flags
pub mod dentry_flags {
    /// Dentry is valid
    pub const DC_VALID: u32 = 1 << 0;
    
    /// Dentry is negative (deleted)
    pub const DC_NEGATIVE: u32 = 1 << 1;
    
    /// Dentry is referenced
    pub const DC_REFERENCED: u32 = 1 << 2;
    
    /// Dentry is unhashed
    pub const DC_UNHASHED: u32 = 1 << 3;
    
    /// Dentry is mounted
    pub const DC_MOUNTED: u32 = 1 << 4;
    
    /// Dentry is root
    pub const DC_ROOT: u32 = 1 << 5;
}

/// Qstr - Quick string for dentry name
pub struct Qstr {
    /// Hash value
    pub hash: u32,
    /// Name length
    pub len: u32,
    /// Name pointer
    pub name: *const u8,
}

impl Qstr {
    pub const fn new() -> Self {
        Qstr {
            hash: 0,
            len: 0,
            name: ptr::null(),
        }
    }
    
    /// Calculate hash for a name
    pub fn hash_name(name: &[u8]) -> u32 {
        let mut hash: u32 = 0;
        
        // FNV-1a hash
        for &byte in name {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        
        hash
    }
    
    /// Initialize from name
    pub fn init(&mut self, name: &[u8]) {
        self.hash = Self::hash_name(name);
        self.len = name.len() as u32;
        self.name = name.as_ptr();
    }
}

/// Dentry - Directory entry
pub struct Dentry {
    /// Dentry flags
    pub flags: AtomicU32,
    
    /// Reference count
    pub ref_count: AtomicU32,
    
    /// Name
    pub name: Qstr,
    
    /// Inode number
    pub ino: u64,
    
    /// Parent dentry
    pub parent: *mut Dentry,
    
    /// Super block
    pub sb: u64,
    
    /// Hash chain next
    pub hash_next: *mut Dentry,
    
    /// LRU list pointers
    pub lru_prev: *mut Dentry,
    pub lru_next: *mut Dentry,
    
    /// Child list (for directories)
    pub child_head: *mut Dentry,
    pub child_next: *mut Dentry,
    
    /// Mount point
    pub mount: u64,
}

impl Dentry {
    pub const fn new() -> Self {
        Dentry {
            flags: AtomicU32::new(0),
            ref_count: AtomicU32::new(1),
            name: Qstr::new(),
            ino: 0,
            parent: ptr::null_mut(),
            sb: 0,
            hash_next: ptr::null_mut(),
            lru_prev: ptr::null_mut(),
            lru_next: ptr::null_mut(),
            child_head: ptr::null_mut(),
            child_next: ptr::null_mut(),
            mount: 0,
        }
    }
    
    #[inline]
    pub fn is_valid(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & dentry_flags::DC_VALID) != 0
    }
    
    #[inline]
    pub fn is_negative(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & dentry_flags::DC_NEGATIVE) != 0
    }
    
    #[inline]
    pub fn is_mounted(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & dentry_flags::DC_MOUNTED) != 0
    }
    
    #[inline]
    pub fn set_valid(&self) {
        self.flags.fetch_or(dentry_flags::DC_VALID, Ordering::AcqRel);
    }
    
    #[inline]
    pub fn set_negative(&self) {
        self.flags.fetch_or(dentry_flags::DC_NEGATIVE, Ordering::AcqRel);
    }
    
    #[inline]
    pub fn set_mounted(&self) {
        self.flags.fetch_or(dentry_flags::DC_MOUNTED, Ordering::AcqRel);
    }
}

/// Dentry hash key
pub struct DentryKey {
    /// Parent inode
    pub parent_ino: u64,
    /// Name hash
    pub name_hash: u32,
    /// Super block
    pub sb: u64,
}

impl DentryKey {
    pub fn new(parent_ino: u64, name_hash: u32, sb: u64) -> Self {
        DentryKey { parent_ino, name_hash, sb }
    }
    
    /// Calculate hash bucket index
    pub fn hash(&self) -> usize {
        let mut h = self.parent_ino;
        h ^= self.name_hash as u64;
        h ^= self.sb;
        h ^= h >> 32;
        (h as usize) & (dcache_config::HASH_SIZE - 1)
    }
}

/// LRU list for dentry cache
pub struct DentryLruList {
    pub head: *mut Dentry,
    pub tail: *mut Dentry,
    pub count: AtomicU32,
}

impl DentryLruList {
    pub const fn new() -> Self {
        DentryLruList {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            count: AtomicU32::new(0),
        }
    }
    
    pub fn add_tail(&mut self, dentry: *mut Dentry) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*dentry).lru_prev = self.tail;
            (*dentry).lru_next = ptr::null_mut();
            
            if !self.tail.is_null() {
                (*self.tail).lru_next = dentry;
            } else {
                self.head = dentry;
            }
            self.tail = dentry;
            
            self.count.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    pub fn remove(&mut self, dentry: *mut Dentry) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if !(*dentry).lru_prev.is_null() {
                (*(*dentry).lru_prev).lru_next = (*dentry).lru_next;
            } else {
                self.head = (*dentry).lru_next;
            }
            
            if !(*dentry).lru_next.is_null() {
                (*(*dentry).lru_next).lru_prev = (*dentry).lru_prev;
            } else {
                self.tail = (*dentry).lru_prev;
            }
            
            (*dentry).lru_prev = ptr::null_mut();
            (*dentry).lru_next = ptr::null_mut();
            
            self.count.fetch_sub(1, Ordering::AcqRel);
        }
    }
    
    pub fn move_to_tail(&mut self, dentry: *mut Dentry) {
        self.remove(dentry);
        self.add_tail(dentry);
    }
    
    pub fn pop_head(&mut self) -> *mut Dentry {
        let dentry = self.head;
        if !dentry.is_null() {
            self.remove(dentry);
        }
        dentry
    }
}

/// Dentry cache statistics
pub struct DcacheStats {
    pub lookups: AtomicU64,
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub creates: AtomicU64,
    pub evictions: AtomicU64,
}

/// Dentry cache
pub struct DentryCache {
    /// Hash table
    pub hash_table: [*mut Dentry; dcache_config::HASH_SIZE],
    
    /// LRU list
    pub lru_list: DentryLruList,
    
    /// Total dentry count
    pub nr_dentry: AtomicU32,
    
    /// Maximum dentry count
    pub max_dentry: u32,
    
    /// Statistics
    pub stats: DcacheStats,
    
    /// Initialized flag
    pub initialized: AtomicBool,
}

impl DentryCache {
    pub const fn new() -> Self {
        DentryCache {
            hash_table: [ptr::null_mut(); dcache_config::HASH_SIZE],
            lru_list: DentryLruList::new(),
            nr_dentry: AtomicU32::new(0),
            max_dentry: dcache_config::MAX_DENTRY_COUNT,
            stats: DcacheStats {
                lookups: AtomicU64::new(0),
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
                creates: AtomicU64::new(0),
                evictions: AtomicU64::new(0),
            },
            initialized: AtomicBool::new(false),
        }
    }
    
    /// Initialize dentry cache
    pub fn init(&mut self) {
        self.initialized.store(true, Ordering::Release);
    }
    
    /// Look up a dentry in the cache
    pub fn lookup(&mut self, key: &DentryKey, name: &[u8]) -> *mut Dentry {
        self.stats.lookups.fetch_add(1, Ordering::Relaxed);
        
        let hash = key.hash();
        let mut dentry = self.hash_table[hash];
        
        while !dentry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                // Check if this is the dentry we're looking for
                if (*dentry).sb == key.sb {
                    // Check parent
                    let parent_ino = if !(*dentry).parent.is_null() {
                        (*(*dentry).parent).ino
                    } else {
                        0
                    };
                    
                    if parent_ino == key.parent_ino {
                        // Check name
                        if (*dentry).name.len as usize == name.len() {
                            let dentry_name = core::slice::from_raw_parts(
                                (*dentry).name.name,
                                name.len()
                            );
                            
                            if dentry_name == name {
                                // Cache hit
                                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                                (*dentry).ref_count.fetch_add(1, Ordering::AcqRel);
                                self.lru_list.move_to_tail(dentry);
                                return dentry;
                            }
                        }
                    }
                }
                
                dentry = (*dentry).hash_next;
            }
        }
        
        // Cache miss
        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        ptr::null_mut()
    }
    
    /// Add a dentry to the cache
    pub fn add(&mut self, dentry: *mut Dentry) -> bool {
        if dentry.is_null() {
            return false;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Calculate hash
            let parent_ino = if !(*dentry).parent.is_null() {
                (*(*dentry).parent).ino
            } else {
                0
            };
            
            let key = DentryKey::new(
                parent_ino,
                (*dentry).name.hash,
                (*dentry).sb
            );
            
            let hash = key.hash();
            
            // Add to hash table
            (*dentry).hash_next = self.hash_table[hash];
            self.hash_table[hash] = dentry;
            
            // Add to LRU list
            self.lru_list.add_tail(dentry);
            
            self.nr_dentry.fetch_add(1, Ordering::AcqRel);
            self.stats.creates.fetch_add(1, Ordering::Relaxed);
            
            // Check if we need to evict
            if self.nr_dentry.load(Ordering::Acquire) > self.max_dentry {
                self.evict();
            }
        }
        
        true
    }
    
    /// Remove a dentry from the cache
    pub fn remove(&mut self, dentry: *mut Dentry) {
        if dentry.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Calculate hash
            let parent_ino = if !(*dentry).parent.is_null() {
                (*(*dentry).parent).ino
            } else {
                0
            };
            
            let key = DentryKey::new(
                parent_ino,
                (*dentry).name.hash,
                (*dentry).sb
            );
            
            let hash = key.hash();
            
            // Remove from hash table
            let mut prev: *mut Dentry = ptr::null_mut();
            let mut current = self.hash_table[hash];
            
            while !current.is_null() {
                if current == dentry {
                    if prev.is_null() {
                        self.hash_table[hash] = (*current).hash_next;
                    } else {
                        (*prev).hash_next = (*current).hash_next;
                    }
                    break;
                }
                prev = current;
                current = (*current).hash_next;
            }
            
            // Remove from LRU list
            self.lru_list.remove(dentry);
            
            self.nr_dentry.fetch_sub(1, Ordering::AcqRel);
        }
    }
    
    /// Evict dentries from cache
    fn evict(&mut self) {
        let target = self.nr_dentry.load(Ordering::Acquire) - self.max_dentry;
        
        for _ in 0..target {
            let dentry = self.lru_list.pop_head();
            if dentry.is_null() {
                break;
            }
            
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                // Check reference count
                if (*dentry).ref_count.load(Ordering::Acquire) > 1 {
                    // Still in use, move to tail
                    self.lru_list.add_tail(dentry);
                    continue;
                }
                
                // Remove from hash table
                self.remove(dentry);
                
                // Free dentry
                // TODO: Call slab allocator to free
                
                self.stats.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    
    /// Get cache hit rate
    pub fn get_hit_rate(&self) -> u32 {
        let hits = self.stats.hits.load(Ordering::Relaxed);
        let misses = self.stats.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            return 0;
        }
        
        ((hits * 1000) / total) as u32
    }
}

/// Global dentry cache
static DCACHE: crate::sync_oncelock::OnceLock<DentryCache> = crate::sync_oncelock::OnceLock::new();

/// Get the dentry cache
pub fn dcache() -> &'static DentryCache {
    DCACHE.get_or_init(DentryCache::new)
}

/// Initialize dentry cache
pub fn init_dcache() {
    dcache().init();
}

/// Look up a dentry
pub fn dcache_lookup(parent_ino: u64, name: &[u8], sb: u64) -> *mut Dentry {
    let name_hash = Qstr::hash_name(name);
    let key = DentryKey::new(parent_ino, name_hash, sb);
    dcache().lookup(&key, name)
}
