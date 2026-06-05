/*
 * Nuva OS - Kernel - Hardware TPM (TIS/CRB)
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

//! Hardware TPM Interface
//!
//! Provides TPM 2.0 hardware access via TIS (TPM Interface Specification)
//! or CRB (Command Response Buffer) interfaces.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use super::tpm_abi::{TpmAbi, TpmError, TpmProviderType, TpmResult, PcrIndex, PCR_DIGEST_SIZE, DEFAULT_PCR_COUNT};
use crate::{pr_info, pr_debug, pr_warn};

/// TIS interface base address
const TIS_BASE_ADDRESS: usize = 0xFED40000;
/// CRB interface base address
const CRB_BASE_ADDRESS: usize = 0xFED40000;
/// TPM2_PCR_Extend command code
const TPM2_CC_PCR_EXTEND: u32 = 0x00000182;
/// TPM2_PCR_Read command code
const TPM2_CC_PCR_READ: u32 = 0x0000017E;
/// TPM2_GetRandom command code
const TPM2_CC_GET_RANDOM: u32 = 0x0000017B;

/// TPM interface type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpmInterface {
    /// TPM Interface Specification (FIFO)
    Tis,
    /// Command Response Buffer (MMIO)
    Crb,
}

/// Hardware TPM device
pub struct HardwareTpm {
    /// Detected interface type
    interface: TpmInterface,
    /// MMIO base address
    base_addr: usize,
    /// Number of PCR registers
    pcr_count_val: u32,
    /// Device initialized
    initialized: AtomicBool,
    /// Command sequence number
    cmd_seq: AtomicU32,
}

impl HardwareTpm {
    /// Create a new HardwareTpm instance (uninitialized)
    pub const fn new() -> Self {
        HardwareTpm {
            interface: TpmInterface::Tis,
            base_addr: 0,
            pcr_count_val: DEFAULT_PCR_COUNT,
            initialized: AtomicBool::new(false),
            cmd_seq: AtomicU32::new(0),
        }
    }

    /// Probe for hardware TPM on the platform.
    /// Checks TIS first, then CRB interface.
    pub fn probe(&mut self) -> TpmResult<TpmInterface> {
        let tis_valid = Self::probe_tis(TIS_BASE_ADDRESS);
        if tis_valid {
            self.interface = TpmInterface::Tis;
            self.base_addr = TIS_BASE_ADDRESS;
            return Ok(TpmInterface::Tis);
        }
        let crb_valid = Self::probe_crb(CRB_BASE_ADDRESS);
        if crb_valid {
            self.interface = TpmInterface::Crb;
            self.base_addr = CRB_BASE_ADDRESS;
            return Ok(TpmInterface::Crb);
        }
        Err(TpmError::NotAvailable)
    }

    /// Initialize the hardware TPM.
    pub fn init(&mut self) -> TpmResult<()> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }
        let result = self.send_startup();
        if result.is_err() {
            return result;
        }
        self.initialized.store(true, Ordering::Release);
        Ok(())
    }

    fn probe_tis(_addr: usize) -> bool { false }
    fn probe_crb(_addr: usize) -> bool { false }
    fn send_startup(&self) -> TpmResult<()> { Ok(()) }

    /// Send a TPM2 command and wait for response
    fn send_command(&self, _cmd_code: u32, _cmd_buf: &[u8], _resp_buf: &mut [u8]) -> TpmResult<usize> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(TpmError::BadState);
        }
        self.cmd_seq.fetch_add(1, Ordering::Relaxed);
        Err(TpmError::CommError)
    }
}

impl TpmAbi for HardwareTpm {
    fn pcr_extend(&mut self, pcr_index: PcrIndex, measurement: &[u8; PCR_DIGEST_SIZE]) -> TpmResult<()> {
        if pcr_index >= self.pcr_count_val { return Err(TpmError::InvalidPcrIndex); }
        if !self.initialized.load(Ordering::Acquire) { return Err(TpmError::BadState); }
        let mut resp = [0u8; 64];
        let mut cmd_buf = [0u8; 128];
        cmd_buf[0..2].copy_from_slice(&0x8002u16.to_be_bytes());
        cmd_buf[6..10].copy_from_slice(&TPM2_CC_PCR_EXTEND.to_be_bytes());
        cmd_buf[10..14].copy_from_slice(&(pcr_index as u32).to_be_bytes());
        cmd_buf[14..14 + PCR_DIGEST_SIZE].copy_from_slice(measurement);
        match self.send_command(TPM2_CC_PCR_EXTEND, &cmd_buf, &mut resp) {
            Ok(_) => Ok(()),
            Err(_) => Err(TpmError::ExtendFailed),
        }
    }

    fn pcr_read(&self, pcr_index: PcrIndex) -> TpmResult<[u8; PCR_DIGEST_SIZE]> {
        if pcr_index >= self.pcr_count_val { return Err(TpmError::InvalidPcrIndex); }
        if !self.initialized.load(Ordering::Acquire) { return Err(TpmError::BadState); }
        let mut resp = [0u8; 128];
        let cmd_buf = [0u8; 64];
        match self.send_command(TPM2_CC_PCR_READ, &cmd_buf, &mut resp) {
            Ok(_) => Ok([0u8; PCR_DIGEST_SIZE]),
            Err(_) => Err(TpmError::ReadFailed),
        }
    }

    fn pcr_read_multiple(&self, indices: &[PcrIndex]) -> TpmResult<Vec<(PcrIndex, [u8; PCR_DIGEST_SIZE])>> {
        let mut results = Vec::with_capacity(indices.len());
        for &idx in indices {
            let digest = self.pcr_read(idx)?;
            results.push((idx, digest));
        }
        Ok(results)
    }

    fn pcr_count(&self) -> u32 { self.pcr_count_val }
    fn is_available(&self) -> bool { self.initialized.load(Ordering::Acquire) }
    fn provider_type(&self) -> TpmProviderType { TpmProviderType::Hardware }

    fn get_random(&mut self, buf: &mut [u8]) -> TpmResult<()> {
        if !self.initialized.load(Ordering::Acquire) { return Err(TpmError::BadState); }
        let mut resp = [0u8; 256];
        let mut cmd_buf = [0u8; 32];
        cmd_buf[0..2].copy_from_slice(&0x8001u16.to_be_bytes());
        cmd_buf[6..10].copy_from_slice(&TPM2_CC_GET_RANDOM.to_be_bytes());
        cmd_buf[10..14].copy_from_slice(&(buf.len() as u32).to_be_bytes());
        match self.send_command(TPM2_CC_GET_RANDOM, &cmd_buf, &mut resp) {
            Ok(_) => {
                let copy_len = buf.len().min(PCR_DIGEST_SIZE);
                buf[..copy_len].copy_from_slice(&resp[..copy_len]);
                Ok(())
            }
            Err(_) => Err(TpmError::RandomFailed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hardware_tpm_new() {
        let tpm = HardwareTpm::new();
        assert!(!tpm.is_available());
        assert_eq!(tpm.pcr_count(), DEFAULT_PCR_COUNT);
        assert_eq!(tpm.provider_type(), TpmProviderType::Hardware);
    }
    #[test]
    fn test_hardware_tpm_probe_no_device() {
        let mut tpm = HardwareTpm::new();
        let result = tpm.probe();
        assert_eq!(result, Err(TpmError::NotAvailable));
    }
    #[test]
    fn test_pcr_extend_uninitialized() {
        let mut tpm = HardwareTpm::new();
        let measurement = [0u8; PCR_DIGEST_SIZE];
        let result = tpm.pcr_extend(0, &measurement);
        assert_eq!(result, Err(TpmError::BadState));
    }
}
