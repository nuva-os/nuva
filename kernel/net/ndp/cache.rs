/*
 * Nuva OS - Kernel - Net - Ndp - Cache
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
 *
 * Nuva OS - Kernel - NDP Neighbor Cache
 *
 * Neighbor cache with BTreeMap for O(log n) lookup and VecDeque-based
 * LRU eviction. Uses no_std-compatible alloc collections.
 */

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;

use crate::kernel::net::ipv6::Ipv6Addr;
use super::nud::NudState;

/// Cache key: IPv6 address + interface index
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheKey {
    /// Target IPv6 address
    pub addr: Ipv6Addr,
    /// Interface index
    pub ifindex: u32,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(addr: Ipv6Addr, ifindex: u32) -> Self {
        CacheKey { addr, ifindex }
    }
}

/// Neighbor cache entry
#[derive(Debug, Clone)]
pub struct NeighborEntry {
    /// Link-layer address (MAC address)
    pub mac_addr: [u8; 6],
    /// NUD reachability state
    pub state: NudState,
    /// Expiry time (monotonic ticks, 0 = no expiry)
    pub expiry_time: u64,
    /// Number of reachability confirmations received
    pub confirmations: u32,
    /// Whether this entry is for a router
    pub is_router: bool,
}

impl NeighborEntry {
    /// Create a new neighbor entry in the given NUD state
    pub fn new(mac_addr: [u8; 6], state: NudState, is_router: bool) -> Self {
        NeighborEntry {
            mac_addr,
            state,
            expiry_time: 0,
            confirmations: 0,
            is_router,
        }
    }

    /// Check if the entry is usable for packet transmission
    pub fn is_usable(&self) -> bool {
        self.state.is_usable()
    }

    /// Check if the entry has expired
    pub fn is_expired(&self, now: u64) -> bool {
        self.expiry_time > 0 && now >= self.expiry_time
    }
}

/// Neighbor cache with LRU eviction
pub struct NeighborCache {
    /// Main storage: BTreeMap for ordered lookup
    entries: BTreeMap<CacheKey, NeighborEntry>,
    /// LRU ordering: most-recently-used at the back, least at the front
    lru_order: VecDeque<CacheKey>,
    /// Maximum number of entries
    max_entries: usize,
}

impl NeighborCache {
    /// Create a new neighbor cache with the given capacity
    pub fn new(max_entries: usize) -> Self {
        NeighborCache {
            entries: BTreeMap::new(),
            lru_order: VecDeque::new(),
            max_entries,
        }
    }

    /// Look up a neighbor entry by key. Returns a clone of the entry if found.
    /// Updates LRU ordering on hit.
    pub fn lookup(&mut self, key: &CacheKey) -> Option<NeighborEntry> {
        if let Some(entry) = self.entries.get(key) {
            self.lru_touch(key);
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Look up a neighbor MAC address by key (fast path for packet transmission).
    /// Returns None if the entry does not exist or is not in a usable state.
    pub fn lookup_mac(&mut self, key: &CacheKey) -> Option<[u8; 6]> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_usable() {
                self.lru_touch(key);
                return Some(entry.mac_addr);
            }
        }
        None
    }

    /// Insert or update a neighbor entry
    pub fn insert(&mut self, key: CacheKey, entry: NeighborEntry) {
        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), entry);
            self.lru_touch(&key);
        } else {
            if self.entries.len() >= self.max_entries {
                self.lru_evict_one();
            }
            self.entries.insert(key.clone(), entry);
            self.lru_order.push_back(key);
        }
    }

    /// Create an INCOMPLETE entry (address resolution in progress)
    pub fn create_incomplete(&mut self, key: CacheKey) {
        if self.entries.contains_key(&key) {
            return;
        }
        if self.entries.len() >= self.max_entries {
            self.lru_evict_one();
        }
        let entry = NeighborEntry {
            mac_addr: [0; 6],
            state: NudState::Incomplete,
            expiry_time: 0,
            confirmations: 0,
            is_router: false,
        };
        self.entries.insert(key.clone(), entry);
        self.lru_order.push_back(key);
    }

    /// Update the NUD state of an existing entry
    pub fn update_state(&mut self, key: &CacheKey, new_state: NudState) -> bool {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.state = new_state;
            self.lru_touch(key);
            true
        } else {
            false
        }
    }

    /// Update the MAC address of an existing entry
    pub fn update_mac(&mut self, key: &CacheKey, mac_addr: [u8; 6]) -> bool {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.mac_addr = mac_addr;
            self.lru_touch(key);
            true
        } else {
            false
        }
    }

    /// Remove a neighbor entry
    pub fn remove(&mut self, key: &CacheKey) -> Option<NeighborEntry> {
        if let Some(entry) = self.entries.remove(key) {
            self.lru_order.retain(|k| k != key);
            Some(entry)
        } else {
            None
        }
    }

    /// Evict the least-recently-used entry
    fn lru_evict_one(&mut self) {
        if let Some(key) = self.lru_order.pop_front() {
            self.entries.remove(&key);
        }
    }

    /// Move a key to the MRU (most recently used) position
    fn lru_touch(&mut self, key: &CacheKey) {
        self.lru_order.retain(|k| k != key);
        self.lru_order.push_back(key.clone());
    }

    /// Get the current number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the maximum capacity
    pub fn capacity(&self) -> usize {
        self.max_entries
    }

    /// Collect all expired entries (for periodic cleanup)
    pub fn collect_expired(&self, now: u64) -> Vec<CacheKey> {
        let mut expired = Vec::new();
        for (key, entry) in &self.entries {
            if entry.is_expired(now) {
                expired.push(key.clone());
            }
        }
        expired
    }

    /// Mark an entry as a router
    pub fn set_router(&mut self, key: &CacheKey, is_router: bool) -> bool {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.is_router = is_router;
            true
        } else {
            false
        }
    }
}
