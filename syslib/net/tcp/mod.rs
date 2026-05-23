/*
 * Nuva OS - TCP Protocol Implementation
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

//! TCP Protocol Implementation (RFC 793)
/*!*/
//! Complete TCP implementation with:
//! - Full state machine (11 states)
//! - Three-way handshake
//! - Data transfer with flow control
//! - Connection termination
//! - Congestion control (Reno, CUBIC, BBR)

pub mod state;
pub mod connect;
pub mod transfer;
pub mod close;
pub mod congestion;

pub use state::*;
pub use connect::*;
pub use transfer::*;
pub use close::*;
pub use congestion::*;

/// TCP Header
#[repr(C, packed)]
pub struct TcpHeader {
    /// Source port
    pub src_port: u16,

    /// Destination port
    pub dst_port: u16,

    /// Sequence number
    pub seq: u32,

    /// Acknowledgment number
    pub ack: u32,

    /// Data offset (4 bits) + Reserved (4 bits) + Flags (8 bits)
    pub offset_flags: u16,

    /// Window size
    pub window: u16,

    /// Checksum
    pub checksum: u16,

    /// Urgent pointer
    pub urgent: u16,
}

/// TCP Flags
pub mod tcp_flags {
    pub const FIN: u16 = 0x0001;
    pub const SYN: u16 = 0x0002;
    pub const RST: u16 = 0x0004;
    pub const PSH: u16 = 0x0008;
    pub const ACK: u16 = 0x0010;
    pub const URG: u16 = 0x0020;
    pub const ECE: u16 = 0x0040;
    pub const CWR: u16 = 0x0080;
}

impl TcpHeader {
    /// Create new TCP header
    pub fn new(src_port: u16, dst_port: u16, seq: u32, ack: u32) -> Self {
        Self {
            src_port,
            dst_port,
            seq,
            ack,
            offset_flags: (5 << 12) | tcp_flags::ACK, // 20 bytes header
            window: 65535,
            checksum: 0,
            urgent: 0,
        }
    }

    /// Get data offset (header length in bytes)
    pub fn data_offset(&self) -> u16 {
        (self.offset_flags >> 12) * 4
    }

    /// Set data offset
    pub fn set_data_offset(&mut self, offset: u16) {
        self.offset_flags = (self.offset_flags & 0x0FFF) | ((offset / 4) << 12);
    }

    /// Check if SYN flag is set
    pub fn is_syn(&self) -> bool {
        (self.offset_flags & tcp_flags::SYN) != 0
    }

    /// Set SYN flag
    pub fn set_syn(&mut self) {
        self.offset_flags |= tcp_flags::SYN;
    }

    /// Check if ACK flag is set
    pub fn is_ack(&self) -> bool {
        (self.offset_flags & tcp_flags::ACK) != 0
    }

    /// Set ACK flag
    pub fn set_ack(&mut self) {
        self.offset_flags |= tcp_flags::ACK;
    }

    /// Check if FIN flag is set
    pub fn is_fin(&self) -> bool {
        (self.offset_flags & tcp_flags::FIN) != 0
    }

    /// Set FIN flag
    pub fn set_fin(&mut self) {
        self.offset_flags |= tcp_flags::FIN;
    }

    /// Check if RST flag is set
    pub fn is_rst(&self) -> bool {
        (self.offset_flags & tcp_flags::RST) != 0
    }

    /// Set RST flag
    pub fn set_rst(&mut self) {
        self.offset_flags |= tcp_flags::RST;
    }

    /// Calculate checksum
    pub fn calculate_checksum(&mut self, data: &[u8], src_ip: u32, dst_ip: u32) {
        // Pseudo-header checksum
        let mut sum: u32 = 0;

        // Source and destination IP
        sum += ((src_ip >> 16) & 0xFFFF) as u32;
        sum += (src_ip & 0xFFFF) as u32;
        sum += ((dst_ip >> 16) & 0xFFFF) as u32;
        sum += (dst_ip & 0xFFFF) as u32;

        // Protocol and length
        sum += 6; // TCP protocol number
        let total_len = core::mem::size_of::<TcpHeader>() + data.len();
        sum += total_len as u32;

        // Header checksum
        // SAFETY: unsafe block required for low-level memory or hardware access
        let header_bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const TcpHeader as *const u8,
                core::mem::size_of::<TcpHeader>(),
            )
        };

        for i in (0..header_bytes.len()).step_by(2) {
            let word = if i + 1 < header_bytes.len() {
                ((header_bytes[i] as u32) << 8) | (header_bytes[i + 1] as u32)
            } else {
                (header_bytes[i] as u32) << 8
            };
            sum += word;
        }

        // Data checksum
        for i in (0..data.len()).step_by(2) {
            let word = if i + 1 < data.len() {
                ((data[i] as u32) << 8) | (data[i + 1] as u32)
            } else {
                (data[i] as u32) << 8
            };
            sum += word;
        }

        // Fold 32-bit sum to 16 bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        self.checksum = (!(sum as u16)).to_be();
    }
}

/// TCP Statistics
pub struct TcpStats {
    /// Active connections opened
    pub active_opens: core::sync::atomic::AtomicU64,

    /// Passive connections opened
    pub passive_opens: core::sync::atomic::AtomicU64,

    /// Connection attempts failed
    pub attempt_fails: core::sync::atomic::AtomicU64,

    /// Connections reset
    pub estab_resets: core::sync::atomic::AtomicU64,

    /// Current established connections
    pub curr_estab: core::sync::atomic::AtomicU32,

    /// Segments received
    pub in_segs: core::sync::atomic::AtomicU64,

    /// Segments sent
    pub out_segs: core::sync::atomic::AtomicU64,

    /// Segments retransmitted
    pub retrans_segs: core::sync::atomic::AtomicU64,

    /// Bytes received
    pub in_bytes: core::sync::atomic::AtomicU64,

    /// Bytes sent
    pub out_bytes: core::sync::atomic::AtomicU64,
}

impl TcpStats {
    pub const fn new() -> Self {
        use core::sync::atomic::{AtomicU32, AtomicU64};

        Self {
            active_opens: AtomicU64::new(0),
            passive_opens: AtomicU64::new(0),
            attempt_fails: AtomicU64::new(0),
            estab_resets: AtomicU64::new(0),
            curr_estab: AtomicU32::new(0),
            in_segs: AtomicU64::new(0),
            out_segs: AtomicU64::new(0),
            retrans_segs: AtomicU64::new(0),
            in_bytes: AtomicU64::new(0),
            out_bytes: AtomicU64::new(0),
        }
    }
}

/// Global TCP statistics
static TCP_STATS: TcpStats = TcpStats::new();

/// Get TCP statistics
pub fn get_tcp_stats() -> &'static TcpStats {
    &TCP_STATS
}

/// Initialize TCP
pub fn init_tcp() {
    // Initialize TCP subsystem
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_header_new() {
        let header = TcpHeader::new(12345, 80, 1000, 2000);
        assert_eq!(header.src_port, 12345);
        assert_eq!(header.dst_port, 80);
        assert_eq!(header.seq, 1000);
        assert_eq!(header.ack, 2000);
    }

    #[test]
    fn test_tcp_header_flags() {
        let mut header = TcpHeader::new(12345, 80, 1000, 2000);

        header.set_syn();
        assert!(header.is_syn());

        header.set_fin();
        assert!(header.is_fin());

        header.set_rst();
        assert!(header.is_rst());

        assert!(header.is_ack()); // ACK was set in new()
    }

    #[test]
    fn test_tcp_header_data_offset() {
        let mut header = TcpHeader::new(12345, 80, 1000, 2000);

        assert_eq!(header.data_offset(), 20); // Default 5 * 4 = 20

        header.set_data_offset(32);
        assert_eq!(header.data_offset(), 32);
    }

    #[test]
    fn test_tcp_stats() {
        let stats = get_tcp_stats();

        assert_eq!(stats.active_opens.load(core::sync::atomic::Ordering::Relaxed), 0);
        assert_eq!(stats.passive_opens.load(core::sync::atomic::Ordering::Relaxed), 0);
    }
}
