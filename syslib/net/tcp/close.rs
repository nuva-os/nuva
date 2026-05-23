/*
 * Nuva OS - TCP Connection Termination
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

//! TCP Connection Termination
/*!*/
//! Implements active close, passive close, and simultaneous close.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use super::state::{TcpState, TcpStateMachine, TcpEvent, StateTransition};

/// Maximum FIN retransmissions
pub const MAX_FIN_RETRANSMITS: u32 = 5;

/// 2MSL timeout (60 seconds)
pub const TWO_MSL_TIMEOUT: u64 = 60_000;

/// FIN wait timeout (10 minutes)
pub const FIN_WAIT_TIMEOUT: u64 = 600_000;

/// Close result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseResult {
    /// FIN sent, waiting for ACK
    FinSent { seq: u32 },

    /// Connection closed immediately
    Closed,

    /// Already closing
    AlreadyClosing,

    /// Invalid state
    InvalidState,
}

/// FIN received result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinReceivedResult {
    /// Send ACK, enter CLOSE_WAIT
    CloseWait { ack: u32 },

    /// Send ACK, enter TIME_WAIT
    TimeWait { ack: u32 },

    /// Send ACK, enter CLOSING
    Closing { ack: u32 },

    /// Invalid state
    InvalidState,
}

/// ACK received result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckReceivedResult {
    /// Enter FINWait2
    FinWait2,

    /// Enter TimeWait
    TimeWait,

    /// Enter Closed
    Closed,

    /// No state change
    NoChange,
}

/// TCP connection closer
pub struct TcpCloser {
    state_machine: TcpStateMachine,

    /// FIN sequence number
    fin_seq: AtomicU32,

    /// FIN retransmit count
    fin_retransmits: AtomicU32,

    /// Close initiated time
    close_time: AtomicU64,

    /// FIN received time
    fin_rcv_time: AtomicU64,

    /// 2MSL expiration time
    tw_expire_time: AtomicU64,
}

impl TcpCloser {
    pub const fn new() -> Self {
        Self {
            state_machine: TcpStateMachine::new(),
            fin_seq: AtomicU32::new(0),
            fin_retransmits: AtomicU32::new(0),
            close_time: AtomicU64::new(0),
            fin_rcv_time: AtomicU64::new(0),
            tw_expire_time: AtomicU64::new(0),
        }
    }

    /// Get current state
    pub fn get_state(&self) -> TcpState {
        self.state_machine.get_state()
    }

    /// Initiate active close
    pub fn active_close(
        &self,
        snd_nxt: u32,
        timestamp: u64,
    ) -> CloseResult {
        let state = self.get_state();

        if state != TcpState::Established {
            return CloseResult::InvalidState;
        }

        // Store FIN sequence number
        self.fin_seq.store(snd_nxt, Ordering::Relaxed);
        self.close_time.store(timestamp, Ordering::Relaxed);

        // Process close event
        let result = self.state_machine.process_event(TcpEvent::Close, timestamp);

        match result {
            StateTransition::SendResponse { new_state, send_fin, .. } => {
                assert_eq!(new_state, TcpState::FinWait1);
                assert!(send_fin);

                CloseResult::FinSent { seq: snd_nxt }
            }
            _ => CloseResult::InvalidState,
        }
    }

    /// Handle received FIN (passive close)
    pub fn handle_fin(
        &self,
        seq: u32,
        rcv_nxt: u32,
        timestamp: u64,
    ) -> FinReceivedResult {
        let state = self.get_state();

        // Store FIN received time
        self.fin_rcv_time.store(timestamp, Ordering::Relaxed);

        match state {
            TcpState::Established => {
                // Passive close path
                let result = self.state_machine.process_event(TcpEvent::RcvFin, timestamp);

                match result {
                    StateTransition::SendResponse { new_state, send_ack, .. } => {
                        assert_eq!(new_state, TcpState::CloseWait);
                        assert!(send_ack);

                        FinReceivedResult::CloseWait {
                            ack: seq.wrapping_add(1),
                        }
                    }
                    _ => FinReceivedResult::InvalidState,
                }
            }
            TcpState::FinWait1 => {
                // Simultaneous close
                let result = self.state_machine.process_event(TcpEvent::RcvFin, timestamp);

                match result {
                    StateTransition::SendResponse { new_state, send_ack, .. } => {
                        assert_eq!(new_state, TcpState::Closing);
                        assert!(send_ack);

                        FinReceivedResult::Closing {
                            ack: seq.wrapping_add(1),
                        }
                    }
                    _ => FinReceivedResult::InvalidState,
                }
            }
            TcpState::FinWait2 => {
                // Normal close completion
                let result = self.state_machine.process_event(TcpEvent::RcvFin, timestamp);

                match result {
                    StateTransition::SendResponse { new_state, send_ack, .. } => {
                        assert_eq!(new_state, TcpState::TimeWait);
                        assert!(send_ack);

                        // Set 2MSL expiration
                        self.tw_expire_time.store(timestamp + TWO_MSL_TIMEOUT, Ordering::Relaxed);

                        FinReceivedResult::TimeWait {
                            ack: seq.wrapping_add(1),
                        }
                    }
                    _ => FinReceivedResult::InvalidState,
                }
            }
            _ => FinReceivedResult::InvalidState,
        }
    }

    /// Handle received ACK
    pub fn handle_ack(
        &self,
        ack: u32,
        timestamp: u64,
    ) -> AckReceivedResult {
        let state = self.get_state();
        let fin_seq = self.fin_seq.load(Ordering::Relaxed);

        // Verify ACK acknowledges our FIN
        if ack != fin_seq.wrapping_add(1) {
            return AckReceivedResult::NoChange;
        }

        match state {
            TcpState::FinWait1 => {
                // Check if we also received FIN
                let result = self.state_machine.process_event(TcpEvent::RcvAck, timestamp);

                match result {
                    StateTransition::Ok(TcpState::FinWait2) => AckReceivedResult::FinWait2,
                    _ => AckReceivedResult::NoChange,
                }
            }
            TcpState::Closing => {
                let result = self.state_machine.process_event(TcpEvent::RcvAck, timestamp);

                match result {
                    StateTransition::Ok(TcpState::TimeWait) => {
                        self.tw_expire_time.store(timestamp + TWO_MSL_TIMEOUT, Ordering::Relaxed);
                        AckReceivedResult::TimeWait
                    }
                    _ => AckReceivedResult::NoChange,
                }
            }
            TcpState::LastAck => {
                let result = self.state_machine.process_event(TcpEvent::RcvAck, timestamp);

                match result {
                    StateTransition::Ok(TcpState::Closed) => AckReceivedResult::Closed,
                    _ => AckReceivedResult::NoChange,
                }
            }
            _ => AckReceivedResult::NoChange,
        }
    }

    /// Send FIN from CLOSE_WAIT state
    pub fn send_fin(
        &self,
        snd_nxt: u32,
        timestamp: u64,
    ) -> CloseResult {
        let state = self.get_state();

        if state != TcpState::CloseWait {
            return CloseResult::InvalidState;
        }

        // Store FIN sequence number
        self.fin_seq.store(snd_nxt, Ordering::Relaxed);

        // Process close event
        let result = self.state_machine.process_event(TcpEvent::Close, timestamp);

        match result {
            StateTransition::SendResponse { new_state, send_fin, .. } => {
                assert_eq!(new_state, TcpState::LastAck);
                assert!(send_fin);

                CloseResult::FinSent { seq: snd_nxt }
            }
            _ => CloseResult::InvalidState,
        }
    }

    /// Check for 2MSL timeout
    pub fn check_timewait_timeout(&self, timestamp: u64) -> bool {
        let state = self.get_state();
        if state != TcpState::TimeWait {
            return false;
        }

        let expire_time = self.tw_expire_time.load(Ordering::Relaxed);
        if timestamp >= expire_time {
            // Process timeout
            let result = self.state_machine.process_event(TcpEvent::Timeout, timestamp);
            matches!(result, StateTransition::Ok(TcpState::Closed))
        } else {
            false
        }
    }

    /// Check for FIN_WAIT2 timeout
    pub fn check_finwait2_timeout(&self, timestamp: u64) -> bool {
        let state = self.get_state();
        if state != TcpState::FinWait2 {
            return false;
        }

        let close_time = self.close_time.load(Ordering::Relaxed);
        if timestamp >= close_time + FIN_WAIT_TIMEOUT {
            // Timeout: close connection
            self.state_machine.process_event(TcpEvent::Timeout, timestamp);
            true
        } else {
            false
        }
    }

    /// Handle RST during close
    pub fn handle_rst(&self, timestamp: u64) {
        self.state_machine.process_event(TcpEvent::RcvRst, timestamp);
    }

    /// Get FIN retransmit count
    pub fn get_fin_retransmits(&self) -> u32 {
        self.fin_retransmits.load(Ordering::Relaxed)
    }

    /// Increment FIN retransmit count
    pub fn increment_fin_retransmits(&self) {
        self.fin_retransmits.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if max FIN retransmits exceeded
    pub fn exceeded_max_fin_retransmits(&self) -> bool {
        self.fin_retransmits.load(Ordering::Relaxed) >= MAX_FIN_RETRANSMITS
    }

    /// Get time remaining in TIME_WAIT
    pub fn get_timewait_remaining(&self, timestamp: u64) -> u64 {
        let state = self.get_state();
        if state != TcpState::TimeWait {
            return 0;
        }

        let expire_time = self.tw_expire_time.load(Ordering::Relaxed);
        expire_time.saturating_sub(timestamp)
    }
}

/// TCP close state tracker
pub struct CloseStateTracker {
    /// Active close initiated
    active_close: bool,

    /// FIN sent
    fin_sent: bool,

    /// FIN acknowledged
    fin_acked: bool,

    /// FIN received
    fin_received: bool,

    /// FIN-ACK sent
    finack_sent: bool,
}

impl CloseStateTracker {
    pub const fn new() -> Self {
        Self {
            active_close: false,
            fin_sent: false,
            fin_acked: false,
            fin_received: false,
            finack_sent: false,
        }
    }

    /// Mark active close initiated
    pub fn mark_active_close(&mut self) {
        self.active_close = true;
    }

    /// Mark FIN sent
    pub fn mark_fin_sent(&mut self) {
        self.fin_sent = true;
    }

    /// Mark FIN acknowledged
    pub fn mark_fin_acked(&mut self) {
        self.fin_acked = true;
    }

    /// Mark FIN received
    pub fn mark_fin_received(&mut self) {
        self.fin_received = true;
    }

    /// Mark FIN-ACK sent
    pub fn mark_finack_sent(&mut self) {
        self.finack_sent = true;
    }

    /// Check if active close
    pub fn is_active_close(&self) -> bool {
        self.active_close
    }

    /// Check if simultaneous close
    pub fn is_simultaneous_close(&self) -> bool {
        self.fin_sent && self.fin_received && !self.fin_acked
    }

    /// Check if close complete
    pub fn is_close_complete(&self) -> bool {
        self.fin_acked && self.fin_received
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_closer_new() {
        let closer = TcpCloser::new();
        assert_eq!(closer.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_active_close() {
        let closer = TcpCloser::new();

        // Set to ESTABLISHED
        closer.state_machine.process_event(TcpEvent::ActiveOpen, 0);
        closer.state_machine.process_event(TcpEvent::RcvSynAck, 1);

        let result = closer.active_close(1000, 2);
        match result {
            CloseResult::FinSent { seq } => assert_eq!(seq, 1000),
            _ => panic!("Expected FinSent"),
        }

        assert_eq!(closer.get_state(), TcpState::FinWait1);
    }

    #[test]
    fn test_passive_close() {
        let closer = TcpCloser::new();

        // Set to ESTABLISHED
        closer.state_machine.process_event(TcpEvent::PassiveOpen, 0);
        closer.state_machine.process_event(TcpEvent::RcvSyn, 1);
        closer.state_machine.process_event(TcpEvent::RcvAck, 2);

        // Receive FIN
        let result = closer.handle_fin(1000, 1000, 3);
        match result {
            FinReceivedResult::CloseWait { ack } => assert_eq!(ack, 1001),
            _ => panic!("Expected CloseWait"),
        }

        assert_eq!(closer.get_state(), TcpState::CloseWait);
    }

    #[test]
    fn test_active_close_complete() {
        let closer = TcpCloser::new();

        // Set to ESTABLISHED
        closer.state_machine.process_event(TcpEvent::ActiveOpen, 0);
        closer.state_machine.process_event(TcpEvent::RcvSynAck, 1);

        // Active close
        closer.active_close(1000, 2);
        assert_eq!(closer.get_state(), TcpState::FinWait1);

        // Receive ACK for FIN
        let result = closer.handle_ack(1001, 3);
        assert_eq!(result, AckReceivedResult::FinWait2);
        assert_eq!(closer.get_state(), TcpState::FinWait2);

        // Receive FIN
        let result = closer.handle_fin(2000, 2000, 4);
        match result {
            FinReceivedResult::TimeWait { ack } => assert_eq!(ack, 2001),
            _ => panic!("Expected TimeWait"),
        }
        assert_eq!(closer.get_state(), TcpState::TimeWait);

        // 2MSL timeout
        assert!(closer.check_timewait_timeout(4 + TWO_MSL_TIMEOUT));
        assert_eq!(closer.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_passive_close_complete() {
        let closer = TcpCloser::new();

        // Set to ESTABLISHED
        closer.state_machine.process_event(TcpEvent::PassiveOpen, 0);
        closer.state_machine.process_event(TcpEvent::RcvSyn, 1);
        closer.state_machine.process_event(TcpEvent::RcvAck, 2);

        // Receive FIN
        closer.handle_fin(1000, 1000, 3);
        assert_eq!(closer.get_state(), TcpState::CloseWait);

        // Send FIN
        let result = closer.send_fin(2000, 4);
        match result {
            CloseResult::FinSent { seq } => assert_eq!(seq, 2000),
            _ => panic!("Expected FinSent"),
        }
        assert_eq!(closer.get_state(), TcpState::LastAck);

        // Receive ACK
        let result = closer.handle_ack(2001, 5);
        assert_eq!(result, AckReceivedResult::Closed);
        assert_eq!(closer.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_simultaneous_close() {
        let closer = TcpCloser::new();

        // Set to ESTABLISHED
        closer.state_machine.process_event(TcpEvent::ActiveOpen, 0);
        closer.state_machine.process_event(TcpEvent::RcvSynAck, 1);

        // Active close
        closer.active_close(1000, 2);
        assert_eq!(closer.get_state(), TcpState::FinWait1);

        // Receive FIN before ACK (simultaneous close)
        let result = closer.handle_fin(2000, 2000, 3);
        match result {
            FinReceivedResult::Closing { ack } => assert_eq!(ack, 2001),
            _ => panic!("Expected Closing"),
        }
        assert_eq!(closer.get_state(), TcpState::Closing);

        // Receive ACK
        let result = closer.handle_ack(1001, 4);
        assert_eq!(result, AckReceivedResult::TimeWait);
        assert_eq!(closer.get_state(), TcpState::TimeWait);

        // 2MSL timeout
        assert!(closer.check_timewait_timeout(4 + TWO_MSL_TIMEOUT));
        assert_eq!(closer.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_close_state_tracker() {
        let mut tracker = CloseStateTracker::new();

        tracker.mark_active_close();
        assert!(tracker.is_active_close());
        assert!(!tracker.is_simultaneous_close());

        tracker.mark_fin_sent();
        tracker.mark_fin_received();
        assert!(tracker.is_simultaneous_close());

        tracker.mark_fin_acked();
        assert!(tracker.is_close_complete());
    }

    #[test]
    fn test_fin_retransmits() {
        let closer = TcpCloser::new();

        assert_eq!(closer.get_fin_retransmits(), 0);

        closer.increment_fin_retransmits();
        assert_eq!(closer.get_fin_retransmits(), 1);

        closer.increment_fin_retransmits();
        closer.increment_fin_retransmits();
        closer.increment_fin_retransmits();
        closer.increment_fin_retransmits();
        assert!(closer.exceeded_max_fin_retransmits());
    }

    #[test]
    fn test_timewait_remaining() {
        let closer = TcpCloser::new();

        // Set to TIME_WAIT
        closer.state_machine.process_event(TcpEvent::ActiveOpen, 0);
        closer.state_machine.process_event(TcpEvent::RcvSynAck, 1);
        closer.active_close(1000, 2);
        closer.handle_ack(1001, 3);
        closer.handle_fin(2000, 2000, 4);

        assert_eq!(closer.get_state(), TcpState::TimeWait);

        let remaining = closer.get_timewait_remaining(4);
        assert_eq!(remaining, TWO_MSL_TIMEOUT);

        let remaining = closer.get_timewait_remaining(4 + 30000);
        assert_eq!(remaining, TWO_MSL_TIMEOUT - 30000);
    }
}
