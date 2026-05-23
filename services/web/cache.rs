/*
 * Nuva OS - SystemService - Web - Cache
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

//! HTTP cache management based on NuvaFS storage.
//! Supports cache hit lookup, expiration/refresh, and storage quota enforcement.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

use super::error::{HttpHeader, HttpResponse, HttpStatus, Url, WebError};

/// Cache entry metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// URL of the cached resource
    pub url: String,
    /// HTTP status code of the cached response
    pub status: HttpStatus,
    /// Response headers (for cache validation)
    pub headers: Vec<HttpHeader>,
    /// Cached body data
    pub body: Vec<u8>,
    /// Timestamp when cached (monotonic microseconds)
    pub cached_at_us: u64,
    /// Expiration timestamp (monotonic microseconds, 0 = never expires)
    pub expires_at_us: u64,
    /// ETag for revalidation
    pub etag: Option<String>,
    /// Last-Modified date header
    pub last_modified: Option<String>,
    /// Content type hint
    pub content_type: String,
    /// Body size in bytes
    pub size: u64,
}

impl CacheEntry {
    /// Check if this cache entry has expired
    pub fn is_expired(&self, now_us: u64) -> bool {
        if self.expires_at_us == 0 {
            return false;
        }
        now_us >= self.expires_at_us
    }

    /// Check if this entry can be revalidated (has ETag or Last-Modified)
    pub fn can_revalidate(&self) -> bool {
        self.etag.is_some() || self.last_modified.is_some()
    }

    /// Get the value of a specific header
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.iter().find(|h| h.name.eq_ignore_ascii_case(name)).map(|h| &h.value)
    }
}

/// Cache storage backend (abstracted over NuvaFS)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStorage {
    /// In-memory cache (fast, volatile)
    Memory,
    /// NuvaFS-backed persistent cache
    NuvaFs,
}

/// Cache policy for a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
    /// Always cache if possible
    CacheFirst,
    /// Always fetch from network, update cache
    NetworkFirst,
    /// Cache only, never network (offline mode)
    CacheOnly,
    /// Network only, never cache
    NetworkOnly,
    /// Stale-while-revalidate: serve cache, update in background
    StaleWhileRevalidate,
}

/// HTTP cache manager
pub struct HttpCache {
    /// In-memory cache entries indexed by URL string
    entries: BTreeMap<String, CacheEntry>,
    /// Maximum total cache size in bytes
    max_size: u64,
    /// Current total cache size in bytes
    current_size: AtomicU64,
    /// Maximum age for cache entries in microseconds (default: 1 hour)
    default_max_age_us: u64,
    /// Cache hit count
    hit_count: AtomicU64,
    /// Cache miss count
    miss_count: AtomicU64,
    /// Total entries evicted
    eviction_count: AtomicU64,
    /// Storage backend
    storage: CacheStorage,
}

/// Default maximum cache size: 50 MB
const DEFAULT_MAX_CACHE_SIZE: u64 = 50 * 1024 * 1024;

/// Default maximum age: 1 hour in microseconds
const DEFAULT_MAX_AGE_US: u64 = 3_600_000_000;

impl HttpCache {
    /// Create a new HTTP cache with default settings
    pub fn new() -> Self {
        HttpCache {
            entries: BTreeMap::new(),
            max_size: DEFAULT_MAX_CACHE_SIZE,
            current_size: AtomicU64::new(0),
            default_max_age_us: DEFAULT_MAX_AGE_US,
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            eviction_count: AtomicU64::new(0),
            storage: CacheStorage::Memory,
        }
    }

    /// Create a cache with custom size limit
    pub fn with_max_size(max_size: u64) -> Self {
        let mut cache = HttpCache::new();
        cache.max_size = max_size;
        cache
    }

    /// Look up a cached response by URL
    pub fn get(&self, url: &Url, now_us: u64) -> CacheLookupResult {
        let key = url.origin() + &url.path;

        match self.entries.get(&key) {
            Some(entry) => {
                if entry.is_expired(now_us) {
                    if entry.can_revalidate() {
                        CacheLookupResult::StaleNeedsRevalidation {
                            cached: entry.clone(),
                        }
                    } else {
                        CacheLookupResult::Expired
                    }
                } else {
                    self.hit_count.fetch_add(1, Ordering::Relaxed);
                    CacheLookupResult::Hit {
                        entry: entry.clone(),
                    }
                }
            }
            None => {
                self.miss_count.fetch_add(1, Ordering::Relaxed);
                CacheLookupResult::Miss
            }
        }
    }

    /// Store a response in the cache
    pub fn put(&mut self, url: &Url, response: &HttpResponse, now_us: u64) -> Result<(), WebError> {
        let key = url.origin() + &url.path;
        let body_size = response.body.len() as u64;

        // Check if we need to evict entries to make room
        let current = self.current_size.load(Ordering::Relaxed);
        if current + body_size > self.max_size {
            self.evict_to_fit(body_size)?;
        }

        // Parse cache-control headers to determine max-age
        let max_age = self.parse_max_age(response);
        let expires_at = if max_age == 0 { 0 } else { now_us + max_age };

        let etag = response.get_header("ETag").cloned();
        let last_modified = response.get_header("Last-Modified").cloned();
        let content_type = response.get_header("Content-Type").cloned().unwrap_or_default();

        // Remove old entry if overwriting
        if let Some(old) = self.entries.remove(&key) {
            self.current_size.fetch_sub(old.size, Ordering::Relaxed);
        }

        let entry = CacheEntry {
            url: key.clone(),
            status: response.status,
            headers: response.headers.clone(),
            body: response.body.clone(),
            cached_at_us: now_us,
            expires_at_us: expires_at,
            etag,
            last_modified,
            content_type,
            size: body_size,
        };

        self.current_size.fetch_add(body_size, Ordering::Relaxed);
        self.entries.insert(key, entry);

        Ok(())
    }

    /// Remove a cached entry by URL
    pub fn remove(&mut self, url: &Url) -> Result<(), WebError> {
        let key = url.origin() + &url.path;
        if let Some(entry) = self.entries.remove(&key) {
            self.current_size.fetch_sub(entry.size, Ordering::Relaxed);
            Ok(())
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Invalidate a cached entry (mark as expired) without removing it
    pub fn invalidate(&mut self, url: &Url, now_us: u64) -> Result<(), WebError> {
        let key = url.origin() + &url.path;
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.expires_at_us = now_us;
            Ok(())
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size.store(0, Ordering::Relaxed);
    }

    /// Get the total number of cached entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get the current total cache size in bytes
    pub fn current_size(&self) -> u64 {
        self.current_size.load(Ordering::Relaxed)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.entries.len(),
            total_size: self.current_size.load(Ordering::Relaxed),
            max_size: self.max_size,
            hit_count: self.hit_count.load(Ordering::Relaxed),
            miss_count: self.miss_count.load(Ordering::Relaxed),
            eviction_count: self.eviction_count.load(Ordering::Relaxed),
        }
    }

    /// Evict entries to make room for a new entry of the given size
    fn evict_to_fit(&mut self, needed: u64) -> Result<(), WebError> {
        // Simple LRU-like eviction: remove oldest entries first
        let mut entries: Vec<(String, u64)> = self.entries
            .iter()
            .map(|(k, v)| (k.clone(), v.cached_at_us))
            .collect();
        entries.sort_by_key(|&(_, t)| t);

        for (key, _) in entries {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_size.fetch_sub(entry.size, Ordering::Relaxed);
                self.eviction_count.fetch_add(1, Ordering::Relaxed);
            }

            let current = self.current_size.load(Ordering::Relaxed);
            if current + needed <= self.max_size {
                break;
            }
        }

        let current = self.current_size.load(Ordering::Relaxed);
        if current + needed > self.max_size {
            return Err(WebError::CacheError);
        }

        Ok(())
    }

    /// Parse max-age from Cache-Control header
    fn parse_max_age(&self, response: &HttpResponse) -> u64 {
        if let Some(cc) = response.get_header("Cache-Control") {
            // Look for "max-age=N" directive
            for directive in cc.split(',') {
                let trimmed = directive.trim();
                if trimmed.starts_with("max-age=") {
                    if let Ok(n) = trimmed[8..].parse::<u64>() {
                        return n * 1_000_000;
                    }
                }
                if trimmed == "no-cache" || trimmed == "no-store" {
                    return 0;
                }
            }
        }

        // Check Expires header
        if let Some(_expires) = response.get_header("Expires") {
            // In a full implementation, parse HTTP date and compute delta
        }

        self.default_max_age_us
    }
}

/// Cache lookup result
#[derive(Debug, Clone)]
pub enum CacheLookupResult {
    /// Cache hit - return cached entry
    Hit { entry: CacheEntry },
    /// Cache miss - need to fetch from network
    Miss,
    /// Entry expired and cannot be revalidated
    Expired,
    /// Entry is stale but can be revalidated (ETag/Last-Modified)
    StaleNeedsRevalidation { cached: CacheEntry },
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached entries
    pub entry_count: usize,
    /// Total cache size in bytes
    pub total_size: u64,
    /// Maximum cache size in bytes
    pub max_size: u64,
    /// Cache hit count
    pub hit_count: u64,
    /// Cache miss count
    pub miss_count: u64,
    /// Eviction count
    pub eviction_count: u64,
}

impl CacheStats {
    /// Compute cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f32 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f32 / total as f32
        }
    }
}
