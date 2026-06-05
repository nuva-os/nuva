/*
 * Nuva OS - Kernel - Net - Route
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
 * Nuva OS - Kernel - Routing
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * IP routing implementation.
 */

use crate::{pr_debug, pr_info, pr_warn};
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// Route Entry
#[repr(C)]
pub struct RouteEntry {
    /// Destination network
    pub dst: u32,
    /// Destination mask
    pub mask: u32,
    /// Gateway
    pub gateway: u32,
    /// Output device index
    pub ifindex: u32,
    /// Metric (priority)
    pub metric: u32,
    /// Flags
    pub flags: RouteFlags,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Use count
    pub use_count: AtomicU64,
    /// Last use time
    pub last_use: AtomicU64,
    /// Next entry
    pub next: *mut RouteEntry,
}

/// Route Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct RouteFlags: u32 {
        /// Route is up
        const RTF_UP = 0x0001;
        /// Gateway
        const RTF_GATEWAY = 0x0002;
        /// Host route
        const RTF_HOST = 0x0004;
        /// Reinstate route
        const RTF_REINSTATE = 0x0008;
        /// Dynamically installed
        const RTF_DYNAMIC = 0x0010;
        /// Modified
        const RTF_MODIFIED = 0x0020;
        /// MTU valid
        const RTF_MTU = 0x0040;
        /// Window valid
        const RTF_WINDOW = 0x0080;
        /// IRTT valid
        const RTF_IRTT = 0x0100;
        /// Reject route
        const RTF_REJECT = 0x0200;
        /// Multipath
        const RTF_MULTIPATH = 0x0400;
        /// Local delivery
        const RTF_LOCAL = 0x8000;
        /// Broadcast
        const RTF_BROADCAST = 0x10000;
        /// Anycast
        const RTF_ANYCAST = 0x20000;
    }
}

impl RouteEntry {
    pub fn new(dst: u32, mask: u32, gateway: u32, ifindex: u32) -> Self {
        let flags = if gateway != 0 {
            RouteFlags::RTF_UP | RouteFlags::RTF_GATEWAY
        } else {
            RouteFlags::RTF_UP
        };

        RouteEntry {
            dst,
            mask,
            gateway,
            ifindex,
            metric: 0,
            flags,
            ref_count: AtomicU32::new(1),
            use_count: AtomicU64::new(0),
            last_use: AtomicU64::new(0),
            next: core::ptr::null_mut(),
        }
    }

    /// Check if address matches
    pub fn matches(&self, addr: u32) -> bool {
        (addr & self.mask) == (self.dst & self.mask)
    }

    /// Get prefix length
    pub fn prefix_len(&self) -> u8 {
        let mut len = 0u8;
        let mut mask = self.mask;
        while mask != 0 {
            len += 1;
            mask <<= 1;
        }
        len
    }
}

/// Route Table
pub struct RouteTable {
    /// Route entries
    entries: [*mut RouteEntry; 256],
    /// Hash buckets (alias for entries access)
    buckets: [*mut RouteEntry; 256],
    /// Entry count
    entry_count: AtomicU32,
    /// Default route
    default_route: *mut RouteEntry,
}

impl RouteTable {
    pub const fn new() -> Self {
        RouteTable {
            entries: [core::ptr::null_mut(); 256],
            buckets: [core::ptr::null_mut(); 256],
            entry_count: AtomicU32::new(0),
            default_route: core::ptr::null_mut(),
        }
    }

    /// Hash function
    fn hash(addr: u32) -> usize {
        // Use top byte as hash
        ((addr >> 24) & 0xFF) as usize
    }

    /// Lookup route
    pub fn lookup(&self, dst: u32) -> Option<&'static RouteEntry> {
        // First, check exact hash bucket
        let idx = Self::hash(dst);
        let mut best: Option<&RouteEntry> = None;
        let mut best_len = 0u8;

        // Search hash bucket
        let mut entry = self.entries[idx];
        while !entry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*entry).matches(dst) {
                    let len = (*entry).prefix_len();
                    if len > best_len {
                        best_len = len;
                        best = Some(&*entry);
                    }
                }
                entry = (*entry).next;
            }
        }

        // Search all entries for longer matches
        for i in 0..256 {
            if i == idx {
                continue;
            }
            entry = self.entries[i];
            while !entry.is_null() {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    if (*entry).matches(dst) {
                        let len = (*entry).prefix_len();
                        if len > best_len {
                            best_len = len;
                            best = Some(&*entry);
                        }
                    }
                    entry = (*entry).next;
                }
            }
        }

        // Fall back to default route
        if best.is_none() && !self.default_route.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                best = Some(&*self.default_route);
            }
        }

        best
    }

    /// Add route
    pub fn add(&mut self, route: *mut RouteEntry) -> i32 {
        if route.is_null() {
            return Errno::Eperm.to_ret_i32();
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let idx = Self::hash((*route).dst);
            (*route).next = self.entries[idx];
            self.entries[idx] = route;
        }

        self.entry_count.fetch_add(1, Ordering::AcqRel);
        0
    }

    /// Delete route
    pub fn delete(&mut self, dst: u32, mask: u32) -> i32 {
        let idx = Self::hash(dst);
        let mut prev: *mut RouteEntry = core::ptr::null_mut();
        let mut entry = self.entries[idx];

        while !entry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*entry).dst == dst && (*entry).mask == mask {
                    if prev.is_null() {
                        self.entries[idx] = (*entry).next;
                    } else {
                        (*prev).next = (*entry).next;
                    }
                    self.entry_count.fetch_sub(1, Ordering::AcqRel);
                    return 0;
                }
                prev = entry;
                entry = (*entry).next;
            }
        }

        -1
    }

    /// Set default route
    pub fn set_default(&mut self, gateway: u32, ifindex: u32) {
        // Create default route (0.0.0.0/0)
        // SAFETY: unsafe block required for low-level memory or hardware access
        let entry = unsafe {
            let ptr = alloc_route_entry();
            if ptr.is_null() {
                log_warn!("Failed to allocate default route entry");
                return;
            }

            let entry = &mut *ptr;
            entry.dst = 0;
            entry.mask = 0;
            entry.gateway = gateway;
            entry.ifindex = ifindex;
            entry.metric = 0;
            entry.flags = RouteFlags::RTF_UP;
            entry.ref_count = AtomicU32::new(1);
            entry.use_count = AtomicU64::new(0);
            entry.last_use = AtomicU64::new(0);
            entry.next = core::ptr::null_mut();

            ptr
        };

        // Add to table
        let idx = Self::hash(0);
        if self.buckets[idx].is_null() {
            self.buckets[idx] = entry;
        } else {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let mut tail = self.buckets[idx];
                while !(*tail).next.is_null() {
                    tail = (*tail).next;
                }
                (*tail).next = entry;
            }
        }

        self.entry_count.fetch_add(1, Ordering::AcqRel);

        log_debug!(
            "Default route set: gateway={:#x}, ifindex={}",
            gateway,
            ifindex
        );
    }

    /// Get entry count
    pub fn count(&self) -> u32 {
        self.entry_count.load(Ordering::Acquire)
    }
}

/// Routing Manager
pub struct RouteManager {
    /// Route table
    pub table: RouteTable,
    /// Local address table
    pub local_addrs: [LocalAddr; 16],
    /// Local address count
    pub local_count: AtomicU32,
}

/// Local Address
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LocalAddr {
    pub addr: u32,
    pub mask: u32,
    pub ifindex: u32,
    pub scope: u8,
}

impl RouteManager {
    pub const fn new() -> Self {
        RouteManager {
            table: RouteTable::new(),
            local_addrs: [LocalAddr {
                addr: 0,
                mask: 0,
                ifindex: 0,
                scope: 0,
            }; 16],
            local_count: AtomicU32::new(0),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Routing initialized");
    }

    /// Route lookup
    pub fn route(&self, dst: u32) -> Option<&'static RouteEntry> {
        self.table.lookup(dst)
    }

    /// Add route
    pub fn add_route(&mut self, dst: u32, mask: u32, gateway: u32, ifindex: u32) -> i32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let entry = unsafe {
            let ptr = alloc_route_entry();
            if ptr.is_null() {
                log_warn!("Failed to allocate route entry");
                return Errno::Eperm.to_ret_i32();
            }

            let entry = &mut *ptr;
            entry.dst = dst;
            entry.mask = mask;
            entry.gateway = gateway;
            entry.ifindex = ifindex;
            entry.metric = 1;
            entry.flags = RouteFlags::RTF_UP;
            entry.ref_count = AtomicU32::new(1);
            entry.use_count = AtomicU64::new(0);
            entry.last_use = AtomicU64::new(0);
            entry.next = core::ptr::null_mut();

            ptr
        };

        // Add to table
        let idx = RouteTable::hash(dst);
        if self.table.buckets[idx].is_null() {
            self.table.buckets[idx] = entry;
        } else {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let mut tail = self.table.buckets[idx];
                while !(*tail).next.is_null() {
                    tail = (*tail).next;
                }
                (*tail).next = entry;
            }
        }

        self.table.entry_count.fetch_add(1, Ordering::AcqRel);

        log_debug!(
            "Route added: dst={}/{}, gateway={}, ifindex={}",
            self.format_ip(dst),
            self.mask_to_prefix(mask),
            self.format_ip(gateway),
            ifindex
        );

        0
    }

    /// Add local address
    pub fn add_local(&mut self, addr: u32, mask: u32, ifindex: u32) -> i32 {
        let count = self.local_count.load(Ordering::Acquire);
        if count >= 16 {
            return Errno::Eperm.to_ret_i32();
        }

        self.local_addrs[count as usize] = LocalAddr {
            addr,
            mask,
            ifindex,
            scope: 0,
        };
        self.local_count.fetch_add(1, Ordering::AcqRel);
        0
    }

    /// Check if address is local
    pub fn is_local(&self, addr: u32) -> bool {
        let count = self.local_count.load(Ordering::Acquire);
        for i in 0..count as usize {
            if self.local_addrs[i].addr == addr {
                return true;
            }
        }
        false
    }

    /// Format IP address
    fn format_ip(&self, ip: u32) -> String {
        format_ip(ip)
    }

    /// Convert mask to prefix length
    fn mask_to_prefix(&self, mask: u32) -> u32 {
        mask_to_prefix(mask)
    }
}

/// Global route manager
static ROUTE_MANAGER: core::sync::OnceLock<RouteManager> = core::sync::OnceLock::new();

/// Get route manager
pub fn route_manager() -> &'static RouteManager {
    ROUTE_MANAGER.get_or_init(RouteManager::new)
}

pub fn init_route_manager() -> &'static RouteManager {
    ROUTE_MANAGER.get_or_init(RouteManager::new)
}

/// Initialize routing
pub fn init_route() {
    let mgr = route_manager();
    mgr.init();
}

/// Allocate route entry
// SAFETY: The caller must ensure ENTRY_POOL and POOL_IDX are properly
// initialized and that no concurrent access occurs.
unsafe fn alloc_route_entry() -> *mut RouteEntry {
    // Simplified implementation: use static memory pool
    const POOL_SIZE: usize = 256;
    static mut ENTRY_POOL: [RouteEntry; POOL_SIZE] = [const {
        RouteEntry {
            dst: 0,
            mask: 0,
            gateway: 0,
            ifindex: 0,
            metric: 0,
            flags: RouteFlags::empty(),
            next: core::ptr::null_mut(),
            last_use: AtomicU64::new(0),
            ref_count: AtomicU32::new(0),
            use_count: AtomicU64::new(0),
        }
    }; POOL_SIZE];
    static mut POOL_IDX: usize = 0;
    static mut FREE_LIST: *mut RouteEntry = core::ptr::null_mut();

    // Try to get from free list first
    if !FREE_LIST.is_null() {
        let entry = FREE_LIST;
        FREE_LIST = (*entry).next;
        return entry;
    }

    // Try to allocate from pool
    let idx = POOL_IDX;
    if idx >= POOL_SIZE {
        return core::ptr::null_mut();
    }
    POOL_IDX = idx + 1;

    ENTRY_POOL.as_mut_ptr().add(idx)
}

/// Format IP address
fn format_ip(ip: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (ip >> 24) & 0xFF,
        (ip >> 16) & 0xFF,
        (ip >> 8) & 0xFF,
        ip & 0xFF
    )
}

/// Convert mask to prefix length
fn mask_to_prefix(mask: u32) -> u32 {
    if mask == 0 {
        return 0;
    }

    let mut prefix = 0u32;
    let mut m = mask;

    while m != 0 {
        prefix += 1;
        m <<= 1;
    }

    prefix
}
