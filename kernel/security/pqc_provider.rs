/*
 * Nuva OS - Kernel - PQC Provider Abstraction
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

//! PQC Provider Abstraction Layer
//! Unified interface for post-quantum cryptographic operations,
//! supporting both HAL FFI and software reference implementations.

use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// PQC algorithm identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcAlgorithm {
    Kyber512,
    Kyber768,
    Kyber1024,
    Dilithium2,
    Dilithium3,
    Dilithium5,
}

impl PqcAlgorithm {
    pub fn is_kem(&self) -> bool {
        matches!(self, PqcAlgorithm::Kyber512 | PqcAlgorithm::Kyber768 | PqcAlgorithm::Kyber1024)
    }

    pub fn is_signature(&self) -> bool {
        matches!(self, PqcAlgorithm::Dilithium2 | PqcAlgorithm::Dilithium3 | PqcAlgorithm::Dilithium5)
    }

    pub fn public_key_size(&self) -> usize {
        match self {
            PqcAlgorithm::Kyber512 => 800,
            PqcAlgorithm::Kyber768 => 1184,
            PqcAlgorithm::Kyber1024 => 1568,
            PqcAlgorithm::Dilithium2 => 1312,
            PqcAlgorithm::Dilithium3 => 1952,
            PqcAlgorithm::Dilithium5 => 2592,
        }
    }

    pub fn secret_key_size(&self) -> usize {
        match self {
            PqcAlgorithm::Kyber512 => 1632,
            PqcAlgorithm::Kyber768 => 2400,
            PqcAlgorithm::Kyber1024 => 3168,
            PqcAlgorithm::Dilithium2 => 2528,
            PqcAlgorithm::Dilithium3 => 4000,
            PqcAlgorithm::Dilithium5 => 4864,
        }
    }

    pub fn ciphertext_size(&self) -> usize {
        match self {
            PqcAlgorithm::Kyber512 => 768,
            PqcAlgorithm::Kyber768 => 1088,
            PqcAlgorithm::Kyber1024 => 1568,
            _ => 0,
        }
    }

    pub fn signature_size(&self) -> usize {
        match self {
            PqcAlgorithm::Dilithium2 => 2420,
            PqcAlgorithm::Dilithium3 => 3293,
            PqcAlgorithm::Dilithium5 => 4595,
            _ => 0,
        }
    }

    pub fn nist_level(&self) -> u8 {
        match self {
            PqcAlgorithm::Kyber512 | PqcAlgorithm::Dilithium2 => 1,
            PqcAlgorithm::Kyber768 | PqcAlgorithm::Dilithium3 => 3,
            PqcAlgorithm::Kyber1024 | PqcAlgorithm::Dilithium5 => 5,
        }
    }

    pub fn fips_standard(&self) -> &'static str {
        if self.is_kem() { "FIPS 203 (ML-KEM)" } else { "FIPS 204 (ML-DSA)" }
    }
}

/// PQC operation error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcError {
    KeyGenFailed,
    EncapsFailed,
    DecapsFailed,
    SignFailed,
    VerifyFailed,
    InvalidKey,
    InvalidCiphertext,
    InvalidSignature,
    AlgorithmNotSupported,
    BufferTooSmall,
    InternalError,
}

/// PQC result type
pub type PqcResult<T> = Result<T, PqcError>;

/// PQC key pair with automatic zeroization on drop
pub struct PqcKeyPair {
    pub algorithm: PqcAlgorithm,
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

impl PqcKeyPair {
    pub fn zeroize(&mut self) {
        for b in self.secret_key.iter_mut() {
            unsafe { core::ptr::write_volatile(b, 0) };
        }
    }
}

impl Drop for PqcKeyPair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// KEM encapsulation result with automatic zeroization
pub struct KemEncapResult {
    pub ciphertext: Vec<u8>,
    pub shared_secret: Vec<u8>,
}

impl KemEncapResult {
    pub fn zeroize(&mut self) {
        for b in self.shared_secret.iter_mut() {
            unsafe { core::ptr::write_volatile(b, 0) };
        }
    }
}

impl Drop for KemEncapResult {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// PQC Provider trait - unified interface for all PQC operations
pub trait PqcProvider: Send + Sync {
    fn keygen(&self, algo: PqcAlgorithm) -> PqcResult<PqcKeyPair>;
    fn encaps(&self, algo: PqcAlgorithm, pk: &[u8]) -> PqcResult<KemEncapResult>;
    fn decaps(&self, algo: PqcAlgorithm, sk: &[u8], ct: &[u8]) -> PqcResult<Vec<u8>>;
    fn sign(&self, algo: PqcAlgorithm, sk: &[u8], msg: &[u8]) -> PqcResult<Vec<u8>>;
    fn verify(&self, algo: PqcAlgorithm, pk: &[u8], msg: &[u8], sig: &[u8]) -> PqcResult<bool>;
    fn provider_name(&self) -> &'static str;
}

/// HAL FFI-based PQC provider (primary)
pub struct HalPqcProvider {
    ops_count: AtomicU64,
}

impl HalPqcProvider {
    pub const fn new() -> Self {
        Self { ops_count: AtomicU64::new(0) }
    }
}

impl PqcProvider for HalPqcProvider {
    fn keygen(&self, algo: PqcAlgorithm) -> PqcResult<PqcKeyPair> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let pk_size = algo.public_key_size();
        let sk_size = algo.secret_key_size();
        let mut pk = vec![0u8; pk_size];
        let mut sk = vec![0u8; sk_size];
        let ret = unsafe {
            pqc_hal_keygen(algo as u32, pk.as_mut_ptr(), pk.len(), sk.as_mut_ptr(), sk.len())
        };
        if ret != 0 { return Err(PqcError::KeyGenFailed); }
        Ok(PqcKeyPair { algorithm: algo, public_key: pk, secret_key: sk })
    }

    fn encaps(&self, algo: PqcAlgorithm, pk: &[u8]) -> PqcResult<KemEncapResult> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let ct_size = algo.ciphertext_size();
        let mut ct = vec![0u8; ct_size];
        let mut ss = vec![0u8; 32];
        let ret = unsafe {
            pqc_hal_encaps(algo as u32, pk.as_ptr(), pk.len(), ct.as_mut_ptr(), ct.len(), ss.as_mut_ptr())
        };
        if ret != 0 { return Err(PqcError::EncapsFailed); }
        Ok(KemEncapResult { ciphertext: ct, shared_secret: ss })
    }

    fn decaps(&self, algo: PqcAlgorithm, sk: &[u8], ct: &[u8]) -> PqcResult<Vec<u8>> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let mut ss = vec![0u8; 32];
        let ret = unsafe {
            pqc_hal_decaps(algo as u32, sk.as_ptr(), sk.len(), ct.as_ptr(), ct.len(), ss.as_mut_ptr())
        };
        if ret != 0 { return Err(PqcError::DecapsFailed); }
        Ok(ss)
    }

    fn sign(&self, algo: PqcAlgorithm, sk: &[u8], msg: &[u8]) -> PqcResult<Vec<u8>> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let sig_size = algo.signature_size();
        let mut sig = vec![0u8; sig_size];
        let mut sig_len: usize = sig_size;
        let ret = unsafe {
            pqc_hal_sign(algo as u32, sk.as_ptr(), sk.len(), msg.as_ptr(), msg.len(), sig.as_mut_ptr(), &mut sig_len)
        };
        if ret != 0 { return Err(PqcError::SignFailed); }
        sig.truncate(sig_len);
        Ok(sig)
    }

    fn verify(&self, algo: PqcAlgorithm, pk: &[u8], msg: &[u8], sig: &[u8]) -> PqcResult<bool> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let ret = unsafe {
            pqc_hal_verify(algo as u32, pk.as_ptr(), pk.len(), msg.as_ptr(), msg.len(), sig.as_ptr(), sig.len())
        };
        Ok(ret == 0)
    }

    fn provider_name(&self) -> &'static str { "hal_ffi" }
}

/// Software reference PQC provider (fallback)
pub struct SoftwarePqcProvider {
    ops_count: AtomicU64,
}

impl SoftwarePqcProvider {
    pub const fn new() -> Self {
        Self { ops_count: AtomicU64::new(0) }
    }
}

impl PqcProvider for SoftwarePqcProvider {
    fn keygen(&self, algo: PqcAlgorithm) -> PqcResult<PqcKeyPair> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let pk_size = algo.public_key_size();
        let sk_size = algo.secret_key_size();
        let mut pk = vec![0u8; pk_size];
        let mut sk = vec![0u8; sk_size];
        let ret = unsafe {
            pqc_soft_keygen(algo as u32, pk.as_mut_ptr(), pk.len(), sk.as_mut_ptr(), sk.len())
        };
        if ret != 0 { return Err(PqcError::KeyGenFailed); }
        Ok(PqcKeyPair { algorithm: algo, public_key: pk, secret_key: sk })
    }

    fn encaps(&self, algo: PqcAlgorithm, pk: &[u8]) -> PqcResult<KemEncapResult> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let ct_size = algo.ciphertext_size();
        let mut ct = vec![0u8; ct_size];
        let mut ss = vec![0u8; 32];
        let ret = unsafe {
            pqc_soft_encaps(algo as u32, pk.as_ptr(), pk.len(), ct.as_mut_ptr(), ct.len(), ss.as_mut_ptr())
        };
        if ret != 0 { return Err(PqcError::EncapsFailed); }
        Ok(KemEncapResult { ciphertext: ct, shared_secret: ss })
    }

    fn decaps(&self, algo: PqcAlgorithm, sk: &[u8], ct: &[u8]) -> PqcResult<Vec<u8>> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let mut ss = vec![0u8; 32];
        let ret = unsafe {
            pqc_soft_decaps(algo as u32, sk.as_ptr(), sk.len(), ct.as_ptr(), ct.len(), ss.as_mut_ptr())
        };
        if ret != 0 { return Err(PqcError::DecapsFailed); }
        Ok(ss)
    }

    fn sign(&self, algo: PqcAlgorithm, sk: &[u8], msg: &[u8]) -> PqcResult<Vec<u8>> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let sig_size = algo.signature_size();
        let mut sig = vec![0u8; sig_size];
        let mut sig_len: usize = sig_size;
        let ret = unsafe {
            pqc_soft_sign(algo as u32, sk.as_ptr(), sk.len(), msg.as_ptr(), msg.len(), sig.as_mut_ptr(), &mut sig_len)
        };
        if ret != 0 { return Err(PqcError::SignFailed); }
        sig.truncate(sig_len);
        Ok(sig)
    }

    fn verify(&self, algo: PqcAlgorithm, pk: &[u8], msg: &[u8], sig: &[u8]) -> PqcResult<bool> {
        self.ops_count.fetch_add(1, Ordering::Relaxed);
        let ret = unsafe {
            pqc_soft_verify(algo as u32, pk.as_ptr(), pk.len(), msg.as_ptr(), msg.len(), sig.as_ptr(), sig.len())
        };
        Ok(ret == 0)
    }

    fn provider_name(&self) -> &'static str { "software_ref" }
}

extern "C" {
    fn pqc_hal_keygen(algo: u32, pk: *mut u8, pk_len: usize, sk: *mut u8, sk_len: usize) -> i32;
    fn pqc_hal_encaps(algo: u32, pk: *const u8, pk_len: usize, ct: *mut u8, ct_len: usize, ss: *mut u8) -> i32;
    fn pqc_hal_decaps(algo: u32, sk: *const u8, sk_len: usize, ct: *const u8, ct_len: usize, ss: *mut u8) -> i32;
    fn pqc_hal_sign(algo: u32, sk: *const u8, sk_len: usize, msg: *const u8, msg_len: usize, sig: *mut u8, sig_len: *mut usize) -> i32;
    fn pqc_hal_verify(algo: u32, pk: *const u8, pk_len: usize, msg: *const u8, msg_len: usize, sig: *const u8, sig_len: usize) -> i32;
    fn pqc_soft_keygen(algo: u32, pk: *mut u8, pk_len: usize, sk: *mut u8, sk_len: usize) -> i32;
    fn pqc_soft_encaps(algo: u32, pk: *const u8, pk_len: usize, ct: *mut u8, ct_len: usize, ss: *mut u8) -> i32;
    fn pqc_soft_decaps(algo: u32, sk: *const u8, sk_len: usize, ct: *const u8, ct_len: usize, ss: *mut u8) -> i32;
    fn pqc_soft_sign(algo: u32, sk: *const u8, sk_len: usize, msg: *const u8, msg_len: usize, sig: *mut u8, sig_len: *mut usize) -> i32;
    fn pqc_soft_verify(algo: u32, pk: *const u8, pk_len: usize, msg: *const u8, msg_len: usize, sig: *const u8, sig_len: usize) -> i32;
}
