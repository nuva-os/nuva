/*
 * Nuva OS - Kernel - Net - Icmp
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
 * Nuva OS - Kernel - ICMP Protocol
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Internet Control Message Protocol (ICMP) implementation.
 */

use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::{pr_debug, pr_info};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// ICMP Header
#[repr(C, packed)]
pub struct IcmpHeader {
    /// Type
    pub icmp_type: u8,
    /// Code
    pub icmp_code: u8,
    /// Checksum
    pub checksum: u16,
    /// Rest of header (depends on type)
    pub un: IcmpUn,
}

/// ICMP Union
#[repr(C)]
pub union IcmpUn {
    /// Echo
    pub echo: IcmpEcho,
    /// Gateway
    pub gateway: u32,
    /// Frag
    pub frag: IcmpFrag,
    /// Reserved
    pub reserved: u32,
}

/// ICMP Echo
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct IcmpEcho {
    pub id: u16,
    pub sequence: u16,
}

/// ICMP Fragment
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct IcmpFrag {
    pub unused: u16,
    pub mtu: u16,
}

impl IcmpHeader {
    /// Header size
    pub const SIZE: usize = 8;
    
    /// Create echo request
    pub fn echo_request(id: u16, seq: u16) -> Self {
        IcmpHeader {
            icmp_type: IcmpType::EchoRequest as u8,
            icmp_code: 0,
            checksum: 0,
            un: IcmpUn {
                echo: IcmpEcho {
                    id: id.to_be(),
                    sequence: seq.to_be(),
                },
            },
        }
    }
    
    /// Create echo reply
    pub fn echo_reply(id: u16, seq: u16) -> Self {
        IcmpHeader {
            icmp_type: IcmpType::EchoReply as u8,
            icmp_code: 0,
            checksum: 0,
            un: IcmpUn {
                echo: IcmpEcho {
                    id: id.to_be(),
                    sequence: seq.to_be(),
                },
            },
        }
    }
    
    /// Create destination unreachable
    pub fn dest_unreachable(code: u8) -> Self {
        IcmpHeader {
            icmp_type: IcmpType::DestUnreach as u8,
            icmp_code: code,
            checksum: 0,
            un: IcmpUn { reserved: 0 },
        }
    }
    
    /// Create time exceeded
    pub fn time_exceeded(code: u8) -> Self {
        IcmpHeader {
            icmp_type: IcmpType::TimeExceeded as u8,
            icmp_code: code,
            checksum: 0,
            un: IcmpUn { reserved: 0 },
        }
    }
}

/// ICMP Type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmpType {
    /// Echo reply
    EchoReply = 0,
    /// Destination unreachable
    DestUnreach = 3,
    /// Source quench (deprecated)
    SourceQuench = 4,
    /// Redirect
    Redirect = 5,
    /// Echo request
    EchoRequest = 8,
    /// Router advertisement
    RouterAdvert = 9,
    /// Router solicitation
    RouterSolicit = 10,
    /// Time exceeded
    TimeExceeded = 11,
    /// Parameter problem
    ParamProblem = 12,
    /// Timestamp request
    TimestampRequest = 13,
    /// Timestamp reply
    TimestampReply = 14,
    /// Information request (obsolete)
    InfoRequest = 15,
    /// Information reply (obsolete)
    InfoReply = 16,
    /// Address mask request (obsolete)
    AddrMaskRequest = 17,
    /// Address mask reply (obsolete)
    AddrMaskReply = 18,
}

/// ICMP Destination Unreachable Codes
pub mod icmp_dest_unreach {
    /// Network unreachable
    pub const NET_UNREACHABLE: u8 = 0;
    /// Host unreachable
    pub const HOST_UNREACHABLE: u8 = 1;
    /// Protocol unreachable
    pub const PROTOCOL_UNREACHABLE: u8 = 2;
    /// Port unreachable
    pub const PORT_UNREACHABLE: u8 = 3;
    /// Fragmentation needed
    pub const FRAG_NEEDED: u8 = 4;
    /// Source route failed
    pub const SOURCE_ROUTE_FAILED: u8 = 5;
    /// Network unknown
    pub const NET_UNKNOWN: u8 = 6;
    /// Host unknown
    pub const HOST_UNKNOWN: u8 = 7;
    /// Source host isolated
    pub const SOURCE_ISOLATED: u8 = 8;
    /// Network admin prohibited
    pub const NET_ADMIN_PROHIBITED: u8 = 9;
    /// Host admin prohibited
    pub const HOST_ADMIN_PROHIBITED: u8 = 10;
    /// Network unreachable for TOS
    pub const NET_UNREACH_TOS: u8 = 11;
    /// Host unreachable for TOS
    pub const HOST_UNREACH_TOS: u8 = 12;
    /// Communication admin prohibited
    pub const COMM_ADMIN_PROHIBITED: u8 = 13;
    /// Host precedence violation
    pub const HOST_PRECEDENCE_VIOLATION: u8 = 14;
    /// Precedence cutoff in effect
    pub const PRECEDENCE_CUTOFF: u8 = 15;
}

/// ICMP Redirect Codes
pub mod icmp_redirect {
    /// Redirect for network
    pub const NETWORK: u8 = 0;
    /// Redirect for host
    pub const HOST: u8 = 1;
    /// Redirect for TOS and network
    pub const NET_TOS: u8 = 2;
    /// Redirect for TOS and host
    pub const HOST_TOS: u8 = 3;
}

/// ICMP Time Exceeded Codes
pub mod icmp_time_exceed {
    /// TTL exceeded in transit
    pub const TTL_IN_TRANSIT: u8 = 0;
    /// Fragment reassembly time exceeded
    pub const FRAG_REASSEMBLY: u8 = 1;
}

/// ICMP Statistics
pub struct IcmpStats {
    /// Messages received
    pub in_msgs: AtomicU64,
    /// Errors received
    pub in_errors: AtomicU64,
    /// Destination unreachable received
    pub in_dest_unreachs: AtomicU64,
    /// Time exceeded received
    pub in_time_excds: AtomicU64,
    /// Parameter problem received
    pub in_parmprobs: AtomicU64,
    /// Source quench received
    pub in_srcquenchs: AtomicU64,
    /// Redirect received
    pub in_redirects: AtomicU64,
    /// Echo request received
    pub in_echos: AtomicU64,
    /// Echo reply received
    pub in_echoreps: AtomicU64,
    /// Timestamp request received
    pub in_timestamps: AtomicU64,
    /// Timestamp reply received
    pub in_timestampreps: AtomicU64,
    /// Address mask request received
    pub in_addrmasks: AtomicU64,
    /// Address mask reply received
    pub in_addrmaskreps: AtomicU64,
    /// Messages sent
    pub out_msgs: AtomicU64,
    /// Errors sent
    pub out_errors: AtomicU64,
    /// Destination unreachable sent
    pub out_dest_unreachs: AtomicU64,
    /// Time exceeded sent
    pub out_time_excds: AtomicU64,
    /// Parameter problem sent
    pub out_parmprobs: AtomicU64,
    /// Source quench sent
    pub out_srcquenchs: AtomicU64,
    /// Redirect sent
    pub out_redirects: AtomicU64,
    /// Echo request sent
    pub out_echos: AtomicU64,
    /// Echo reply sent
    pub out_echoreps: AtomicU64,
    /// Timestamp request sent
    pub out_timestamps: AtomicU64,
    /// Timestamp reply sent
    pub out_timestampreps: AtomicU64,
    /// Address mask request sent
    pub out_addrmasks: AtomicU64,
    /// Address mask reply sent
    pub out_addrmaskreps: AtomicU64,
}

impl IcmpStats {
    pub const fn new() -> Self {
        IcmpStats {
            in_msgs: AtomicU64::new(0),
            in_errors: AtomicU64::new(0),
            in_dest_unreachs: AtomicU64::new(0),
            in_time_excds: AtomicU64::new(0),
            in_parmprobs: AtomicU64::new(0),
            in_srcquenchs: AtomicU64::new(0),
            in_redirects: AtomicU64::new(0),
            in_echos: AtomicU64::new(0),
            in_echoreps: AtomicU64::new(0),
            in_timestamps: AtomicU64::new(0),
            in_timestampreps: AtomicU64::new(0),
            in_addrmasks: AtomicU64::new(0),
            in_addrmaskreps: AtomicU64::new(0),
            out_msgs: AtomicU64::new(0),
            out_errors: AtomicU64::new(0),
            out_dest_unreachs: AtomicU64::new(0),
            out_time_excds: AtomicU64::new(0),
            out_parmprobs: AtomicU64::new(0),
            out_srcquenchs: AtomicU64::new(0),
            out_redirects: AtomicU64::new(0),
            out_echos: AtomicU64::new(0),
            out_echoreps: AtomicU64::new(0),
            out_timestamps: AtomicU64::new(0),
            out_timestampreps: AtomicU64::new(0),
            out_addrmasks: AtomicU64::new(0),
            out_addrmaskreps: AtomicU64::new(0),
        }
    }
}

/// ICMP Manager
pub struct IcmpManager {
    /// Statistics
    pub stats: IcmpStats,
    /// Rate limit
    pub rate_limit: u32,
}

impl IcmpManager {
    pub const fn new() -> Self {
        IcmpManager {
            stats: IcmpStats::new(),
            rate_limit: 1000, // 1000 per second
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        log_info!("ICMP initialized");
    }
    
    /// Process received packet
    pub fn receive(&mut self, data: &[u8]) -> i32 {
        self.stats.in_msgs.fetch_add(1, Ordering::AcqRel);
        
        if data.len() < IcmpHeader::SIZE {
            self.stats.in_errors.fetch_add(1, Ordering::AcqRel);
            return Errno::Eperm.to_ret_i32();
        }
        
        // Process ICMP message
        let icmp_type = data[0];
        let icmp_code = data[1];
        let icmp_checksum = u16::from_be_bytes([data[2], data[3]]);
        
        log_debug!("ICMP receive: type={}, code={}, checksum={:#x}", 
                  icmp_type, icmp_code, icmp_checksum);
        
        match icmp_type {
            0 => { // Echo reply
                log_debug!("Received echo reply (pong)");
                self.stats.in_echos.fetch_add(1, Ordering::AcqRel);
            }
            3 => { // Destination unreachable
                log_debug!("Destination unreachable, code={}", icmp_code);
                self.stats.in_dest_unreachs.fetch_add(1, Ordering::AcqRel);
            }
            8 => { // Echo request (ping)
                log_debug!("Received echo request (ping)");
                self.stats.in_msgs.fetch_add(1, Ordering::AcqRel);
                
                // Send echo reply
                // let id = u16::from_be_bytes([data[4], data[5]]);
                // let seq = u16::from_be_bytes([data[6], data[7]]);
                // self.send_echo_reply(src_ip, id, seq, &data[8..]);
            }
            11 => { // Time exceeded
                log_debug!("Time exceeded, code={}", icmp_code);
            }
            _ => {
                log_debug!("Unknown ICMP type: {}", icmp_type);
            }
        }
        
        0
    }
    
    /// Send echo request (ping)
    pub fn ping(&mut self, dst: u32, id: u16, seq: u16) -> i32 {
        self.stats.out_echos.fetch_add(1, Ordering::AcqRel);
        self.stats.out_msgs.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("ICMP ping: dst={}, id={}, seq={}", self.format_ip(dst), id, seq);
        
        // Build ICMP echo request
        let mut packet = [0u8; 8];
        packet[0] = 8; // Type: Echo request
        packet[1] = 0; // Code
        packet[2..4].copy_from_slice(&0u16.to_be_bytes()); // Checksum (placeholder)
        packet[4..6].copy_from_slice(&id.to_be_bytes());
        packet[6..8].copy_from_slice(&seq.to_be_bytes());
        
        // Calculate checksum
        let checksum = self.calculate_checksum(&packet);
        packet[2..4].copy_from_slice(&checksum.to_be_bytes());
        
        // Send via IP layer
        // crate::kernel::net::ip::send(&packet, dst);
        
        0
    }
    
    /// Send destination unreachable
    pub fn send_dest_unreach(&mut self, code: u8, dst: u32) -> i32 {
        self.stats.out_dest_unreachs.fetch_add(1, Ordering::AcqRel);
        self.stats.out_msgs.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("ICMP dest unreach: dst={}, code={}", self.format_ip(dst), code);
        
        // Build ICMP destination unreachable message
        let mut packet = [0u8; 8];
        packet[0] = 3; // Type: Destination unreachable
        packet[1] = code; // Code
        packet[2..4].copy_from_slice(&0u16.to_be_bytes()); // Checksum (placeholder)
        packet[4..8].copy_from_slice(&0u32.to_be_bytes()); // Unused
        
        // Calculate checksum
        let checksum = self.calculate_checksum(&packet);
        packet[2..4].copy_from_slice(&checksum.to_be_bytes());
        
        // Send via IP layer
        // crate::kernel::net::ip::send(&packet, dst);
        
        0
    }
    
    /// Calculate ICMP checksum
    fn calculate_checksum(&self, data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        
        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            } else {
                sum += (chunk[0] as u32) << 8;
            }
        }
        
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        !sum as u16
    }
    
    /// Format IP address
    fn format_ip(&self, ip: u32) -> String {
        format!("{}.{}.{}.{}", 
                (ip >> 24) & 0xFF,
                (ip >> 16) & 0xFF,
                (ip >> 8) & 0xFF,
                ip & 0xFF)
    }
}

/// Global ICMP manager
static ICMP_MANAGER: crate::sync_oncelock::OnceLock<IcmpManager> = crate::sync_oncelock::OnceLock::new();

/// Get ICMP manager
pub fn icmp_manager() -> &'static IcmpManager {
    ICMP_MANAGER.get_or_init(IcmpManager::new)
}

pub fn init_icmp_manager() -> &'static IcmpManager {
    ICMP_MANAGER.get_or_init(IcmpManager::new)
}

/// Initialize ICMP
pub fn init_icmp() {
    let mgr = icmp_manager();
    mgr.init();
}
