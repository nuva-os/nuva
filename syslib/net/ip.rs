/*
 * Nuva OS - SystemLibrary - Network IP
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

//! IP Protocol Implementation
//!
//! Implements the IPv4 protocol, network layer packet handling.
//!
//! # Features
//!
//! - IPv4 header parsing and construction
//! - IP checksum computation
//! - IP fragmentation and reassembly (in progress)
//! - IP routing (in progress)
//!
//! # IPv4 Header Format
//!
//! ```text
//! 0 1 2 3
//! 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |Version| IHL | TOS | Total Length |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! | Identification |Flags| Fragment Offset |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! | TTL | Protocol| Header Checksum |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! | Source Address |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! | Destination Address |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! ```

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// IPv4 Header
#[repr(C, packed)]
pub struct Ipv4Header {
    /// Version (4 bits) + Header length (4 bits)
    pub ver_ihl: u8,
    /// Type of Service
    pub tos: u8,
    /// Total Length
    pub total_len: u16,
    /// Identification
    pub id: u16,
    /// Flags (3 bits) + Fragment Offset (13 bits)
    pub flags_frag: u16,
    /// Time To Live
    pub ttl: u8,
    /// Protocol
    pub protocol: u8,
    /// Header Checksum
    pub checksum: u16,
    /// Source IP
    pub src_addr: u32,
    /// Destination IP
    pub dst_addr: u32,
}

/// IP Protocol numbers
pub mod ip_protocol {
    pub const ICMP: u8 = 1;
    pub const TCP: u8 = 6;
    pub const UDP: u8 = 17;
}

/// IP Fragment Flags
pub mod ip_flags {
    pub const MF: u16 = 0x2000; // More Fragments
    pub const DF: u16 = 0x4000; // Don't Fragment
    pub const FRAG_MASK: u16 = 0x1FFF;
}

/// IP Statistics
pub struct IpStats {
    /// Received packet count
    pub rx_packets: AtomicU64,
    /// Sent packet count
    pub tx_packets: AtomicU64,
    /// Received byte count
    pub rx_bytes: AtomicU64,
    /// Sent byte count
    pub tx_bytes: AtomicU64,
    /// Fragment count
    pub fragments: AtomicU32,
}

impl IpStats {
    pub const fn new() -> Self {
        IpStats {
            rx_packets: AtomicU64::new(0),
            tx_packets: AtomicU64::new(0),
            rx_bytes: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            fragments: AtomicU32::new(0),
        }
    }
}

/// Global IP Statistics
static mut IP_STATS: IpStats = IpStats::new();

pub fn get_ip_stats() -> &'static IpStats {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &IP_STATS }
}

/// Initialize IP
pub fn init_ip() {
    log_info!("IP protocol initialized");
}

/// IP Address utility functions
impl Ipv4Header {
    /// Create a new IPv4 Header
    pub fn new(src: u32, dst: u32, protocol: u8, payload_len: u16) -> Self {
        let mut header = Ipv4Header {
            ver_ihl: 0x45, // IPv4, 20 bytes header
            tos: 0,
            total_len: (20 + payload_len).to_be(),
            id: 0,
            flags_frag: 0x4000, // Don't Fragment
            ttl: 64,
            protocol,
            checksum: 0,
            src_addr: src.to_be(),
            dst_addr: dst.to_be(),
        };

        header.checksum = header.calculate_checksum();
        header
    }

    /// Compute the header checksum
    pub fn calculate_checksum(&self) -> u16 {
        let mut sum: u32 = 0;
        let ptr = self as *const Ipv4Header as *const u16;

        // Sum the header in 16-bit words
        for i in 0..10 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                sum += u16::from_be(*ptr.add(i)) as u32;
            }
        }

        // Fold carry bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        // Take one's complement
        (!(sum as u16)).to_be()
    }

    /// Verify the header checksum
    pub fn verify_checksum(&self) -> bool {
        self.calculate_checksum() == 0
    }

    /// Get the IP version
    pub fn version(&self) -> u8 {
        self.ver_ihl >> 4
    }

    /// Get the header length in bytes
    pub fn header_len(&self) -> u8 {
        (self.ver_ihl & 0x0F) * 4
    }

    /// Get the total length
    pub fn total_length(&self) -> u16 {
        u16::from_be(self.total_len)
    }

    /// Get the payload length
    pub fn payload_len(&self) -> u16 {
        self.total_length() - self.header_len() as u16
    }

    /// Check if the packet is fragmented
    pub fn is_fragmented(&self) -> bool {
        let flags = u16::from_be(self.flags_frag);
        (flags & ip_flags::MF) != 0 || (flags & ip_flags::FRAG_MASK) != 0
    }

    /// Get the fragment offset
    pub fn fragment_offset(&self) -> u16 {
        u16::from_be(self.flags_frag) & ip_flags::FRAG_MASK
    }
}

/// Convert an IP address to a string
pub fn ip_to_string(ip: u32) -> [u8; 16] {
    let mut buf = [0u8; 16];
    let bytes = ip.to_be_bytes();
    let mut pos = 0;

    for i in 0..4 {
        let mut num = bytes[i];
        let mut digits = [0u8; 3];
        let mut len = 0;

        if num == 0 {
            digits[0] = b'0';
            len = 1;
        } else {
            while num > 0 {
                digits[len] = b'0' + (num % 10);
                num /= 10;
                len += 1;
            }
        }

        // Reverse the digit order
        for j in 0..len / 2 {
            let tmp = digits[j];
            digits[j] = digits[len - 1 - j];
            digits[len - 1 - j] = tmp;
        }

        for j in 0..len {
            buf[pos] = digits[j];
            pos += 1;
        }

        if i < 3 {
            buf[pos] = b'.';
            pos += 1;
        }
    }

    buf
}

/// Parse a string into an IP address
pub fn string_to_ip(s: &[u8]) -> Option<u32> {
    let mut result: u32 = 0;
    let mut octet: u32 = 0;
    let mut dot_count = 0;

    for &c in s {
        if c == b'.' {
            if octet > 255 {
                return None;
            }
            result = (result << 8) | octet;
            octet = 0;
            dot_count += 1;
        } else if c >= b'0' && c <= b'9' {
            octet = octet * 10 + (c - b'0') as u32;
            if octet > 255 {
                return None;
            }
        } else if c == 0 {
            break;
        } else {
            return None;
        }
    }

    if dot_count != 3 || octet > 255 {
        return None;
    }

    Some(((result << 8) | octet).to_be())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_header_new() {
        let header = Ipv4Header::new(0x0A000001, 0x0A000002, ip_protocol::TCP, 100);

        assert_eq!(header.version(), 4);
        assert_eq!(header.header_len(), 20);
        assert_eq!(header.protocol, ip_protocol::TCP);
        assert_eq!(header.ttl, 64);
        assert_eq!(header.total_length(), 120);
        assert_eq!(header.payload_len(), 100);
    }

    #[test]
    fn test_ipv4_header_checksum() {
        let header = Ipv4Header::new(0x0A000001, 0x0A000002, ip_protocol::TCP, 100);
        assert!(header.verify_checksum());
    }

    #[test]
    fn test_ipv4_header_fragment() {
        let mut header = Ipv4Header::new(0x0A000001, 0x0A000002, ip_protocol::TCP, 100);

        // Default: not fragmented
        assert!(!header.is_fragmented());

        // Set fragment flags
        header.flags_frag = (ip_flags::MF | 100).to_be();
        assert!(header.is_fragmented());
        assert_eq!(header.fragment_offset(), 100);
    }

    #[test]
    fn test_ip_protocol_constants() {
        assert_eq!(ip_protocol::ICMP, 1);
        assert_eq!(ip_protocol::TCP, 6);
        assert_eq!(ip_protocol::UDP, 17);
    }

    #[test]
    fn test_ip_flags() {
        assert_eq!(ip_flags::MF, 0x2000);
        assert_eq!(ip_flags::DF, 0x4000);
        assert_eq!(ip_flags::FRAG_MASK, 0x1FFF);
    }

    #[test]
    fn test_ip_to_string() {
        let ip = 0x0A000001; // 10.0.0.1
        let s = ip_to_string(ip);

        // Validate format
        assert!(s[0] == b'1' || s[0] == b'0');
    }

    #[test]
    fn test_string_to_ip() {
        // Test valid IP
        let ip = string_to_ip(b"10.0.0.1");
        assert!(ip.is_some());

        // Test invalid IP
        let ip = string_to_ip(b"256.0.0.1");
        assert!(ip.is_none());

        let ip = string_to_ip(b"10.0.0");
        assert!(ip.is_none());
    }

    #[test]
    fn test_ip_stats() {
        let stats = IpStats::new();

        assert_eq!(stats.rx_packets.load(Ordering::Relaxed), 0);
        assert_eq!(stats.tx_packets.load(Ordering::Relaxed), 0);

        stats.rx_packets.fetch_add(1, Ordering::Relaxed);
        assert_eq!(stats.rx_packets.load(Ordering::Relaxed), 1);
    }
}
