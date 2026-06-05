/*
 * Nuva OS - Kernel - Net - Tcp
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
 * Nuva OS - Kernel - TCP Protocol
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Transmission Control Protocol (TCP) implementation.
 * Full state machine per RFC 793 with timers and congestion control.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_debug, pr_info, pr_warn};

use crate::posix::errno::Errno;
/// TCP Header
#[repr(C, packed)]
pub struct TcpHeader {
    /// Source port
    pub source: u16,
    /// Destination port
    pub dest: u16,
    /// Sequence number
    pub seq: u32,
    /// Acknowledgment number
    pub ack_seq: u32,
    /// Data offset (4 bits) + Reserved (4 bits) + Flags (8 bits)
    pub doff_flags: u16,
    /// Window size
    pub window: u16,
    /// Checksum
    pub check: u16,
    /// Urgent pointer
    pub urg_ptr: u16,
}

impl TcpHeader {
    /// Header minimum size
    pub const MIN_SIZE: usize = 20;

    /// Create new TCP header
    pub fn new(source: u16, dest: u16, seq: u32, ack_seq: u32, flags: u16, window: u16) -> Self {
        TcpHeader {
            source: source.to_be(),
            dest: dest.to_be(),
            seq: seq.to_be(),
            ack_seq: ack_seq.to_be(),
            doff_flags: ((5 << 12) | flags).to_be(),
            window: window.to_be(),
            check: 0,
            urg_ptr: 0,
        }
    }

    /// Get source port (host byte order)
    pub fn get_source(&self) -> u16 {
        u16::from_be(self.source)
    }

    /// Get destination port (host byte order)
    pub fn get_dest(&self) -> u16 {
        u16::from_be(self.dest)
    }

    /// Get sequence number (host byte order)
    pub fn get_seq(&self) -> u32 {
        u32::from_be(self.seq)
    }

    /// Get acknowledgment number (host byte order)
    pub fn get_ack_seq(&self) -> u32 {
        u32::from_be(self.ack_seq)
    }

    /// Get data offset (in bytes)
    pub fn doff(&self) -> u8 {
        ((u16::from_be(self.doff_flags) >> 12) & 0x0F) as u8 * 4
    }

    /// Get flags
    pub fn flags(&self) -> u16 {
        u16::from_be(self.doff_flags) & 0x00FF
    }

    /// Get window size (host byte order)
    pub fn get_window(&self) -> u16 {
        u16::from_be(self.window)
    }

    /// Check if SYN flag is set
    pub fn is_syn(&self) -> bool {
        (self.flags() & tcp_flags::SYN) != 0
    }

    /// Check if ACK flag is set
    pub fn is_ack(&self) -> bool {
        (self.flags() & tcp_flags::ACK) != 0
    }

    /// Check if FIN flag is set
    pub fn is_fin(&self) -> bool {
        (self.flags() & tcp_flags::FIN) != 0
    }

    /// Check if RST flag is set
    pub fn is_rst(&self) -> bool {
        (self.flags() & tcp_flags::RST) != 0
    }
}

/// TCP Flags
pub mod tcp_flags {
    /// FIN
    pub const FIN: u16 = 0x01;
    /// SYN
    pub const SYN: u16 = 0x02;
    /// RST
    pub const RST: u16 = 0x04;
    /// PSH
    pub const PSH: u16 = 0x08;
    /// ACK
    pub const ACK: u16 = 0x10;
    /// URG
    pub const URG: u16 = 0x20;
    /// ECE
    pub const ECE: u16 = 0x40;
    /// CWR
    pub const CWR: u16 = 0x80;
}

/// TCP State (RFC 793, all 11 states)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    /// Closed
    Closed = 0,
    /// Listen
    Listen = 1,
    /// SYN Sent
    SynSent = 2,
    /// SYN Received
    SynReceived = 3,
    /// Established
    Established = 4,
    /// FIN Wait 1
    FinWait1 = 5,
    /// FIN Wait 2
    FinWait2 = 6,
    /// Close Wait
    CloseWait = 7,
    /// Closing
    Closing = 8,
    /// Last ACK
    LastAck = 9,
    /// Time Wait
    TimeWait = 10,
}

impl TcpState {
    /// Check if connection can send data
    pub fn can_send(&self) -> bool {
        matches!(self, TcpState::Established | TcpState::CloseWait)
    }

    /// Check if connection can receive data
    pub fn can_receive(&self) -> bool {
        matches!(
            self,
            TcpState::Established | TcpState::FinWait1 | TcpState::FinWait2
        )
    }

    /// Check if connection is closing
    pub fn is_closing(&self) -> bool {
        matches!(
            self,
            TcpState::FinWait1
                | TcpState::FinWait2
                | TcpState::Closing
                | TcpState::TimeWait
                | TcpState::CloseWait
                | TcpState::LastAck
        )
    }

    /// Get state name
    pub fn as_str(&self) -> &'static str {
        match self {
            TcpState::Closed => "CLOSED",
            TcpState::Listen => "LISTEN",
            TcpState::SynSent => "SYN_SENT",
            TcpState::SynReceived => "SYN_RECEIVED",
            TcpState::Established => "ESTABLISHED",
            TcpState::FinWait1 => "FIN_WAIT_1",
            TcpState::FinWait2 => "FIN_WAIT_2",
            TcpState::CloseWait => "CLOSE_WAIT",
            TcpState::Closing => "CLOSING",
            TcpState::LastAck => "LAST_ACK",
            TcpState::TimeWait => "TIME_WAIT",
        }
    }
}

/// TCP State Machine — tracks and validates state transitions per RFC 793.
///
/// This struct wraps the raw u32 state value and enforces valid state
/// transitions. Any invalid transition is logged and clamped.
pub struct TcpStateMachine {
    /// Current TCP state (stored as u32 for atomic compatibility)
    state: AtomicU32,
}

impl TcpStateMachine {
    pub const fn new(initial: TcpState) -> Self {
        TcpStateMachine {
            state: AtomicU32::new(initial as u32),
        }
    }

    /// Get the current state
    pub fn get_state(&self) -> TcpState {
        match self.state.load(Ordering::Acquire) {
            0 => TcpState::Closed,
            1 => TcpState::Listen,
            2 => TcpState::SynSent,
            3 => TcpState::SynReceived,
            4 => TcpState::Established,
            5 => TcpState::FinWait1,
            6 => TcpState::FinWait2,
            7 => TcpState::CloseWait,
            8 => TcpState::Closing,
            9 => TcpState::LastAck,
            10 => TcpState::TimeWait,
            _ => TcpState::Closed,
        }
    }

    /// Validate and perform a state transition.
    /// Returns true if the transition was valid, false if it was clamped (logged).
    pub fn transition(&self, new_state: TcpState) -> bool {
        let old = self.get_state();
        if Self::is_valid_transition(old, new_state) {
            self.state.store(new_state as u32, Ordering::Release);
            true
        } else {
            pr_warn!(
                "TCP: invalid state transition {} -> {}",
                old.as_str(),
                new_state.as_str()
            );
            // Clamp to known reachable states for robustness
            let clamped = Self::clamp_transition(old, new_state);
            self.state.store(clamped as u32, Ordering::Release);
            false
        }
    }

    /// Directly set the state without validation (e.g. for RST processing).
    pub fn force_set(&self, new_state: TcpState) {
        self.state.store(new_state as u32, Ordering::Release);
    }

    /// Check if a transition from `old` to `new` is valid per RFC 793.
    ///
    /// This covers the complete state transition diagram:
    /// - CLOSED -> LISTEN (passive open)
    /// - CLOSED -> SYN_SENT (active open)
    /// - LISTEN -> SYN_RECEIVED (received SYN)
    /// - LISTEN -> CLOSED (close)
    /// - SYN_SENT -> CLOSED (timeout/reset)
    /// - SYN_SENT -> SYN_RECEIVED (simultaneous open)
    /// - SYN_SENT -> ESTABLISHED (received SYN-ACK)
    /// - SYN_RECEIVED -> ESTABLISHED (received ACK of SYN)
    /// - SYN_RECEIVED -> FIN_WAIT_1 (active close while in SYN_RECEIVED)
    /// - SYN_RECEIVED -> CLOSED (reset)
    /// - ESTABLISHED -> FIN_WAIT_1 (active close)
    /// - ESTABLISHED -> CLOSE_WAIT (received FIN)
    /// - FIN_WAIT_1 -> FIN_WAIT_2 (received ACK of FIN)
    /// - FIN_WAIT_1 -> CLOSING (received FIN+ACK simultaneously)
    /// - FIN_WAIT_1 -> CLOSED (reset)
    /// - FIN_WAIT_2 -> TIME_WAIT (received FIN)
    /// - CLOSING -> TIME_WAIT (received ACK of FIN)
    /// - CLOSE_WAIT -> LAST_ACK (active close after receiving FIN)
    /// - LAST_ACK -> CLOSED (received ACK of FIN)
    /// - TIME_WAIT -> CLOSED (2MSL timeout)
    ///
    /// All other transitions return false.
    pub fn is_valid_transition(old: TcpState, new: TcpState) -> bool {
        use TcpState::*;
        match (old, new) {
            // Passive open
            (Closed, Listen) => true,
            // Active open
            (Closed, SynSent) => true,
            // Close while listening
            (Listen, Closed) => true,
            // Received SYN in LISTEN (passive open continuation)
            (Listen, SynReceived) => true,
            // Received SYN-ACK in SYN_SENT (active open completion)
            (SynSent, Established) => true,
            // Received SYN in SYN_SENT (simultaneous open)
            (SynSent, SynReceived) => true,
            // Timeout / reset while SYN_SENT
            (SynSent, Closed) => true,
            // Received ACK of SYN in SYN_RECEIVED
            (SynReceived, Established) => true,
            // Close during SYN_RECEIVED
            (SynReceived, FinWait1) => true,
            // Reset in SYN_RECEIVED
            (SynReceived, Closed) => true,
            // Active close from ESTABLISHED
            (Established, FinWait1) => true,
            // Received FIN in ESTABLISHED (passive close)
            (Established, CloseWait) => true,
            // Reset in ESTABLISHED
            (Established, Closed) => true,
            // Received ACK of FIN in FIN_WAIT_1
            (FinWait1, FinWait2) => true,
            // Received FIN+ACK simultaneously
            (FinWait1, Closing) => true,
            // Received FIN in FIN_WAIT_1
            (FinWait1, TimeWait) => true,
            // Reset in FIN_WAIT_1
            (FinWait1, Closed) => true,
            // Received FIN in FIN_WAIT_2
            (FinWait2, TimeWait) => true,
            // Reset in FIN_WAIT_2
            (FinWait2, Closed) => true,
            // Active close from CLOSE_WAIT
            (CloseWait, LastAck) => true,
            // Reset in CLOSE_WAIT
            (CloseWait, Closed) => true,
            // ACK of FIN received in CLOSING
            (Closing, TimeWait) => true,
            // Reset in CLOSING
            (Closing, Closed) => true,
            // ACK of FIN received in LAST_ACK
            (LastAck, Closed) => true,
            // 2MSL timeout in TIME_WAIT
            (TimeWait, Closed) => true,
            // Self-transition (timer re-fires, benign duplicate)
            (s1, s2) if s1 == s2 => true,
            // All other transitions are invalid
            _ => false,
        }
    }

    /// Clamp an invalid transition to the nearest reachable state.
    /// This provides robustness when unexpected segments arrive.
    fn clamp_transition(old: TcpState, new: TcpState) -> TcpState {
        use TcpState::*;
        // For bidirectional CLOSED transitions, allow
        match (old, new) {
            // If attempting to go to ESTABLISHED from unexpected state, try SYN_RECEIVED
            (Closed, Established) => SynReceived,
            // If attempting to go to LISTEN from non-CLOSED, stay where we are
            (_, Listen) => old,
            // Default: allow self-transition
            _ => old,
        }
    }

    /// Get the raw atomic state field for compatibility with TcpConnection
    pub fn atomic_state(&self) -> &AtomicU32 {
        &self.state
    }
}

/// Retransmit queue entry — holds a segment that may need retransmission.
#[derive(Clone)]
pub struct RetransmitEntry {
    /// Sequence number of the first byte in this segment
    pub seq: u32,
    /// Length of data in this segment
    pub len: u16,
    /// Segment data (header not included, just payload)
    pub data: [u8; 64],
    /// Number of times this segment has been retransmitted
    pub rtx_count: u8,
    /// Timestamp (ms) of last transmission
    pub last_sent_ms: u64,
    /// Whether this segment contains SYN flag
    pub is_syn: bool,
    /// Whether this segment contains FIN flag
    pub is_fin: bool,
}

impl RetransmitEntry {
    pub const fn new() -> Self {
        RetransmitEntry {
            seq: 0,
            len: 0,
            data: [0; 64],
            rtx_count: 0,
            last_sent_ms: 0,
            is_syn: false,
            is_fin: false,
        }
    }
}

/// Out-of-order segment queue entry — buffered segment that arrived ahead of rcv_nxt.
#[derive(Clone)]
pub struct OutOfOrderEntry {
    /// Sequence number of the first byte
    pub seq: u32,
    /// Length of data
    pub len: u16,
    /// Segment data
    pub data: [u8; 128],
}

impl OutOfOrderEntry {
    pub const fn new() -> Self {
        OutOfOrderEntry {
            seq: 0,
            len: 0,
            data: [0; 128],
        }
    }
}

/// TCP segment arrival result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentResult {
    /// No action needed
    Ok,
    /// Send SYN-ACK (passive open received SYN)
    SendSynAck,
    /// Send ACK (completing handshake or acknowledging data)
    SendAck,
    /// Send FIN-ACK (acknowledging received FIN)
    SendFinAck,
    /// Send RST (reset connection)
    SendRst,
    /// Connection established
    Established,
    /// Connection closed
    Closed,
    /// Error: invalid segment for current state
    Invalid,
    /// Error: bad checksum
    BadChecksum,
    /// Retransmit missing segment (Fast Retransmit)
    Retransmit,
}

/// TCP timer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpTimerType {
    /// Retransmission timer
    Retransmit,
    /// Keepalive timer
    Keepalive,
    /// TIME_WAIT 2MSL timer
    TimeWait,
    /// Delayed ACK timer
    DelayedAck,
    /// Persist timer (zero window probe)
    Persist,
}

/// TCP timer configuration
pub struct TcpTimerConfig {
    /// Minimum RTO in ms
    pub rto_min: u32,
    /// Maximum RTO in ms
    pub rto_max: u32,
    /// Initial RTO in ms
    pub rto_initial: u32,
    /// Keepalive idle time in ms
    pub keepalive_idle: u32,
    /// Keepalive interval in ms
    pub keepalive_interval: u32,
    /// Keepalive probe count
    pub keepalive_probes: u32,
    /// TIME_WAIT timeout in ms (2MSL)
    pub timewait_timeout: u32,
    /// Delayed ACK timeout in ms
    pub delayed_ack_timeout: u32,
}

impl TcpTimerConfig {
    pub const fn default() -> Self {
        Self {
            rto_min: 200,
            rto_max: 60000,
            rto_initial: 1000,
            keepalive_idle: 7200000,
            keepalive_interval: 75000,
            keepalive_probes: 9,
            timewait_timeout: 60000,
            delayed_ack_timeout: 200,
        }
    }
}

/// TCP timer state
pub struct TcpTimers {
    /// Retransmission timer expiration (ms timestamp)
    pub rto_expire: AtomicU64,
    /// Number of retransmissions
    pub rtx_count: AtomicU32,
    /// Maximum retransmissions before giving up
    pub max_rtx: AtomicU32,
    /// Keepalive timer expiration
    pub keepalive_expire: AtomicU64,
    /// Keepalive probe count
    pub keepalive_probes: AtomicU32,
    /// TIME_WAIT expiration
    pub timewait_expire: AtomicU64,
    /// Delayed ACK expiration
    pub delayed_ack_expire: AtomicU64,
    /// Persist timer expiration
    pub persist_expire: AtomicU64,
}

impl TcpTimers {
    pub const fn new() -> Self {
        Self {
            rto_expire: AtomicU64::new(0),
            rtx_count: AtomicU32::new(0),
            max_rtx: AtomicU32::new(15),
            keepalive_expire: AtomicU64::new(0),
            keepalive_probes: AtomicU32::new(0),
            timewait_expire: AtomicU64::new(0),
            delayed_ack_expire: AtomicU64::new(0),
            persist_expire: AtomicU64::new(0),
        }
    }

    /// Start retransmission timer
    pub fn start_rto(&self, rto_ms: u32, now_ms: u64) {
        let expire = now_ms + rto_ms as u64;
        self.rto_expire.store(expire, Ordering::Release);
        self.rtx_count.store(0, Ordering::Release);
    }

    /// Check if retransmission timer expired
    pub fn rto_expired(&self, now_ms: u64) -> bool {
        let expire = self.rto_expire.load(Ordering::Acquire);
        expire > 0 && now_ms >= expire
    }

    /// Cancel retransmission timer
    pub fn cancel_rto(&self) {
        self.rto_expire.store(0, Ordering::Release);
    }

    /// Increment retransmit count, returns true if max exceeded
    pub fn increment_rtx(&self) -> bool {
        let count = self.rtx_count.fetch_add(1, Ordering::AcqRel) + 1;
        count >= self.max_rtx.load(Ordering::Acquire)
    }

    /// Start keepalive timer
    pub fn start_keepalive(&self, idle_ms: u64, now_ms: u64) {
        let expire = now_ms + idle_ms;
        self.keepalive_expire.store(expire, Ordering::Release);
        self.keepalive_probes.store(0, Ordering::Release);
    }

    /// Check if keepalive timer expired
    pub fn keepalive_expired(&self, now_ms: u64) -> bool {
        let expire = self.keepalive_expire.load(Ordering::Acquire);
        expire > 0 && now_ms >= expire
    }

    /// Start TIME_WAIT timer
    pub fn start_timewait(&self, timeout_ms: u64, now_ms: u64) {
        let expire = now_ms + timeout_ms;
        self.timewait_expire.store(expire, Ordering::Release);
    }

    /// Check if TIME_WAIT expired
    pub fn timewait_expired(&self, now_ms: u64) -> bool {
        let expire = self.timewait_expire.load(Ordering::Acquire);
        expire > 0 && now_ms >= expire
    }

    /// Start delayed ACK timer
    pub fn start_delayed_ack(&self, timeout_ms: u64, now_ms: u64) {
        let expire = now_ms + timeout_ms;
        self.delayed_ack_expire.store(expire, Ordering::Release);
    }

    /// Check if delayed ACK timer expired
    pub fn delayed_ack_expired(&self, now_ms: u64) -> bool {
        let expire = self.delayed_ack_expire.load(Ordering::Acquire);
        expire > 0 && now_ms >= expire
    }

    /// Start persist timer
    pub fn start_persist(&self, rto_ms: u32, now_ms: u64) {
        let expire = now_ms + rto_ms as u64;
        self.persist_expire.store(expire, Ordering::Release);
    }

    /// Check if persist timer expired
    pub fn persist_expired(&self, now_ms: u64) -> bool {
        let expire = self.persist_expire.load(Ordering::Acquire);
        expire > 0 && now_ms >= expire
    }
}

/// TCP Connection
#[repr(C)]
pub struct TcpConnection {
    /// Local address
    pub local_addr: u32,
    /// Local port
    pub local_port: u16,
    /// Remote address
    pub remote_addr: u32,
    /// Remote port
    pub remote_port: u16,
    /// State
    pub state: AtomicU32,
    /// Send unacknowledged
    pub snd_una: AtomicU32,
    /// Send next
    pub snd_nxt: AtomicU32,
    /// Send window
    pub snd_wnd: AtomicU32,
    /// Receive next
    pub rcv_nxt: AtomicU32,
    /// Receive window
    pub rcv_wnd: AtomicU32,
    /// ISS (Initial Send Sequence)
    pub iss: AtomicU32,
    /// IRS (Initial Receive Sequence)
    pub irs: AtomicU32,
    /// RTT (Round Trip Time) in ms
    pub srtt: AtomicU32,
    /// RTT variance
    pub rttvar: AtomicU32,
    /// RTO (Retransmission Timeout) in ms
    pub rto: AtomicU32,
    /// Congestion window
    pub cwnd: AtomicU32,
    /// Slow start threshold
    pub ssthresh: AtomicU32,
    /// Last activity timestamp (ms)
    pub last_activity: AtomicU64,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Send next (non-atomic for direct use)
    pub send_nxt: u32,
    /// Receive next (non-atomic for direct use)
    pub recv_nxt: u32,
    /// Window (non-atomic for direct use)
    pub window: u16,
    /// Maximum segment size
    pub mss: u16,
    /// Duplicate ACK count
    pub dup_ack_count: AtomicU32,
    /// Last ACK received
    pub last_ack: AtomicU32,
    /// Timers
    pub timers: TcpTimers,
}

impl TcpConnection {
    pub fn new(local_addr: u32, local_port: u16, remote_addr: u32, remote_port: u16) -> Self {
        TcpConnection {
            local_addr,
            local_port,
            remote_addr,
            remote_port,
            state: AtomicU32::new(TcpState::Closed as u32),
            snd_una: AtomicU32::new(0),
            snd_nxt: AtomicU32::new(0),
            snd_wnd: AtomicU32::new(65535),
            rcv_nxt: AtomicU32::new(0),
            rcv_wnd: AtomicU32::new(65535),
            iss: AtomicU32::new(0),
            irs: AtomicU32::new(0),
            srtt: AtomicU32::new(0),
            rttvar: AtomicU32::new(0),
            rto: AtomicU32::new(1000),
            cwnd: AtomicU32::new(1460),
            ssthresh: AtomicU32::new(65535),
            last_activity: AtomicU64::new(0),
            ref_count: AtomicU32::new(1),
            send_nxt: 0,
            recv_nxt: 0,
            window: 65535,
            mss: 1460,
            dup_ack_count: AtomicU32::new(0),
            last_ack: AtomicU32::new(0),
            timers: TcpTimers::new(),
        }
    }

    /// Get current state
    pub fn get_state(&self) -> TcpState {
        match self.state.load(Ordering::Acquire) {
            0 => TcpState::Closed,
            1 => TcpState::Listen,
            2 => TcpState::SynSent,
            3 => TcpState::SynReceived,
            4 => TcpState::Established,
            5 => TcpState::FinWait1,
            6 => TcpState::FinWait2,
            7 => TcpState::CloseWait,
            8 => TcpState::Closing,
            9 => TcpState::LastAck,
            10 => TcpState::TimeWait,
            _ => TcpState::Closed,
        }
    }

    /// Set state
    fn set_state(&self, state: TcpState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Handle incoming SYN segment
    pub fn on_syn(&self, seq: u32, _now_ms: u64) -> SegmentResult {
        let state = self.get_state();
        match state {
            TcpState::Listen => {
                self.irs.store(seq, Ordering::Release);
                self.rcv_nxt.store(seq.wrapping_add(1), Ordering::Release);
                self.set_state(TcpState::SynReceived);
                SegmentResult::SendSynAck
            }
            TcpState::SynSent => {
                self.irs.store(seq, Ordering::Release);
                self.rcv_nxt.store(seq.wrapping_add(1), Ordering::Release);
                self.set_state(TcpState::SynReceived);
                SegmentResult::SendSynAck
            }
            _ => SegmentResult::Invalid,
        }
    }

    /// Handle incoming SYN-ACK segment (client side)
    pub fn on_syn_ack(&self, seq: u32, ack: u32, _now_ms: u64) -> SegmentResult {
        let state = self.get_state();
        match state {
            TcpState::SynSent => {
                let expected_ack = self.iss.load(Ordering::Acquire).wrapping_add(1);
                if ack != expected_ack {
                    return SegmentResult::Invalid;
                }
                self.irs.store(seq, Ordering::Release);
                self.rcv_nxt.store(seq.wrapping_add(1), Ordering::Release);
                self.snd_una.store(ack, Ordering::Release);
                self.set_state(TcpState::Established);
                SegmentResult::SendAck
            }
            _ => SegmentResult::Invalid,
        }
    }

    /// Handle incoming ACK segment
    pub fn on_ack(&self, ack: u32, _now_ms: u64) -> SegmentResult {
        let state = self.get_state();
        match state {
            TcpState::SynReceived => {
                let expected_ack = self.iss.load(Ordering::Acquire).wrapping_add(1);
                if ack != expected_ack {
                    return SegmentResult::Invalid;
                }
                self.snd_una.store(ack, Ordering::Release);
                self.set_state(TcpState::Established);
                SegmentResult::Established
            }
            TcpState::Established => {
                let snd_una = self.snd_una.load(Ordering::Acquire);
                let snd_nxt = self.snd_nxt.load(Ordering::Acquire);
                let cwnd = self.cwnd.load(Ordering::Acquire);
                let ssthresh = self.ssthresh.load(Ordering::Acquire);
                let mss: u32 = 1460; // Maximum Segment Size for Ethernet

                if ack == snd_una {
                    // Duplicate ACK
                    let last = self.last_ack.load(Ordering::Acquire);
                    if ack == last {
                        let dup_ack = self.dup_ack_count.fetch_add(1, Ordering::Relaxed) + 1;

                        // Fast Retransmit: 3 duplicate ACKs trigger retransmit
                        if dup_ack == 3 {
                            // Fast Retransmit: reduce ssthresh and retransmit
                            let flight_size = snd_nxt.wrapping_sub(snd_una);
                            let new_ssthresh = core::cmp::max(flight_size / 2, 2 * mss);
                            self.ssthresh.store(new_ssthresh, Ordering::Release);
                            // Fast Recovery: set cwnd to ssthresh + 3*MSS
                            self.cwnd.store(new_ssthresh + 3 * mss, Ordering::Release);
                            // Retransmit the missing segment
                            return SegmentResult::Retransmit;
                        } else if dup_ack > 3 {
                            // Fast Recovery: inflate cwnd by 1 MSS per additional dup ACK
                            self.cwnd.store(cwnd + mss, Ordering::Release);
                        }
                    } else {
                        self.dup_ack_count.store(0, Ordering::Relaxed);
                        self.last_ack.store(ack, Ordering::Relaxed);
                    }
                } else if ack > snd_una && ack <= snd_nxt {
                    // New ACK: advance snd_una
                    self.snd_una.store(ack, Ordering::Relaxed);
                    self.dup_ack_count.store(0, Ordering::Relaxed);
                    self.last_ack.store(ack, Ordering::Relaxed);
                    self.timers.cancel_rto();

                    // Congestion control: increase cwnd
                    let dup_ack = self.dup_ack_count.load(Ordering::Acquire);
                    if dup_ack >= 3 {
                        // Exiting Fast Recovery: set cwnd = ssthresh
                        let cur_ssthresh = self.ssthresh.load(Ordering::Acquire);
                        self.cwnd.store(cur_ssthresh, Ordering::Release);
                    } else if cwnd < ssthresh {
                        // Slow Start: increase cwnd by 1 MSS per ACK (exponential growth)
                        self.cwnd.store(cwnd + mss, Ordering::Release);
                    } else {
                        // Congestion Avoidance: increase cwnd by MSS*(MSS/cwnd) per ACK
                        // (additive growth, approximately 1 MSS per RTT)
                        let increment = core::cmp::max(mss * mss / core::cmp::max(cwnd, 1), 1);
                        self.cwnd.store(cwnd + increment, Ordering::Release);
                    }
                }
                SegmentResult::Ok
            }
            TcpState::FinWait1 => {
                let fin_seq = self.snd_nxt.load(Ordering::Acquire);
                if ack == fin_seq.wrapping_add(1) {
                    self.set_state(TcpState::FinWait2);
                    SegmentResult::Ok
                } else {
                    SegmentResult::Ok
                }
            }
            TcpState::Closing => {
                let fin_seq = self.snd_nxt.load(Ordering::Acquire);
                if ack == fin_seq.wrapping_add(1) {
                    self.set_state(TcpState::TimeWait);
                    SegmentResult::Ok
                } else {
                    SegmentResult::Ok
                }
            }
            TcpState::LastAck => {
                let fin_seq = self.snd_nxt.load(Ordering::Acquire);
                if ack == fin_seq.wrapping_add(1) {
                    self.set_state(TcpState::Closed);
                    SegmentResult::Closed
                } else {
                    SegmentResult::Ok
                }
            }
            _ => SegmentResult::Ok,
        }
    }

    /// Handle incoming FIN segment
    pub fn on_fin(&self, seq: u32, _now_ms: u64) -> SegmentResult {
        let state = self.get_state();
        self.rcv_nxt.store(seq.wrapping_add(1), Ordering::Release);
        match state {
            TcpState::Established => {
                self.set_state(TcpState::CloseWait);
                SegmentResult::SendFinAck
            }
            TcpState::FinWait1 => {
                self.set_state(TcpState::Closing);
                SegmentResult::SendAck
            }
            TcpState::FinWait2 => {
                self.set_state(TcpState::TimeWait);
                SegmentResult::SendFinAck
            }
            TcpState::CloseWait => SegmentResult::Ok,
            TcpState::TimeWait => SegmentResult::SendAck,
            _ => SegmentResult::Invalid,
        }
    }

    /// Handle incoming RST segment
    pub fn on_rst(&self, _now_ms: u64) -> SegmentResult {
        let state = self.get_state();
        match state {
            TcpState::SynSent => {
                self.set_state(TcpState::Closed);
                SegmentResult::Closed
            }
            TcpState::SynReceived => {
                self.set_state(TcpState::Closed);
                SegmentResult::Closed
            }
            TcpState::Established
            | TcpState::FinWait1
            | TcpState::FinWait2
            | TcpState::CloseWait => {
                self.set_state(TcpState::Closed);
                SegmentResult::Closed
            }
            TcpState::Closing | TcpState::LastAck | TcpState::TimeWait => {
                self.set_state(TcpState::Closed);
                SegmentResult::Closed
            }
            _ => SegmentResult::Invalid,
        }
    }

    /// Process incoming segment based on current state and flags.
    /// This is the main segment arrival handler per RFC 793.
    pub fn segment_arrives(
        &self,
        header: &TcpHeader,
        _payload_len: usize,
        src_addr: u32,
        dst_addr: u32,
        now_ms: u64,
    ) -> SegmentResult {
        let state = self.get_state();
        let flags = header.flags();
        let seq = header.get_seq();
        let ack = header.get_ack_seq();

        if header.is_rst() {
            if state == TcpState::SynSent {
                if header.is_ack() {
                    return self.on_rst(now_ms);
                }
                return SegmentResult::Ok;
            }
            if state == TcpState::Listen {
                return SegmentResult::Ok;
            }
            return self.on_rst(now_ms);
        }

        if header.is_syn() {
            if state == TcpState::SynSent && header.is_ack() {
                return self.on_syn_ack(seq, ack, now_ms);
            }
            return self.on_syn(seq, now_ms);
        }

        if state == TcpState::Listen {
            return SegmentResult::Ok;
        }

        if state == TcpState::SynSent {
            if header.is_ack() {
                let expected_ack = self.iss.load(Ordering::Acquire).wrapping_add(1);
                if ack != expected_ack {
                    return SegmentResult::SendRst;
                }
                return self.on_syn_ack(seq, ack, now_ms);
            }
            return SegmentResult::Ok;
        }

        let rcv_nxt = self.rcv_nxt.load(Ordering::Acquire);
        let rcv_wnd = self.rcv_wnd.load(Ordering::Acquire);
        if seq != rcv_nxt && (seq < rcv_nxt || seq >= rcv_nxt + rcv_wnd) {
            if header.is_ack() {
                return SegmentResult::SendAck;
            }
            return SegmentResult::Ok;
        }

        if header.is_fin() {
            return self.on_fin(seq, now_ms);
        }

        if header.is_ack() {
            let result = self.on_ack(ack, now_ms);
            if result != SegmentResult::Ok {
                return result;
            }
        }

        if flags & tcp_flags::PSH != 0 || _payload_len > 0 {
            let new_rcv_nxt = rcv_nxt.wrapping_add(_payload_len as u32);
            self.rcv_nxt.store(new_rcv_nxt, Ordering::Release);
        }

        let snd_wnd = header.get_window() as u32;
        self.snd_wnd.store(snd_wnd, Ordering::Release);

        self.last_activity.store(now_ms, Ordering::Release);

        if header.is_ack() {
            SegmentResult::Ok
        } else {
            SegmentResult::SendAck
        }
    }

    /// Update RTT estimation using Jacobson/Karels algorithm
    pub fn update_rtt(&self, measured_rtt_ms: u32) {
        let srtt = self.srtt.load(Ordering::Acquire);
        let rttvar = self.rttvar.load(Ordering::Acquire);

        if srtt == 0 {
            self.srtt.store(measured_rtt_ms, Ordering::Release);
            self.rttvar.store(measured_rtt_ms / 2, Ordering::Release);
        } else {
            let delta = if measured_rtt_ms > srtt {
                measured_rtt_ms - srtt
            } else {
                srtt - measured_rtt_ms
            };
            let new_rttvar = (3 * rttvar + delta) / 4;
            let new_srtt = (7 * srtt + measured_rtt_ms) / 8;
            self.rttvar.store(new_rttvar, Ordering::Release);
            self.srtt.store(new_srtt, Ordering::Release);
        }

        let srtt = self.srtt.load(Ordering::Acquire);
        let rttvar = self.rttvar.load(Ordering::Acquire);
        let rto = (srtt + 4 * rttvar).max(200).min(60000);
        self.rto.store(rto, Ordering::Release);
    }

    /// Process timer events, returns true if connection should be closed
    pub fn process_timers(&self, now_ms: u64, config: &TcpTimerConfig) -> bool {
        let state = self.get_state();

        if state == TcpState::TimeWait {
            if self.timers.timewait_expired(now_ms) {
                self.set_state(TcpState::Closed);
                return true;
            }
            return false;
        }

        if state == TcpState::Established && self.timers.rto_expired(now_ms) {
            let exceeded = self.timers.increment_rtx();
            if exceeded {
                self.set_state(TcpState::Closed);
                return true;
            }
            let rto = self.rto.load(Ordering::Acquire);
            let new_rto = (rto * 2).min(config.rto_max);
            self.rto.store(new_rto, Ordering::Release);
            self.timers.start_rto(new_rto, now_ms);
            let cwnd = self.cwnd.load(Ordering::Acquire);
            let ssthresh = cwnd / 2;
            self.ssthresh.store(ssthresh.max(2 * self.mss as u32), Ordering::Release);
            self.cwnd.store(self.mss as u32, Ordering::Release);
        }

        if state == TcpState::Established && self.timers.keepalive_expired(now_ms) {
            let probes = self.timers.keepalive_probes.fetch_add(1, Ordering::AcqRel);
            if probes >= config.keepalive_probes {
                self.set_state(TcpState::Closed);
                return true;
            }
            self.timers.start_keepalive(config.keepalive_interval as u64, now_ms);
        }

        if state == TcpState::Established && self.timers.delayed_ack_expired(now_ms) {
            // Delayed ACK timeout: must send ACK now
        }

        if self.snd_wnd.load(Ordering::Acquire) == 0 && self.timers.persist_expired(now_ms) {
            // Send zero window probe
            let rto = self.rto.load(Ordering::Acquire);
            let next_rto = (rto * 2).min(config.rto_max);
            self.rto.store(next_rto, Ordering::Release);
            self.timers.start_persist(next_rto, now_ms);
        }

        false
    }
}

/// TCP Control Block (TCB) — comprehensive per-connection state.
///
/// The TCB wraps a `TcpConnection` together with all the data structures
/// needed for reliable, ordered octet stream delivery: retransmit queue,
/// out-of-order segment reassembly buffer, and connection metrics.
pub struct TcpControlBlock {
    /// Core connection state (sequence numbers, timers, congestion state)
    pub conn: TcpConnection,
    /// State machine for validated state transitions
    pub sm: TcpStateMachine,
    /// Retransmission queue (circular buffer of unacknowledged segments)
    pub rtx_queue: [RetransmitEntry; 32],
    /// Number of valid entries in rtx_queue (head index at 0, count used entries)
    pub rtx_queue_head: core::sync::atomic::AtomicU8,
    pub rtx_queue_len: core::sync::atomic::AtomicU8,
    /// Out-of-order segment queue (segments that arrived after a gap)
    pub ooo_queue: [OutOfOrderEntry; 16],
    /// Number of valid entries in ooo_queue
    pub ooo_queue_len: core::sync::atomic::AtomicU8,
    /// Connection creation timestamp (ms)
    pub created_ms: AtomicU64,
    /// Total bytes sent
    pub bytes_sent: AtomicU64,
    /// Total bytes received
    pub bytes_received: AtomicU64,
    /// Whether this TCB is in use
    pub in_use: AtomicU32,
}

impl TcpControlBlock {
    pub fn new(local_addr: u32, local_port: u16, remote_addr: u32, remote_port: u16) -> Self {
        TcpControlBlock {
            conn: TcpConnection::new(local_addr, local_port, remote_addr, remote_port),
            sm: TcpStateMachine::new(TcpState::Closed),
            rtx_queue: [
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(), RetransmitEntry::new(),
                RetransmitEntry::new(), RetransmitEntry::new(),
            ],
            rtx_queue_head: core::sync::atomic::AtomicU8::new(0),
            rtx_queue_len: core::sync::atomic::AtomicU8::new(0),
            ooo_queue: [
                OutOfOrderEntry::new(), OutOfOrderEntry::new(), OutOfOrderEntry::new(),
                OutOfOrderEntry::new(), OutOfOrderEntry::new(), OutOfOrderEntry::new(),
                OutOfOrderEntry::new(), OutOfOrderEntry::new(), OutOfOrderEntry::new(),
                OutOfOrderEntry::new(), OutOfOrderEntry::new(), OutOfOrderEntry::new(),
                OutOfOrderEntry::new(), OutOfOrderEntry::new(), OutOfOrderEntry::new(),
                OutOfOrderEntry::new(),
            ],
            ooo_queue_len: core::sync::atomic::AtomicU8::new(0),
            created_ms: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            in_use: AtomicU32::new(1),
        }
    }

    /// Get current TCP state
    pub fn get_state(&self) -> TcpState {
        self.sm.get_state()
    }

    /// Enqueue a segment to the retransmit queue.
    /// Returns true if enqueued successfully, false if queue is full.
    pub fn enqueue_rtx(&self, entry: RetransmitEntry) -> bool {
        let len = self.rtx_queue_len.load(Ordering::Acquire);
        if len >= 32 {
            pr_warn!("TCP: retransmit queue full, dropping segment seq={}", entry.seq);
            return false;
        }
        // Simple append at the tail
        let head = self.rtx_queue_head.load(Ordering::Acquire) as usize;
        let idx = ((head + len as usize) % 32) as usize;
        // SAFETY: We only write to indices within bounds.
        unsafe {
            let ptr = self.rtx_queue.as_ptr() as *mut RetransmitEntry;
            ptr.add(idx).write(entry);
        }
        self.rtx_queue_len.fetch_add(1, Ordering::AcqRel);
        true
    }

    /// Remove acknowledged segments from the retransmit queue.
    /// Removes all entries with (seq + len) <= ack.
    pub fn ack_rtx_queue(&self, ack: u32) {
        loop {
            let len = self.rtx_queue_len.load(Ordering::Acquire);
            if len == 0 {
                break;
            }
            let head = self.rtx_queue_head.load(Ordering::Acquire) as usize;
            // SAFETY: index is validated by queue length invariant
            let entry = unsafe {
                let ptr = self.rtx_queue.as_ptr() as *const RetransmitEntry;
                ptr.add(head).read()
            };
            let seg_end = entry.seq.wrapping_add(entry.len as u32);
            // Use wrapping comparison: if ack is beyond segment end, it's fully ACKed
            if ack.wrapping_sub(seg_end) < ack.wrapping_sub(entry.seq) || ack == seg_end {
                // Segment fully ACKed, remove from queue
                self.rtx_queue_head.store(((head + 1) % 32) as u8, Ordering::Release);
                self.rtx_queue_len.fetch_sub(1, Ordering::AcqRel);
            } else {
                // This segment is not yet ACKed, stop here (queue is ordered)
                break;
            }
        }
    }

    /// Get the oldest unacknowledged segment from the retransmit queue.
    pub fn oldest_rtx_entry(&self) -> Option<RetransmitEntry> {
        let len = self.rtx_queue_len.load(Ordering::Acquire);
        if len == 0 {
            return None;
        }
        let head = self.rtx_queue_head.load(Ordering::Acquire) as usize;
        // SAFETY: head index is validated by queue length invariant
        let entry = unsafe {
            let ptr = self.rtx_queue.as_ptr() as *const RetransmitEntry;
            ptr.add(head).read()
        };
        Some(entry)
    }

    /// Enqueue an out-of-order segment.
    /// Returns true if enqueued successfully.
    pub fn enqueue_ooo(&self, seq: u32, data: &[u8]) -> bool {
        let len = self.ooo_queue_len.load(Ordering::Acquire);
        if len >= 16 || data.len() > 128 {
            pr_warn!("TCP: OOO queue full or segment too large, dropping seq={}", seq);
            return false;
        }
        let idx = len as usize;
        // SAFETY: index is within bounds
        unsafe {
            let ptr = self.ooo_queue.as_ptr() as *mut OutOfOrderEntry;
            let mut entry = ptr.add(idx);
            (*entry).seq = seq;
            (*entry).len = data.len() as u16;
            let dst = (*entry).data.as_mut_ptr();
            let src = data.as_ptr();
            let copy_len = core::cmp::min(data.len(), 128);
            core::ptr::copy_nonoverlapping(src, dst, copy_len);
        }
        self.ooo_queue_len.fetch_add(1, Ordering::AcqRel);
        true
    }

    /// Flush out-of-order segments that are now in sequence (seq == rcv_nxt).
    /// Returns data that can be delivered to the socket.
    pub fn flush_ooo(&self, rcv_nxt: u32, buf: &mut [u8]) -> usize {
        let mut written: usize = 0;
        // Sort by sequence number (simple insertion sort for small queue)
        // Then flush contiguous segments.
        let n = self.ooo_queue_len.load(Ordering::Acquire) as usize;
        let mut next_seq = rcv_nxt;

        // Dequeue and deliver segments in order
        loop {
            let n = self.ooo_queue_len.load(Ordering::Acquire) as usize;
            if n == 0 {
                break;
            }
            // Find segment matching next_seq
            let mut found = false;
            for i in 0..n {
                // SAFETY: index is within bounds
                let raw = unsafe {
                    let ptr = self.ooo_queue.as_ptr() as *const OutOfOrderEntry;
                    ptr.add(i).read()
                };
                if raw.seq == next_seq {
                    let copy_len = core::cmp::min(raw.len as usize, buf.len() - written);
                    unsafe {
                        let src = raw.data.as_ptr();
                        let dst = buf.as_mut_ptr().add(written);
                        core::ptr::copy_nonoverlapping(src, dst, copy_len);
                    }
                    written += copy_len;
                    next_seq = next_seq.wrapping_add(raw.len as u32);
                    // Remove this entry by shifting remaining entries down
                    for j in i..(n - 1) {
                        unsafe {
                            let ptr = self.ooo_queue.as_ptr() as *mut OutOfOrderEntry;
                            let next = ptr.add(j + 1).read();
                            ptr.add(j).write(next);
                        }
                    }
                    self.ooo_queue_len.fetch_sub(1, Ordering::AcqRel);
                    found = true;
                    break;
                }
            }
            if !found {
                break; // Gap still exists, stop flushing
            }
        }
        written
    }
}

/// TCP Options
#[repr(C)]
pub struct TcpOptions {
    /// Maximum segment size
    pub mss: u16,
    /// Window scale
    pub wscale: u8,
    /// SACK permitted
    pub sack_perm: bool,
    /// Timestamps
    pub timestamps: bool,
}

impl Default for TcpOptions {
    fn default() -> Self {
        TcpOptions {
            mss: 1460,
            wscale: 0,
            sack_perm: false,
            timestamps: false,
        }
    }
}

/// TCP Statistics
pub struct TcpStats {
    /// Active opens
    pub active_opens: AtomicU64,
    /// Passive opens
    pub passive_opens: AtomicU64,
    /// Attempt fails
    pub attempt_fails: AtomicU64,
    /// Estab resets
    pub estab_resets: AtomicU64,
    /// Current established
    pub curr_estab: AtomicU32,
    /// Segments received
    pub in_segs: AtomicU64,
    /// Segments sent
    pub out_segs: AtomicU64,
    /// Retransmitted segments
    pub retrans_segs: AtomicU64,
    /// Input errors
    pub in_errs: AtomicU64,
    /// Output resets
    pub out_rsts: AtomicU64,
}

impl TcpStats {
    pub const fn new() -> Self {
        TcpStats {
            active_opens: AtomicU64::new(0),
            passive_opens: AtomicU64::new(0),
            attempt_fails: AtomicU64::new(0),
            estab_resets: AtomicU64::new(0),
            curr_estab: AtomicU32::new(0),
            in_segs: AtomicU64::new(0),
            out_segs: AtomicU64::new(0),
            retrans_segs: AtomicU64::new(0),
            in_errs: AtomicU64::new(0),
            out_rsts: AtomicU64::new(0),
        }
    }
}

/// TCP Manager
pub struct TcpManager {
    /// Statistics
    pub stats: TcpStats,
    /// Maximum segment size
    pub default_mss: u16,
    /// Time wait timeout (in seconds)
    pub time_wait_timeout: u32,
    /// Timer configuration
    pub timer_config: TcpTimerConfig,
}

impl TcpManager {
    pub const fn new() -> Self {
        TcpManager {
            stats: TcpStats::new(),
            default_mss: 1460,
            time_wait_timeout: 60,
            timer_config: TcpTimerConfig::default(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("TCP initialized");
    }

    /// Process received segment
    pub fn receive(&mut self, data: &[u8]) -> i32 {
        self.stats.in_segs.fetch_add(1, Ordering::AcqRel);

        if data.len() < 20 {
            log_warn!("TCP segment too short");
            return Errno::Eperm.to_ret_i32();
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq_num = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let flags = data[13];
        let window = u16::from_be_bytes([data[14], data[15]]);

        log_debug!(
            "TCP receive: src={}, dst={}, seq={}, ack={}, flags={:#x}, window={}",
            src_port, dst_port, seq_num, ack_num, flags, window
        );

        0
    }

    /// Send segment
    pub fn send(&mut self, conn: &TcpConnection, data: &[u8]) -> i32 {
        self.stats.out_segs.fetch_add(1, Ordering::AcqRel);

        let mut header = [0u8; 20];
        header[0..2].copy_from_slice(&conn.local_port.to_be_bytes());
        header[2..4].copy_from_slice(&conn.remote_port.to_be_bytes());
        header[4..8].copy_from_slice(&conn.send_nxt.to_be_bytes());
        header[8..12].copy_from_slice(&conn.recv_nxt.to_be_bytes());
        header[12] = 5 << 4;
        header[13] = 0x10;
        header[14..16].copy_from_slice(&conn.window.to_be_bytes());

        let checksum = self.calculate_checksum(&header, data);
        header[16..18].copy_from_slice(&checksum.to_be_bytes());

        log_debug!(
            "TCP send: len={}, seq={}, ack={}, window={}",
            data.len(), conn.send_nxt, conn.recv_nxt, conn.window
        );

        0
    }

    /// Active open (connect) — send SYN segment
    pub fn connect(&mut self, conn: &mut TcpConnection) -> i32 {
        self.stats.active_opens.fetch_add(1, Ordering::AcqRel);

        log_debug!(
            "TCP connect: {}:{} -> {}:{}",
            conn.local_addr, conn.local_port, conn.remote_addr, conn.remote_port
        );

        let iss = conn.iss.load(Ordering::Acquire);
        conn.send_nxt = iss.wrapping_add(1);
        conn.snd_una.store(iss, Ordering::Release);
        conn.snd_nxt.store(conn.send_nxt, Ordering::Release);
        conn.rcv_nxt.store(0, Ordering::Release);

        let mut syn_header = [0u8; 24];
        syn_header[0..2].copy_from_slice(&conn.local_port.to_be_bytes());
        syn_header[2..4].copy_from_slice(&conn.remote_port.to_be_bytes());
        syn_header[4..8].copy_from_slice(&iss.to_be_bytes());
        syn_header[8..12].copy_from_slice(&0u32.to_be_bytes());
        syn_header[12] = 6 << 4;
        syn_header[13] = (tcp_flags::SYN) as u8;
        syn_header[14..16].copy_from_slice(&conn.window.to_be_bytes());
        syn_header[20] = 2;
        syn_header[21] = 4;
        syn_header[22..24].copy_from_slice(&conn.mss.to_be_bytes());

        let checksum = self.calculate_checksum_with_pseudo(
            &syn_header,
            &[],
            conn.local_addr,
            conn.remote_addr,
        );
        syn_header[16..18].copy_from_slice(&checksum.to_be_bytes());

        self.stats.out_segs.fetch_add(1, Ordering::AcqRel);
        self.ip_send(conn.local_addr, conn.remote_addr, &syn_header, &[]);

        conn.state.store(TcpState::SynSent as u32, Ordering::Release);
        conn.timers.start_rto(conn.rto.load(Ordering::Acquire), 0);

        log_debug!(
            "SYN sent: iss={}, mss={}, waiting for SYN-ACK",
            iss, conn.mss
        );

        0
    }

    /// Send raw bytes to IP layer (stub for IP integration)
    fn ip_send(&self, _src: u32, _dst: u32, _header: &[u8], _data: &[u8]) {
        // SAFETY: placeholder for IP layer output; will be replaced by ip_output() when IP stack is integrated
    }

    /// Passive open (listen)
    pub fn listen(&mut self, conn: &mut TcpConnection) -> i32 {
        conn.state.store(TcpState::Listen as u32, Ordering::Release);
        log_debug!("TCP listen on {}:{}", conn.local_addr, conn.local_port);
        0
    }

    /// Accept connection
    pub fn accept(&mut self) -> i32 {
        self.stats.passive_opens.fetch_add(1, Ordering::AcqRel);
        log_debug!("TCP accept: waiting for connection");
        0
    }

    /// Close connection
    pub fn close(&mut self, conn: &mut TcpConnection) -> i32 {
        let state = conn.get_state();

        match state {
            TcpState::Established => {
                conn.state.store(TcpState::FinWait1 as u32, Ordering::Release);
                log_debug!("FIN sent, waiting for ACK");
            }
            TcpState::CloseWait => {
                conn.state.store(TcpState::LastAck as u32, Ordering::Release);
                log_debug!("FIN sent from CLOSE_WAIT, waiting for ACK");
            }
            TcpState::Listen => {
                conn.state.store(TcpState::Closed as u32, Ordering::Release);
            }
            TcpState::SynSent => {
                conn.state.store(TcpState::Closed as u32, Ordering::Release);
            }
            _ => {
                log_warn!("TCP close in unexpected state: {:?}", state);
                return Errno::Eperm.to_ret_i32();
            }
        }

        0
    }

    /// Calculate TCP checksum (with pseudo-header)
    pub fn calculate_checksum_with_pseudo(
        &self,
        header: &[u8],
        data: &[u8],
        src_addr: u32,
        dst_addr: u32,
    ) -> u16 {
        let mut sum: u32 = 0;

        // Pseudo-header
        sum += ((src_addr >> 16) & 0xFFFF) as u32;
        sum += (src_addr & 0xFFFF) as u32;
        sum += ((dst_addr >> 16) & 0xFFFF) as u32;
        sum += (dst_addr & 0xFFFF) as u32;
        sum += 6; // TCP protocol
        let total_len = (header.len() + data.len()) as u32;
        sum += (total_len >> 16) & 0xFFFF;
        sum += total_len & 0xFFFF;

        // Header
        for chunk in header.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            } else {
                sum += (chunk[0] as u32) << 8;
            }
        }

        // Data
        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            } else {
                sum += (chunk[0] as u32) << 8;
            }
        }

        // Fold 32-bit sum to 16 bits
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }

    /// Calculate TCP checksum (without pseudo-header, for compatibility)
    fn calculate_checksum(&self, header: &[u8], data: &[u8]) -> u16 {
        let mut sum: u32 = 0;

        for chunk in header.chunks(2) {
            if chunk.len() == 2 {
                sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
            }
        }

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
}

/// Global TCP manager
static TCP_MANAGER: core::sync::OnceLock<TcpManager> = core::sync::OnceLock::new();

/// Get TCP manager
pub fn tcp_manager() -> &'static TcpManager {
    TCP_MANAGER.get_or_init(TcpManager::new)
}

pub fn init_tcp_manager() -> &'static TcpManager {
    TCP_MANAGER.get_or_init(TcpManager::new)
}

/// Initialize TCP
pub fn init_tcp() {
    let mgr = tcp_manager();
    mgr.init();
}
