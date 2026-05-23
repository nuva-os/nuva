/*
 * Nuva OS - SystemLibrary - Data
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

//! Key-Value Store

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Getcurrenttimebetween(ms)
pub fn get_current_time() -> u64 {
 // SimplifiedImplementation:Returnitemincrease Timestamp
 // realactualshouldthesecondarysystemsystemtimeclockor TSC Get
 use core::sync::atomic::{AtomicU64, Ordering};
 static COUNTER: AtomicU64 = AtomicU64::new(0);
 COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Key-Value StoreConfig
#[derive(Debug, Clone)]
pub struct KVStoreConfig {
 pub path: [u8; 256],
 pub path_len: u8,
 pub max_size: u64,
 pub cache_size: u32,
 pub sync_writes: bool,
 pub compress: bool,
}

/// keyvaluestripentry
#[derive(Debug, Clone)]
pub struct KVEntry {
 pub key: [u8; 64],
 pub key_len: u8,
 pub value: [u8; 1024],
 pub value_len: u16,
 pub created_at: u64,
 pub updated_at: u64,
 pub ttl: u32,
 pub flags: u32,
}

/// stripentryFlag
pub const ENTRY_FLAG_DELETED: u32 = 1 << 0;
pub const ENTRY_FLAG_COMPRESSED: u32 = 1 << 1;
pub const ENTRY_FLAG_ENCRYPTED: u32 = 1 << 2;

impl KVEntry {
 pub fn new(key: &[u8], value: &[u8]) -> Self {
 let mut key_buf = [0u8; 64];
 let key_len = key.len().min(63);
 key_buf[..key_len].copy_from_slice(&key[..key_len]);
 
 let mut value_buf = [0u8; 1024];
 let value_len = value.len().min(1023);
 value_buf[..value_len].copy_from_slice(&value[..value_len]);
 
 Self {
 key: key_buf,
 key_len: key_len as u8,
 value: value_buf,
 value_len: value_len as u16,
 created_at: 0,
 updated_at: 0,
 ttl: 0,
 flags: 0,
 }
 }

 pub fn key(&self) -> &[u8] {
 &self.key[..self.key_len as usize]
 }

 pub fn value(&self) -> &[u8] {
 &self.value[..self.value_len as usize]
 }

 pub fn is_deleted(&self) -> bool {
 self.flags & ENTRY_FLAG_DELETED != 0
 }

 pub fn is_expired(&self, now: u64) -> bool {
 self.ttl > 0 && now > self.created_at + self.ttl as u64
 }
}

/// Key-Value Store
pub struct KVStore {
 entries: [Option<KVEntry>; 4096],
 num_entries: AtomicU32,
 config: KVStoreConfig,
 stats: KVStoreStats,
}

impl Clone for KVStore {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            num_entries: AtomicU32::new(self.num_entries.load(core::sync::atomic::Ordering::Relaxed)),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl KVStore {
 pub fn new(config: KVStoreConfig) -> Self {
 Self {
 entries: core::array::from_fn(|_| None),
 num_entries: AtomicU32::new(0),
 config,
 stats: KVStoreStats::new(),
 }
 }

 pub fn init(&mut self) {
 crate::log_info!("KV store initialized");
 }

 /// Setkeyvalue
 pub fn set(&mut self, key: &[u8], value: &[u8]) -> bool {
 // CheckifalreadyExists
 for i in 0..self.num_entries.load(Ordering::Relaxed) as usize {
 if let Some(ref entry) = self.entries[i] {
 if entry.key() == key {
 // Updatefinitestripentry
 let mut new_entry = KVEntry::new(key, value);
 new_entry.created_at = entry.created_at;
 new_entry.updated_at = get_current_time();
 self.entries[i] = Some(new_entry);
 self.stats.updates.fetch_add(1, Ordering::Relaxed);
 return true;
 }
 }
 }
 
 // addPlusnewstripentry
 let idx = self.num_entries.load(Ordering::Relaxed) as usize;
 if idx < 4096 {
 let entry = KVEntry::new(key, value);
 self.entries[idx] = Some(entry);
 self.num_entries.fetch_add(1, Ordering::Relaxed);
 self.stats.inserts.fetch_add(1, Ordering::Relaxed);
 return true;
 }
 
 self.stats.errors.fetch_add(1, Ordering::Relaxed);
 false
 }

 /// Getvalue
 pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
 for i in 0..self.num_entries.load(Ordering::Relaxed) as usize {
 if let Some(ref entry) = self.entries[i] {
 if entry.key() == key && !entry.is_deleted() {
 self.stats.hits.fetch_add(1, Ordering::Relaxed);
 return Some(entry.value());
 }
 }
 }
 
 self.stats.misses.fetch_add(1, Ordering::Relaxed);
 None
 }

 /// Deletekey
 pub fn delete(&mut self, key: &[u8]) -> bool {
 for i in 0..self.num_entries.load(Ordering::Relaxed) as usize {
 if let Some(ref mut entry) = self.entries[i] {
 if entry.key() == key {
 entry.flags |= ENTRY_FLAG_DELETED;
 self.stats.deletes.fetch_add(1, Ordering::Relaxed);
 return true;
 }
 }
 }
 false
 }

 /// Checkkeyifexist
 pub fn exists(&self, key: &[u8]) -> bool {
 for i in 0..self.num_entries.load(Ordering::Relaxed) as usize {
 if let Some(ref entry) = self.entries[i] {
 if entry.key() == key && !entry.is_deleted() {
 return true;
 }
 }
 }
 false
 }

 /// Set TTL
 pub fn set_ttl(&mut self, key: &[u8], ttl: u32) -> bool {
 for i in 0..self.num_entries.load(Ordering::Relaxed) as usize {
 if let Some(ref mut entry) = self.entries[i] {
 if entry.key() == key {
 entry.ttl = ttl;
 return true;
 }
 }
 }
 false
 }

 /// Getallkey
 pub fn keys(&self) -> Vec<&[u8]> {
 let mut keys = Vec::new();
 for i in 0..self.num_entries.load(Ordering::Relaxed) as usize {
 if let Some(ref entry) = self.entries[i] {
 if !entry.is_deleted() {
 keys.push(entry.key());
 }
 }
 }
 keys
 }

 /// Clearexist
 pub fn clear(&mut self) {
 for i in 0..self.num_entries.load(Ordering::Relaxed) as usize {
 self.entries[i] = None;
 }
 self.num_entries.store(0, Ordering::Relaxed);
 }

 /// Get statistics
 pub fn stats(&self) -> &KVStoreStats {
 &self.stats
 }

 /// clearadministrationoverperiodstripentry
 pub fn cleanup_expired(&mut self, now: u64) -> u32 {
 let mut cleaned = 0u32;
 
 for i in 0..self.num_entries.load(Ordering::Relaxed) as usize {
 if let Some(ref entry) = self.entries[i] {
 if entry.is_expired(now) {
 self.entries[i] = None;
 cleaned += 1;
 }
 }
 }
 
 cleaned
 }
}

/// Key-Value Storestatistics
pub struct KVStoreStats {
 pub inserts: AtomicU32,
 pub updates: AtomicU32,
 pub deletes: AtomicU32,
 pub hits: AtomicU32,
 pub misses: AtomicU32,
 pub errors: AtomicU32,
}

impl Clone for KVStoreStats {
    fn clone(&self) -> Self {
        Self {
            inserts: AtomicU32::new(self.inserts.load(core::sync::atomic::Ordering::Relaxed)),
            updates: AtomicU32::new(self.updates.load(core::sync::atomic::Ordering::Relaxed)),
            deletes: AtomicU32::new(self.deletes.load(core::sync::atomic::Ordering::Relaxed)),
            hits: AtomicU32::new(self.hits.load(core::sync::atomic::Ordering::Relaxed)),
            misses: AtomicU32::new(self.misses.load(core::sync::atomic::Ordering::Relaxed)),
            errors: AtomicU32::new(self.errors.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl KVStoreStats {
 pub const fn new() -> Self {
 Self {
 inserts: AtomicU32::new(0),
 updates: AtomicU32::new(0),
 deletes: AtomicU32::new(0),
 hits: AtomicU32::new(0),
 misses: AtomicU32::new(0),
 errors: AtomicU32::new(0),
 }
 }

 pub fn total_operations(&self) -> u32 {
 self.inserts.load(Ordering::Relaxed)
 + self.updates.load(Ordering::Relaxed)
 + self.deletes.load(Ordering::Relaxed)
 }

 pub fn hit_rate(&self) -> f32 {
 let hits = self.hits.load(Ordering::Relaxed);
 let misses = self.misses.load(Ordering::Relaxed);
 let total = hits + misses;
 
 if total > 0 {
 hits as f32 / total as f32
 } else {
 0.0
 }
 }
}

/// Key-Value StoreManager
pub struct KVStoreManager {
 stores: [Option<KVStore>; 16],
 num_stores: AtomicU32,
}

impl KVStoreManager {
 pub const fn new() -> Self {
 Self {
 stores: [const { None }; 16],
 num_stores: AtomicU32::new(0),
 }
 }

 pub fn create_store(&mut self, config: KVStoreConfig) -> Option<u32> {
 let id = self.num_stores.load(Ordering::Relaxed);
 if id < 16 {
 let store = KVStore::new(config);
 self.stores[id as usize] = Some(store);
 self.num_stores.fetch_add(1, Ordering::Relaxed);
 return Some(id);
 }
 None
 }

 pub fn get_store(&mut self, id: u32) -> Option<&mut KVStore> {
 if id < self.num_stores.load(Ordering::Relaxed) {
 self.stores[id as usize].as_mut()
 } else {
 None
 }
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 fn default_config() -> KVStoreConfig {
 KVStoreConfig {
 path: [0u8; 256],
 path_len: 0,
 max_size: 1024 * 1024,
 cache_size: 4096,
 sync_writes: true,
 compress: false,
 }
 }

 #[test]
 fn test_kv_entry_new() {
 let entry = KVEntry::new(b"test_key", b"test_value");

 assert_eq!(entry.key(), b"test_key");
 assert_eq!(entry.value(), b"test_value");
 assert!(!entry.is_deleted());
 }

 #[test]
 fn test_kv_entry_flags() {
 let mut entry = KVEntry::new(b"key", b"value");

 assert!(!entry.is_deleted());

 entry.flags |= ENTRY_FLAG_DELETED;
 assert!(entry.is_deleted());

 entry.flags |= ENTRY_FLAG_COMPRESSED;
 assert!(entry.flags & ENTRY_FLAG_COMPRESSED != 0);
 }

 #[test]
 fn test_kv_entry_expiration() {
 let mut entry = KVEntry::new(b"key", b"value");

 // none TTL
 assert!(!entry.is_expired(1000));

 // Set TTL
 entry.created_at = 100;
 entry.ttl = 500;

 assert!(!entry.is_expired(500)); // overperiod
 assert!(entry.is_expired(700)); // alreadyoverperiod
 }

 #[test]
 fn test_kv_store_new() {
 let config = default_config();
 let store = KVStore::new(config);

 assert_eq!(store.num_entries.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_kv_store_set_get() {
 let config = default_config();
 let mut store = KVStore::new(config);

 // Setkeyvalue
 assert!(store.set(b"key1", b"value1"));
 assert!(store.set(b"key2", b"value2"));

 // Getvalue
 assert_eq!(store.get(b"key1"), Some(&b"value1"[..]));
 assert_eq!(store.get(b"key2"), Some(&b"value2"[..]));
 assert_eq!(store.get(b"key3"), None);
 }

 #[test]
 fn test_kv_store_update() {
 let config = default_config();
 let mut store = KVStore::new(config);

 store.set(b"key", b"value1");
 assert_eq!(store.get(b"key"), Some(&b"value1"[..]));

 store.set(b"key", b"value2");
 assert_eq!(store.get(b"key"), Some(&b"value2"[..]));
 }

 #[test]
 fn test_kv_store_delete() {
 let config = default_config();
 let mut store = KVStore::new(config);

 store.set(b"key", b"value");
 assert!(store.exists(b"key"));

 assert!(store.delete(b"key"));
 assert!(!store.exists(b"key"));
 assert_eq!(store.get(b"key"), None);
 }

 #[test]
 fn test_kv_store_exists() {
 let config = default_config();
 let mut store = KVStore::new(config);

 assert!(!store.exists(b"key"));

 store.set(b"key", b"value");
 assert!(store.exists(b"key"));
 }

 #[test]
 fn test_kv_store_ttl() {
 let config = default_config();
 let mut store = KVStore::new(config);

 store.set(b"key", b"value");
 assert!(store.set_ttl(b"key", 100));

 // Check TTL Set
 for i in 0..store.num_entries.load(Ordering::Relaxed) as usize {
 if let Some(ref entry) = store.entries[i] {
 if entry.key() == b"key" {
 assert_eq!(entry.ttl, 100);
 }
 }
 }
 }

 #[test]
 fn test_kv_store_clear() {
 let config = default_config();
 let mut store = KVStore::new(config);

 store.set(b"key1", b"value1");
 store.set(b"key2", b"value2");

 store.clear();

 assert_eq!(store.num_entries.load(Ordering::Relaxed), 0);
 assert!(!store.exists(b"key1"));
 assert!(!store.exists(b"key2"));
 }

 #[test]
 fn test_kv_store_stats() {
 let config = default_config();
 let mut store = KVStore::new(config);

 store.set(b"key1", b"value1");
 store.set(b"key2", b"value2");
 store.get(b"key1");
 store.get(b"key3"); // miss
 store.delete(b"key1");

 let stats = store.stats();
 assert_eq!(stats.inserts.load(Ordering::Relaxed), 2);
 assert_eq!(stats.hits.load(Ordering::Relaxed), 1);
 assert_eq!(stats.misses.load(Ordering::Relaxed), 1);
 assert_eq!(stats.deletes.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_kv_store_stats_hit_rate() {
 let stats = KVStoreStats::new();

 assert_eq!(stats.hit_rate(), 0.0);

 stats.hits.fetch_add(8, Ordering::Relaxed);
 stats.misses.fetch_add(2, Ordering::Relaxed);

 // 8 / (8 + 2) = 0.8
 let rate = stats.hit_rate();
 assert!(rate > 0.79 && rate < 0.81);
 }

 #[test]
 fn test_kv_store_manager() {
 let mut manager = KVStoreManager::new();

 let config = default_config();
 let id1 = manager.create_store(config.clone());
 assert!(id1.is_some());

 let id2 = manager.create_store(config);
 assert!(id2.is_some());

 assert_ne!(id1, id2);
 }

 #[test]
 fn test_kv_store_manager_get_store() {
 let mut manager = KVStoreManager::new();

 let config = default_config();
 let id = manager.create_store(config).unwrap();

 let store = manager.get_store(id);
 assert!(store.is_some());

 let store = manager.get_store(999);
 assert!(store.is_none());
 }
}