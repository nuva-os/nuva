/*
 * Nuva OS - TCP Connection Establishment
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

//! TCP Connection Establishment (RFC 793 Three-Way Handshake)
/*!*/
//! Implements active open, passive open, and simultaneous open.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use super::state::{TcpState, TcpStateMachine, TcpEvent, StateTransition};

/// TCP Flags
pub mod flags {
    pub const FIN: u16 = 0x0001;
    pub const SYN: u16 = 0x0002;
    pub const RST: u16 = 0x0004;
    pub const PSH: u16 = 0x0008;
    pub const ACK: u16 = 0x0010;
    pub const URG: u16 = 0x0020;
}

/// Maximum Segment Size (default)
pub const DEFAULT_MSS: u32 = 1460;

/// Default window size
pub const DEFAULT_WINDOW: u32 = 65535;

/// Maximum SYN retransmissions
pub const MAX_SYN_RETRANSMITS: u32 = 5;

/// Initial sequence number generator
pub struct IsnGenerator {
    counter: AtomicU32,
    boot_time: AtomicU64,
}

impl IsnGenerator {
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            boot_time: AtomicU64::new(0),
        }
    }

    /// Generate initial sequence number
    pub fn generate(&self, timestamp: u64) -> u32 {
        // ISN = M + F(local_ip, local_port, remote_ip, remote_port)
        // Simplified: ISN = timestamp-based
        let boot_time = self.boot_time.load(Ordering::Relaxed);
        let elapsed = timestamp.saturating_sub(boot_time);
        let isn = (elapsed as u32).wrapping_add(self.counter.fetch_add(1, Ordering::Relaxed));
        isn
    }

    /// Initialize with boot time
    pub fn init(&self, boot_time: u64) {
        self.boot_time.store(boot_time, Ordering::Relaxed);
    }
}

/// TCP Connection Control Block (TCB)
pub struct TcpControlBlock {
    /// State machine
    pub state_machine: TcpStateMachine,

    /// Local IP address
    pub local_ip: u32,

    /// Local port
    pub local_port: u16,

    /// Remote IP address
    pub remote_ip: u32,

    /// Remote port
    pub remote_port: u16,

    /// Send sequence number (SND.NXT)
    pub snd_nxt: AtomicU32,

    /// Send initial sequence number (SND.ISS)
    pub snd_iss: AtomicU32,

    /// Send unacknowledged (SND.UNA)
    pub snd_una: AtomicU32,

    /// Send window (SND.WND)
    pub snd_wnd: AtomicU32,

    /// Receive sequence number (RCV.NXT)
    pub rcv_nxt: AtomicU32,

    /// Receive initial sequence number (RCV.IRS)
    pub rcv_irs: AtomicU32,

    /// Receive window (RCV.WND)
    pub rcv_wnd: AtomicU32,

    /// Maximum segment size
    pub mss: AtomicU32,

    /// Window scale factor
    pub wscale: AtomicU32,

    /// Timestamps enabled
    pub timestamps_enabled: AtomicU32,

    /// SACK permitted
    pub sack_permitted: AtomicU32,

    /// Connection established time
    pub established_time: AtomicU64,
}

impl TcpControlBlock {
    /// Create new TCB
    pub fn new(local_ip: u32, local_port: u16) -> Self {
        Self {
            state_machine: TcpStateMachine::new(),
            local_ip,
            local_port,
            remote_ip: 0,
            remote_port: 0,
            snd_nxt: AtomicU32::new(0),
            snd_iss: AtomicU32::new(0),
            snd_una: AtomicU32::new(0),
            snd_wnd: AtomicU32::new(DEFAULT_WINDOW),
            rcv_nxt: AtomicU32::new(0),
            rcv_irs: AtomicU32::new(0),
            rcv_wnd: AtomicU32::new(DEFAULT_WINDOW),
            mss: AtomicU32::new(DEFAULT_MSS),
            wscale: AtomicU32::new(0),
            timestamps_enabled: AtomicU32::new(0),
            sack_permitted: AtomicU32::new(0),
            established_time: AtomicU64::new(0),
        }
    }

    /// Get current state
    pub fn get_state(&self) -> TcpState {
        self.state_machine.get_state()
    }

    /// Active open (client)
    pub fn active_open(
        &mut self,
        remote_ip: u32,
        remote_port: u16,
        isn: u32,
        timestamp: u64,
    ) -> Result<ActiveOpenResult, TcpConnectError> {
        if self.get_state() != TcpState::Closed {
            return Err(TcpConnectError::AlreadyConnected);
        }

        self.remote_ip = remote_ip;
        self.remote_port = remote_port;

        // Set initial sequence number
        self.snd_iss.store(isn, Ordering::Relaxed);
        self.snd_nxt.store(isn, Ordering::Relaxed);
        self.snd_una.store(isn, Ordering::Relaxed);

        // Process active open event
        let result = self.state_machine.process_event(TcpEvent::ActiveOpen, timestamp);

        match result {
            StateTransition::Ok(TcpState::SynSent) => {
                Ok(ActiveOpenResult::SynSent {
                    seq: isn,
                    mss: self.mss.load(Ordering::Relaxed),
                    wscale: self.wscale.load(Ordering::Relaxed),
                })
            }
            _ => Err(TcpConnectError::StateError),
        }
    }

    /// Passive open (server)
    pub fn passive_open(&mut self, timestamp: u64) -> Result<(), TcpConnectError> {
        if self.get_state() != TcpState::Closed {
            return Err(TcpConnectError::AlreadyConnected);
        }

        let result = self.state_machine.process_event(TcpEvent::PassiveOpen, timestamp);

        match result {
            StateTransition::Ok(TcpState::Listen) => Ok(()),
            _ => Err(TcpConnectError::StateError),
        }
    }

    /// Handle incoming SYN (server side)
    pub fn handle_syn(
        &mut self,
        remote_ip: u32,
        remote_port: u16,
        seq: u32,
        mss: Option<u32>,
        wscale: Option<u32>,
        timestamp: u64,
    ) -> Result<SynReceivedResult, TcpConnectError> {
        let state = self.get_state();
        if state != TcpState::Listen {
            return Err(TcpConnectError::InvalidState);
        }

        self.remote_ip = remote_ip;
        self.remote_port = remote_port;

        // Set receive initial sequence number
        self.rcv_irs.store(seq, Ordering::Relaxed);
        self.rcv_nxt.store(seq.wrapping_add(1), Ordering::Relaxed);

        // Update options
        if let Some(m) = mss {
            self.mss.store(m.min(DEFAULT_MSS), Ordering::Relaxed);
        }
        if let Some(w) = wscale {
            self.wscale.store(w.min(14), Ordering::Relaxed);
        }

        // Generate ISN for SYN-ACK
        let isn = self.snd_iss.load(Ordering::Relaxed);
        self.snd_nxt.store(isn, Ordering::Relaxed);

        // Process SYN event
        let result = self.state_machine.process_event(TcpEvent::RcvSyn, timestamp);

        match result {
            StateTransition::SendResponse { new_state, send_syn, send_ack, .. } => {
                assert_eq!(new_state, TcpState::SynReceived);
                assert!(send_syn && send_ack);

                Ok(SynReceivedResult {
                    seq: isn,
                    ack: seq.wrapping_add(1),
                    mss: self.mss.load(Ordering::Relaxed),
                    wscale: self.wscale.load(Ordering::Relaxed),
                })
            }
            _ => Err(TcpConnectError::StateError),
        }
    }

    /// Handle incoming SYN-ACK (client side)
    pub fn handle_syn_ack(
        &mut self,
        seq: u32,
        ack: u32,
        mss: Option<u32>,
        wscale: Option<u32>,
        timestamp: u64,
    ) -> Result<SynAckReceivedResult, TcpConnectError> {
        let state = self.get_state();
        if state != TcpState::SynSent {
            return Err(TcpConnectError::InvalidState);
        }

        // Verify ACK
        let expected_ack = self.snd_iss.load(Ordering::Relaxed).wrapping_add(1);
        if ack != expected_ack {
            return Err(TcpConnectError::InvalidAck);
        }

        // Set receive initial sequence number
        self.rcv_irs.store(seq, Ordering::Relaxed);
        self.rcv_nxt.store(seq.wrapping_add(1), Ordering::Relaxed);

        // Update SND.UNA
        self.snd_una.store(ack, Ordering::Relaxed);

        // Update options
        if let Some(m) = mss {
            self.mss.store(m.min(DEFAULT_MSS), Ordering::Relaxed);
        }
        if let Some(w) = wscale {
            self.wscale.store(w.min(14), Ordering::Relaxed);
        }

        // Process SYN-ACK event
        let result = self.state_machine.process_event(TcpEvent::RcvSynAck, timestamp);

        match result {
            StateTransition::SendResponse { new_state, send_ack, .. } => {
                assert_eq!(new_state, TcpState::Established);
                assert!(send_ack);

                self.established_time.store(timestamp, Ordering::Relaxed);

                Ok(SynAckReceivedResult {
                    seq: self.snd_nxt.load(Ordering::Relaxed),
                    ack: seq.wrapping_add(1),
                })
            }
            _ => Err(TcpConnectError::StateError),
        }
    }

    /// Handle incoming ACK (server side, completing handshake)
    pub fn handle_ack(
        &mut self,
        seq: u32,
        ack: u32,
        timestamp: u64,
    ) -> Result<(), TcpConnectError> {
        let state = self.get_state();
        if state != TcpState::SynReceived {
            return Err(TcpConnectError::InvalidState);
        }

        // Verify ACK
        let expected_ack = self.snd_iss.load(Ordering::Relaxed).wrapping_add(1);
        if ack != expected_ack {
            return Err(TcpConnectError::InvalidAck);
        }

        // Update SND.UNA
        self.snd_una.store(ack, Ordering::Relaxed);

        // Update RCV.NXT
        self.rcv_nxt.store(seq, Ordering::Relaxed);

        // Process ACK event
        let result = self.state_machine.process_event(TcpEvent::RcvAck, timestamp);

        match result {
            StateTransition::Ok(TcpState::Established) => {
                self.established_time.store(timestamp, Ordering::Relaxed);
                Ok(())
            }
            _ => Err(TcpConnectError::StateError),
        }
    }

    /// Handle simultaneous open
    pub fn handle_simultaneous_syn(
        &mut self,
        remote_ip: u32,
        remote_port: u16,
        seq: u32,
        timestamp: u64,
    ) -> Result<SimultaneousOpenResult, TcpConnectError> {
        let state = self.get_state();
        if state != TcpState::SynSent {
            return Err(TcpConnectError::InvalidState);
        }

        self.remote_ip = remote_ip;
        self.remote_port = remote_port;

        // Set receive initial sequence number
        self.rcv_irs.store(seq, Ordering::Relaxed);
        self.rcv_nxt.store(seq.wrapping_add(1), Ordering::Relaxed);

        // Process SYN event (simultaneous open)
        let result = self.state_machine.process_event(TcpEvent::RcvSyn, timestamp);

        match result {
            StateTransition::SendResponse { new_state, send_ack, .. } => {
                assert_eq!(new_state, TcpState::SynReceived);
                assert!(send_ack);

                Ok(SimultaneousOpenResult {
                    seq: self.snd_nxt.load(Ordering::Relaxed),
                    ack: seq.wrapping_add(1),
                })
            }
            _ => Err(TcpConnectError::StateError),
        }
    }

    /// Handle RST
    pub fn handle_rst(&mut self, timestamp: u64) {
        self.state_machine.process_event(TcpEvent::RcvRst, timestamp);
    }

    /// Check if connection is established
    pub fn is_established(&self) -> bool {
        self.get_state() == TcpState::Established
    }
}

/// Active open result
#[derive(Debug, Clone, Copy)]
pub struct ActiveOpenResult {
    pub seq: u32,
    pub mss: u32,
    pub wscale: u32,
}

/// SYN received result
#[derive(Debug, Clone, Copy)]
pub struct SynReceivedResult {
    pub seq: u32,
    pub ack: u32,
    pub mss: u32,
    pub wscale: u32,
}

/// SYN-ACK received result
#[derive(Debug, Clone, Copy)]
pub struct SynAckReceivedResult {
    pub seq: u32,
    pub ack: u32,
}

/// Simultaneous open result
#[derive(Debug, Clone, Copy)]
pub struct SimultaneousOpenResult {
    pub seq: u32,
    pub ack: u32,
}

/// TCP connection error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpConnectError {
    /// Already connected
    AlreadyConnected,

    /// Invalid state for operation
    InvalidState,

    /// Invalid ACK
    InvalidAck,

    /// State machine error
    StateError,

    /// Connection refused
    ConnectionRefused,

    /// Timeout
    Timeout,
}

/// TCP listener
pub struct TcpListener {
    local_ip: u32,
    local_port: u16,
    backlog: u32,
    pending_connections: [Option<TcpControlBlock>; 32],
    pending_count: AtomicU32,
}

impl TcpListener {
    pub fn new(local_ip: u32, local_port: u16, backlog: u32) -> Self {
        Self {
            local_ip,
            local_port,
            backlog: backlog.min(32),
            pending_connections: [None; 32],
            pending_count: AtomicU32::new(0),
        }
    }

    /// Start listening
    pub fn listen(&mut self, timestamp: u64) -> Result<(), TcpConnectError> {
        // Create TCB for listening
        let mut tcb = TcpControlBlock::new(self.local_ip, self.local_port);
        tcb.passive_open(timestamp)?;

        // Store as first pending connection
        self.pending_connections[0] = Some(tcb);
        self.pending_count.store(1, Ordering::Relaxed);

        Ok(())
    }

    /// Accept incoming connection
    pub fn accept(&mut self) -> Option<TcpControlBlock> {
        for i in 0..32 {
            if let Some(ref tcb) = self.pending_connections[i] {
                if tcb.is_established() {
                    return self.pending_connections[i].take();
                }
            }
        }
        None
    }

    /// Handle incoming SYN
    pub fn handle_syn(
        &mut self,
        remote_ip: u32,
        remote_port: u16,
        seq: u32,
        mss: Option<u32>,
        wscale: Option<u32>,
        timestamp: u64,
    ) -> Option<SynReceivedResult> {
        let count = self.pending_count.load(Ordering::Relaxed);
        if count >= self.backlog {
            return None; // Backlog full
        }

        // Find free slot
        for i in 0..32 {
            if self.pending_connections[i].is_none() {
                let mut tcb = TcpControlBlock::new(self.local_ip, self.local_port);
                if tcb.passive_open(timestamp).is_ok() {
                    if let Ok(result) = tcb.handle_syn(remote_ip, remote_port, seq, mss, wscale, timestamp) {
                        self.pending_connections[i] = Some(tcb);
                        self.pending_count.fetch_add(1, Ordering::Relaxed);
                        return Some(result);
                    }
                }
                break;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isn_generator() {
        let gen = IsnGenerator::new();
        gen.init(1000);

        let isn1 = gen.generate(2000);
        let isn2 = gen.generate(2001);

        assert_ne!(isn1, isn2);
    }

    #[test]
    fn test_tcb_new() {
        let tcb = TcpControlBlock::new(0x0A000001, 8080);
        assert_eq!(tcb.local_ip, 0x0A000001);
        assert_eq!(tcb.local_port, 8080);
        assert_eq!(tcb.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_tcb_active_open() {
        let mut tcb = TcpControlBlock::new(0x0A000001, 12345);

        let result = tcb.active_open(0xC0A80001, 80, 1000, 0);
        assert!(result.is_ok());
        assert_eq!(tcb.get_state(), TcpState::SynSent);
        assert_eq!(tcb.remote_ip, 0xC0A80001);
        assert_eq!(tcb.remote_port, 80);
    }

    #[test]
    fn test_tcb_passive_open() {
        let mut tcb = TcpControlBlock::new(0x0A000001, 80);

        let result = tcb.passive_open(0);
        assert!(result.is_ok());
        assert_eq!(tcb.get_state(), TcpState::Listen);
    }

    #[test]
    fn test_tcb_three_way_handshake_server() {
        let mut tcb = TcpControlBlock::new(0x0A000001, 80);

        // Passive open
        tcb.passive_open(0).unwrap();
        assert_eq!(tcb.get_state(), TcpState::Listen);

        // Receive SYN
        let result = tcb.handle_syn(0xC0A80001, 12345, 1000, Some(1460), None, 1);
        assert!(result.is_ok());
        assert_eq!(tcb.get_state(), TcpState::SynReceived);

        // Receive ACK
        let result = tcb.handle_ack(1001, tcb.snd_iss.load(Ordering::Relaxed) + 1, 2);
        assert!(result.is_ok());
        assert_eq!(tcb.get_state(), TcpState::Established);
    }

    #[test]
    fn test_tcb_three_way_handshake_client() {
        let mut tcb = TcpControlBlock::new(0x0A000001, 12345);

        // Active open
        tcb.active_open(0xC0A80001, 80, 1000, 0).unwrap();
        assert_eq!(tcb.get_state(), TcpState::SynSent);

        // Receive SYN-ACK
        let result = tcb.handle_syn_ack(2000, 1001, Some(1460), None, 1);
        assert!(result.is_ok());
        assert_eq!(tcb.get_state(), TcpState::Established);
    }

    #[test]
    fn test_tcp_listener() {
        let mut listener = TcpListener::new(0x0A000001, 80, 10);

        // Start listening
        listener.listen(0).unwrap();

        // Handle SYN
        let result = listener.handle_syn(0xC0A80001, 12345, 1000, None, None, 1);
        assert!(result.is_some());
    }

    #[test]
    fn test_tcb_rst() {
        let mut tcb = TcpControlBlock::new(0x0A000001, 12345);

        // Establish connection
        tcb.active_open(0xC0A80001, 80, 1000, 0).unwrap();
        tcb.handle_syn_ack(2000, 1001, None, None, 1).unwrap();
        assert_eq!(tcb.get_state(), TcpState::Established);

        // Receive RST
        tcb.handle_rst(2);
        assert_eq!(tcb.get_state(), TcpState::Closed);
    }
}
