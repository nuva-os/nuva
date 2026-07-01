/*
 * Nuva OS - Kernel - Net - Firewall
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
 * Nuva OS - Kernel - Network Firewall
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Network firewall implementation for packet filtering.
 */

use crate::{pr_debug, pr_info, pr_warn};
use core::sync::atomic::{AtomicU32, Ordering};

/// Firewall Action
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallAction {
    Accept = 0,
    Drop = 1,
    Reject = 2,
}

/// Firewall Protocol
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallProtocol {
    Any = 0,
    TCP = 6,
    UDP = 17,
    ICMP = 1,
    ICMPv6 = 58,
}

/// Firewall Rule Flags
pub mod firewall_flags {
    pub const INBOUND: u32 = 0x01;
    pub const OUTBOUND: u32 = 0x02;
    pub const FORWARD: u32 = 0x04;
    pub const LOG: u32 = 0x08;
}

/// Firewall Rule
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FirewallRule {
    /// Rule ID
    pub id: u32,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Flags
    pub flags: u32,
    /// Action
    pub action: FirewallAction,
    /// Protocol
    pub protocol: FirewallProtocol,
    /// Source IP address
    pub src_ip: u32,
    /// Source IP mask
    pub src_mask: u32,
    /// Source port (0 = any)
    pub src_port: u16,
    /// Destination IP address
    pub dst_ip: u32,
    /// Destination IP mask
    pub dst_mask: u32,
    /// Destination port (0 = any)
    pub dst_port: u16,
    /// Next rule
    pub next: *mut FirewallRule,
}

impl FirewallRule {
    /// Create new firewall rule
    pub fn new(
        id: u32,
        priority: u32,
        flags: u32,
        action: FirewallAction,
        protocol: FirewallProtocol,
    ) -> Self {
        FirewallRule {
            id,
            priority,
            flags,
            action,
            protocol,
            src_ip: 0,
            src_mask: 0,
            src_port: 0,
            dst_ip: 0,
            dst_mask: 0,
            dst_port: 0,
            next: core::ptr::null_mut(),
        }
    }

    /// Check if rule matches packet
    pub fn matches(
        &self,
        src_ip: u32,
        dst_ip: u32,
        src_port: u16,
        dst_port: u16,
        protocol: u8,
    ) -> bool {
        // Check protocol
        if self.protocol != FirewallProtocol::Any && self.protocol as u8 != protocol {
            return false;
        }

        // Check source IP
        if self.src_ip != 0 && (src_ip & self.src_mask) != (self.src_ip & self.src_mask) {
            return false;
        }

        // Check destination IP
        if self.dst_ip != 0 && (dst_ip & self.dst_mask) != (self.dst_ip & self.dst_mask) {
            return false;
        }

        // Check source port
        if self.src_port != 0 && src_port != self.src_port {
            return false;
        }

        // Check destination port
        if self.dst_port != 0 && dst_port != self.dst_port {
            return false;
        }

        true
    }
}

/// Firewall Statistics
pub struct FirewallStats {
    /// Total packets processed
    pub total_packets: AtomicU32,
    /// Accepted packets
    pub accepted: AtomicU32,
    /// Dropped packets
    pub dropped: AtomicU32,
    /// Rejected packets
    pub rejected: AtomicU32,
    /// Logged packets
    pub logged: AtomicU32,
}

impl FirewallStats {
    pub const fn new() -> Self {
        FirewallStats {
            total_packets: AtomicU32::new(0),
            accepted: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
            rejected: AtomicU32::new(0),
            logged: AtomicU32::new(0),
        }
    }
}

/// Firewall Manager
pub struct FirewallManager {
    /// Rule list
    pub rules: *mut FirewallRule,
    /// Rule count
    pub rule_count: AtomicU32,
    /// Next rule ID
    pub next_id: AtomicU32,
    /// Statistics
    pub stats: FirewallStats,
    /// Default action
    pub default_action: FirewallAction,
}

impl FirewallManager {
    pub const fn new() -> Self {
        FirewallManager {
            rules: core::ptr::null_mut(),
            rule_count: AtomicU32::new(0),
            next_id: AtomicU32::new(1),
            stats: FirewallStats::new(),
            default_action: FirewallAction::Accept,
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Firewall initialized");
        log_info!("Default action: {:?}", self.default_action);
    }

    /// Set default action
    pub fn set_default_action(&mut self, action: FirewallAction) {
        self.default_action = action;
        log_info!("Firewall default action set to: {:?}", action);
    }

    /// Add rule
    pub fn add_rule(&mut self, mut rule: FirewallRule) -> u32 {
        rule.id = self.next_id.fetch_add(1, Ordering::AcqRel);

        // SAFETY: unsafe block required for low-level memory or hardware access
        let rule_ptr = unsafe {
            let ptr = alloc_firewall_rule();
            if ptr.is_null() {
                log_warn!("Failed to allocate firewall rule");
                return 0;
            }

            *ptr = rule;
            ptr
        };

        // Insert rule in priority order
        let mut prev: *mut FirewallRule = core::ptr::null_mut();
        let mut current = self.rules;

        while !current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*current).priority >= rule.priority {
                    break;
                }
                prev = current;
                current = (*current).next;
            }
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*rule_ptr).next = current;

            if prev.is_null() {
                self.rules = rule_ptr;
            } else {
                (*prev).next = rule_ptr;
            }
        }

        self.rule_count.fetch_add(1, Ordering::AcqRel);

        log_debug!(
            "Firewall rule added: id={}, priority={}, action={:?}",
            rule.id,
            rule.priority,
            rule.action
        );

        rule.id
    }

    /// Delete rule
    pub fn delete_rule(&mut self, id: u32) -> bool {
        let mut prev: *mut FirewallRule = core::ptr::null_mut();
        let mut current = self.rules;

        while !current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*current).id == id {
                    // Remove from list
                    if prev.is_null() {
                        self.rules = (*current).next;
                    } else {
                        (*prev).next = (*current).next;
                    }

                    // Free rule
                    free_firewall_rule(current);

                    self.rule_count.fetch_sub(1, Ordering::AcqRel);

                    log_debug!("Firewall rule deleted: id={}", id);

                    return true;
                }

                prev = current;
                current = (*current).next;
            }
        }

        false
    }

    /// Filter packet
    pub fn filter(
        &mut self,
        src_ip: u32,
        dst_ip: u32,
        src_port: u16,
        dst_port: u16,
        protocol: u8,
    ) -> FirewallAction {
        self.stats.total_packets.fetch_add(1, Ordering::AcqRel);

        let mut current = self.rules;

        while !current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let rule = &*current;

                if rule.matches(src_ip, dst_ip, src_port, dst_port, protocol) {
                    let action = rule.action;

                    // Log if requested
                    if (rule.flags & firewall_flags::LOG) != 0 {
                        self.stats.logged.fetch_add(1, Ordering::AcqRel);
                        log_debug!("Firewall log: src={:#x}, dst={:#x}, src_port={}, dst_port={}, proto={}, action={:?}", 
                                  src_ip, dst_ip, src_port, dst_port, protocol, action);
                    }

                    // Update statistics
                    match action {
                        FirewallAction::Accept => {
                            self.stats.accepted.fetch_add(1, Ordering::AcqRel);
                        }
                        FirewallAction::Drop => {
                            self.stats.dropped.fetch_add(1, Ordering::AcqRel);
                        }
                        FirewallAction::Reject => {
                            self.stats.rejected.fetch_add(1, Ordering::AcqRel);
                        }
                    }

                    return action;
                }

                current = (*current).next;
            }
        }

        // No rule matched, use default action
        match self.default_action {
            FirewallAction::Accept => {
                self.stats.accepted.fetch_add(1, Ordering::AcqRel);
            }
            FirewallAction::Drop => {
                self.stats.dropped.fetch_add(1, Ordering::AcqRel);
            }
            FirewallAction::Reject => {
                self.stats.rejected.fetch_add(1, Ordering::AcqRel);
            }
        }

        self.default_action
    }

    /// Get rule count
    pub fn rule_count(&self) -> u32 {
        self.rule_count.load(Ordering::Acquire)
    }

    /// Get statistics
    pub fn get_stats(&self) -> &FirewallStats {
        &self.stats
    }
}

/// Firewall rule pool
static mut FW_RULE_POOL: [FirewallRule; 256] = [FirewallRule {
    id: 0,
    priority: 0,
    flags: 0,
    action: FirewallAction::Accept,
    protocol: FirewallProtocol::Any,
    src_ip: 0,
    src_mask: 0,
    src_port: 0,
    dst_ip: 0,
    dst_mask: 0,
    dst_port: 0,
    next: core::ptr::null_mut(),
}; 256];
static mut FW_POOL_IDX: usize = 0;
static mut FW_FREE_LIST: *mut FirewallRule = core::ptr::null_mut();

/// Allocate firewall rule
// SAFETY: The caller must ensure FW_RULE_POOL and FW_FREE_LIST are
// properly initialized and that no concurrent access occurs.
unsafe fn alloc_firewall_rule() -> *mut FirewallRule {
    const POOL_SIZE: usize = 256;

    // Try to get from free list first
    if !FW_FREE_LIST.is_null() {
        let rule = FW_FREE_LIST;
        FW_FREE_LIST = (*rule).next;
        return rule;
    }

    // Try to allocate from pool
    let idx = FW_POOL_IDX;
    if idx >= POOL_SIZE {
        return core::ptr::null_mut();
    }
    FW_POOL_IDX = idx + 1;

    FW_RULE_POOL.as_mut_ptr().add(idx)
}

/// Free firewall rule
// SAFETY: The caller must ensure rule was allocated by alloc_firewall_rule
// and is not concurrently accessed.
unsafe fn free_firewall_rule(rule: *mut FirewallRule) {
    if rule.is_null() {
        return;
    }

    // Add to free list
    (*rule).next = FW_FREE_LIST;
    FW_FREE_LIST = rule;
}

/// Global firewall manager
static FIREWALL_MANAGER: crate::sync_oncelock::OnceLock<FirewallManager> = crate::sync_oncelock::OnceLock::new();

/// Get firewall manager
pub fn firewall_manager() -> &'static FirewallManager {
    FIREWALL_MANAGER.get_or_init(FirewallManager::new)
}

pub fn init_firewall_manager() -> &'static FirewallManager {
    FIREWALL_MANAGER.get_or_init(FirewallManager::new)
}

/// Initialize firewall
pub fn init_firewall() {
    let mgr = firewall_manager();
    mgr.init();
}
