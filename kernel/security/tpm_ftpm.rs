/*
 * Nuva OS - Kernel - Firmware TPM (fTPM)
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

//! Firmware TPM (fTPM) Software Fallback
//!
//! Provides a software-emulated TPM when no hardware TPM is available.
//! PCR registers are stored in SpinLock-protected memory with SHA-256
//! extend semantics: PCR_new = SHA256(PCR_old || measurement).
//!
//! This replaces the FNV-1a placeholder hash used in the original
//! measured_boot implementation with proper SHA-256 PCR extension.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use spin::Mutex as SpinLock;

use super::tpm_abi::{TpmAbi, TpmError, TpmProviderType, TpmResult, PcrIndex, PCR_DIGEST_SIZE, DEFAULT_PCR_COUNT};
use super::sha256::sha256_digest;
use crate::{pr_info, pr_debug};

/// Number of PCR registers (24 for TPM 2.0 default)
const FTPM_PCR_COUNT: usize = DEFAULT_PCR_COUNT as usize;

/// PCR register bank: 24 PCR registers, each holding a SHA-256 digest
type PcrBank = [[u8; PCR_DIGEST_SIZE]; FTPM_PCR_COUNT];

/// Firmware TPM state (protected by SpinLock)
struct FirmwareTpmState {
    /// PCR register bank
    pcrs: PcrBank,
    /// PRNG state for get_random (xoshiro256++)
    prng_state: [u64; 4],
}

impl FirmwareTpmState {
    const fn new() -> Self {
        FirmwareTpmState {
            pcrs: [[0u8; PCR_DIGEST_SIZE]; FTPM_PCR_COUNT],
            prng_state: [0x0123456789ABCDEF, 0xFEDCBA9876543210,
                        0x89ABCDEF01234567, 0x76543210FEDCBA98],
        }
    }
}

/// Firmware TPM (fTPM) software implementation
///
/// Provides TPM-like functionality in software when no hardware
/// TPM is available. PCR registers use SHA-256 extend semantics.
/// All state is protected by a SpinLock for concurrent access.
pub struct FirmwareTpm {
    /// Internal state protected by spinlock
    state: SpinLock<FirmwareTpmState>,
    /// Initialized flag
    initialized: AtomicBool,
    /// Extend operation count
    extend_count: AtomicU64,
}

impl FirmwareTpm {
    /// Create a new firmware TPM instance
    pub const fn new() -> Self {
        FirmwareTpm {
            state: SpinLock::new(FirmwareTpmState::new()),
            initialized: AtomicBool::new(false),
            extend_count: AtomicU64::new(0),
        }
    }

    /// Initialize the firmware TPM
    pub fn init(&self) -> TpmResult<()> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }
        // Initialize all PCR registers to zero
        { let mut state = self.state.lock(); state.pcrs = [[0u8; PCR_DIGEST_SIZE]; FTPM_PCR_COUNT]; }
        self.initialized.store(true, Ordering::Release);
        Ok(())
    }

    /// xoshiro256++ PRNG step
    fn xoshiro256pp(s: &mut [u64; 4]) -> u64 {
        let result = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1].wrapping_shl(17);
        s[2] ^= s[0]; s[3] ^= s[1]; s[1] ^= s[2]; s[0] ^= s[3];
        s[2] = t;
        s[3] = s[3].rotate_left(45);
        result
    }
}

impl TpmAbi for FirmwareTpm {
    fn pcr_extend(&mut self, pcr_index: PcrIndex, measurement: &[u8; PCR_DIGEST_SIZE]) -> TpmResult<()> {
        if pcr_index as usize >= FTPM_PCR_COUNT { return Err(TpmError::InvalidPcrIndex); }
        if !self.initialized.load(Ordering::Acquire) { return Err(TpmError::BadState); }
        let idx = pcr_index as usize;
        let mut state = self.state.lock();
        // SHA-256 PCR extend: PCR_new = SHA256(PCR_old || measurement)
        let mut concat = [0u8; PCR_DIGEST_SIZE * 2];
        concat[..PCR_DIGEST_SIZE].copy_from_slice(&state.pcrs[idx]);
        concat[PCR_DIGEST_SIZE..].copy_from_slice(measurement);
        state.pcrs[idx] = sha256_digest(&concat);
        drop(state);
        self.extend_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn pcr_read(&self, pcr_index: PcrIndex) -> TpmResult<[u8; PCR_DIGEST_SIZE]> {
        if pcr_index as usize >= FTPM_PCR_COUNT { return Err(TpmError::InvalidPcrIndex); }
        if !self.initialized.load(Ordering::Acquire) { return Err(TpmError::BadState); }
        let state = self.state.lock();
        Ok(state.pcrs[pcr_index as usize])
    }

    fn pcr_read_multiple(&self, indices: &[PcrIndex]) -> TpmResult<Vec<(PcrIndex, [u8; PCR_DIGEST_SIZE])>> {
        let mut results = Vec::with_capacity(indices.len());
        for &idx in indices {
            let digest = self.pcr_read(idx)?;
            results.push((idx, digest));
        }
        Ok(results)
    }

    fn pcr_count(&self) -> u32 { DEFAULT_PCR_COUNT }
    fn is_available(&self) -> bool { self.initialized.load(Ordering::Acquire) }
    fn provider_type(&self) -> TpmProviderType { TpmProviderType::Firmware }

    fn get_random(&mut self, buf: &mut [u8]) -> TpmResult<()> {
        if !self.initialized.load(Ordering::Acquire) { return Err(TpmError::BadState); }
        let mut state = self.state.lock();
        for chunk in buf.chunks_mut(8) {
            let val = Self::xoshiro256pp(&mut state.prng_state);
            let src = val.to_le_bytes();
            let copy_len = chunk.len().min(8);
            chunk[..copy_len].copy_from_slice(&src[..copy_len]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_firmware_tpm_new() {
        let tpm = FirmwareTpm::new();
        assert!(!tpm.is_available());
        assert_eq!(tpm.pcr_count(), DEFAULT_PCR_COUNT);
        assert_eq!(tpm.provider_type(), TpmProviderType::Firmware);
    }
    #[test]
    fn test_firmware_tpm_init() {
        let tpm = FirmwareTpm::new();
        assert!(tpm.init().is_ok());
        assert!(tpm.is_available());
    }
    #[test]
    fn test_firmware_tpm_pcr_extend_read() {
        let mut tpm = FirmwareTpm::new();
        tpm.init().unwrap();
        let measurement = [0xABu8; PCR_DIGEST_SIZE];
        assert!(tpm.pcr_extend(0, &measurement).is_ok());
        let pcr_val = tpm.pcr_read(0).unwrap();
        // PCR should no longer be all zeros after extend
        assert_ne!(pcr_val, [0u8; PCR_DIGEST_SIZE]);
    }
    #[test]
    fn test_firmware_tpm_invalid_pcr() {
        let mut tpm = FirmwareTpm::new();
        tpm.init().unwrap();
        let measurement = [0u8; PCR_DIGEST_SIZE];
        assert_eq!(tpm.pcr_extend(DEFAULT_PCR_COUNT, &measurement), Err(TpmError::InvalidPcrIndex));
    }
}
