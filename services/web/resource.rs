/*
 * Nuva OS - SystemService - Web - Resource
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

//! Resource loading and limitation management.
//! Tracks per-page memory and CPU usage, enforces resource budgets,
//! and terminates JS execution when limits are exceeded while preserving
//! the DOM structure for graceful degradation.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::error::WebError;
use super::js_engine::JsEngine;
use super::page::PageId;

/// Resource limit configuration for a page
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    /// Maximum total memory in bytes for the page
    pub memory_limit: u64,
    /// Maximum JS heap size in bytes
    pub js_heap_limit: u64,
    /// Maximum JS execution time in microseconds
    pub js_timeout_us: u64,
    /// Maximum number of concurrent network requests
    pub max_concurrent_requests: u32,
    /// Maximum number of DOM nodes
    pub max_dom_nodes: u32,
    /// Maximum number of CSS rules
    pub max_css_rules: u32,
    /// Maximum total network bytes
    pub max_network_bytes: u64,
    /// CPU time limit in microseconds (0 = unlimited)
    pub cpu_limit_us: u64,
}

impl ResourceLimits {
    /// Default resource limits
    pub const DEFAULT: ResourceLimits = ResourceLimits {
        memory_limit: 64 * 1024 * 1024,
        js_heap_limit: 32 * 1024 * 1024,
        js_timeout_us: 5_000_000,
        max_concurrent_requests: 6,
        max_dom_nodes: 100_000,
        max_css_rules: 50_000,
        max_network_bytes: 100 * 1024 * 1024,
        cpu_limit_us: 0,
    };

    /// Relaxed limits for background tabs
    pub const BACKGROUND: ResourceLimits = ResourceLimits {
        memory_limit: 32 * 1024 * 1024,
        js_heap_limit: 16 * 1024 * 1024,
        js_timeout_us: 2_000_000,
        max_concurrent_requests: 2,
        max_dom_nodes: 50_000,
        max_css_rules: 25_000,
        max_network_bytes: 50 * 1024 * 1024,
        cpu_limit_us: 10_000_000,
    };
}

/// Resource usage snapshot for a page
#[derive(Debug, Clone)]
pub struct ResourceSnapshot {
    /// Page ID
    pub page_id: PageId,
    /// Current total memory usage in bytes
    pub memory_used: u64,
    /// Current JS heap usage in bytes
    pub js_heap_used: u64,
    /// Current DOM node count
    pub dom_nodes: u32,
    /// Current CSS rule count
    pub css_rules: u32,
    /// Network bytes transferred
    pub network_bytes: u64,
    /// Active network requests
    pub active_requests: u32,
    /// CPU time consumed in microseconds
    pub cpu_time_us: u64,
    /// JS execution time in microseconds
    pub js_exec_time_us: u64,
    /// Number of JS scripts executed
    pub js_script_count: u32,
    /// Timestamp of this snapshot (monotonic us)
    pub timestamp_us: u64,
}

/// Resource violation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceViolation {
    /// Total memory limit exceeded
    MemoryLimitExceeded,
    /// JS heap limit exceeded
    JsHeapLimitExceeded,
    /// JS execution timeout
    JsTimeout,
    /// Too many concurrent network requests
    TooManyRequests,
    /// Too many DOM nodes
    TooManyDomNodes,
    /// Too many CSS rules
    TooManyCssRules,
    /// Network bytes limit exceeded
    NetworkBytesExceeded,
    /// CPU time limit exceeded
    CpuLimitExceeded,
}

/// Resource violation event
#[derive(Debug, Clone)]
pub struct ViolationEvent {
    /// Page that violated
    pub page_id: PageId,
    /// Type of violation
    pub violation: ResourceViolation,
    /// Current usage value
    pub current: u64,
    /// Limit that was exceeded
    pub limit: u64,
    /// Timestamp
    pub timestamp_us: u64,
}

/// Per-page resource tracker
pub struct PageResourceTracker {
    /// Page ID
    pub page_id: PageId,
    /// Active resource limits
    pub limits: ResourceLimits,
    /// Current memory usage
    pub memory_used: AtomicU64,
    /// Current JS heap usage
    pub js_heap_used: AtomicU64,
    /// Current DOM node count
    pub dom_nodes: AtomicU32,
    /// Current CSS rule count
    pub css_rules: AtomicU32,
    /// Network bytes transferred
    pub network_bytes: AtomicU64,
    /// Active network requests
    pub active_requests: AtomicU32,
    /// CPU time consumed
    pub cpu_time_us: AtomicU64,
    /// JS execution time
    pub js_exec_time_us: AtomicU64,
    /// JS script count
    pub js_script_count: AtomicU32,
    /// Whether JS has been terminated due to resource limits
    pub js_terminated: bool,
}

impl PageResourceTracker {
    /// Create a new resource tracker for a page
    pub fn new(page_id: PageId, limits: ResourceLimits) -> Self {
        PageResourceTracker {
            page_id,
            limits,
            memory_used: AtomicU64::new(0),
            js_heap_used: AtomicU64::new(0),
            dom_nodes: AtomicU32::new(0),
            css_rules: AtomicU32::new(0),
            network_bytes: AtomicU64::new(0),
            active_requests: AtomicU32::new(0),
            cpu_time_us: AtomicU64::new(0),
            js_exec_time_us: AtomicU64::new(0),
            js_script_count: AtomicU32::new(0),
            js_terminated: false,
        }
    }

    /// Take a snapshot of current resource usage
    pub fn snapshot(&self, timestamp_us: u64) -> ResourceSnapshot {
        ResourceSnapshot {
            page_id: self.page_id,
            memory_used: self.memory_used.load(Ordering::Relaxed),
            js_heap_used: self.js_heap_used.load(Ordering::Relaxed),
            dom_nodes: self.dom_nodes.load(Ordering::Relaxed),
            css_rules: self.css_rules.load(Ordering::Relaxed),
            network_bytes: self.network_bytes.load(Ordering::Relaxed),
            active_requests: self.active_requests.load(Ordering::Relaxed),
            cpu_time_us: self.cpu_time_us.load(Ordering::Relaxed),
            js_exec_time_us: self.js_exec_time_us.load(Ordering::Relaxed),
            js_script_count: self.js_script_count.load(Ordering::Relaxed),
            timestamp_us,
        }
    }

    /// Check all resource limits and return violations
    pub fn check_limits(&self) -> Vec<ResourceViolation> {
        let mut violations = Vec::new();

        if self.memory_used.load(Ordering::Relaxed) > self.limits.memory_limit {
            violations.push(ResourceViolation::MemoryLimitExceeded);
        }
        if self.js_heap_used.load(Ordering::Relaxed) > self.limits.js_heap_limit {
            violations.push(ResourceViolation::JsHeapLimitExceeded);
        }
        if self.dom_nodes.load(Ordering::Relaxed) > self.limits.max_dom_nodes {
            violations.push(ResourceViolation::TooManyDomNodes);
        }
        if self.css_rules.load(Ordering::Relaxed) > self.limits.max_css_rules {
            violations.push(ResourceViolation::TooManyCssRules);
        }
        if self.active_requests.load(Ordering::Relaxed) > self.limits.max_concurrent_requests {
            violations.push(ResourceViolation::TooManyRequests);
        }
        if self.network_bytes.load(Ordering::Relaxed) > self.limits.max_network_bytes {
            violations.push(ResourceViolation::NetworkBytesExceeded);
        }
        if self.limits.cpu_limit_us > 0 && self.cpu_time_us.load(Ordering::Relaxed) > self.limits.cpu_limit_us {
            violations.push(ResourceViolation::CpuLimitExceeded);
        }

        violations
    }

    /// Record memory allocation
    pub fn allocate_memory(&self, size: u64) -> Result<(), WebError> {
        let current = self.memory_used.load(Ordering::Relaxed);
        if current + size > self.limits.memory_limit {
            return Err(WebError::MemoryLimitExceeded);
        }
        self.memory_used.fetch_add(size, Ordering::Relaxed);
        Ok(())
    }

    /// Record memory deallocation
    pub fn free_memory(&self, size: u64) {
        let current = self.memory_used.load(Ordering::Relaxed);
        let freed = if size > current { current } else { size };
        self.memory_used.fetch_sub(freed, Ordering::Relaxed);
    }

    /// Record a network request start
    pub fn request_started(&self) -> Result<(), WebError> {
        let current = self.active_requests.load(Ordering::Relaxed);
        if current >= self.limits.max_concurrent_requests {
            return Err(WebError::InvalidArgument);
        }
        self.active_requests.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Record a network request completion
    pub fn request_completed(&self, bytes: u64) {
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
        self.network_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record DOM node addition
    pub fn dom_node_added(&self) -> Result<(), WebError> {
        let current = self.dom_nodes.load(Ordering::Relaxed);
        if current >= self.limits.max_dom_nodes {
            return Err(WebError::MemoryLimitExceeded);
        }
        self.dom_nodes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Record DOM node removal
    pub fn dom_node_removed(&self) {
        let current = self.dom_nodes.load(Ordering::Relaxed);
        if current > 0 {
            self.dom_nodes.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Terminate JS execution due to resource limits
    /// Preserves DOM structure for graceful degradation
    pub fn terminate_js(&mut self) -> Result<(), WebError> {
        if self.js_terminated {
            return Ok(());
        }
        self.js_terminated = true;
        // In a full implementation, this would:
        // 1. Signal the JS context to stop execution
        // 2. Cancel all pending timers (setTimeout/setInterval)
        // 3. Abort all pending fetch() promises
        // 4. Keep the DOM tree intact for rendering
        // 5. Fire a "resourceviolation" event on the page
        Ok(())
    }

    /// Check if JS execution is allowed
    pub fn js_allowed(&self) -> bool {
        !self.js_terminated
    }
}

/// Global resource manager
pub struct ResourceManager {
    /// Per-page resource trackers
    trackers: BTreeMap<u64, PageResourceTracker>,
    /// Total violations across all pages
    total_violations: AtomicU64,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        ResourceManager {
            trackers: BTreeMap::new(),
            total_violations: AtomicU64::new(0),
        }
    }

    /// Register a page for resource tracking
    pub fn register_page(&mut self, page_id: PageId, limits: ResourceLimits) {
        let tracker = PageResourceTracker::new(page_id, limits);
        self.trackers.insert(page_id.0, tracker);
    }

    /// Unregister a page
    pub fn unregister_page(&mut self, page_id: PageId) {
        self.trackers.remove(&page_id.0);
    }

    /// Get the resource tracker for a page
    pub fn get_tracker(&self, page_id: PageId) -> Option<&PageResourceTracker> {
        self.trackers.get(&page_id.0)
    }

    /// Get a mutable tracker for a page
    pub fn get_tracker_mut(&mut self, page_id: PageId) -> Option<&mut PageResourceTracker> {
        self.trackers.get_mut(&page_id.0)
    }

    /// Update resource limits for a page
    pub fn set_resource_limits(&mut self, page_id: PageId, limits: ResourceLimits) -> Result<(), WebError> {
        if let Some(tracker) = self.trackers.get_mut(&page_id.0) {
            tracker.limits = limits;
            Ok(())
        } else {
            Err(WebError::ResourceNotFound)
        }
    }

    /// Check all pages for violations and enforce limits
    pub fn enforce_limits(&mut self, js_engine: &mut JsEngine) -> Vec<ViolationEvent> {
        let mut events = Vec::new();

        for (_, tracker) in self.trackers.iter_mut() {
            let violations = tracker.check_limits();
            for violation in violations {
                self.total_violations.fetch_add(1, Ordering::Relaxed);

                // Terminate JS on critical violations
                match violation {
                    ResourceViolation::MemoryLimitExceeded
                    | ResourceViolation::JsHeapLimitExceeded
                    | ResourceViolation::JsTimeout
                    | ResourceViolation::CpuLimitExceeded => {
                        let _ = tracker.terminate_js();
                    }
                    _ => {}
                }

                events.push(ViolationEvent {
                    page_id: tracker.page_id,
                    violation,
                    current: 0,
                    limit: 0,
                    timestamp_us: 0,
                });
            }
        }

        let _ = js_engine;
        events
    }

    /// Get total tracked page count
    pub fn tracked_page_count(&self) -> usize {
        self.trackers.len()
    }
}
