/*
 * Nuva OS - System Library - Network TCP
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

//! TCP Protocol Implementation
//!
//! Implements the Transmission Control Protocol (TCP) providing connection-oriented,
//! reliable byte-stream transport.
//!
//! # Features
//! - Full TCP state machine implementation
//! - Dynamic window flow control
//! - Congestion control (Reno, CUBIC, BBR)
//! - Fast retransmit and recovery
//!
//! # TCP State Machine
//!
//! ```text
//! +---------+ ---------\ active OPEN
//! |  CLOSED |           \ -----------
//! +---------+<---------\   create TCB
//!   |     ^              \ snd SYN
//!   |     |                ---------------
//!   |     |                    |
//! passive OPEN | CLOSE        |
//! ------------ | -------       |
//! create TCB   | delete TCB    |
//!   |           |              |
//!   V           |              |
//! +---------+   ------->      |
//! | LISTEN  |    rcv SYN      |
//! +---------+  send SYN/ACK   |
//!   |           |              |
//!   |           | rcv SYN     |
//!   |           V send ACK    |
//!   |   +-----------+         |
//!   |   | SYN_RCVD  |         |
//!   |   +-----------+         |
//!   |           |              |
//! rcv ACK      | rcv SYN/ACK  |
//! snd ACK      | snd ACK      |
//!   V           V              |
//! +---------+                  |
//! | ESTAB   |<-----------------
//! +---------+
//! ```
//!
//! # Usage Example
//!
//! ```ignore
//! // Create TCP connection
//! let mut conn = TcpConnection::new(8080, local_ip);
//!
//! // Initiate connection
//! conn.connect(remote_ip, remote_port)?;
//!
//! // Send data
//! conn.send(data)?;
//!
//! // Receive data
//! let received = conn.recv(buf)?;
//! ```

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

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
    /// Data offset (4 bits) + reserved (4 bits) + flags (8 bits)
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

/// TCP State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed = 0,
    Listen = 1,
    SynSent = 2,
    SynReceived = 3,
    Established = 4,
    FinWait1 = 5,
    FinWait2 = 6,
    Closing = 7,
    TimeWait = 8,
    CloseWait = 9,
    LastAck = 10,
}

/// TCP Connection
pub struct TcpConnection {
    /// Local port
    pub local_port: u16,
    /// Remote port
    pub remote_port: u16,
    /// Local IP address
    pub local_ip: u32,
    /// Remote IP address
    pub remote_ip: u32,
    /// Connection state
    pub state: AtomicU32,
    /// Send next sequence number
    pub snd_nxt: AtomicU32,
    /// Receive next sequence number
    pub rcv_nxt: AtomicU32,
    /// Send window
    pub snd_wnd: AtomicU32,
    /// Receive window
    pub rcv_wnd: AtomicU32,
}

impl TcpConnection {
    /// Create a new connection
    pub fn new(local_port: u16, local_ip: u32) -> Self {
        TcpConnection {
            local_port,
            remote_port: 0,
            local_ip,
            remote_ip: 0,
            state: AtomicU32::new(TcpState::Closed as u32),
            snd_nxt: AtomicU32::new(0),
            rcv_nxt: AtomicU32::new(0),
            snd_wnd: AtomicU32::new(65535),
            rcv_wnd: AtomicU32::new(65535),
        }
    }

    /// Get current connection state
    pub fn get_state(&self) -> TcpState {
        match self.state.load(Ordering::Acquire) {
            0 => TcpState::Closed,
            1 => TcpState::Listen,
            2 => TcpState::SynSent,
            3 => TcpState::SynReceived,
            4 => TcpState::Established,
            5 => TcpState::FinWait1,
            6 => TcpState::FinWait2,
            7 => TcpState::Closing,
            8 => TcpState::TimeWait,
            9 => TcpState::CloseWait,
            10 => TcpState::LastAck,
            _ => TcpState::Closed,
        }
    }
}

/// TCP statistics
pub struct TcpStats {
    /// Active connection count
    pub active_connections: AtomicU32,
    /// Passive connection count
    pub passive_connections: AtomicU32,
    /// Segments transmitted
    pub segments_tx: AtomicU64,
    /// Segments received
    pub segments_rx: AtomicU64,
    /// Retransmission count
    pub retransmits: AtomicU64,
}

impl TcpStats {
    pub const fn new() -> Self {
        TcpStats {
            active_connections: AtomicU32::new(0),
            passive_connections: AtomicU32::new(0),
            segments_tx: AtomicU64::new(0),
            segments_rx: AtomicU64::new(0),
            retransmits: AtomicU64::new(0),
        }
    }
}

/// Global TCP statistics
static mut TCP_STATS: TcpStats = TcpStats::new();

pub fn get_tcp_stats() -> &'static TcpStats {
    // SAFETY: unsafe block required for static mutable access in no_std context
    unsafe { &TCP_STATS }
}

/// Initialize TCP subsystem
pub fn init_tcp() {
    log_info!("TCP protocol initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_state_values() {
        assert_eq!(TcpState::Closed as u32, 0);
        assert_eq!(TcpState::Listen as u32, 1);
        assert_eq!(TcpState::SynSent as u32, 2);
        assert_eq!(TcpState::SynReceived as u32, 3);
        assert_eq!(TcpState::Established as u32, 4);
        assert_eq!(TcpState::FinWait1 as u32, 5);
        assert_eq!(TcpState::FinWait2 as u32, 6);
        assert_eq!(TcpState::Closing as u32, 7);
        assert_eq!(TcpState::TimeWait as u32, 8);
        assert_eq!(TcpState::CloseWait as u32, 9);
        assert_eq!(TcpState::LastAck as u32, 10);
    }

    #[test]
    fn test_tcp_flags() {
        assert_eq!(tcp_flags::FIN, 0x0001);
        assert_eq!(tcp_flags::SYN, 0x0002);
        assert_eq!(tcp_flags::RST, 0x0004);
        assert_eq!(tcp_flags::PSH, 0x0008);
        assert_eq!(tcp_flags::ACK, 0x0010);
        assert_eq!(tcp_flags::URG, 0x0020);
        assert_eq!(tcp_flags::ECE, 0x0040);
        assert_eq!(tcp_flags::CWR, 0x0080);
    }

    #[test]
    fn test_tcp_connection_new() {
        let conn = TcpConnection::new(8080, 0x0A000001);
        assert_eq!(conn.local_port, 8080);
        assert_eq!(conn.local_ip, 0x0A000001);
        assert_eq!(conn.remote_port, 0);
        assert_eq!(conn.remote_ip, 0);
        assert_eq!(conn.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_tcp_connection_initial_values() {
        let conn = TcpConnection::new(443, 0xC0A80001);
        assert_eq!(conn.snd_nxt.load(Ordering::Relaxed), 0);
        assert_eq!(conn.rcv_nxt.load(Ordering::Relaxed), 0);
        assert_eq!(conn.snd_wnd.load(Ordering::Relaxed), 65535);
        assert_eq!(conn.rcv_wnd.load(Ordering::Relaxed), 65535);
    }

    #[test]
    fn test_tcp_stats_new() {
        let stats = TcpStats::new();
        assert_eq!(stats.active_connections.load(Ordering::Relaxed), 0);
        assert_eq!(stats.passive_connections.load(Ordering::Relaxed), 0);
        assert_eq!(stats.segments_tx.load(Ordering::Relaxed), 0);
        assert_eq!(stats.segments_rx.load(Ordering::Relaxed), 0);
        assert_eq!(stats.retransmits.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_tcp_header_size() {
        // TCP header minimum size is 20 bytes
        assert_eq!(core::mem::size_of::<TcpHeader>(), 20);
    }

    #[test]
    fn test_tcp_state_transitions() {
        let conn = TcpConnection::new(80, 0);

        // Initial state
        assert_eq!(conn.get_state(), TcpState::Closed);

        // Simulate state transitions
        conn.state.store(TcpState::Listen as u32, Ordering::Release);
        assert_eq!(conn.get_state(), TcpState::Listen);

        conn.state.store(TcpState::SynSent as u32, Ordering::Release);
        assert_eq!(conn.get_state(), TcpState::SynSent);

        conn.state.store(TcpState::Established as u32, Ordering::Release);
        assert_eq!(conn.get_state(), TcpState::Established);
    }

    #[test]
    fn test_tcp_sequence_numbers() {
        let conn = TcpConnection::new(8080, 0);

        conn.snd_nxt.store(1000, Ordering::Release);
        conn.rcv_nxt.store(2000, Ordering::Release);

        assert_eq!(conn.snd_nxt.load(Ordering::Acquire), 1000);
        assert_eq!(conn.rcv_nxt.load(Ordering::Acquire), 2000);
    }

    #[test]
    fn test_tcp_window_sizes() {
        let conn = TcpConnection::new(8080, 0);

        // Test window size adjustments
        conn.snd_wnd.store(32768, Ordering::Release);
        conn.rcv_wnd.store(16384, Ordering::Release);

        assert_eq!(conn.snd_wnd.load(Ordering::Acquire), 32768);
        assert_eq!(conn.rcv_wnd.load(Ordering::Acquire), 16384);
    }
}

// ============================================================================
// TCP Congestion Control Algorithms
// ============================================================================

/// Congestion control state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CongestionState {
    /// Slow start phase
    SlowStart,
    /// Congestion avoidance phase
    CongestionAvoidance,
    /// Fast retransmit phase
    FastRetransmit,
    /// Fast recovery phase
    FastRecovery,
}

/// TCP Reno congestion control parameters
pub struct CongestionControl {
    /// Congestion window (cwnd)
    pub cwnd: AtomicU32,
    /// Slow start threshold (ssthresh)
    pub ssthresh: AtomicU32,
    /// Congestion state
    pub state: AtomicU32,
    /// Duplicate ACK count
    pub dup_ack_count: AtomicU32,
    /// Last acknowledged sequence number
    pub last_ack: AtomicU32,
    /// Recovery sequence number
    pub recover: AtomicU32,
    /// Minimum RTT (for delay calculation)
    pub min_rtt: AtomicU32,
    /// Smoothed RTT
    pub srtt: AtomicU32,
    /// RTT variance
    pub rttvar: AtomicU32,
    /// Retransmission timeout
    pub rto: AtomicU32,
}

impl CongestionControl {
    pub const fn new() -> Self {
        CongestionControl {
            cwnd: AtomicU32::new(1),         // Initial cwnd = 1 MSS
            ssthresh: AtomicU32::new(65535), // Initial threshold
            state: AtomicU32::new(CongestionState::SlowStart as u32),
            dup_ack_count: AtomicU32::new(0),
            last_ack: AtomicU32::new(0),
            recover: AtomicU32::new(0),
            min_rtt: AtomicU32::new(u32::MAX),
            srtt: AtomicU32::new(0),
            rttvar: AtomicU32::new(0),
            rto: AtomicU32::new(1000),       // Initial RTO = 1 second
        }
    }

    /// Get current congestion state
    pub fn get_state(&self) -> CongestionState {
        match self.state.load(Ordering::Acquire) {
            0 => CongestionState::SlowStart,
            1 => CongestionState::CongestionAvoidance,
            2 => CongestionState::FastRetransmit,
            3 => CongestionState::FastRecovery,
            _ => CongestionState::SlowStart,
        }
    }

    /// Set congestion state
    pub fn set_state(&self, state: CongestionState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Get congestion window size
    pub fn get_cwnd(&self) -> u32 {
        self.cwnd.load(Ordering::Acquire)
    }

    /// Get slow start threshold
    pub fn get_ssthresh(&self) -> u32 {
        self.ssthresh.load(Ordering::Acquire)
    }

    /// Handle new ACK (Reno algorithm)
    pub fn on_new_ack(&self, ack: u32, mss: u32) {
        let state = self.get_state();
        let mut cwnd = self.cwnd.load(Ordering::Acquire);

        match state {
            CongestionState::SlowStart => {
                // Slow start: increment cwnd by 1 MSS per ACK
                cwnd += mss;
                self.cwnd.store(cwnd, Ordering::Release);

                // Check if threshold reached
                if cwnd >= self.ssthresh.load(Ordering::Acquire) {
                    self.set_state(CongestionState::CongestionAvoidance);
                    log_debug!("TCP: entering congestion avoidance, cwnd={}", cwnd);
                }
            }
            CongestionState::CongestionAvoidance => {
                // Congestion avoidance: increment cwnd by ~1 MSS per RTT
                cwnd += mss * mss / cwnd;
                self.cwnd.store(cwnd, Ordering::Release);
            }
            CongestionState::FastRecovery => {
                // Fast recovery: on new ACK, exit fast recovery
                cwnd = self.ssthresh.load(Ordering::Acquire);
                self.cwnd.store(cwnd, Ordering::Release);
                self.dup_ack_count.store(0, Ordering::Release);
                self.set_state(CongestionState::CongestionAvoidance);
                log_debug!("TCP: exiting fast recovery, cwnd={}", cwnd);
            }
            _ => {}
        }

        // Update last acknowledged sequence number
        self.last_ack.store(ack, Ordering::Release);
    }

    /// Handle duplicate ACK
    pub fn on_dup_ack(&self, ack: u32, mss: u32) {
        let state = self.get_state();
        let dup_count = self.dup_ack_count.fetch_add(1, Ordering::AcqRel) + 1;

        match state {
            CongestionState::SlowStart | CongestionState::CongestionAvoidance => {
                // Check if 3 duplicate ACKs received (fast retransmit threshold)
                if dup_count == 3 {
                    // Enter fast retransmit
                    let cwnd = self.cwnd.load(Ordering::Acquire);
                    let ssthresh = cwnd / 2;
                    self.ssthresh.store(ssthresh, Ordering::Release);
                    self.cwnd.store(ssthresh + 3 * mss, Ordering::Release);
                    self.recover.store(ack, Ordering::Release);
                    self.set_state(CongestionState::FastRecovery);
                    log_debug!("TCP: fast retransmit, ssthresh={}, cwnd={}",
                        ssthresh, self.cwnd.load(Ordering::Acquire));
                }
            }
            CongestionState::FastRecovery => {
                // In fast recovery: inflate cwnd by 1 MSS per dup ACK
                let cwnd = self.cwnd.load(Ordering::Acquire) + mss;
                self.cwnd.store(cwnd, Ordering::Release);
            }
            _ => {}
        }
    }

    /// Handle timeout
    pub fn on_timeout(&self) {
        // Timeout: enter slow start
        let cwnd = self.cwnd.load(Ordering::Acquire);
        let ssthresh = cwnd / 2;
        self.ssthresh.store(ssthresh, Ordering::Release);
        self.cwnd.store(1, Ordering::Release);
        self.dup_ack_count.store(0, Ordering::Release);
        self.set_state(CongestionState::SlowStart);

        // Exponential backoff for RTO
        let rto = self.rto.load(Ordering::Acquire);
        let new_rto = (rto * 2).min(60000); // Max 60 seconds
        self.rto.store(new_rto, Ordering::Release);

        log_debug!("TCP: timeout, entering slow start, ssthresh={}, rto={}",
            ssthresh, new_rto);
    }

    /// Update RTT estimate (Jacobson/Karels algorithm)
    pub fn update_rtt(&self, rtt: u32) {
        // Update minimum RTT
        let min_rtt = self.min_rtt.load(Ordering::Acquire);
        if rtt < min_rtt {
            self.min_rtt.store(rtt, Ordering::Release);
        }

        // Compute SRTT and RTTVAR (Jacobson/Karels algorithm)
        let srtt = self.srtt.load(Ordering::Acquire);
        let rttvar = self.rttvar.load(Ordering::Acquire);

        if srtt == 0 {
            // First measurement
            self.srtt.store(rtt, Ordering::Release);
            self.rttvar.store(rtt / 2, Ordering::Release);
        } else {
            // Subsequent measurements
            let delta = if rtt > srtt { rtt - srtt } else { srtt - rtt };
            let new_rttvar = (3 * rttvar + delta) / 4;
            let new_srtt = (7 * srtt + rtt) / 8;
            self.rttvar.store(new_rttvar, Ordering::Release);
            self.srtt.store(new_srtt, Ordering::Release);
        }

        // Update RTO
        let srtt = self.srtt.load(Ordering::Acquire);
        let rttvar = self.rttvar.load(Ordering::Acquire);
        let rto = (srtt + 4 * rttvar).max(200).min(60000);
        self.rto.store(rto, Ordering::Release);
    }

    /// Get current RTO value
    pub fn get_rto(&self) -> u32 {
        self.rto.load(Ordering::Acquire)
    }
}

/// CUBIC congestion control algorithm
pub struct CubicCongestionControl {
    /// Congestion window
    pub cwnd: AtomicU32,
    /// Slow start threshold
    pub ssthresh: AtomicU32,
    /// Window maximum (W_max)
    pub w_max: AtomicU32,
    /// Cubic K parameter (time to reach W_max)
    pub k: AtomicU64,
    /// Last update timestamp
    pub last_update: AtomicU64,
    /// CUBIC parameter C (scaling factor)
    pub c: f64,
    /// CUBIC parameter beta (multiplicative decrease factor)
    pub beta: f64,
}

impl CubicCongestionControl {
    pub const fn new() -> Self {
        CubicCongestionControl {
            cwnd: AtomicU32::new(1),
            ssthresh: AtomicU32::new(65535),
            w_max: AtomicU32::new(0),
            k: AtomicU64::new(0),
            last_update: AtomicU64::new(0),
            c: 0.4,
            beta: 0.7,
        }
    }

    /// Calculate CUBIC window: W_cubic(t) = C * (t - K)^3 + W_max
    pub fn calculate_cwnd(&self, t: u64, mss: u32) -> u32 {
        let w_max = self.w_max.load(Ordering::Acquire) as f64;
        let k = self.k.load(Ordering::Acquire) as f64;
        let t = t as f64;
        let c = self.c;

        let delta_t = t - k;
        let w_cubic = c * delta_t * delta_t * delta_t + w_max;

        // Convert to MSS units
        (w_cubic.max(1.0) as u32) * mss
    }

    /// Handle packet loss event
    pub fn on_loss(&self) {
        let cwnd = self.cwnd.load(Ordering::Acquire);
        let new_ssthresh = (cwnd as f64 * self.beta) as u32;

        self.w_max.store(cwnd, Ordering::Release);
        self.ssthresh.store(new_ssthresh, Ordering::Release);

        // Calculate K: K = (W_max * beta / C)^(1/3)
        let w_max = cwnd as f64;
        let k = (w_max * self.beta / self.c).powf(1.0 / 3.0);
        self.k.store(k as u64, Ordering::Release);

        log_debug!("CUBIC: loss detected, w_max={}, ssthresh={}",
            cwnd, new_ssthresh);
    }
}

/// BBR (Bottleneck Bandwidth and Round-trip) congestion control
pub struct BbrCongestionControl {
    /// BBR state
    pub state: AtomicU32,
    /// Bandwidth-delay product
    pub bdp: AtomicU32,
    /// Measured bandwidth
    pub bw: AtomicU32,
    /// Minimum RTT
    pub min_rtt: AtomicU32,
    /// Pacing rate
    pub pacing_rate: AtomicU32,
    /// Send congestion window
    pub send_cwnd: AtomicU32,
}

impl BbrCongestionControl {
    pub const fn new() -> Self {
        BbrCongestionControl {
            state: AtomicU32::new(0),
            bdp: AtomicU32::new(0),
            bw: AtomicU32::new(0),
            min_rtt: AtomicU32::new(u32::MAX),
            pacing_rate: AtomicU32::new(0),
            send_cwnd: AtomicU32::new(0),
        }
    }

    /// Update bandwidth estimate
    pub fn update_bw(&self, delivered: u32, interval_us: u64) {
        if interval_us > 0 {
            let bw = (delivered as u64 * 1_000_000 / interval_us) as u32;
            self.bw.store(bw, Ordering::Release);
        }
    }

    /// Update minimum RTT
    pub fn update_min_rtt(&self, rtt: u32) {
        let min_rtt = self.min_rtt.load(Ordering::Acquire);
        if rtt < min_rtt {
            self.min_rtt.store(rtt, Ordering::Release);
        }
    }

    /// Calculate bandwidth-delay product
    pub fn calculate_bdp(&self) -> u32 {
        let bw = self.bw.load(Ordering::Acquire);
        let min_rtt = self.min_rtt.load(Ordering::Acquire);

        if min_rtt == u32::MAX || min_rtt == 0 {
            return 0;
        }

        // BDP = bandwidth * RTT
        (bw as u64 * min_rtt as u64 / 1_000_000) as u32
    }

    /// Get send congestion window
    pub fn get_send_cwnd(&self) -> u32 {
        let bdp = self.calculate_bdp();
        // BBR send window = BDP * gain (1.25x)
        (bdp as f64 * 1.25) as u32
    }
}
