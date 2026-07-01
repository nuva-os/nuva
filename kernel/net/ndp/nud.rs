/*
 * Nuva OS - Kernel - Net - Ndp - Nud
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
 *
 * Nuva OS - Kernel - NDP NUD State Machine
 *
 * Neighbor Unreachability Detection (NUD) state machine per RFC 4861 Section 7.3.
 * Six-state FSM: Incomplete -> Reachable -> Stale -> Delay -> Probe -> Failed.
 */

use crate::kernel::net::ipv6::Ipv6Addr;
use alloc::vec::Vec;

/// NUD states per RFC 4861 Section 7.3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudState {
    /// Address resolution is in progress (no link-layer address yet)
    Incomplete,
    /// Neighbor is reachable (confirmed recently)
    Reachable,
    /// Neighbor may be unreachable (reachability is uncertain)
    Stale,
    /// Delay state before probing (short grace period for upper-layer confirmation)
    Delay,
    /// Actively probing neighbor reachability (sending NS)
    Probe,
    /// Neighbor is confirmed unreachable (max probes exceeded)
    Failed,
}

impl NudState {
    /// Check if the state allows packet transmission
    pub fn is_usable(&self) -> bool {
        matches!(self, NudState::Reachable | NudState::Stale | NudState::Delay | NudState::Probe)
    }
}

/// NUD events that drive state transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudEvent {
    /// Upper-layer protocol confirmed reachability (e.g., TCP ACK received)
    ConfirmFromUpperLayer,
    /// Received a Neighbor Advertisement
    ReceiveNA,
    /// A packet is being transmitted to this neighbor
    TransmitPacket,
    /// ReachableTime timer expired
    ReachableTimeout,
    /// DelayFirstProbeTime timer expired
    DelayTimeout,
    /// RetransTimer expired (send another probe)
    ProbeTimeout,
    /// Maximum number of probes reached without response
    MaxProbesReached,
}

/// Actions produced by NUD state transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudAction {
    /// Send a Neighbor Solicitation
    SendNS,
    /// Start a timer (parameter: duration in ms)
    SetTimer,
    /// Clear the neighbor cache entry
    ClearEntry,
    /// Mark the entry as failed (unreachable)
    MarkFailed,
    /// No action required
    None,
}

/// NUD state machine for a single neighbor entry
pub struct NudMachine {
    /// Current NUD state
    pub state: NudState,
    /// Number of NS probes sent in current Probe cycle
    pub probe_count: u32,
    /// Maximum number of probes before failure
    pub max_probes: u32,
}

impl NudMachine {
    /// Create a new NUD machine starting in the Incomplete state
    pub fn new(max_probes: u32) -> Self {
        NudMachine {
            state: NudState::Incomplete,
            probe_count: 0,
            max_probes,
        }
    }

    /// Create a new NUD machine starting in the Stale state
    pub fn new_stale(max_probes: u32) -> Self {
        NudMachine {
            state: NudState::Stale,
            probe_count: 0,
            max_probes,
        }
    }

    /// Perform a state transition based on the given event.
    /// Returns a list of actions to take.
    /// Implements the FSM from RFC 4861 Section 7.3, Figure 7.
    pub fn transition(&mut self, event: NudEvent) -> alloc::vec::Vec<NudAction> {
        use NudState::*;
        use NudEvent::*;
        use NudAction::*;

        let mut actions = alloc::vec::Vec::new();

        match self.state {
            Incomplete => {
                match event {
                    ReceiveNA => {
                        self.state = Reachable;
                        actions.push(SetTimer);
                    }
                    MaxProbesReached => {
                        self.state = Failed;
                        actions.push(MarkFailed);
                    }
                    ProbeTimeout => {
                        self.probe_count += 1;
                        if self.probe_count >= self.max_probes {
                            self.state = Failed;
                            actions.push(MarkFailed);
                        } else {
                            actions.push(SendNS);
                            actions.push(SetTimer);
                        }
                    }
                    _ => {}
                }
            }
            Reachable => {
                match event {
                    ReachableTimeout => {
                        self.state = Stale;
                    }
                    ConfirmFromUpperLayer => {
                        actions.push(SetTimer);
                    }
                    ReceiveNA => {
                        actions.push(SetTimer);
                    }
                    _ => {}
                }
            }
            Stale => {
                match event {
                    TransmitPacket => {
                        self.state = Delay;
                        actions.push(SetTimer);
                    }
                    ReceiveNA => {
                        self.state = Reachable;
                        actions.push(SetTimer);
                    }
                    ConfirmFromUpperLayer => {
                        self.state = Reachable;
                        actions.push(SetTimer);
                    }
                    _ => {}
                }
            }
            Delay => {
                match event {
                    DelayTimeout => {
                        self.state = Probe;
                        self.probe_count = 1;
                        actions.push(SendNS);
                        actions.push(SetTimer);
                    }
                    ConfirmFromUpperLayer => {
                        self.state = Reachable;
                        actions.push(SetTimer);
                    }
                    ReceiveNA => {
                        self.state = Reachable;
                        actions.push(SetTimer);
                    }
                    _ => {}
                }
            }
            Probe => {
                match event {
                    ProbeTimeout => {
                        self.probe_count += 1;
                        if self.probe_count >= self.max_probes {
                            self.state = Failed;
                            actions.push(MarkFailed);
                        } else {
                            actions.push(SendNS);
                            actions.push(SetTimer);
                        }
                    }
                    ReceiveNA => {
                        self.state = Reachable;
                        self.probe_count = 0;
                        actions.push(SetTimer);
                    }
                    ConfirmFromUpperLayer => {
                        self.state = Reachable;
                        self.probe_count = 0;
                        actions.push(SetTimer);
                    }
                    MaxProbesReached => {
                        self.state = Failed;
                        actions.push(MarkFailed);
                    }
                    _ => {}
                }
            }
            Failed => {
                // No transitions out of Failed; entry must be re-created
            }
        }

        actions
    }

    /// Get the current NUD state
    pub fn get_state(&self) -> NudState {
        self.state
    }

    /// Reset the machine to Incomplete state (for re-resolution)
    pub fn reset(&mut self) {
        self.state = NudState::Incomplete;
        self.probe_count = 0;
    }
}

/// NUD timer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudTimer {
    /// ReachableTime timer
    ReachableTimer,
    /// DELAY_FIRST_PROBE_TIME timer
    DelayTimer,
    /// RetransTimer for NS probes
    ProbeTimer,
}

impl NudTimer {
    /// Handle timer expiration by producing the corresponding NUD event
    pub fn to_event(&self) -> NudEvent {
        match self {
            NudTimer::ReachableTimer => NudEvent::ReachableTimeout,
            NudTimer::DelayTimer => NudEvent::DelayTimeout,
            NudTimer::ProbeTimer => NudEvent::ProbeTimeout,
        }
    }
}
