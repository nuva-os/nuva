/*
 * Nuva OS - Kernel - Net - Icmpv6
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
 * Nuva OS - Kernel - ICMPv6 Protocol
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Internet Control Message Protocol version 6 (ICMPv6) implementation.
 */

use core::sync::atomic::{AtomicU64, Ordering};

use crate::kernel::net::ipv6::Ipv6Addr;
use crate::{pr_debug, pr_info, pr_warn};

use crate::posix::errno::Errno;
/// ICMPv6 Header
#[repr(C, packed)]
pub struct Icmpv6Header {
    /// Type
    pub type_: u8,
    /// Code
    pub code: u8,
    /// Checksum
    pub checksum: u16,
}

impl Icmpv6Header {
    /// Header size
    pub const SIZE: usize = 4;
    
    /// Create new ICMPv6 header
    pub fn new(type_: u8, code: u8) -> Self {
        Icmpv6Header {
            type_,
            code,
            checksum: 0,
        }
    }
}

/// ICMPv6 Message Types
pub mod icmpv6_type {
    pub const DST_UNREACH: u8 = 1;      // Destination Unreachable
    pub const PACKET_TOO_BIG: u8 = 2;    // Packet Too Big
    pub const TIME_EXCEEDED: u8 = 3;     // Time Exceeded
    pub const PARAM_PROBLEM: u8 = 4;     // Parameter Problem
    pub const ECHO_REQUEST: u8 = 128;    // Echo Request
    pub const ECHO_REPLY: u8 = 129;      // Echo Reply
    pub const MLD_QUERY: u8 = 130;       // Multicast Listener Query
    pub const MLD_REPORT: u8 = 131;      // Multicast Listener Report
    pub const MLD_DONE: u8 = 132;        // Multicast Listener Done
    pub const ND_RS: u8 = 133;           // Router Solicitation
    pub const ND_RA: u8 = 134;           // Router Advertisement
    pub const ND_NS: u8 = 135;           // Neighbor Solicitation
    pub const ND_NA: u8 = 136;           // Neighbor Advertisement
    pub const ND_REDIRECT: u8 = 137;     // Redirect Message
}

/// ICMPv6 Destination Unreachable Codes
pub mod icmpv6_code_dst_unreach {
    pub const NOROUTE: u8 = 0;           // No route to destination
    pub const ADM_PROHIBITED: u8 = 1;    // Communication with destination administratively prohibited
    pub const NOT_NEIGHBOUR: u8 = 2;     // Beyond scope of source address
    pub const ADDR_UNREACH: u8 = 3;      // Address unreachable
    pub const PORT_UNREACH: u8 = 4;      // Port unreachable
}

/// ICMPv6 Echo Request/Reply
#[repr(C, packed)]
pub struct Icmpv6Echo {
    /// Identifier
    pub identifier: u16,
    /// Sequence number
    pub sequence: u16,
    /// Data
    pub data: [u8; 0],
}

impl Icmpv6Echo {
    /// Create new ICMPv6 echo message
    pub fn new(identifier: u16, sequence: u16) -> Self {
        Icmpv6Echo {
            identifier: identifier.to_be(),
            sequence: sequence.to_be(),
            data: [],
        }
    }
    
    /// Get identifier
    pub fn get_identifier(&self) -> u16 {
        u16::from_be(self.identifier)
    }
    
    /// Get sequence number
    pub fn get_sequence(&self) -> u16 {
        u16::from_be(self.sequence)
    }
}

/// ICMPv6 Neighbor Solicitation
#[repr(C, packed)]
pub struct Icmpv6NeighborSolicitation {
    /// Reserved
    pub reserved: u32,
    /// Target address
    pub target_addr: Ipv6Addr,
}

impl Icmpv6NeighborSolicitation {
    /// Create new neighbor solicitation
    pub fn new(target_addr: Ipv6Addr) -> Self {
        Icmpv6NeighborSolicitation {
            reserved: 0,
            target_addr,
        }
    }
}

/// ICMPv6 Neighbor Advertisement
#[repr(C, packed)]
pub struct Icmpv6NeighborAdvertisement {
    /// Flags (Router, Solicited, Override)
    pub flags: u32,
    /// Target address
    pub target_addr: Ipv6Addr,
}

impl Icmpv6NeighborAdvertisement {
    /// Create new neighbor advertisement
    pub fn new(target_addr: Ipv6Addr, router: bool, solicited: bool, override_: bool) -> Self {
        let mut flags = 0u32;
        if router { flags |= 0x80000000; }
        if solicited { flags |= 0x40000000; }
        if override_ { flags |= 0x20000000; }
        
        Icmpv6NeighborAdvertisement {
            flags: flags.to_be(),
            target_addr,
        }
    }
    
    /// Check if router flag is set
    pub fn is_router(&self) -> bool {
        (u32::from_be(self.flags) & 0x80000000) != 0
    }
    
    /// Check if solicited flag is set
    pub fn is_solicited(&self) -> bool {
        (u32::from_be(self.flags) & 0x40000000) != 0
    }
    
    /// Check if override flag is set
    pub fn is_override(&self) -> bool {
        (u32::from_be(self.flags) & 0x20000000) != 0
    }
}

/// ICMPv6 Router Advertisement
#[repr(C, packed)]
pub struct Icmpv6RouterAdvertisement {
    /// Current hop limit
    pub hop_limit: u8,
    /// Flags (Managed, Other)
    pub flags: u8,
    /// Router lifetime
    pub router_lifetime: u16,
    /// Reachable time
    pub reachable_time: u32,
    /// Retrans timer
    pub retrans_timer: u32,
}

impl Icmpv6RouterAdvertisement {
    /// Create new router advertisement
    pub fn new(hop_limit: u8, managed: bool, other: bool, lifetime: u16) -> Self {
        let mut flags = 0u8;
        if managed { flags |= 0x80; }
        if other { flags |= 0x40; }
        
        Icmpv6RouterAdvertisement {
            hop_limit,
            flags,
            router_lifetime: lifetime.to_be(),
            reachable_time: 0,
            retrans_timer: 0,
        }
    }
    
    /// Get hop limit
    pub fn get_hop_limit(&self) -> u8 {
        self.hop_limit
    }
    
    /// Check if managed flag is set
    pub fn is_managed(&self) -> bool {
        (self.flags & 0x80) != 0
    }
    
    /// Check if other flag is set
    pub fn is_other(&self) -> bool {
        (self.flags & 0x40) != 0
    }
    
    /// Get router lifetime
    pub fn get_router_lifetime(&self) -> u16 {
        u16::from_be(self.router_lifetime)
    }
}

/// ICMPv6 Statistics
pub struct Icmpv6Stats {
    /// Messages received
    pub in_msgs: AtomicU64,
    /// Messages sent
    pub out_msgs: AtomicU64,
    /// Errors
    pub in_errors: AtomicU64,
    /// Echo requests received
    pub in_echos: AtomicU64,
    /// Echo requests sent
    pub out_echos: AtomicU64,
    /// Echo replies received
    pub in_echo_replies: AtomicU64,
    /// Echo replies sent
    pub out_echo_replies: AtomicU64,
    /// Destination unreachables received
    pub in_dest_unreachs: AtomicU64,
    /// Destination unreachables sent
    pub out_dest_unreachs: AtomicU64,
}

impl Icmpv6Stats {
    pub const fn new() -> Self {
        Icmpv6Stats {
            in_msgs: AtomicU64::new(0),
            out_msgs: AtomicU64::new(0),
            in_errors: AtomicU64::new(0),
            in_echos: AtomicU64::new(0),
            out_echos: AtomicU64::new(0),
            in_echo_replies: AtomicU64::new(0),
            out_echo_replies: AtomicU64::new(0),
            in_dest_unreachs: AtomicU64::new(0),
            out_dest_unreachs: AtomicU64::new(0),
        }
    }
}

/// ICMPv6 Manager
pub struct Icmpv6Manager {
    /// Statistics
    pub stats: Icmpv6Stats,
    /// Echo identifier
    pub echo_identifier: u16,
    /// Echo sequence number
    pub echo_sequence: u16,
}

impl Icmpv6Manager {
    pub const fn new() -> Self {
        Icmpv6Manager {
            stats: Icmpv6Stats::new(),
            echo_identifier: 0,
            echo_sequence: 0,
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        log_info!("ICMPv6 initialized");
    }
    
    /// Process received message
    pub fn receive(&mut self, data: &[u8], src_addr: Ipv6Addr, dst_addr: Ipv6Addr) -> i32 {
        self.stats.in_msgs.fetch_add(1, Ordering::AcqRel);
        
        // Parse ICMPv6 header
        if data.len() < Icmpv6Header::SIZE {
            log_warn!("ICMPv6 message too short");
            self.stats.in_errors.fetch_add(1, Ordering::AcqRel);
            return Errno::Eperm.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        let header = unsafe { &*(data.as_ptr() as *const Icmpv6Header) };
        let type_ = header.type_;
        let code = header.code;
        
        log_debug!("ICMPv6 receive: type={}, code={}, src={}, dst={}", 
                  type_, code, src_addr.to_string(), dst_addr.to_string());
        
        match type_ {
            icmpv6_type::ECHO_REPLY => {
                log_debug!("Received echo reply (pong)");
                self.stats.in_echo_replies.fetch_add(1, Ordering::AcqRel);
            }
            icmpv6_type::ECHO_REQUEST => {
                log_debug!("Received echo request (ping)");
                self.stats.in_echos.fetch_add(1, Ordering::AcqRel);
                
                // Send echo reply
                if data.len() >= Icmpv6Header::SIZE + 4 {
                    // SAFETY: unsafe block required for low-level memory or hardware access
                    let echo = unsafe { &*(data[Icmpv6Header::SIZE..].as_ptr() as *const Icmpv6Echo) };
                    let identifier = echo.get_identifier();
                    let sequence = echo.get_sequence();
                    self.send_echo_reply(src_addr, identifier, sequence, &data[Icmpv6Header::SIZE + 4..]);
                }
            }
            icmpv6_type::DST_UNREACH => {
                log_debug!("Destination unreachable, code={}", code);
                self.stats.in_dest_unreachs.fetch_add(1, Ordering::AcqRel);
            }
            icmpv6_type::PACKET_TOO_BIG => {
                log_debug!("Packet too big");
            }
            icmpv6_type::TIME_EXCEEDED => {
                log_debug!("Time exceeded, code={}", code);
            }
            icmpv6_type::PARAM_PROBLEM => {
                log_debug!("Parameter problem, code={}", code);
            }
            icmpv6_type::ND_NS => {
                log_debug!("Received neighbor solicitation");
            }
            icmpv6_type::ND_NA => {
                log_debug!("Received neighbor advertisement");
            }
            icmpv6_type::ND_RA => {
                log_debug!("Received router advertisement");
            }
            _ => {
                log_debug!("Unknown ICMPv6 type: {}", type_);
            }
        }
        
        0
    }
    
    /// Send echo request (ping)
    pub fn ping(&mut self, dst: Ipv6Addr) -> i32 {
        self.stats.out_echos.fetch_add(1, Ordering::AcqRel);
        self.stats.out_msgs.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("ICMPv6 ping: dst={}, id={}, seq={}", 
                  dst.to_string(), self.echo_identifier, self.echo_sequence);
        
        // Build ICMPv6 echo request
        let mut header = Icmpv6Header::new(icmpv6_type::ECHO_REQUEST, 0);
        let echo = Icmpv6Echo::new(self.echo_identifier, self.echo_sequence);
        
        // Calculate checksum
        // let checksum = self.calculate_checksum(&header, &echo, &[], dst);
        // header.checksum = checksum.to_be();
        
        // Send via IPv6 layer
        // crate::net::ipv6::send(&[header, &echo], dst, ipv6_next_header::ICMPV6);
        
        self.echo_sequence = self.echo_sequence.wrapping_add(1);
        
        0
    }
    
    /// Send echo reply
    fn send_echo_reply(&mut self, dst: Ipv6Addr, identifier: u16, sequence: u16, data: &[u8]) {
        self.stats.out_echo_replies.fetch_add(1, Ordering::AcqRel);
        self.stats.out_msgs.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("ICMPv6 echo reply: dst={}, id={}, seq={}", 
                  dst.to_string(), identifier, sequence);
        
        // Build ICMPv6 echo reply
        let mut header = Icmpv6Header::new(icmpv6_type::ECHO_REPLY, 0);
        let echo = Icmpv6Echo::new(identifier, sequence);
        
        // Calculate checksum
        // let checksum = self.calculate_checksum(&header, &echo, data, dst);
        // header.checksum = checksum.to_be();
        
        // Send via IPv6 layer
        // crate::net::ipv6::send(&[header, &echo, data], dst, ipv6_next_header::ICMPV6);
    }
    
    /// Send destination unreachable
    pub fn send_dest_unreach(&mut self, code: u8, dst: Ipv6Addr) -> i32 {
        self.stats.out_dest_unreachs.fetch_add(1, Ordering::AcqRel);
        self.stats.out_msgs.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("ICMPv6 dest unreach: dst={}, code={}", dst.to_string(), code);
        
        // Build ICMPv6 destination unreachable message
        let mut header = Icmpv6Header::new(icmpv6_type::DST_UNREACH, code);
        
        // Calculate checksum
        // let checksum = self.calculate_checksum(&header, &[], &[], dst);
        // header.checksum = checksum.to_be();
        
        // Send via IPv6 layer
        // crate::net::ipv6::send(&header, dst, ipv6_next_header::ICMPV6);
        
        0
    }
}

/// Global ICMPv6 manager
static ICMPV6_MANAGER: core::sync::OnceLock<Icmpv6Manager> = core::sync::OnceLock::new();

/// Get ICMPv6 manager
pub fn icmpv6_manager() -> &'static Icmpv6Manager {
    ICMPV6_MANAGER.get_or_init(Icmpv6Manager::new)
}

pub fn init_icmpv6_manager() -> &'static Icmpv6Manager {
    ICMPV6_MANAGER.get_or_init(Icmpv6Manager::new)
}

/// Initialize ICMPv6
pub fn init_icmpv6() {
    let mgr = icmpv6_manager();
    mgr.init();
}
