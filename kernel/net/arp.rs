/*
 * Nuva OS - Kernel - Net - Arp
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
 * Nuva OS - Kernel - ARP Protocol
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Address Resolution Protocol (ARP) implementation.
 */

use crate::{pr_debug, pr_info, pr_warn};
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// ARP Header
#[repr(C, packed)]
pub struct ArpHeader {
    /// Hardware type
    pub htype: u16,
    /// Protocol type
    pub ptype: u16,
    /// Hardware address length
    pub hlen: u8,
    /// Protocol address length
    pub plen: u8,
    /// Operation
    pub oper: u16,
    /// Sender hardware address
    pub sha: [u8; 6],
    /// Sender protocol address
    pub spa: [u8; 4],
    /// Target hardware address
    pub tha: [u8; 6],
    /// Target protocol address
    pub tpa: [u8; 4],
}

impl ArpHeader {
    /// Header size
    pub const SIZE: usize = 28;

    /// Create ARP request
    pub fn request(src_mac: &[u8; 6], src_ip: &[u8; 4], target_ip: &[u8; 4]) -> Self {
        let mut sha = [0u8; 6];
        let mut spa = [0u8; 4];
        let mut tpa = [0u8; 4];
        sha.copy_from_slice(src_mac);
        spa.copy_from_slice(src_ip);
        tpa.copy_from_slice(target_ip);

        ArpHeader {
            htype: 1u16.to_be(),      // Ethernet
            ptype: 0x0800u16.to_be(), // IPv4
            hlen: 6,
            plen: 4,
            oper: 1u16.to_be(), // Request
            sha,
            spa,
            tha: [0; 6],
            tpa,
        }
    }

    /// Create ARP reply
    pub fn reply(
        src_mac: &[u8; 6],
        src_ip: &[u8; 4],
        target_mac: &[u8; 6],
        target_ip: &[u8; 4],
    ) -> Self {
        let mut sha = [0u8; 6];
        let mut spa = [0u8; 4];
        let mut tha = [0u8; 6];
        let mut tpa = [0u8; 4];
        sha.copy_from_slice(src_mac);
        spa.copy_from_slice(src_ip);
        tha.copy_from_slice(target_mac);
        tpa.copy_from_slice(target_ip);

        ArpHeader {
            htype: 1u16.to_be(),
            ptype: 0x0800u16.to_be(),
            hlen: 6,
            plen: 4,
            oper: 2u16.to_be(), // Reply
            sha,
            spa,
            tha,
            tpa,
        }
    }

    /// Get operation (host byte order)
    pub fn get_oper(&self) -> u16 {
        u16::from_be(self.oper)
    }
}

/// ARP Operation
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpOperation {
    /// Request
    Request = 1,
    /// Reply
    Reply = 2,
    /// RARP request
    RarpRequest = 3,
    /// RARP reply
    RarpReply = 4,
    /// InARP request
    InarpRequest = 8,
    /// InARP reply
    InarpReply = 9,
    /// NAK
    Nak = 10,
}

/// ARP State
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpState {
    /// Incomplete
    Incomplete = 0,
    /// Reachable
    Reachable = 1,
    /// Stale
    Stale = 2,
    /// Delay
    Delay = 3,
    /// Probe
    Probe = 4,
    /// Failed
    Failed = 5,
    /// Permanent
    Permanent = 6,
}

/// ARP Entry
#[repr(C)]
pub struct ArpEntry {
    /// IP address
    pub ip_addr: u32,
    /// MAC address
    pub mac_addr: [u8; 6],
    /// State
    pub state: AtomicU32,
    /// Last updated (jiffies)
    pub updated: AtomicU64,
    /// Last used (jiffies)
    pub used: AtomicU64,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Flags
    pub flags: u32,
    /// Device index
    pub ifindex: u32,
    /// Next entry
    pub next: *mut ArpEntry,
    pub timestamp: u64,
}

impl Clone for ArpEntry {
    fn clone(&self) -> Self {
        Self {
            ip_addr: self.ip_addr.clone(),
            mac_addr: self.mac_addr.clone(),
            state: AtomicU32::new(self.state.load(core::sync::atomic::Ordering::Relaxed)),
            updated: AtomicU64::new(self.updated.load(core::sync::atomic::Ordering::Relaxed)),
            used: AtomicU64::new(self.used.load(core::sync::atomic::Ordering::Relaxed)),
            ref_count: AtomicU32::new(self.ref_count.load(core::sync::atomic::Ordering::Relaxed)),
            flags: self.flags.clone(),
            ifindex: self.ifindex.clone(),
            next: self.next.clone(),
            timestamp: 0,
        }
    }
}

impl ArpEntry {
    pub fn new(ip_addr: u32, mac_addr: &[u8; 6], ifindex: u32) -> Self {
        let mut mac = [0u8; 6];
        mac.copy_from_slice(mac_addr);

        ArpEntry {
            ip_addr,
            mac_addr: mac,
            state: AtomicU32::new(ArpState::Reachable as u32),
            updated: AtomicU64::new(0),
            used: AtomicU64::new(0),
            ref_count: AtomicU32::new(1),
            flags: 0,
            ifindex,
            next: core::ptr::null_mut(),
            timestamp: 0,
        }
    }
}

/// ARP Table
pub struct ArpTable {
    /// Hash buckets
    buckets: [*mut ArpEntry; 256],
    /// Entry count
    entry_count: AtomicU32,
    /// Last GC time
    last_gc: AtomicU64,
}

impl ArpTable {
    pub const fn new() -> Self {
        ArpTable {
            buckets: [core::ptr::null_mut(); 256],
            entry_count: AtomicU32::new(0),
            last_gc: AtomicU64::new(0),
        }
    }

    /// Hash function
    fn hash(ip: u32) -> usize {
        // Simple hash using XOR of bytes
        let b0 = (ip & 0xFF) as u8;
        let b1 = ((ip >> 8) & 0xFF) as u8;
        let b2 = ((ip >> 16) & 0xFF) as u8;
        let b3 = ((ip >> 24) & 0xFF) as u8;
        (b0 ^ b1 ^ b2 ^ b3) as usize
    }

    /// Lookup entry
    pub fn lookup(&self, ip: u32, ifindex: u32) -> Option<[u8; 6]> {
        let idx = Self::hash(ip);
        let mut entry = self.buckets[idx];

        while !entry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*entry).ip_addr == ip && (*entry).ifindex == ifindex {
                    let state = (*entry).state.load(Ordering::Acquire);
                    if state == ArpState::Reachable as u32
                        || state == ArpState::Permanent as u32
                        || state == ArpState::Stale as u32
                    {
                        (*entry).used.fetch_add(1, Ordering::AcqRel);
                        return Some((*entry).mac_addr);
                    }
                }
                entry = (*entry).next;
            }
        }

        None
    }

    /// Add entry
    pub fn add(&mut self, ip: u32, mac: &[u8; 6], ifindex: u32, permanent: bool) {
        let idx = Self::hash(ip);

        // Check if entry already exists
        let mut entry = self.buckets[idx];
        while !entry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*entry).ip_addr == ip && (*entry).ifindex == ifindex {
                    // Update existing entry
                    (*entry).mac_addr.copy_from_slice(mac);
                    let state = if permanent {
                        ArpState::Permanent
                    } else {
                        ArpState::Reachable
                    };
                    (*entry).state.store(state as u32, Ordering::Release);
                    return;
                }
                entry = (*entry).next;
            }
        }

        // Create new entry
        // SAFETY: unsafe block required for low-level memory or hardware access
        let new_entry = unsafe {
            let ptr = alloc_arp_entry();
            if ptr.is_null() {
                log_warn!("Failed to allocate ARP entry");
                return;
            }

            let entry = &mut *ptr;
            entry.ip_addr = ip;
            entry.mac_addr = *mac;
            entry.ifindex = ifindex;
            entry.state = AtomicU32::new(ArpState::Reachable as u32);
            entry.timestamp = 0;
            entry.next = core::ptr::null_mut();

            ptr
        };

        // Add to bucket
        if self.buckets[idx].is_null() {
            self.buckets[idx] = new_entry;
        } else {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let mut tail = self.buckets[idx];
                while !(*tail).next.is_null() {
                    tail = (*tail).next;
                }
                (*tail).next = new_entry;
            }
        }

        self.entry_count.fetch_add(1, Ordering::AcqRel);

        log_debug!(
            "ARP entry added: IP={}, MAC={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.format_ip(ip),
            mac[0],
            mac[1],
            mac[2],
            mac[3],
            mac[4],
            mac[5]
        );
    }

    /// Delete entry
    pub fn delete(&mut self, ip: u32, ifindex: u32) -> bool {
        let idx = Self::hash(ip);
        let mut prev: *mut ArpEntry = core::ptr::null_mut();
        let mut entry = self.buckets[idx];

        while !entry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*entry).ip_addr == ip && (*entry).ifindex == ifindex {
                    // Remove from list
                    if prev.is_null() {
                        self.buckets[idx] = (*entry).next;
                    } else {
                        (*prev).next = (*entry).next;
                    }
                    // Free entry
                    free_arp_entry(entry);
                    self.entry_count.fetch_sub(1, Ordering::AcqRel);

                    log_debug!("ARP entry deleted: IP={}", self.format_ip(ip));

                    return true;
                }
                prev = entry;
                entry = (*entry).next;
            }
        }

        false
    }

    /// Format IP address
    fn format_ip(&self, ip: u32) -> String {
        format!(
            "{}.{}.{}.{}",
            (ip >> 24) & 0xFF,
            (ip >> 16) & 0xFF,
            (ip >> 8) & 0xFF,
            ip & 0xFF
        )
    }

    /// Get entry count
    pub fn count(&self) -> u32 {
        self.entry_count.load(Ordering::Acquire)
    }
}

/// Global ARP table
static ARP_TABLE: core::sync::OnceLock<ArpTable> = core::sync::OnceLock::new();

/// ARP entry pool
static mut ARP_ENTRY_POOL: [ArpEntry; 256] = [const {
    ArpEntry {
        ip_addr: 0,
        mac_addr: [0; 6],
        ifindex: 0,
        state: AtomicU32::new(ArpState::Incomplete as u32),
        updated: AtomicU64::new(0),
        used: AtomicU64::new(0),
        ref_count: AtomicU32::new(0),
        flags: 0,
        timestamp: 0,
        next: core::ptr::null_mut(),
    }
}; 256];
static mut ARP_POOL_IDX: usize = 0;
static mut ARP_FREE_LIST: *mut ArpEntry = core::ptr::null_mut();

/// Get ARP table
pub fn arp_table() -> &'static ArpTable {
    ARP_TABLE.get_or_init(ArpTable::new)
}

/// Allocate ARP entry
// SAFETY: The caller must ensure ARP_ENTRY_POOL and ARP_FREE_LIST are
// properly initialized and that no concurrent access occurs.
unsafe fn alloc_arp_entry() -> *mut ArpEntry {
    const POOL_SIZE: usize = 256;

    // Try to get from free list first
    if !ARP_FREE_LIST.is_null() {
        let entry = ARP_FREE_LIST;
        ARP_FREE_LIST = (*entry).next;
        return entry;
    }

    // Try to allocate from pool
    let idx = ARP_POOL_IDX;
    if idx >= POOL_SIZE {
        return core::ptr::null_mut();
    }
    ARP_POOL_IDX = idx + 1;

    ARP_ENTRY_POOL.as_mut_ptr().add(idx)
}

/// Free ARP entry
// SAFETY: The caller must ensure entry was allocated by alloc_arp_entry
// and is not concurrently accessed.
unsafe fn free_arp_entry(entry: *mut ArpEntry) {
    if entry.is_null() {
        return;
    }

    // Add to free list
    (*entry).next = ARP_FREE_LIST;
    ARP_FREE_LIST = entry;
}

/// Initialize ARP
pub fn init_arp() {
    log_info!("ARP initialized");
}
