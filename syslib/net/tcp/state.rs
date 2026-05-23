/*
 * Nuva OS - TCP State Machine Implementation
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

//! TCP State Machine Implementation (RFC 793)
/*!*/
//! Complete TCP state machine with all states and transitions.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// TCP States (RFC 793)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TcpState {
    /// Connection closed
    Closed = 0,

    /// Listening for incoming connections
    Listen = 1,

    /// SYN sent, waiting for SYN-ACK
    SynSent = 2,

    /// SYN received, waiting for ACK
    SynReceived = 3,

    /// Connection established
    Established = 4,

    /// FIN sent, waiting for ACK
    FinWait1 = 5,

    /// FIN acknowledged, waiting for FIN
    FinWait2 = 6,

    /// FIN received, waiting for application close
    CloseWait = 7,

    /// FIN sent after receiving FIN, waiting for ACK
    Closing = 8,

    /// FIN acknowledged, waiting for FIN ACK
    LastAck = 9,

    /// Waiting for 2MSL timeout
    TimeWait = 10,
}

impl TcpState {
    /// Check if connection is established
    pub fn is_established(&self) -> bool {
        *self == TcpState::Established
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

    /// Check if connection is closed
    pub fn is_closed(&self) -> bool {
        *self == TcpState::Closed
    }

    /// Get state name as string
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

/// TCP Events that trigger state transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpEvent {
    /// Application requests to open connection (active)
    ActiveOpen,

    /// Application requests to listen (passive)
    PassiveOpen,

    /// Received SYN segment
    RcvSyn,

    /// Received SYN-ACK segment
    RcvSynAck,

    /// Received ACK segment
    RcvAck,

    /// Received FIN segment
    RcvFin,

    /// Received FIN-ACK segment
    RcvFinAck,

    /// Received RST segment
    RcvRst,

    /// Application requests to close
    Close,

    /// Timer expired (2MSL, retransmit, etc.)
    Timeout,

    /// Send SYN
    SendSyn,

    /// Send FIN
    SendFin,
}

/// TCP State transition result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTransition {
    /// Transition successful
    Ok(TcpState),

    /// Invalid transition
    Invalid,

    /// Send response segment
    SendResponse {
        new_state: TcpState,
        send_syn: bool,
        send_ack: bool,
        send_fin: bool,
        send_rst: bool,
    },

    /// Connection error
    Error(TcpError),
}

/// TCP Error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpError {
    /// Connection reset
    ConnectionReset,

    /// Connection refused
    ConnectionRefused,

    /// Timeout
    Timeout,

    /// Invalid state
    InvalidState,

    /// Invalid segment
    InvalidSegment,
}

/// TCP State Machine
pub struct TcpStateMachine {
    state: AtomicU32,
    state_entered_time: AtomicU64,
    syn_count: AtomicU32,
    fin_count: AtomicU32,
}

impl TcpStateMachine {
    /// Create new state machine in CLOSED state
    pub const fn new() -> Self {
        Self {
            state: AtomicU32::new(TcpState::Closed as u32),
            state_entered_time: AtomicU64::new(0),
            syn_count: AtomicU32::new(0),
            fin_count: AtomicU32::new(0),
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

    /// Set state (internal)
    fn set_state(&self, state: TcpState, timestamp: u64) {
        self.state.store(state as u32, Ordering::Release);
        self.state_entered_time.store(timestamp, Ordering::Release);
    }

    /// Get time in current state (ms)
    pub fn time_in_state(&self, current_time: u64) -> u64 {
        current_time.saturating_sub(self.state_entered_time.load(Ordering::Acquire))
    }

    /// Process event and transition state
    pub fn process_event(&self, event: TcpEvent, timestamp: u64) -> StateTransition {
        let current_state = self.get_state();

        let result = match current_state {
            TcpState::Closed => self.process_closed(event),
            TcpState::Listen => self.process_listen(event),
            TcpState::SynSent => self.process_syn_sent(event),
            TcpState::SynReceived => self.process_syn_received(event),
            TcpState::Established => self.process_established(event),
            TcpState::FinWait1 => self.process_fin_wait1(event),
            TcpState::FinWait2 => self.process_fin_wait2(event),
            TcpState::CloseWait => self.process_close_wait(event),
            TcpState::Closing => self.process_closing(event),
            TcpState::LastAck => self.process_last_ack(event),
            TcpState::TimeWait => self.process_time_wait(event, timestamp),
        };

        // Update state if transition successful
        if let StateTransition::Ok(new_state) = result {
            self.set_state(new_state, timestamp);
        } else if let StateTransition::SendResponse { new_state, .. } = result {
            self.set_state(new_state, timestamp);
        }

        result
    }

    /// Process event in CLOSED state
    fn process_closed(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::ActiveOpen => {
                self.syn_count.fetch_add(1, Ordering::Relaxed);
                StateTransition::Ok(TcpState::SynSent)
            }
            TcpEvent::PassiveOpen => StateTransition::Ok(TcpState::Listen),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in LISTEN state
    fn process_listen(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::RcvSyn => {
                // Send SYN-ACK
                StateTransition::SendResponse {
                    new_state: TcpState::SynReceived,
                    send_syn: true,
                    send_ack: true,
                    send_fin: false,
                    send_rst: false,
                }
            }
            TcpEvent::Close => StateTransition::Ok(TcpState::Closed),
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in SYN_SENT state
    fn process_syn_sent(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::RcvSynAck => {
                // Send ACK
                StateTransition::SendResponse {
                    new_state: TcpState::Established,
                    send_syn: false,
                    send_ack: true,
                    send_fin: false,
                    send_rst: false,
                }
            }
            TcpEvent::RcvSyn => {
                // Simultaneous open: send ACK
                StateTransition::SendResponse {
                    new_state: TcpState::SynReceived,
                    send_syn: false,
                    send_ack: true,
                    send_fin: false,
                    send_rst: false,
                }
            }
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            TcpEvent::Timeout => {
                // Retransmit SYN
                self.syn_count.fetch_add(1, Ordering::Relaxed);
                StateTransition::Ok(TcpState::SynSent)
            }
            TcpEvent::Close => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in SYN_RECEIVED state
    fn process_syn_received(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::RcvAck => StateTransition::Ok(TcpState::Established),
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            TcpEvent::Timeout => {
                // Retransmit SYN-ACK
                StateTransition::Ok(TcpState::SynReceived)
            }
            TcpEvent::Close => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in ESTABLISHED state
    fn process_established(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::Close => {
                self.fin_count.fetch_add(1, Ordering::Relaxed);
                // Send FIN
                StateTransition::SendResponse {
                    new_state: TcpState::FinWait1,
                    send_syn: false,
                    send_ack: false,
                    send_fin: true,
                    send_rst: false,
                }
            }
            TcpEvent::RcvFin => {
                // Send ACK
                StateTransition::SendResponse {
                    new_state: TcpState::CloseWait,
                    send_syn: false,
                    send_ack: true,
                    send_fin: false,
                    send_rst: false,
                }
            }
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in FIN_WAIT_1 state
    fn process_fin_wait1(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::RcvAck => StateTransition::Ok(TcpState::FinWait2),
            TcpEvent::RcvFin => {
                // Simultaneous close: send ACK
                StateTransition::SendResponse {
                    new_state: TcpState::Closing,
                    send_syn: false,
                    send_ack: true,
                    send_fin: false,
                    send_rst: false,
                }
            }
            TcpEvent::RcvFinAck => {
                // Send ACK
                StateTransition::SendResponse {
                    new_state: TcpState::TimeWait,
                    send_syn: false,
                    send_ack: true,
                    send_fin: false,
                    send_rst: false,
                }
            }
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in FIN_WAIT_2 state
    fn process_fin_wait2(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::RcvFin => {
                // Send ACK
                StateTransition::SendResponse {
                    new_state: TcpState::TimeWait,
                    send_syn: false,
                    send_ack: true,
                    send_fin: false,
                    send_rst: false,
                }
            }
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in CLOSE_WAIT state
    fn process_close_wait(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::Close => {
                self.fin_count.fetch_add(1, Ordering::Relaxed);
                // Send FIN
                StateTransition::SendResponse {
                    new_state: TcpState::LastAck,
                    send_syn: false,
                    send_ack: false,
                    send_fin: true,
                    send_rst: false,
                }
            }
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in CLOSING state
    fn process_closing(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::RcvAck => StateTransition::Ok(TcpState::TimeWait),
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in LAST_ACK state
    fn process_last_ack(&self, event: TcpEvent) -> StateTransition {
        match event {
            TcpEvent::RcvAck => StateTransition::Ok(TcpState::Closed),
            TcpEvent::RcvRst => StateTransition::Ok(TcpState::Closed),
            _ => StateTransition::Invalid,
        }
    }

    /// Process event in TIME_WAIT state
    fn process_time_wait(&self, event: TcpEvent, timestamp: u64) -> StateTransition {
        match event {
            TcpEvent::Timeout => {
                // 2MSL timeout expired
                StateTransition::Ok(TcpState::Closed)
            }
            TcpEvent::RcvFin => {
                // Retransmitted FIN, send ACK and restart 2MSL
                StateTransition::SendResponse {
                    new_state: TcpState::TimeWait,
                    send_syn: false,
                    send_ack: true,
                    send_fin: false,
                    send_rst: false,
                }
            }
            _ => StateTransition::Invalid,
        }
    }

    /// Check if can send data
    pub fn can_send(&self) -> bool {
        matches!(
            self.get_state(),
            TcpState::Established | TcpState::CloseWait
        )
    }

    /// Check if can receive data
    pub fn can_receive(&self) -> bool {
        matches!(
            self.get_state(),
            TcpState::Established
                | TcpState::FinWait1
                | TcpState::FinWait2
        )
    }

    /// Get SYN retransmit count
    pub fn get_syn_count(&self) -> u32 {
        self.syn_count.load(Ordering::Relaxed)
    }

    /// Get FIN retransmit count
    pub fn get_fin_count(&self) -> u32 {
        self.fin_count.load(Ordering::Relaxed)
    }
}

/// TCP State machine validator
pub struct StateValidator;

impl StateValidator {
    /// Validate state transition
    pub fn is_valid_transition(from: TcpState, to: TcpState) -> bool {
        match from {
            TcpState::Closed => {
                matches!(to, TcpState::Listen | TcpState::SynSent)
            }
            TcpState::Listen => {
                matches!(to, TcpState::SynReceived | TcpState::Closed)
            }
            TcpState::SynSent => {
                matches!(
                    to,
                    TcpState::Established
                        | TcpState::SynReceived
                        | TcpState::Closed
                )
            }
            TcpState::SynReceived => {
                matches!(to, TcpState::Established | TcpState::Closed)
            }
            TcpState::Established => {
                matches!(
                    to,
                    TcpState::FinWait1 | TcpState::CloseWait | TcpState::Closed
                )
            }
            TcpState::FinWait1 => {
                matches!(
                    to,
                    TcpState::FinWait2 | TcpState::Closing | TcpState::TimeWait | TcpState::Closed
                )
            }
            TcpState::FinWait2 => {
                matches!(to, TcpState::TimeWait | TcpState::Closed)
            }
            TcpState::CloseWait => {
                matches!(to, TcpState::LastAck | TcpState::Closed)
            }
            TcpState::Closing => {
                matches!(to, TcpState::TimeWait | TcpState::Closed)
            }
            TcpState::LastAck => {
                matches!(to, TcpState::Closed)
            }
            TcpState::TimeWait => {
                matches!(to, TcpState::Closed)
            }
        }
    }

    /// Get all valid next states
    pub fn get_valid_next_states(state: TcpState) -> &'static [TcpState] {
        match state {
            TcpState::Closed => &[TcpState::Listen, TcpState::SynSent],
            TcpState::Listen => &[TcpState::SynReceived, TcpState::Closed],
            TcpState::SynSent => &[TcpState::Established, TcpState::SynReceived, TcpState::Closed],
            TcpState::SynReceived => &[TcpState::Established, TcpState::Closed],
            TcpState::Established => &[TcpState::FinWait1, TcpState::CloseWait, TcpState::Closed],
            TcpState::FinWait1 => &[TcpState::FinWait2, TcpState::Closing, TcpState::TimeWait, TcpState::Closed],
            TcpState::FinWait2 => &[TcpState::TimeWait, TcpState::Closed],
            TcpState::CloseWait => &[TcpState::LastAck, TcpState::Closed],
            TcpState::Closing => &[TcpState::TimeWait, TcpState::Closed],
            TcpState::LastAck => &[TcpState::Closed],
            TcpState::TimeWait => &[TcpState::Closed],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_state_values() {
        assert_eq!(TcpState::Closed as u8, 0);
        assert_eq!(TcpState::Listen as u8, 1);
        assert_eq!(TcpState::SynSent as u8, 2);
        assert_eq!(TcpState::SynReceived as u8, 3);
        assert_eq!(TcpState::Established as u8, 4);
        assert_eq!(TcpState::FinWait1 as u8, 5);
        assert_eq!(TcpState::FinWait2 as u8, 6);
        assert_eq!(TcpState::CloseWait as u8, 7);
        assert_eq!(TcpState::Closing as u8, 8);
        assert_eq!(TcpState::LastAck as u8, 9);
        assert_eq!(TcpState::TimeWait as u8, 10);
    }

    #[test]
    fn test_tcp_state_is_established() {
        assert!(TcpState::Established.is_established());
        assert!(!TcpState::Closed.is_established());
        assert!(!TcpState::Listen.is_established());
    }

    #[test]
    fn test_tcp_state_is_closing() {
        assert!(TcpState::FinWait1.is_closing());
        assert!(TcpState::FinWait2.is_closing());
        assert!(TcpState::Closing.is_closing());
        assert!(TcpState::TimeWait.is_closing());
        assert!(TcpState::CloseWait.is_closing());
        assert!(TcpState::LastAck.is_closing());
        assert!(!TcpState::Established.is_closing());
    }

    #[test]
    fn test_tcp_state_as_str() {
        assert_eq!(TcpState::Closed.as_str(), "CLOSED");
        assert_eq!(TcpState::Listen.as_str(), "LISTEN");
        assert_eq!(TcpState::SynSent.as_str(), "SYN_SENT");
        assert_eq!(TcpState::Established.as_str(), "ESTABLISHED");
    }

    #[test]
    fn test_state_machine_new() {
        let sm = TcpStateMachine::new();
        assert_eq!(sm.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_state_machine_active_open() {
        let sm = TcpStateMachine::new();

        let result = sm.process_event(TcpEvent::ActiveOpen, 0);
        assert!(matches!(result, StateTransition::Ok(TcpState::SynSent)));
        assert_eq!(sm.get_state(), TcpState::SynSent);
    }

    #[test]
    fn test_state_machine_passive_open() {
        let sm = TcpStateMachine::new();

        let result = sm.process_event(TcpEvent::PassiveOpen, 0);
        assert!(matches!(result, StateTransition::Ok(TcpState::Listen)));
        assert_eq!(sm.get_state(), TcpState::Listen);
    }

    #[test]
    fn test_state_machine_three_way_handshake() {
        let sm = TcpStateMachine::new();

        // Active open
        sm.process_event(TcpEvent::ActiveOpen, 0);
        assert_eq!(sm.get_state(), TcpState::SynSent);

        // Receive SYN-ACK
        let result = sm.process_event(TcpEvent::RcvSynAck, 1);
        assert!(matches!(result, StateTransition::SendResponse { .. }));
        assert_eq!(sm.get_state(), TcpState::Established);
    }

    #[test]
    fn test_state_machine_passive_open_handshake() {
        let sm = TcpStateMachine::new();

        // Passive open
        sm.process_event(TcpEvent::PassiveOpen, 0);
        assert_eq!(sm.get_state(), TcpState::Listen);

        // Receive SYN
        let result = sm.process_event(TcpEvent::RcvSyn, 1);
        assert!(matches!(result, StateTransition::SendResponse { .. }));
        assert_eq!(sm.get_state(), TcpState::SynReceived);

        // Receive ACK
        sm.process_event(TcpEvent::RcvAck, 2);
        assert_eq!(sm.get_state(), TcpState::Established);
    }

    #[test]
    fn test_state_machine_active_close() {
        let sm = TcpStateMachine::new();

        // Establish connection
        sm.process_event(TcpEvent::ActiveOpen, 0);
        sm.process_event(TcpEvent::RcvSynAck, 1);
        assert_eq!(sm.get_state(), TcpState::Established);

        // Active close
        let result = sm.process_event(TcpEvent::Close, 2);
        assert!(matches!(result, StateTransition::SendResponse { .. }));
        assert_eq!(sm.get_state(), TcpState::FinWait1);

        // Receive ACK
        sm.process_event(TcpEvent::RcvAck, 3);
        assert_eq!(sm.get_state(), TcpState::FinWait2);

        // Receive FIN
        sm.process_event(TcpEvent::RcvFin, 4);
        assert_eq!(sm.get_state(), TcpState::TimeWait);

        // 2MSL timeout
        sm.process_event(TcpEvent::Timeout, 5);
        assert_eq!(sm.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_state_machine_passive_close() {
        let sm = TcpStateMachine::new();

        // Establish connection
        sm.process_event(TcpEvent::PassiveOpen, 0);
        sm.process_event(TcpEvent::RcvSyn, 1);
        sm.process_event(TcpEvent::RcvAck, 2);
        assert_eq!(sm.get_state(), TcpState::Established);

        // Receive FIN
        sm.process_event(TcpEvent::RcvFin, 3);
        assert_eq!(sm.get_state(), TcpState::CloseWait);

        // Application close
        sm.process_event(TcpEvent::Close, 4);
        assert_eq!(sm.get_state(), TcpState::LastAck);

        // Receive ACK
        sm.process_event(TcpEvent::RcvAck, 5);
        assert_eq!(sm.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_state_machine_simultaneous_close() {
        let sm = TcpStateMachine::new();

        // Establish connection
        sm.process_event(TcpEvent::ActiveOpen, 0);
        sm.process_event(TcpEvent::RcvSynAck, 1);
        assert_eq!(sm.get_state(), TcpState::Established);

        // Active close
        sm.process_event(TcpEvent::Close, 2);
        assert_eq!(sm.get_state(), TcpState::FinWait1);

        // Receive FIN (simultaneous close)
        sm.process_event(TcpEvent::RcvFin, 3);
        assert_eq!(sm.get_state(), TcpState::Closing);

        // Receive ACK
        sm.process_event(TcpEvent::RcvAck, 4);
        assert_eq!(sm.get_state(), TcpState::TimeWait);

        // 2MSL timeout
        sm.process_event(TcpEvent::Timeout, 5);
        assert_eq!(sm.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_state_machine_rst() {
        let sm = TcpStateMachine::new();

        // Establish connection
        sm.process_event(TcpEvent::ActiveOpen, 0);
        sm.process_event(TcpEvent::RcvSynAck, 1);
        assert_eq!(sm.get_state(), TcpState::Established);

        // Receive RST
        sm.process_event(TcpEvent::RcvRst, 2);
        assert_eq!(sm.get_state(), TcpState::Closed);
    }

    #[test]
    fn test_state_machine_can_send_receive() {
        let sm = TcpStateMachine::new();

        // Closed state
        assert!(!sm.can_send());
        assert!(!sm.can_receive());

        // Established state
        sm.process_event(TcpEvent::ActiveOpen, 0);
        sm.process_event(TcpEvent::RcvSynAck, 1);
        assert!(sm.can_send());
        assert!(sm.can_receive());

        // FinWait1 state
        sm.process_event(TcpEvent::Close, 2);
        assert!(!sm.can_send());
        assert!(sm.can_receive());
    }

    #[test]
    fn test_state_validator() {
        assert!(StateValidator::is_valid_transition(
            TcpState::Closed,
            TcpState::SynSent
        ));
        assert!(StateValidator::is_valid_transition(
            TcpState::SynSent,
            TcpState::Established
        ));
        assert!(!StateValidator::is_valid_transition(
            TcpState::Closed,
            TcpState::Established
        ));
    }

    #[test]
    fn test_state_validator_next_states() {
        let next = StateValidator::get_valid_next_states(TcpState::Closed);
        assert_eq!(next.len(), 2);
        assert!(next.contains(&TcpState::Listen));
        assert!(next.contains(&TcpState::SynSent));
    }
}
