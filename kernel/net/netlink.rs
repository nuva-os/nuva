/*
 * Nuva OS - Kernel - Net - Netlink
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
 * Nuva OS - Kernel - Netlink
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel-user communication socket (Netlink-compatible protocol).
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_info};

/// Address family constants for kernel-user communication sockets.
pub mod address_family {
    /// Kernel-user communication socket family.
    /// Equivalent to AF_NETLINK (16) on Linux-compatible platforms.
    pub const AF_KERN_USER: u16 = 16;
}

/// Netlink Socket Address
#[repr(C)]
pub struct SockAddrNl {
    /// Address family (AF_KERN_USER)
    pub nl_family: u16,
    /// Padding
    pub nl_pad: u16,
    /// Port ID (process ID)
    pub nl_pid: u32,
    /// Multicast groups
    pub nl_groups: u32,
}

impl SockAddrNl {
    pub fn new(pid: u32, groups: u32) -> Self {
        SockAddrNl {
            nl_family: address_family::AF_KERN_USER,
            nl_pad: 0,
            nl_pid: pid,
            nl_groups: groups,
        }
    }
}

/// Netlink Message Header
#[repr(C)]
pub struct NlMsgHdr {
    /// Message length (including header)
    pub nlmsg_len: u32,
    /// Message type
    pub nlmsg_type: u16,
    /// Message flags
    pub nlmsg_flags: u16,
    /// Sequence number
    pub nlmsg_seq: u32,
    /// Sender port ID
    pub nlmsg_pid: u32,
}

impl NlMsgHdr {
    /// Header size
    pub const SIZE: usize = 16;
    
    pub fn new(msg_type: u16, flags: u16, seq: u32, pid: u32, len: u32) -> Self {
        NlMsgHdr {
            nlmsg_len: len + Self::SIZE as u32,
            nlmsg_type: msg_type,
            nlmsg_flags: flags,
            nlmsg_seq: seq,
            nlmsg_pid: pid,
        }
    }
}

/// Netlink Message Flags
pub mod nlmsg_flags {
    /// Request
    pub const REQUEST: u16 = 0x01;
    /// Multi-part message
    pub const MULTI: u16 = 0x02;
    /// Acknowledge
    pub const ACK: u16 = 0x04;
    /// Echo
    pub const ECHO: u16 = 0x08;
    /// Dump inconsistent
    pub const DUMP_INTR: u16 = 0x10;
    /// Dump filtered
    pub const DUMP_FILTERED: u16 = 0x20;
    /// Root user
    pub const ROOT: u16 = 0x100;
    /// Match all
    pub const MATCH: u16 = 0x200;
    /// Atomic operation
    pub const ATOMIC: u16 = 0x400;
    /// Dump done
    pub const DONE: u16 = 0x200;
    /// Error message
    pub const ERROR: u16 = 0x400;
    /// No truncate
    pub const NO_TRUNC: u16 = 0x1000;
}

/// Netlink Protocol Types
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetlinkProtocol {
    /// Route
    Route = 0,
    /// Usersock
    Usersock = 2,
    /// Firewall
    Firewall = 3,
    /// Security module (NSM audit)
    NsmAudit = 7,
    /// Audit
    Audit = 9,
    /// Generic
    Generic = 16,
    /// SCSI
    Scsi = 18,
    /// RDMA
    Rdma = 20,
    /// Crypto
    Crypto = 21,
}

/// Netlink Attribute
#[repr(C)]
pub struct NlAttr {
    /// Attribute length
    pub nla_len: u16,
    /// Attribute type
    pub nla_type: u16,
}

impl NlAttr {
    /// Header size
    pub const SIZE: usize = 4;
    
    pub fn new(attr_type: u16, len: u16) -> Self {
        NlAttr {
            nla_len: len + Self::SIZE as u16,
            nla_type: attr_type,
        }
    }
}

/// Netlink Error
#[repr(C)]
pub struct NlMsgErr {
    /// Error code (negative errno)
    pub error: i32,
    /// Original message header
    pub msg: NlMsgHdr,
}

/// Netlink Socket
pub struct NlSock {
    /// Port ID
    pub pid: u32,
    /// Subscribed groups
    pub groups: AtomicU32,
    /// Receive buffer
    pub rx_buf: *mut u8,
    /// Receive buffer size
    pub rx_buf_size: u32,
    /// Send buffer
    pub tx_buf: *mut u8,
    /// Send buffer size
    pub tx_buf_size: u32,
    /// Reference count
    pub ref_count: AtomicU32,
}

impl NlSock {
    pub fn new(pid: u32) -> Self {
        NlSock {
            pid,
            groups: AtomicU32::new(0),
            rx_buf: core::ptr::null_mut(),
            rx_buf_size: 0,
            tx_buf: core::ptr::null_mut(),
            tx_buf_size: 0,
            ref_count: AtomicU32::new(1),
        }
    }
    
    /// Subscribe to group
    pub fn subscribe(&self, group: u32) {
        self.groups.fetch_or(1 << group, Ordering::AcqRel);
    }
    
    /// Unsubscribe from group
    pub fn unsubscribe(&self, group: u32) {
        self.groups.fetch_and(!(1 << group), Ordering::AcqRel);
    }
}

/// Netlink Statistics
pub struct NlStats {
    /// Messages received
    pub rx_msgs: AtomicU64,
    /// Messages sent
    pub tx_msgs: AtomicU64,
    /// Bytes received
    pub rx_bytes: AtomicU64,
    /// Bytes sent
    pub tx_bytes: AtomicU64,
    /// Errors
    pub errors: AtomicU64,
}

impl NlStats {
    pub const fn new() -> Self {
        NlStats {
            rx_msgs: AtomicU64::new(0),
            tx_msgs: AtomicU64::new(0),
            rx_bytes: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
}

/// Netlink Manager
pub struct NlManager {
    /// Statistics
    pub stats: NlStats,
    /// Socket count
    pub sock_count: AtomicU32,
    /// Next port ID
    pub next_pid: AtomicU32,
}

impl NlManager {
    pub const fn new() -> Self {
        NlManager {
            stats: NlStats::new(),
            sock_count: AtomicU32::new(0),
            next_pid: AtomicU32::new(1),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        log_info!("Netlink initialized");
    }
    
    /// Create socket
    pub fn create_socket(&mut self) -> u32 {
        let pid = self.next_pid.fetch_add(1, Ordering::AcqRel);
        self.sock_count.fetch_add(1, Ordering::AcqRel);
        pid
    }
    
    /// Send message
    pub fn send(&mut self, _msg: &[u8]) -> i32 {
        self.stats.tx_msgs.fetch_add(1, Ordering::AcqRel);
        // TODO: Send netlink message
        0
    }
    
    /// Broadcast to group
    pub fn broadcast(&mut self, _msg: &[u8], _group: u32) -> i32 {
        // TODO: Broadcast to all sockets subscribed to group
        0
    }
    
    /// Unicast to specific PID
    pub fn unicast(&mut self, _msg: &[u8], _pid: u32) -> i32 {
        // TODO: Send to specific socket
        0
    }
}

/// Global netlink manager
static NL_MANAGER: crate::sync_oncelock::OnceLock<NlManager> = crate::sync_oncelock::OnceLock::new();

/// Get netlink manager
pub fn nl_manager() -> &'static NlManager {
    NL_MANAGER.get_or_init(NlManager::new)
}

pub fn init_nl_manager() -> &'static NlManager {
    NL_MANAGER.get_or_init(NlManager::new)
}

/// Initialize netlink
pub fn init_netlink() {
    let mgr = nl_manager();
    mgr.init();
}
