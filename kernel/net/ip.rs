/*
 * Nuva OS - Kernel - IP Protocol
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Internet Protocol (IPv4) implementation.
 */

use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_debug, pr_info, pr_warn};

use crate::posix::errno::Errno;
/// IP Address
#[repr(C)]
pub union IpAddr {
    pub v4: u32,
    pub v6: [u8; 16],
}

impl IpAddr {
    /// Create IPv4 address from bytes
    pub fn from_v4(bytes: [u8; 4]) -> Self {
        IpAddr { v4: u32::from_be_bytes(bytes) }
    }
    
    /// Create IPv6 address from bytes
    pub fn from_v6(bytes: [u8; 16]) -> Self {
        IpAddr { v6: bytes }
    }
    
    /// IPv4 loopback
    pub const fn loopback_v4() -> Self {
        IpAddr { v4: 0x7F000001 } // 127.0.0.1
    }
    
    /// IPv4 any
    pub const fn any_v4() -> Self {
        IpAddr { v4: 0 }
    }
}

/// IP Header
#[repr(C, packed)]
pub struct IpHeader {
    /// Version (4 bits) + IHL (4 bits)
    pub ver_ihl: u8,
    /// Type of service
    pub tos: u8,
    /// Total length
    pub tot_len: u16,
    /// Identification
    pub id: u16,
    /// Flags (3 bits) + Fragment offset (13 bits)
    pub frag_off: u16,
    /// Time to live
    pub ttl: u8,
    /// Protocol
    pub protocol: u8,
    /// Header checksum
    pub check: u16,
    /// Source address
    pub saddr: u32,
    /// Destination address
    pub daddr: u32,
}

impl IpHeader {
    /// Minimum header size
    pub const MIN_SIZE: usize = 20;
    
    /// Create new IP header
    pub fn new(tos: u8, tot_len: u16, id: u16, ttl: u8, protocol: u8, saddr: u32, daddr: u32) -> Self {
        IpHeader {
            ver_ihl: 0x45, // Version 4, IHL 5 (20 bytes)
            tos,
            tot_len: tot_len.to_be(),
            id: id.to_be(),
            frag_off: 0,
            ttl,
            protocol,
            check: 0,
            saddr: saddr.to_be(),
            daddr: daddr.to_be(),
        }
    }
    
    /// Get version
    pub fn version(&self) -> u8 {
        (self.ver_ihl >> 4) & 0x0F
    }
    
    /// Get header length (in bytes)
    pub fn ihl(&self) -> u8 {
        (self.ver_ihl & 0x0F) * 4
    }
    
    /// Get total length (host byte order)
    pub fn tot_len(&self) -> u16 {
        u16::from_be(self.tot_len)
    }
    
    /// Get fragment offset
    pub fn frag_off(&self) -> u16 {
        u16::from_be(self.frag_off) & 0x1FFF
    }
    
    /// Get flags
    pub fn flags(&self) -> u8 {
        ((u16::from_be(self.frag_off) >> 13) & 0x7) as u8
    }
    
    /// Calculate checksum
    pub fn calc_checksum(&mut self) {
        self.check = 0;
        let sum = self.checksum_partial();
        self.check = Self::fold_checksum(sum);
    }
    
    /// Partial checksum
    fn checksum_partial(&self) -> u32 {
        let ptr = self as *const IpHeader as *const u16;
        let mut sum: u32 = 0;
        
        for i in 0..10 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                sum += u16::from_be(*ptr.add(i)) as u32;
            }
        }
        
        sum
    }
    
    /// Fold checksum
    fn fold_checksum(mut sum: u32) -> u16 {
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        (!(sum as u16)).to_be()
    }
    
    /// Verify checksum
    pub fn verify_checksum(&self) -> bool {
        let sum = self.checksum_partial();
        let folded = Self::fold_checksum(sum);
        folded == 0 || folded == 0xFFFF
    }
}

/// IP Flags
pub mod ip_flags {
    /// Reserved
    pub const FLAG_RESERVED: u16 = 0x8000;
    /// Don't fragment
    pub const FLAG_DF: u16 = 0x4000;
    /// More fragments
    pub const FLAG_MF: u16 = 0x2000;
    /// Fragment offset mask
    pub const FRAG_OFF_MASK: u16 = 0x1FFF;
}

/// IP Protocol Numbers
pub mod ip_proto {
    /// ICMP
    pub const ICMP: u8 = 1;
    /// IGMP
    pub const IGMP: u8 = 2;
    /// TCP
    pub const TCP: u8 = 6;
    /// EGP
    pub const EGP: u8 = 8;
    /// UDP
    pub const UDP: u8 = 17;
    /// DCCP
    pub const DCCP: u8 = 33;
    /// IPv6 encapsulation
    pub const IPV6: u8 = 41;
    /// RSVP
    pub const RSVP: u8 = 46;
    /// GRE
    pub const GRE: u8 = 47;
    /// ESP
    pub const ESP: u8 = 50;
    /// AH
    pub const AH: u8 = 51;
    /// SCTP
    pub const SCTP: u8 = 132;
    /// UDPLite
    pub const UDPLITE: u8 = 136;
    /// Raw
    pub const RAW: u8 = 255;
}

/// IP Options
#[repr(C)]
pub struct IpOptions {
    /// Options data
    pub data: [u8; 40],
    /// Options length
    pub len: u8,
}

/// IP Fragment
#[repr(C)]
pub struct IpFragment {
    /// IP ID
    pub id: u16,
    /// Source address
    pub saddr: u32,
    /// Destination address
    pub daddr: u32,
    /// Protocol
    pub protocol: u8,
    /// Last received fragment
    pub last_in: AtomicU32,
    /// Total length
    pub len: u16,
    /// Meat (received bytes)
    pub meat: u16,
    /// First fragment time
    pub stamp: u64,
    /// Next fragment
    pub next: *mut IpFragment,
}

/// IP Statistics
pub struct IpStats {
    /// Packets received
    pub in_recvs: AtomicU64,
    /// Header errors
    pub in_hdr_errors: AtomicU64,
    /// Address errors
    pub in_addr_errors: AtomicU64,
    /// Unknown protocol
    pub in_unknown_protos: AtomicU64,
    /// Discarded input
    pub in_discards: AtomicU64,
    /// Delivered
    pub in_delivers: AtomicU64,
    /// Requests sent
    pub out_requests: AtomicU64,
    /// Discarded output
    pub out_discards: AtomicU64,
    /// Output no route
    pub out_no_routes: AtomicU64,
    /// Reassembled
    pub reasm_reqds: AtomicU64,
    /// Reassembly OK
    pub reasm_oks: AtomicU64,
    /// Reassembly failures
    pub reasm_fails: AtomicU64,
    /// Fragmented OK
    pub frag_oks: AtomicU64,
    /// Fragment failures
    pub frag_fails: AtomicU64,
    /// Fragments created
    pub frag_creates: AtomicU64,
}

impl IpStats {
    pub const fn new() -> Self {
        IpStats {
            in_recvs: AtomicU64::new(0),
            in_hdr_errors: AtomicU64::new(0),
            in_addr_errors: AtomicU64::new(0),
            in_unknown_protos: AtomicU64::new(0),
            in_discards: AtomicU64::new(0),
            in_delivers: AtomicU64::new(0),
            out_requests: AtomicU64::new(0),
            out_discards: AtomicU64::new(0),
            out_no_routes: AtomicU64::new(0),
            reasm_reqds: AtomicU64::new(0),
            reasm_oks: AtomicU64::new(0),
            reasm_fails: AtomicU64::new(0),
            frag_oks: AtomicU64::new(0),
            frag_fails: AtomicU64::new(0),
            frag_creates: AtomicU64::new(0),
        }
    }
}

/// IP Manager
pub struct IpManager {
    /// Default TTL
    pub default_ttl: u8,
    /// Statistics
    pub stats: IpStats,
    pub id: u32,
    pub local_ip: u32,
}

impl IpManager {
    pub const fn new() -> Self {
        IpManager {
            default_ttl: 64,
            stats: IpStats::new(),
                id: 0,
                local_ip: 0,
            }
    }
    
    /// Initialize
    pub fn init(&self) {
        log_info!("IP protocol initialized");
    }
    
    /// Process received packet
    pub fn receive(&mut self, data: &[u8]) -> i32 {
        self.stats.in_recvs.fetch_add(1, Ordering::AcqRel);
        
        // Parse IP header
        if data.len() < 20 {
            log_warn!("IP packet too short");
            return Errno::Eperm.to_ret_i32();
        }
        
        let version_ihl = data[0];
        let version = version_ihl >> 4;
        let ihl = version_ihl & 0x0F;
        
        if version != 4 {
            log_warn!("Invalid IP version: {}", version);
            return Errno::Eperm.to_ret_i32();
        }
        
        let header_len = (ihl as usize) * 4;
        if data.len() < header_len {
            log_warn!("IP header too short");
            return Errno::Eperm.to_ret_i32();
        }
        
        let total_len = u16::from_be_bytes([data[2], data[3]]) as usize;
        let protocol = data[9];
        let src_ip = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
        let dst_ip = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        
        log_debug!("IP receive: src={}, dst={}, proto={}, len={}", 
                  self.format_ip(src_ip), self.format_ip(dst_ip), protocol, total_len);
        
        // Verify checksum
        // let checksum = u16::from_be_bytes([data[10], data[11]]);
        // if !self.verify_checksum(&data[..header_len], checksum) {
        //     log_warn!("IP checksum error");
        //     return Errno::Eperm.to_ret_i32();
        // }
        
        // Route to upper layer based on protocol
        match protocol {
            1 => { // ICMP
                log_debug!("Routing to ICMP");
                // crate::net::icmp::receive(&data[header_len..]);
            }
            6 => { // TCP
                log_debug!("Routing to TCP");
                // crate::net::tcp::receive(&data[header_len..]);
            }
            17 => { // UDP
                log_debug!("Routing to UDP");
                // crate::net::udp::receive(&data[header_len..]);
            }
            _ => {
                log_debug!("Unknown protocol: {}", protocol);
            }
        }
        
        0
    }
    
    /// Send packet
    pub fn send(&mut self, data: &[u8], dst: u32) -> i32 {
        self.stats.out_requests.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("IP send: dst={}, len={}", self.format_ip(dst), data.len());
        
        // Build IP header
        let mut header = [0u8; 20];
        header[0] = 0x45; // Version 4, IHL 5
        header[1] = 0; // TOS
        header[2..4].copy_from_slice(((20 + data.len()) as u16).to_be_bytes().as_ref());
        header[4..6].copy_from_slice(&self.id.to_be_bytes());
        header[6] = 0x40; // Don't fragment
        header[7] = 0;
        header[8] = 64; // TTL
        header[9] = 6; // Protocol (TCP)
        header[12..16].copy_from_slice(&self.local_ip.to_be_bytes());
        header[16..20].copy_from_slice(&dst.to_be_bytes());
        
        // Calculate checksum
        let checksum = self.calculate_checksum(&header);
        header[10..12].copy_from_slice(&checksum.to_be_bytes());
        
        // Send to lower layer (e.g., Ethernet)
        // crate::net::ethernet::send(&header, data, dst);
        
        self.id = self.id.wrapping_add(1);
        
        0
    }
    
    /// Calculate IP checksum
    fn calculate_checksum(&self, header: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        
        for chunk in header.chunks(2) {
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

/// Global IP manager
static IP_MANAGER: core::sync::OnceLock<IpManager> = core::sync::OnceLock::new();

/// Get IP manager
pub fn ip_manager() -> &'static IpManager {
    IP_MANAGER.get_or_init(IpManager::new)
}

pub fn init_ip_manager() -> &'static IpManager {
    IP_MANAGER.get_or_init(IpManager::new)
}

/// Initialize IP
pub fn init_ip() {
    let mgr = ip_manager();
    mgr.init();
}

/// IP address utilities
pub struct IpUtils;

impl IpUtils {
    /// Check if address is loopback
    pub fn is_loopback(addr: u32) -> bool {
        (addr & 0xFF000000) == 0x7F000000
    }
    
    /// Check if address is multicast
    pub fn is_multicast(addr: u32) -> bool {
        (addr & 0xF0000000) == 0xE0000000
    }
    
    /// Check if address is broadcast
    pub fn is_broadcast(addr: u32) -> bool {
        addr == 0xFFFFFFFF
    }
    
    /// Check if address is local (link-local)
    pub fn is_linklocal(addr: u32) -> bool {
        (addr & 0xFFFF0000) == 0xA9FE0000 // 169.254.0.0/16
    }
    
    /// Check if address is private
    pub fn is_private(addr: u32) -> bool {
        let a = (addr >> 24) & 0xFF;
        let b = (addr >> 16) & 0xFF;
        
        // 10.0.0.0/8
        if a == 10 {
            return true;
        }
        // 172.16.0.0/12
        if a == 172 && (16..=31).contains(&b) {
            return true;
        }
        // 192.168.0.0/16
        if a == 192 && b == 168 {
            return true;
        }
        false
    }
    
    /// Convert IP to string
    pub fn to_string(addr: u32, buf: &mut [u8]) -> usize {
        let a = ((addr >> 24) & 0xFF) as u8;
        let b = ((addr >> 16) & 0xFF) as u8;
        let c = ((addr >> 8) & 0xFF) as u8;
        let d = (addr & 0xFF) as u8;
        
        let mut pos = 0;
        pos += Self::write_dec(a, &mut buf[pos..]);
        buf[pos] = b'.';
        pos += 1;
        pos += Self::write_dec(b, &mut buf[pos..]);
        buf[pos] = b'.';
        pos += 1;
        pos += Self::write_dec(c, &mut buf[pos..]);
        buf[pos] = b'.';
        pos += 1;
        pos += Self::write_dec(d, &mut buf[pos..]);
        pos
    }
    
    fn write_dec(mut n: u8, buf: &mut [u8]) -> usize {
        if n == 0 {
            buf[0] = b'0';
            return 1;
        }
        
        let mut digits = [0u8; 3];
        let mut i = 0;
        while n > 0 {
            digits[i] = b'0' + (n % 10);
            n /= 10;
            i += 1;
        }
        
        for j in 0..i {
            buf[j] = digits[i - 1 - j];
        }
        i
    }
}
