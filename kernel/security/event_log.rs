/*
 * Nuva OS - Kernel - Event Log Manager
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

//! Event Log Manager
//!
//! Maintains an append-only measurement event log with rolling hash
//! integrity protection. Supports replay verification to check
//! PCR consistency against the recorded events.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::sha256::sha256_digest;
use super::tpm_abi::{TpmAbi, PcrIndex, PCR_DIGEST_SIZE};

/// Maximum events in the log
pub const MAX_EVENT_LOG_ENTRIES: usize = 256;

/// A single measurement event in the event log
#[derive(Debug, Clone)]
pub struct MeasurementEvent {
    /// Component identifier
    pub component: u32,
    /// PCR index this measurement extends
    pub pcr_index: PcrIndex,
    /// SHA-256 digest of the measured component
    pub digest: [u8; PCR_DIGEST_SIZE],
    /// Size of the measured data in bytes
    pub data_size: u64,
    /// Monotonic timestamp (boot cycle counter)
    pub timestamp: u64,
    /// Whether PCR extend succeeded
    pub pcr_extend_ok: bool,
}

/// Event log manager with rolling hash integrity protection
pub struct EventLogManager {
    /// Event log entries
    events: Vec<MeasurementEvent>,
    /// Rolling integrity hash: H_n = SHA256(H_{n-1} || event_n)
    rolling_hash: [u8; PCR_DIGEST_SIZE],
    /// Total events appended
    event_count: AtomicU64,
}

impl EventLogManager {
    /// Create a new event log manager
    pub fn new() -> Self {
        EventLogManager {
            events: Vec::new(),
            rolling_hash: [0u8; PCR_DIGEST_SIZE],
            event_count: AtomicU64::new(0),
        }
    }

    /// Append a measurement event and update rolling hash.
    ///
    /// The rolling hash is computed as:
    ///   H_n = SHA256(H_{n-1} || serialize(event_n))
    /// This provides tamper-evidence: any modification to an
    /// event or reordering will change the final hash.
    pub fn append(&mut self, event: MeasurementEvent) {
        // Serialize event for rolling hash computation
        let mut event_bytes = [0u8; 8 + 4 + PCR_DIGEST_SIZE + 8 + 8 + 1];
        let mut offset = 0;
        event_bytes[offset..offset+4].copy_from_slice(&event.component.to_le_bytes()); offset += 4;
        event_bytes[offset..offset+4].copy_from_slice(&event.pcr_index.to_le_bytes()); offset += 4;
        event_bytes[offset..offset+PCR_DIGEST_SIZE].copy_from_slice(&event.digest); offset += PCR_DIGEST_SIZE;
        event_bytes[offset..offset+8].copy_from_slice(&event.data_size.to_le_bytes()); offset += 8;
        event_bytes[offset..offset+8].copy_from_slice(&event.timestamp.to_le_bytes()); offset += 8;
        event_bytes[offset] = if event.pcr_extend_ok { 1 } else { 0 };
        // Update rolling hash: H_new = SHA256(H_old || event_bytes)
        let mut concat = [0u8; PCR_DIGEST_SIZE + 53];
        concat[..PCR_DIGEST_SIZE].copy_from_slice(&self.rolling_hash);
        concat[PCR_DIGEST_SIZE..].copy_from_slice(&event_bytes);
        self.rolling_hash = sha256_digest(&concat);
        // Append event (with capacity limit)
        if self.events.len() < MAX_EVENT_LOG_ENTRIES {
            self.events.push(event);
        }
        self.event_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Replay the event log and verify PCR consistency.
    ///
    /// Simulates PCR extension from the event log and compares
    /// the computed PCR values with the actual TPM PCR values.
    /// Returns true if all PCRs match.
    pub fn replay_verify<T: TpmAbi>(&self, tpm: &T) -> bool {
        // Group events by PCR index
        let mut pcr_computed: Vec<(PcrIndex, [u8; PCR_DIGEST_SIZE])> = Vec::new();
        for event in &self.events {
            if !event.pcr_extend_ok { continue; }
            if let Some(entry) = pcr_computed.iter_mut().find(|(idx, _)| *idx == event.pcr_index) {
                // Extend: PCR_new = SHA256(PCR_old || measurement)
                let mut concat = [0u8; PCR_DIGEST_SIZE * 2];
                concat[..PCR_DIGEST_SIZE].copy_from_slice(&entry.1);
                concat[PCR_DIGEST_SIZE..].copy_from_slice(&event.digest);
                entry.1 = sha256_digest(&concat);
            } else {
                // First measurement: extend from zero PCR
                let mut concat = [0u8; PCR_DIGEST_SIZE * 2];
                concat[PCR_DIGEST_SIZE..].copy_from_slice(&event.digest);
                let extended = sha256_digest(&concat);
                pcr_computed.push((event.pcr_index, extended));
            }
        }
        // Compare computed PCRs with actual TPM PCRs
        for (pcr_idx, computed) in &pcr_computed {
            if let Ok(actual) = tpm.pcr_read(*pcr_idx) {
                if actual != *computed { return false; }
            } else {
                return false;
            }
        }
        true
    }

    /// Return the current integrity hash of the log
    pub fn integrity_hash(&self) -> [u8; PCR_DIGEST_SIZE] {
        self.rolling_hash
    }

    /// Get the number of events in the log
    pub fn len(&self) -> usize { self.events.len() }

    /// Check if the log is empty
    pub fn is_empty(&self) -> bool { self.events.is_empty() }

    /// Get reference to events
    pub fn events(&self) -> &Vec<MeasurementEvent> { &self.events }
}
