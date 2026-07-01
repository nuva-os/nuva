/*
 * Nuva OS - Kernel - Mm - Stats
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
 * Nuva OS - Kernel - Memory Statistics
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Memory statistics and monitoring implementation.
 */

use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};
use crate::{pr_info};

/// Memory Statistics
pub struct MemoryStats {
    /// Total physical memory (bytes)
    pub total_memory: AtomicU64,
    
    /// Free memory (bytes)
    pub free_memory: AtomicU64,
    
    /// Available memory (bytes) - memory available for allocation
    pub available_memory: AtomicU64,
    
    /// Cached memory (bytes)
    pub cached_memory: AtomicU64,
    
    /// Buffer memory (bytes)
    pub buffer_memory: AtomicU64,
    
    /// Slab memory (bytes)
    pub slab_memory: AtomicU64,
    
    /// Kernel memory (bytes)
    pub kernel_memory: AtomicU64,
    
    /// User memory (bytes)
    pub user_memory: AtomicU64,
    
    /// Shared memory (bytes)
    pub shared_memory: AtomicU64,
    
    /// Page table memory (bytes)
    pub page_table_memory: AtomicU64,
    
    /// Total pages allocated
    pub total_pages: AtomicU64,
    
    /// Free pages
    pub free_pages: AtomicU64,
    
    /// Active pages
    pub active_pages: AtomicU64,
    
    /// Inactive pages
    pub inactive_pages: AtomicU64,
    
    /// Dirty pages
    pub dirty_pages: AtomicU64,
    
    /// Writeback pages
    pub writeback_pages: AtomicU64,
    
    /// Mapped pages
    pub mapped_pages: AtomicU64,
    
    /// Anonymous pages
    pub anon_pages: AtomicU64,
    
    /// File-backed pages
    pub file_pages: AtomicU64,
    
    /// Slab pages
    pub slab_pages: AtomicU64,
    
    /// Huge pages
    pub huge_pages: AtomicU64,
    
    /// Page faults
    pub page_faults: AtomicU64,
    
    /// Major page faults (requiring I/O)
    pub major_faults: AtomicU64,
    
    /// Minor page faults (not requiring I/O)
    pub minor_faults: AtomicU64,
    
    /// Memory allocations
    pub allocations: AtomicU64,
    
    /// Memory deallocations
    pub deallocations: AtomicU64,
    
    /// Page allocations
    pub page_allocs: AtomicU64,
    
    /// Page deallocations
    pub page_deallocs: AtomicU64,
    
    /// Page compactions
    pub compactions: AtomicU64,
    
    /// Page migrations
    pub migrations: AtomicU64,
    
    /// OOM kills
    pub oom_kills: AtomicU64,
}

impl MemoryStats {
    pub const fn new() -> Self {
        MemoryStats {
            total_memory: AtomicU64::new(0),
            free_memory: AtomicU64::new(0),
            available_memory: AtomicU64::new(0),
            cached_memory: AtomicU64::new(0),
            buffer_memory: AtomicU64::new(0),
            slab_memory: AtomicU64::new(0),
            kernel_memory: AtomicU64::new(0),
            user_memory: AtomicU64::new(0),
            shared_memory: AtomicU64::new(0),
            page_table_memory: AtomicU64::new(0),
            total_pages: AtomicU64::new(0),
            free_pages: AtomicU64::new(0),
            active_pages: AtomicU64::new(0),
            inactive_pages: AtomicU64::new(0),
            dirty_pages: AtomicU64::new(0),
            writeback_pages: AtomicU64::new(0),
            mapped_pages: AtomicU64::new(0),
            anon_pages: AtomicU64::new(0),
            file_pages: AtomicU64::new(0),
            slab_pages: AtomicU64::new(0),
            huge_pages: AtomicU64::new(0),
            page_faults: AtomicU64::new(0),
            major_faults: AtomicU64::new(0),
            minor_faults: AtomicU64::new(0),
            allocations: AtomicU64::new(0),
            deallocations: AtomicU64::new(0),
            page_allocs: AtomicU64::new(0),
            page_deallocs: AtomicU64::new(0),
            compactions: AtomicU64::new(0),
            migrations: AtomicU64::new(0),
            oom_kills: AtomicU64::new(0),
        }
    }
    
    /// Update total memory
    pub fn set_total_memory(&self, total: u64) {
        self.total_memory.store(total, Ordering::Release);
    }
    
    /// Update free memory
    pub fn set_free_memory(&self, free: u64) {
        self.free_memory.store(free, Ordering::Release);
    }
    
    /// Update available memory
    pub fn set_available_memory(&self, available: u64) {
        self.available_memory.store(available, Ordering::Release);
    }
    
    /// Record allocation
    pub fn record_allocation(&self, size: u64) {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        self.available_memory.fetch_sub(size, Ordering::Relaxed);
    }
    
    /// Record deallocation
    pub fn record_deallocation(&self, size: u64) {
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.available_memory.fetch_add(size, Ordering::Relaxed);
    }
    
    /// Record page allocation
    pub fn record_page_alloc(&self) {
        self.page_allocs.fetch_add(1, Ordering::Relaxed);
        self.total_pages.fetch_add(1, Ordering::Relaxed);
        self.free_pages.fetch_sub(1, Ordering::Relaxed);
    }
    
    /// Record page deallocation
    pub fn record_page_dealloc(&self) {
        self.page_deallocs.fetch_add(1, Ordering::Relaxed);
        self.free_pages.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record page fault
    pub fn record_page_fault(&self, major: bool) {
        self.page_faults.fetch_add(1, Ordering::Relaxed);
        if major {
            self.major_faults.fetch_add(1, Ordering::Relaxed);
        } else {
            self.minor_faults.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Get memory usage percentage
    pub fn get_memory_usage_percent(&self) -> u8 {
        let total = self.total_memory.load(Ordering::Acquire);
        if total == 0 {
            return 0;
        }
        
        let used = total - self.free_memory.load(Ordering::Acquire);
        ((used * 100) / total) as u8
    }
    
    /// Get memory statistics as a string
    pub fn to_string(&self) -> String {
        let total = self.total_memory.load(Ordering::Acquire);
        let free = self.free_memory.load(Ordering::Acquire);
        let available = self.available_memory.load(Ordering::Acquire);
        let cached = self.cached_memory.load(Ordering::Acquire);
        let kernel = self.kernel_memory.load(Ordering::Acquire);
        let user = self.user_memory.load(Ordering::Acquire);
        
        let total_mb = total / (1024 * 1024);
        let free_mb = free / (1024 * 1024);
        let available_mb = available / (1024 * 1024);
        let cached_mb = cached / (1024 * 1024);
        let kernel_mb = kernel / (1024 * 1024);
        let user_mb = user / (1024 * 1024);
        
        format!(
            "Memory: total={}MB, free={}MB, available={}MB, cached={}MB, kernel={}MB, user={}MB",
            total_mb, free_mb, available_mb, cached_mb, kernel_mb, user_mb
        )
    }
}

/// Memory Monitoring
pub struct MemoryMonitor {
    /// Statistics
    pub stats: MemoryStats,
    
    /// Monitoring enabled
    pub enabled: AtomicBool,
    
    /// High watermark (bytes)
    pub high_watermark: AtomicU64,
    
    /// Low watermark (bytes)
    pub low_watermark: AtomicU64,
    
    /// Critical watermark (bytes)
    pub critical_watermark: AtomicU64,
}

impl MemoryMonitor {
    pub const fn new() -> Self {
        MemoryMonitor {
            stats: MemoryStats::new(),
            enabled: AtomicBool::new(false),
            high_watermark: AtomicU64::new(0),
            low_watermark: AtomicU64::new(0),
            critical_watermark: AtomicU64::new(0),
        }
    }
    
    /// Initialize
    pub fn init(&mut self, total_memory: u64) {
        self.stats.set_total_memory(total_memory);
        self.stats.set_free_memory(total_memory);
        self.stats.set_available_memory(total_memory);
        
        // Set watermarks
        self.high_watermark.store(total_memory * 80 / 100, Ordering::Release);
        self.low_watermark.store(total_memory * 20 / 100, Ordering::Release);
        self.critical_watermark.store(total_memory * 5 / 100, Ordering::Release);
        
        log_info!("Memory monitor initialized: total={}MB", total_memory / (1024 * 1024));
    }
    
    /// Enable monitoring
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
        log_info!("Memory monitoring enabled");
    }
    
    /// Disable monitoring
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Release);
        log_info!("Memory monitoring disabled");
    }
    
    /// Check memory pressure
    pub fn check_pressure(&self) -> MemoryPressure {
        if !self.enabled.load(Ordering::Acquire) {
            return MemoryPressure::Normal;
        }
        
        let available = self.stats.available_memory.load(Ordering::Acquire);
        let high = self.high_watermark.load(Ordering::Acquire);
        let low = self.low_watermark.load(Ordering::Acquire);
        let critical = self.critical_watermark.load(Ordering::Acquire);
        
        if available < critical {
            MemoryPressure::Critical
        } else if available < low {
            MemoryPressure::Low
        } else if available < high {
            MemoryPressure::High
        } else {
            MemoryPressure::Normal
        }
    }
    
    /// Get memory statistics
    pub fn get_stats(&self) -> &MemoryStats {
        &self.stats
    }
    
    /// Print memory statistics
    pub fn print_stats(&self) {
        log_info!("{}", self.stats.to_string());
        
        let pressure = self.check_pressure();
        log_info!("Memory pressure: {:?}", pressure);
    }
}

/// Memory Pressure Level
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    Normal = 0,
    High = 1,
    Low = 2,
    Critical = 3,
}

/// Global memory monitor
static MEMORY_MONITOR: crate::sync_oncelock::OnceLock<MemoryMonitor> = crate::sync_oncelock::OnceLock::new();

/// Get memory monitor
pub fn memory_monitor() -> &'static MemoryMonitor {
    MEMORY_MONITOR.get_or_init(MemoryMonitor::new)
}

/// Initialize memory monitoring
pub fn init_memory_monitoring(total_memory: u64) {
    let monitor = memory_monitor();
    monitor.init(total_memory);
}
