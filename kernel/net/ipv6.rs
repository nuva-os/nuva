/*
 * Nuva OS - Kernel - IPv6 Protocol
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Internet Protocol version 6 (IPv6) implementation.
 */

use crate::{pr_debug, pr_info, pr_warn};
use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// IPv6 Address (128 bits)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv6Addr {
    /// Address bytes
    pub bytes: [u8; 16],
}

impl Ipv6Addr {
    /// Create new IPv6 address
    pub const fn new(bytes: [u8; 16]) -> Self {
        Ipv6Addr { bytes }
    }

    /// Unspecified address (::)
    pub const UNSPECIFIED: Ipv6Addr = Ipv6Addr { bytes: [0; 16] };

    /// Loopback address (::1)
    pub const LOOPBACK: Ipv6Addr = Ipv6Addr {
        bytes: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    };

    /// All nodes multicast address (ff02::1)
    pub const ALL_NODES: Ipv6Addr = Ipv6Addr {
        bytes: [0xff, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    };

    /// All routers multicast address (ff02::2)
    pub const ALL_ROUTERS: Ipv6Addr = Ipv6Addr {
        bytes: [0xff, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
    };

    /// Check if address is unspecified
    pub fn is_unspecified(&self) -> bool {
        self.bytes == [0; 16]
    }

    /// Check if address is loopback
    pub fn is_loopback(&self) -> bool {
        self.bytes == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
    }

    /// Check if address is multicast
    pub fn is_multicast(&self) -> bool {
        self.bytes[0] == 0xff
    }

    /// Check if address is link-local
    pub fn is_link_local(&self) -> bool {
        self.bytes[0] == 0xfe && (self.bytes[1] & 0xc0) == 0x80
    }

    /// Format IPv6 address to string
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        let mut zero_start = None;
        let mut zero_len = 0;
        let mut current_zero_len = 0;

        // Find longest run of zeros
        for i in 0..8 {
            let word = ((self.bytes[i * 2] as u16) << 8) | (self.bytes[i * 2 + 1] as u16);
            if word == 0 {
                current_zero_len += 1;
            } else {
                if current_zero_len > zero_len && current_zero_len > 1 {
                    zero_len = current_zero_len;
                    zero_start = Some(i - current_zero_len);
                }
                current_zero_len = 0;
            }
        }

        // Check trailing zeros
        if current_zero_len > zero_len && current_zero_len > 1 {
            zero_len = current_zero_len;
            zero_start = Some(8 - current_zero_len);
        }

        // Build string
        let mut i = 0;
        while i < 8 {
            if let Some(start) = zero_start {
                if i == start {
                    result.push_str("::");
                    i += zero_len;
                    continue;
                } else if i > start && i < start + zero_len {
                    i += 1;
                    continue;
                }
            }

            let word = ((self.bytes[i * 2] as u16) << 8) | (self.bytes[i * 2 + 1] as u16);
            if i > 0 && !(zero_start.is_some() && i == zero_start.map_or(0, |s| s) + zero_len) {
                result.push(':');
            }
            result.push_str(&format!("{:x}", word));
            i += 1;
        }

        result
    }
}

/// IPv6 Header
#[repr(C, packed)]
pub struct Ipv6Header {
    /// Version (4 bits) + Traffic class (8 bits) + Flow label (20 bits)
    pub version_tc_fl: u32,
    /// Payload length
    pub payload_len: u16,
    /// Next header
    pub next_header: u8,
    /// Hop limit
    pub hop_limit: u8,
    /// Source address
    pub src_addr: Ipv6Addr,
    /// Destination address
    pub dst_addr: Ipv6Addr,
}

impl Ipv6Header {
    /// Header size
    pub const SIZE: usize = 40;

    /// Create new IPv6 header
    pub fn new(src: Ipv6Addr, dst: Ipv6Addr, payload_len: u16, next_header: u8) -> Self {
        let version_tc_fl: u32 = (6u32 << 28) | (0u32 << 20) | 0u32; // Version 6, TC 0, Flow 0

        Ipv6Header {
            version_tc_fl: version_tc_fl.to_be(),
            payload_len: payload_len.to_be(),
            next_header,
            hop_limit: 64,
            src_addr: src,
            dst_addr: dst,
        }
    }

    /// Get version
    pub fn get_version(&self) -> u8 {
        (u32::from_be(self.version_tc_fl) >> 28) as u8
    }

    /// Get traffic class
    pub fn get_traffic_class(&self) -> u8 {
        ((u32::from_be(self.version_tc_fl) >> 20) & 0xFF) as u8
    }

    /// Get flow label
    pub fn get_flow_label(&self) -> u32 {
        u32::from_be(self.version_tc_fl) & 0xFFFFF
    }

    /// Get payload length
    pub fn get_payload_len(&self) -> u16 {
        u16::from_be(self.payload_len)
    }
}

/// IPv6 Next Header Types
pub mod ipv6_next_header {
    pub const HOPOPT: u8 = 0; // Hop-by-Hop Options
    pub const TCP: u8 = 6; // TCP
    pub const UDP: u8 = 17; // UDP
    pub const IPV6_ROUTE: u8 = 43; // Routing Header
    pub const IPV6_FRAG: u8 = 44; // Fragment Header
    pub const ICMPV6: u8 = 58; // ICMPv6
    pub const NONE: u8 = 59; // No Next Header
    pub const DSTOPTS: u8 = 60; // Destination Options
}

/// IPv6 Statistics
pub struct Ipv6Stats {
    /// Packets received
    pub in_recvs: AtomicU64,
    /// Packets sent
    pub out_requests: AtomicU64,
    /// Forwarded packets
    pub forw_datagrams: AtomicU64,
    /// Input errors
    pub in_errors: AtomicU64,
    /// Output errors
    pub out_errors: AtomicU64,
    /// Discarded packets
    pub in_discards: AtomicU64,
    /// Out discards
    pub out_discards: AtomicU64,
}

impl Ipv6Stats {
    pub const fn new() -> Self {
        Ipv6Stats {
            in_recvs: AtomicU64::new(0),
            out_requests: AtomicU64::new(0),
            forw_datagrams: AtomicU64::new(0),
            in_errors: AtomicU64::new(0),
            out_errors: AtomicU64::new(0),
            in_discards: AtomicU64::new(0),
            out_discards: AtomicU64::new(0),
        }
    }
}

/// IPv6 Manager
pub struct Ipv6Manager {
    /// Statistics
    pub stats: Ipv6Stats,
    /// Local IPv6 address
    pub local_addr: Ipv6Addr,
    /// Default gateway
    pub gateway: Ipv6Addr,
}

impl Ipv6Manager {
    pub const fn new() -> Self {
        Ipv6Manager {
            stats: Ipv6Stats::new(),
            local_addr: Ipv6Addr::UNSPECIFIED,
            gateway: Ipv6Addr::UNSPECIFIED,
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("IPv6 initialized");
    }

    /// Set local IPv6 address
    pub fn set_local_addr(&mut self, addr: Ipv6Addr) {
        self.local_addr = addr;
        log_info!("IPv6 local address: {}", addr.to_string());
    }

    /// Set default gateway
    pub fn set_gateway(&mut self, gateway: Ipv6Addr) {
        self.gateway = gateway;
        log_info!("IPv6 gateway: {}", gateway.to_string());
    }

    /// Process received packet
    pub fn receive(&mut self, data: &[u8]) -> i32 {
        self.stats.in_recvs.fetch_add(1, Ordering::AcqRel);

        // Parse IPv6 header
        if data.len() < Ipv6Header::SIZE {
            log_warn!("IPv6 packet too short");
            self.stats.in_errors.fetch_add(1, Ordering::AcqRel);
            return Errno::Eperm.to_ret_i32();
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        let header = unsafe { &*(data.as_ptr() as *const Ipv6Header) };

        // Verify version
        if header.get_version() != 6 {
            log_warn!("Invalid IPv6 version: {}", header.get_version());
            self.stats.in_errors.fetch_add(1, Ordering::AcqRel);
            return Errno::Enoent.to_ret_i32();
        }

        let payload_len = header.get_payload_len() as usize;
        let total_len = Ipv6Header::SIZE + payload_len;

        if data.len() < total_len {
            log_warn!("IPv6 packet incomplete");
            self.stats.in_errors.fetch_add(1, Ordering::AcqRel);
            return Errno::Esrch.to_ret_i32();
        }

        let src = header.src_addr;
        let dst = header.dst_addr;
        let next_header = header.next_header;

        log_debug!(
            "IPv6 receive: src={}, dst={}, next_header={}, len={}",
            src.to_string(),
            dst.to_string(),
            next_header,
            payload_len
        );

        // Route to upper layer based on next header
        match next_header {
            ipv6_next_header::ICMPV6 => {
                log_debug!("Routing to ICMPv6");
                // crate::net::icmpv6::receive(&data[Ipv6Header::SIZE..]);
            }
            ipv6_next_header::TCP => {
                log_debug!("Routing to TCP");
                // crate::net::tcp::receive(&data[Ipv6Header::SIZE..]);
            }
            ipv6_next_header::UDP => {
                log_debug!("Routing to UDP");
                // crate::net::udp::receive(&data[Ipv6Header::SIZE..]);
            }
            _ => {
                log_debug!("Unknown next header: {}", next_header);
            }
        }

        0
    }

    /// Send packet
    pub fn send(&mut self, data: &[u8], dst: Ipv6Addr, next_header: u8) -> i32 {
        self.stats.out_requests.fetch_add(1, Ordering::AcqRel);

        log_debug!(
            "IPv6 send: dst={}, len={}, next_header={}",
            dst.to_string(),
            data.len(),
            next_header
        );

        // Build IPv6 header
        let header = Ipv6Header::new(self.local_addr, dst, data.len() as u16, next_header);

        // Send to lower layer (e.g., Ethernet)
        // crate::net::ethernet::send_ipv6(&header, data);

        0
    }
}

/// Global IPv6 manager
static IPV6_MANAGER: core::sync::OnceLock<Ipv6Manager> = core::sync::OnceLock::new();

/// Get IPv6 manager
pub fn ipv6_manager() -> &'static Ipv6Manager {
    IPV6_MANAGER.get_or_init(Ipv6Manager::new)
}

pub fn init_ipv6_manager() -> &'static Ipv6Manager {
    IPV6_MANAGER.get_or_init(Ipv6Manager::new)
}

/// Initialize IPv6
pub fn init_ipv6() {
    let mgr = ipv6_manager();
    mgr.init();
}
