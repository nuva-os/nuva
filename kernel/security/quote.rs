/*
 * Nuva OS - Kernel - Quote/Attestation Engine
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

//! Quote/Attestation Engine
//!
//! Generates TPM-style quote tokens for remote attestation.
//! A quote binds AIK-signed PCR values with a nonce and timestamp,
//! providing cryptographic evidence of the platform state.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::tpm_abi::{TpmAbi, TpmError, TpmResult, PcrIndex, PCR_DIGEST_SIZE, DEFAULT_PCR_COUNT};
use super::aik::{AikManager, AikError};
use super::event_log::EventLogManager;
use super::sha256::sha256_digest;

/// Nonce size for quote freshness
pub const NONCE_SIZE: usize = 32;
/// Timestamp size
pub const TIMESTAMP_SIZE: usize = 8;

/// Quote token structure
///
/// Contains the AIK signature over PCR values plus metadata
/// needed for remote attestation verification.
#[derive(Debug, Clone)]
pub struct QuoteToken {
    /// AIK signature over the quote data
    pub signature: Vec<u8>,
    /// PCR values included in the quote
    pub pcr_values: Vec<(PcrIndex, [u8; PCR_DIGEST_SIZE])>,
    /// Nonce for freshness (prevents replay)
    pub nonce: [u8; NONCE_SIZE],
    /// Monotonic timestamp
    pub timestamp: u64,
    /// Hash of the event log at quote time
    pub event_log_hash: [u8; PCR_DIGEST_SIZE],
}

/// Attestation response
///
/// Complete attestation response containing the quote token,
/// event log, and AIK certificate for verifier.
#[derive(Debug, Clone)]
pub struct AttestationResponse {
    /// Quote token
    pub quote: QuoteToken,
    /// Event log entries
    pub event_log: Vec<super::event_log::MeasurementEvent>,
    /// AIK public key hash (serves as certificate reference)
    pub aik_public_key_hash: [u8; PCR_DIGEST_SIZE],
}

/// Quote error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteError {
    /// PCR read failed
    PcrReadFailed,
    /// AIK signing failed
    AikError(AikError),
    /// TPM error
    TpmError(TpmError),
    /// Invalid nonce
    InvalidNonce,
}

/// Quote engine for generating attestation tokens
pub struct QuoteEngine {
    /// Quote generation counter
    quote_count: AtomicU64,
}

impl QuoteEngine {
    /// Create a new quote engine
    pub const fn new() -> Self {
        QuoteEngine { quote_count: AtomicU64::new(0) }
    }

    /// Generate a quote token.
    ///
    /// Pipeline:
    /// 1. Read PCR values from TPM (all 24 PCRs)
    /// 2. Construct signed data: hash(pcr_values || nonce || timestamp || event_log_hash)
    /// 3. AIK sign the data (target: <=100ms total)
    pub fn generate_quote<T: TpmAbi>(
        &self,
        tpm: &T,
        aik: &AikManager,
        event_log: &EventLogManager,
        nonce: &[u8; NONCE_SIZE],
    ) -> Result<QuoteToken, QuoteError> {
        // Step 1: Read all PCR values
        let indices: Vec<PcrIndex> = (0..DEFAULT_PCR_COUNT).collect();
        let pcr_values = tpm.pcr_read_multiple(&indices).map_err(QuoteError::TpmError)?;
        // Step 2: Construct signed data
        let timestamp = self.quote_count.fetch_add(1, Ordering::Relaxed);
        let event_log_hash = event_log.integrity_hash();
        // Serialize: pcr_values || nonce || timestamp || event_log_hash
        let mut sign_data = Vec::new();
        for (_, pcr) in &pcr_values {
            sign_data.extend_from_slice(pcr);
        }
        sign_data.extend_from_slice(nonce);
        sign_data.extend_from_slice(&timestamp.to_le_bytes());
        sign_data.extend_from_slice(&event_log_hash);
        // Step 3: AIK sign
        let signature = aik.sign(&sign_data).map_err(QuoteError::AikError)?;
        Ok(QuoteToken {
            signature,
            pcr_values,
            nonce: *nonce,
            timestamp,
            event_log_hash,
        })
    }

    /// Generate a full attestation response.
    ///
    /// Returns: Quote + Event Log + AIK certificate
    pub fn attest<T: TpmAbi>(
        &self,
        tpm: &T,
        aik: &AikManager,
        event_log: &EventLogManager,
        nonce: &[u8; NONCE_SIZE],
    ) -> Result<AttestationResponse, QuoteError> {
        let quote = self.generate_quote(tpm, aik, event_log, nonce)?;
        let aik_public_key_hash = *aik.public_key_hash();
        Ok(AttestationResponse {
            event_log: event_log.events().clone(),
            aik_public_key_hash,
            quote,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_quote_engine_new() {
        let engine = QuoteEngine::new();
        assert_eq!(engine.quote_count.load(Ordering::Relaxed), 0);
    }
}
