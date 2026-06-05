/*
 * Nuva OS - Kernel - TPM Abstract Interface
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

//! TPM Abstract Interface Layer
//!
//! Provides a hardware-agnostic trait for TPM operations including
//! PCR extend/read, random number generation, and provider detection.
//! Both hardware TPM (TIS/CRB) and firmware TPM (fTPM) implement
//! this trait.

use alloc::vec::Vec;

/// PCR index type
pub type PcrIndex = u32;

/// SHA-256 digest size in bytes
pub const PCR_DIGEST_SIZE: usize = 32;

/// Default number of PCR registers (TPM 2.0 default)
pub const DEFAULT_PCR_COUNT: u32 = 24;

/// TPM provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpmProviderType {
    /// Hardware TPM chip (discrete TPM via TIS/CRB interface)
    Hardware = 0,
    /// Firmware TPM (software-emulated in secure memory)
    Firmware = 1,
}

/// TPM error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpmError {
    /// TPM not available or not detected
    NotAvailable,
    /// Invalid PCR index
    InvalidPcrIndex,
    /// PCR extend operation failed
    ExtendFailed,
    /// PCR read operation failed
    ReadFailed,
    /// TPM initialization failed
    InitFailed,
    /// TPM command timeout
    Timeout,
    /// Random number generation failed
    RandomFailed,
    /// Invalid parameter
    InvalidParam,
    /// TPM is in a bad state
    BadState,
    /// Insufficient buffer space
    BufferOverflow,
    /// Hardware communication error
    CommError,
    /// Unsupported operation
    Unsupported,
}

/// TPM result type alias
pub type TpmResult<T> = Result<T, TpmError>;

/// TPM abstract interface trait
///
/// Defines the core operations required for measured boot and
/// remote attestation. Both hardware and firmware TPMs implement
/// this trait, allowing transparent fallback from HW TPM to fTPM.
pub trait TpmAbi {
    /// Extend a PCR register with a new measurement digest.
    ///
    /// The TPM extend operation computes: PCR_new = SHA256(PCR_old || measurement)
    /// This is the fundamental operation for measured boot.
    fn pcr_extend(&mut self, pcr_index: PcrIndex, measurement: &[u8; PCR_DIGEST_SIZE]) -> TpmResult<()>;

    /// Read the current value of a PCR register.
    fn pcr_read(&self, pcr_index: PcrIndex) -> TpmResult<[u8; PCR_DIGEST_SIZE]>;

    /// Read multiple PCR registers at once.
    fn pcr_read_multiple(&self, indices: &[PcrIndex]) -> TpmResult<Vec<(PcrIndex, [u8; PCR_DIGEST_SIZE])>>;

    /// Get the number of available PCR registers.
    fn pcr_count(&self) -> u32;

    /// Check if the TPM provider is available and operational.
    fn is_available(&self) -> bool;

    /// Get the provider type (hardware or firmware).
    fn provider_type(&self) -> TpmProviderType;

    /// Generate random bytes from the TPM entropy source.
    fn get_random(&mut self, buf: &mut [u8]) -> TpmResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcr_index_type() {
        let idx: PcrIndex = 7;
        assert_eq!(idx, 7u32);
    }

    #[test]
    fn test_tpm_provider_type_values() {
        assert_eq!(TpmProviderType::Hardware as u32, 0);
        assert_eq!(TpmProviderType::Firmware as u32, 1);
    }

    #[test]
    fn test_tpm_error_distinct() {
        let errors = [
            TpmError::NotAvailable,
            TpmError::InvalidPcrIndex,
            TpmError::ExtendFailed,
            TpmError::ReadFailed,
            TpmError::InitFailed,
            TpmError::Timeout,
            TpmError::RandomFailed,
            TpmError::InvalidParam,
            TpmError::BadState,
            TpmError::BufferOverflow,
            TpmError::CommError,
            TpmError::Unsupported,
        ];
        for i in 0..errors.len() {
            for j in (i + 1)..errors.len() {
                assert_ne!(errors[i], errors[j]);
            }
        }
    }
}
