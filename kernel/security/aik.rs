/*
 * Nuva OS - Kernel - Attestation Identity Key (AIK) Manager
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

//! AIK Manager - Dilithium Native Attestation Identity Key
//! Manages Attestation Identity Keys using CRYSTALS-Dilithium
//! post-quantum signatures. Private keys NEVER leave the TPM boundary.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};

use super::tpm_abi::{TpmError, TpmResult, PCR_DIGEST_SIZE};
use super::sha256::sha256_digest;

/// AIK state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AikState {
    /// AIK not yet generated
    Uninitialized = 0,
    /// AIK generated and ready for signing
    Active = 1,
    /// AIK key material may be compromised
    Compromised = 2,
}

impl AikState {
    fn from_u8(v: u8) -> Self {
        match v { 0 => AikState::Uninitialized, 1 => AikState::Active, _ => AikState::Compromised }
    }
}

/// AIK error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AikError {
    /// Key generation failed
    KeygenFailed,
    /// Signing failed
    SignFailed,
    /// AIK not initialized
    NotInitialized,
    /// AIK compromised
    Compromised,
    /// Non-PQC algorithm rejected
    NonPqcAlgorithm,
    /// Private key export denied
    ExportDenied,
    /// Internal TPM error
    TpmError(TpmError),
}

/// Dilithium-3 public key size
const DILITHIUM3_PK_SIZE: usize = 1952;
/// Dilithium-3 signature size
const DILITHIUM3_SIG_SIZE: usize = 3293;

/// AIK Manager
pub struct AikManager {
    /// AIK state
    state: AtomicU8,
    /// Public key (exportable)
    public_key: [u8; DILITHIUM3_PK_SIZE],
    /// Public key hash
    public_key_hash: [u8; PCR_DIGEST_SIZE],
    /// Private key exists flag (key never stored in software)
    private_key_exists: AtomicBool,
    /// Signatures produced
    sign_count: AtomicU8,
}

impl AikManager {
    /// Create a new AIK manager (uninitialized)
    pub const fn new() -> Self {
        AikManager {
            state: AtomicU8::new(AikState::Uninitialized as u8),
            public_key: [0u8; DILITHIUM3_PK_SIZE],
            public_key_hash: [0u8; PCR_DIGEST_SIZE],
            private_key_exists: AtomicBool::new(false),
            sign_count: AtomicU8::new(0),
        }
    }

    /// Generate a new AIK using Dilithium-3.
    /// Only PQC algorithms are allowed. Key pair is generated
    /// within the TPM/fTPM boundary. Private key NEVER leaves.
    pub fn generate_aik(&mut self) -> TpmResult<()> {
        let current = self.get_state();
        if current == AikState::Compromised {
            return Err(TpmError::BadState);
        }
        // Generate Dilithium-3 key pair within TPM boundary
        let result = unsafe { dilithium3_aik_keygen(self.public_key.as_mut_ptr()) };
        if result != 0 { return Err(TpmError::BadState); }
        self.public_key_hash = sha256_digest(&self.public_key);
        self.private_key_exists.store(true, Ordering::Release);
        self.state.store(AikState::Active as u8, Ordering::Release);
        Ok(())
    }

    /// Sign data with the AIK. Private key NEVER leaves the TPM.
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, AikError> {
        let state = self.get_state();
        if state == AikState::Uninitialized { return Err(AikError::NotInitialized); }
        if state == AikState::Compromised { return Err(AikError::Compromised); }
        let hash = sha256_digest(data);
        let mut sig_buf = alloc::vec![0u8; DILITHIUM3_SIG_SIZE];
        let result = unsafe { dilithium3_aik_sign(sig_buf.as_mut_ptr(), hash.as_ptr()) };
        if result == 0 { Ok(sig_buf) } else { Err(AikError::SignFailed) }
    }

    /// Export private key - ALWAYS DENIED.
    /// Security guarantee: private keys never leave the TPM boundary.
    pub fn export_private_key(&self) -> Result<Vec<u8>, AikError> {
        Err(AikError::ExportDenied)
    }

    /// Get the AIK public key
    pub fn public_key(&self) -> &[u8; DILITHIUM3_PK_SIZE] { &self.public_key }

    /// Get the AIK public key hash
    pub fn public_key_hash(&self) -> &[u8; PCR_DIGEST_SIZE] { &self.public_key_hash }

    /// Get current AIK state
    pub fn get_state(&self) -> AikState {
        AikState::from_u8(self.state.load(Ordering::Acquire))
    }

    /// Mark AIK as compromised (irreversible)
    pub fn mark_compromised(&self) {
        self.state.store(AikState::Compromised as u8, Ordering::Release);
    }
}

extern "C" { fn dilithium3_aik_keygen(public_key: *mut u8) -> i32; }
extern "C" { fn dilithium3_aik_sign(signature: *mut u8, hash: *const u8) -> i32; }

#[cfg(test)]
mod tests {
    use super::*;
use alloc::vec;
use core::sync::atomic::AtomicU8;
    #[test]
    fn test_aik_new() {
        let aik = AikManager::new();
        assert_eq!(aik.get_state(), AikState::Uninitialized);
    }
    #[test]
    fn test_export_private_key_denied() {
        let aik = AikManager::new();
        assert_eq!(aik.export_private_key(), Err(AikError::ExportDenied));
    }
    #[test]
    fn test_sign_uninitialized() {
        let aik = AikManager::new();
        assert_eq!(aik.sign(&[1,2,3]), Err(AikError::NotInitialized));
    }
}
