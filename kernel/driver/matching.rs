/*
 * Nuva OS - Kernel - Driver Compatible Hash Table
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

//! Driver compatible string hash table for O(1) device-driver matching.
//!
//! The compatible hash table maps device tree compatible strings
//! to driver descriptors, enabling constant-time driver lookup
//! during device probing instead of linear scanning.

use alloc::string::String;
use alloc::vec::Vec;

use crate::kernel::driver::declarative::DriverDescriptor;

/** Hash table bucket entry.
 *
 * Each entry stores a compatible string and a list of
 * driver descriptors that match that compatible string.
 * Multiple drivers may match the same compatible string
 * (distinguished by priority).
 */
pub struct HashEntry {
    /** Compatible string key */
    pub compatible: String,
    /** Matching driver descriptors, sorted by priority */
    pub drivers: Vec<&'static DriverDescriptor>,
}

/** Driver compatible string hash table.
 *
 * Provides O(1) average-case lookup for driver matching
 * by hashing the compatible string. Falls back to linear
 * scan on hash collisions.
 *
 * The table size should be chosen to be approximately
 * 2x the expected number of unique compatible strings
 * to keep the load factor below 0.5.
 */
pub struct CompatibleHashTable {
    /** Hash table buckets */
    buckets: Vec<Vec<HashEntry>>,
    /** Number of entries in the table */
    count: usize,
}

impl CompatibleHashTable {
    /** Create a new hash table with the given number of buckets.
     *
     * The bucket count should be a power of two for efficient
     * modulo reduction via bitmasking.
     */
    pub fn new(num_buckets: usize) -> Self {
        let num_buckets = if num_buckets == 0 { 16 } else { num_buckets };
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(Vec::new());
        }
        CompatibleHashTable { buckets, count: 0 }
    }

    /** Compute the FNV-1a hash of a compatible string.
     *
     * FNV-1a is chosen for its excellent distribution on
     * short strings (typical compatible strings are < 40 chars)
     * and computational simplicity (no multiplication by
     * non-power-of-two constants).
     */
    pub fn hash_compatible(compatible: &str) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x000001000001b3;

        let mut hash = FNV_OFFSET_BASIS;
        for byte in compatible.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    /** Insert a driver descriptor for the given compatible string.
     *
     * If the compatible string already exists in the table,
     * the driver is added to the existing entry's driver list.
     * Drivers within an entry are sorted by priority (lower = higher).
     */
    pub fn insert(&mut self, compatible: &str, driver: &'static DriverDescriptor) {
        if self.buckets.is_empty() {
            return;
        }

        let hash = Self::hash_compatible(compatible);
        let index = (hash as usize) % self.buckets.len();
        let bucket = &mut self.buckets[index];

        for entry in bucket.iter_mut() {
            if entry.compatible == compatible {
                entry.drivers.push(driver);
                entry.drivers.sort_by_key(|d| d.priority);
                return;
            }
        }

        let mut entry = HashEntry {
            compatible: String::from(compatible),
            drivers: Vec::new(),
        };
        entry.drivers.push(driver);
        bucket.push(entry);
        self.count += 1;
    }

    /** Look up drivers matching the given compatible string.
     *
     * Returns a slice of driver descriptors sorted by priority,
     * or None if no matching drivers are registered.
     */
    pub fn lookup(&self, compatible: &str) -> Option<&[&'static DriverDescriptor]> {
        if self.buckets.is_empty() {
            return None;
        }

        let hash = Self::hash_compatible(compatible);
        let index = (hash as usize) % self.buckets.len();
        let bucket = &self.buckets[index];

        for entry in bucket.iter() {
            if entry.compatible == compatible {
                if entry.drivers.is_empty() {
                    return None;
                }
                return Some(&entry.drivers);
            }
        }

        None
    }

    /** Remove all drivers for the given compatible string.
     *
     * Returns true if an entry was found and removed,
     * false if the compatible string was not in the table.
     */
    pub fn remove(&mut self, compatible: &str) -> bool {
        if self.buckets.is_empty() {
            return false;
        }

        let hash = Self::hash_compatible(compatible);
        let index = (hash as usize) % self.buckets.len();
        let bucket = &mut self.buckets[index];

        let original_len = bucket.len();
        bucket.retain(|e| e.compatible != compatible);
        let removed = bucket.len() != original_len;

        if removed {
            self.count -= 1;
        }

        removed
    }

    /** Get the total number of unique compatible strings in the table */
    pub fn len(&self) -> usize {
        self.count
    }

    /** Check if the table is empty */
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /** Get the current load factor (entries / buckets) */
    pub fn load_factor(&self) -> f32 {
        if self.buckets.is_empty() {
            return 0.0;
        }
        self.count as f32 / self.buckets.len() as f32
    }

    /** Get the number of buckets */
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::driver::declarative::CapabilityFlags;

    fn make_desc(
        name: &'static str,
        compat: &'static [&'static str],
        prio: u32,
    ) -> DriverDescriptor {
        DriverDescriptor {
            name,
            compatible: compat,
            resources: &[],
            capabilities: CapabilityFlags::empty(),
            priority: prio,
            hotplug: false,
        }
    }

    #[test]
    fn test_hash_compatible_deterministic() {
        let h1 = CompatibleHashTable::hash_compatible("vendor,my-device");
        let h2 = CompatibleHashTable::hash_compatible("vendor,my-device");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_compatible_different() {
        let h1 = CompatibleHashTable::hash_compatible("vendor,device-a");
        let h2 = CompatibleHashTable::hash_compatible("vendor,device-b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_insert_and_lookup() {
        static DESC: DriverDescriptor = make_desc("test_drv", &["vendor,test"], 0);

        let mut table = CompatibleHashTable::new(16);
        table.insert("vendor,test", &DESC);

        let result = table.lookup("vendor,test");
        assert!(result.is_some());
        let drivers = result.unwrap();
        assert_eq!(drivers.len(), 1);
        assert_eq!(drivers[0].name, "test_drv");
    }

    #[test]
    fn test_lookup_missing() {
        let table = CompatibleHashTable::new(16);
        let result = table.lookup("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_remove() {
        static DESC: DriverDescriptor = make_desc("rm_drv", &["vendor,rm"], 0);

        let mut table = CompatibleHashTable::new(16);
        table.insert("vendor,rm", &DESC);
        assert_eq!(table.len(), 1);

        let removed = table.remove("vendor,rm");
        assert!(removed);
        assert_eq!(table.len(), 0);
        assert!(table.lookup("vendor,rm").is_none());
    }

    #[test]
    fn test_priority_sorting() {
        static DESC_HI: DriverDescriptor = make_desc("hi", &["vendor,s"], 0);
        static DESC_LO: DriverDescriptor = make_desc("lo", &["vendor,s"], 10);

        let mut table = CompatibleHashTable::new(16);
        table.insert("vendor,s", &DESC_LO);
        table.insert("vendor,s", &DESC_HI);

        let drivers = table.lookup("vendor,s").unwrap();
        assert_eq!(drivers.len(), 2);
        assert_eq!(drivers[0].priority, 0);
        assert_eq!(drivers[1].priority, 10);
    }

    #[test]
    fn test_load_factor() {
        let mut table = CompatibleHashTable::new(16);
        assert!(table.is_empty());

        static DESC: DriverDescriptor = make_desc("lf", &["v,d"], 0);
        table.insert("v,d", &DESC);
        assert!(!table.is_empty());
        assert!(table.load_factor() > 0.0);
    }
}
