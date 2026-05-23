/*
 * Nuva OS - Kernel - TCP Fast Path
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

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

/// TCP flags
pub mod tcp_flags {
    pub const FIN: u8 = 0x01;
    pub const SYN: u8 = 0x02;
    pub const RST: u8 = 0x04;
    pub const PSH: u8 = 0x08;
    pub const ACK: u8 = 0x10;
    pub const URG: u8 = 0x20;
    pub const ECE: u8 = 0x40;
    pub const CWR: u8 = 0x80;
}

/// TCP connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed = 0,
    Listen = 1,
    SynSent = 2,
    SynReceived = 3,
    Established = 4,
    FinWait1 = 5,
    FinWait2 = 6,
    CloseWait = 7,
    Closing = 8,
    LastAck = 9,
    TimeWait = 10,
}

/// TCP fast path conditions
pub struct TcpFastPathConditions {
    /// No out-of-order segments
    pub no_ooo: bool,
    
    /// No special flags (only ACK)
    pub no_special_flags: bool,
    
    /// Window matches expected
    pub window_match: bool,
    
    /// No pending data to send
    pub no_pending_send: bool,
    
    /// Receiver window not zero
    pub rwnd_not_zero: bool,
}

impl TcpFastPathConditions {
    /// Check if all conditions are met for fast path
    #[inline(always)]
    pub fn can_fast_path(&self) -> bool {
        self.no_ooo 
            && self.no_special_flags 
            && self.window_match 
            && self.no_pending_send 
            && self.rwnd_not_zero
    }
}

/// TCP connection (simplified for fast path)
pub struct TcpConnection {
    /// Connection state
    pub state: AtomicU32,
    
    /// Local address
    pub local_addr: u32,
    pub local_port: u16,
    
    /// Remote address
    pub remote_addr: u32,
    pub remote_port: u16,
    
    /// Sequence numbers
    pub snd_una: AtomicU32,    // Send unacknowledged
    pub snd_nxt: AtomicU32,    // Send next
    pub snd_wnd: AtomicU32,    // Send window
    
    pub rcv_nxt: AtomicU32,    // Receive next
    pub rcv_wnd: AtomicU32,    // Receive window
    pub rcv_wup: AtomicU32,    // Receive window update
    
    /// Congestion control
    pub cwnd: AtomicU32,       // Congestion window
    pub ssthresh: AtomicU32,   // Slow start threshold
    
    /// RTT estimation
    pub srtt: AtomicU32,       // Smoothed RTT
    pub rttvar: AtomicU32,     // RTT variance
    pub rto: AtomicU32,        // Retransmission timeout
    
    /// Fast path statistics
    pub fast_path_count: AtomicU64,
    pub slow_path_count: AtomicU64,
    
    /// Connection flags
    pub flags: AtomicU32,
}

/// TCP connection flags
pub mod tcp_conn_flags {
    /// Fast path eligible
    pub const FAST_PATH: u32 = 1 << 0;
    
    /// Delayed ACK enabled
    pub const DELAYED_ACK: u32 = 1 << 1;
    
    /// Nagle algorithm enabled
    pub const NAGLE: u32 = 1 << 2;
    
    /// SACK permitted
    pub const SACK_PERM: u32 = 1 << 3;
    
    /// Timestamps enabled
    pub const TIMESTAMPS: u32 = 1 << 4;
}

impl TcpConnection {
    pub const fn new() -> Self {
        TcpConnection {
            state: AtomicU32::new(TcpState::Closed as u32),
            local_addr: 0,
            local_port: 0,
            remote_addr: 0,
            remote_port: 0,
            snd_una: AtomicU32::new(0),
            snd_nxt: AtomicU32::new(0),
            snd_wnd: AtomicU32::new(0),
            rcv_nxt: AtomicU32::new(0),
            rcv_wnd: AtomicU32::new(65535),  // Default window
            rcv_wup: AtomicU32::new(0),
            cwnd: AtomicU32::new(10),  // Initial cwnd (10 segments)
            ssthresh: AtomicU32::new(65535),
            srtt: AtomicU32::new(0),
            rttvar: AtomicU32::new(0),
            rto: AtomicU32::new(1000),  // 1 second default RTO
            fast_path_count: AtomicU64::new(0),
            slow_path_count: AtomicU64::new(0),
            flags: AtomicU32::new(tcp_conn_flags::FAST_PATH),
        }
    }
    
    /// Check if fast path is eligible
    #[inline(always)]
    pub fn is_fast_path_eligible(&self) -> bool {
        let state = self.state.load(Ordering::Acquire);
        let flags = self.flags.load(Ordering::Acquire);
        
        // Must be in ESTABLISHED state
        if state != TcpState::Established as u32 {
            return false;
        }
        
        // Check fast path flag
        (flags & tcp_conn_flags::FAST_PATH) != 0
    }
    
    /// Check fast path conditions for receive
    #[inline(always)]
    pub fn check_fast_path_rx(&self, seq: u32, ack: u32, flags: u8, win: u16) -> TcpFastPathConditions {
        let rcv_nxt = self.rcv_nxt.load(Ordering::Acquire);
        let snd_una = self.snd_una.load(Ordering::Acquire);
        let snd_wnd = self.snd_wnd.load(Ordering::Acquire);
        
        TcpFastPathConditions {
            // No out-of-order: sequence matches expected
            no_ooo: seq == rcv_nxt,
            
            // No special flags: only ACK set
            no_special_flags: flags == tcp_flags::ACK,
            
            // Window matches expected
            window_match: (win as u32) == snd_wnd,
            
            // No pending data: ACK acknowledges all sent data
            no_pending_send: ack == snd_una,
            
            // Receiver window not zero
            rwnd_not_zero: (win as u32) > 0,
        }
    }
    
    /// Fast path receive - process data directly
    #[inline(always)]
    pub fn fast_path_receive(&mut self, seq: u32, data_len: u32) -> bool {
        // Update receive sequence
        self.rcv_nxt.fetch_add(data_len, Ordering::AcqRel);
        
        // Update statistics
        self.fast_path_count.fetch_add(1, Ordering::Relaxed);
        
        true
    }
    
    /// Fast path send ACK
    #[inline(always)]
    pub fn fast_path_send_ack(&mut self) -> (u32, u32, u16) {
        let rcv_nxt = self.rcv_nxt.load(Ordering::Acquire);
        let snd_una = self.snd_una.load(Ordering::Acquire);
        let rcv_wnd = self.rcv_wnd.load(Ordering::Acquire);
        
        // Return (seq, ack, win)
        (snd_una, rcv_nxt, rcv_wnd as u16)
    }
    
    /// Slow path receive - full processing
    pub fn slow_path_receive(&mut self, _seq: u32, _ack: u32, _flags: u8, _data_len: u32) -> bool {
        // TODO: Full TCP state machine processing
        // - Handle out-of-order segments
        // - Process special flags (FIN, SYN, RST, etc.)
        // - Update congestion control
        // - Handle window updates
        
        self.slow_path_count.fetch_add(1, Ordering::Relaxed);
        true
    }
}

/// TCP fast path processor
pub struct TcpFastPathProcessor {
    /// Number of connections
    pub nr_connections: AtomicU32,
    
    /// Fast path statistics
    pub total_fast_path: AtomicU64,
    pub total_slow_path: AtomicU64,
    pub total_packets: AtomicU64,
    
    /// Enabled flag
    pub enabled: AtomicBool,
}

impl TcpFastPathProcessor {
    pub const fn new() -> Self {
        TcpFastPathProcessor {
            nr_connections: AtomicU32::new(0),
            total_fast_path: AtomicU64::new(0),
            total_slow_path: AtomicU64::new(0),
            total_packets: AtomicU64::new(0),
            enabled: AtomicBool::new(true),
        }
    }
    
    /// Process incoming TCP segment
    #[inline(always)]
    pub fn process_segment(&mut self, conn: &mut TcpConnection, seq: u32, ack: u32, flags: u8, win: u16, data_len: u32) -> bool {
        self.total_packets.fetch_add(1, Ordering::Relaxed);
        
        if !self.enabled.load(Ordering::Acquire) {
            return conn.slow_path_receive(seq, ack, flags, data_len);
        }
        
        // Check if fast path is eligible
        if !conn.is_fast_path_eligible() {
            self.total_slow_path.fetch_add(1, Ordering::Relaxed);
            return conn.slow_path_receive(seq, ack, flags, data_len);
        }
        
        // Check fast path conditions
        let conditions = conn.check_fast_path_rx(seq, ack, flags, win);
        
        if conditions.can_fast_path() {
            // Fast path processing
            self.total_fast_path.fetch_add(1, Ordering::Relaxed);
            conn.fast_path_receive(seq, data_len)
        } else {
            // Slow path processing
            self.total_slow_path.fetch_add(1, Ordering::Relaxed);
            conn.slow_path_receive(seq, ack, flags, data_len)
        }
    }
    
    /// Get fast path hit rate
    pub fn get_fast_path_rate(&self) -> u32 {
        let fast = self.total_fast_path.load(Ordering::Relaxed);
        let slow = self.total_slow_path.load(Ordering::Relaxed);
        let total = fast + slow;
        
        if total == 0 {
            return 0;
        }
        
        ((fast * 1000) / total) as u32
    }
}

/// Zero-copy sendfile support
pub struct ZeroCopyContext {
    /// Source file descriptor
    pub in_fd: u32,
    
    /// Destination socket descriptor
    pub out_fd: u32,
    
    /// File offset
    pub offset: u64,
    
    /// Number of bytes to send
    pub count: u64,
    
    /// Bytes sent
    pub sent: AtomicU64,
}

impl ZeroCopyContext {
    pub const fn new() -> Self {
        ZeroCopyContext {
            in_fd: 0,
            out_fd: 0,
            offset: 0,
            count: 0,
            sent: AtomicU64::new(0),
        }
    }
    
    /// Perform zero-copy sendfile
    /// Directly transfer data from page cache to socket buffer
    /// without copying to user space
    pub fn sendfile(&mut self) -> i64 {
        // TODO: Implement zero-copy sendfile
        // 1. Get page cache entry for file at offset
        // 2. Get socket buffer
        // 3. Map page cache page to socket buffer (splice)
        // 4. Update offset and sent count
        // 5. Return bytes sent
        
        -1  // Not implemented
    }
}

/// Global TCP fast path processor
static TCP_FAST_PATH: core::sync::OnceLock<TcpFastPathProcessor> = core::sync::OnceLock::new();

/// Get TCP fast path processor
pub fn tcp_fast_path() -> &'static TcpFastPathProcessor {
    TCP_FAST_PATH.get_or_init(TcpFastPathProcessor::new)
}

/// Initialize TCP fast path
pub fn init_tcp_fast_path() {
    get_tcp_fast_path().enabled.store(true, Ordering::Release);
}

/// Process TCP segment (fast path entry point)
#[inline(always)]
pub fn tcp_process_segment(conn: &mut TcpConnection, seq: u32, ack: u32, flags: u8, win: u16, data_len: u32) -> bool {
    get_tcp_fast_path().process_segment(conn, seq, ack, flags, win, data_len)
}
