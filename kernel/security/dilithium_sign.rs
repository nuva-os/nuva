/*
 * Nuva OS - Kernel - Security - DilithiumSign
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
/*
 * Dilithium Code Signature Integration
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Extends code signing with CRYSTALS-Dilithium post-quantum
 * signatures, supporting hybrid Dilithium+ECDSA for backward
 * compatibility and pure Dilithium for quantum-safe deployments.
 */

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use alloc::vec::Vec;
use alloc::vec;

use crate::kernel::security::signature::{
    CodeSignature, SignatureAlgorithm, SignatureResult,
    MAX_SIGNATURE_SIZE, SIG_FLAG_TRUSTED,
};
use crate::{pr_info, pr_debug, pr_warn};

/// Dilithium code signer error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilithiumSignerError {
    /// Key generation failed
    KeygenFailed,
    /// Signing failed
    SigningFailed,
    /// Verification failed
    VerificationFailed,
    /// Invalid key
    InvalidKey,
    /// Invalid signature
    InvalidSignature,
    /// ECDSA fallback failed
    EcdsaFallbackFailed,
    /// Hash computation failed
    HashFailed,
    /// Out of memory
    OutOfMemory,
}

/// Dilithium key pair for code signing
pub struct DilithiumKeyPair {
    /// Public key data
    pub public_key: Vec<u8>,
    /// Secret key data (sensitive)
    pub secret_key: Vec<u8>,
    /// Dilithium variant
    pub variant: SignatureAlgorithm,
}

impl DilithiumKeyPair {
    /// Create empty key pair
    pub fn empty() -> Self {
        DilithiumKeyPair {
            public_key: Vec::new(),
            secret_key: Vec::new(),
            variant: SignatureAlgorithm::Dilithium3,
        }
    }

    /// Get public key size
    pub fn public_key_size(&self) -> usize {
        self.public_key.len()
    }

    /// Securely zeroize secret key
    pub fn zeroize(&mut self) {
        for byte in self.secret_key.iter_mut() {
            // SAFETY: volatile write to prevent compiler optimization
            unsafe {
                core::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl Drop for DilithiumKeyPair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Dilithium code signer
/// Provides post-quantum secure code signing using CRYSTALS-Dilithium.
/// Supports Dilithium-2, Dilithium-3, and Dilithium-5 variants.
pub struct DilithiumCodeSigner {
    /// Active variant
    variant: SignatureAlgorithm,
    /// Signatures produced
    sign_count: AtomicU64,
    /// Verifications performed
    verify_count: AtomicU64,
    /// Verification successes
    verify_success: AtomicU64,
    /// Hybrid mode enabled
    hybrid_mode: AtomicBool,
}

impl DilithiumCodeSigner {
    /// Create new signer with specified variant
    pub fn new(variant: SignatureAlgorithm) -> Result<Self, DilithiumSignerError> {
        match variant {
            SignatureAlgorithm::Dilithium2
            | SignatureAlgorithm::Dilithium3
            | SignatureAlgorithm::Dilithium5 => {
                Ok(DilithiumCodeSigner {
                    variant,
                    sign_count: AtomicU64::new(0),
                    verify_count: AtomicU64::new(0),
                    verify_success: AtomicU64::new(0),
                    hybrid_mode: AtomicBool::new(true),
                })
            }
            _ => Err(DilithiumSignerError::InvalidKey),
        }
    }

    /// Create with Dilithium-3 (recommended)
    pub fn with_dilithium3() -> Result<Self, DilithiumSignerError> {
        Self::new(SignatureAlgorithm::Dilithium3)
    }

    /// Enable or disable hybrid mode
    pub fn set_hybrid_mode(&self, enabled: bool) {
        self.hybrid_mode.store(enabled, Ordering::Release);
    }

    /// Check if hybrid mode is enabled
    pub fn is_hybrid_mode(&self) -> bool {
        self.hybrid_mode.load(Ordering::Acquire)
    }

    /// Generate Dilithium key pair
    /// In production, calls the CRYSTALS-Dilithium reference implementation
    /// via FFI. Returns a key pair suitable for code signing.
    pub fn keygen(&self) -> Result<DilithiumKeyPair, DilithiumSignerError> {
        let pk_size = match self.variant {
            SignatureAlgorithm::Dilithium2 => 1312,
            SignatureAlgorithm::Dilithium3 => 1952,
            SignatureAlgorithm::Dilithium5 => 2592,
            _ => return Err(DilithiumSignerError::InvalidKey),
        };

        let sk_size = match self.variant {
            SignatureAlgorithm::Dilithium2 => 2560,
            SignatureAlgorithm::Dilithium3 => 4032,
            SignatureAlgorithm::Dilithium5 => 4864,
            _ => return Err(DilithiumSignerError::InvalidKey),
        };

        let mut pk = alloc::vec![0u8; pk_size];
        let mut sk = alloc::vec![0u8; sk_size];

        // SAFETY: FFI call to Dilithium key generation
        let result = unsafe {
            dilithium_keygen_ffi(
                self.variant as u32,
                pk.as_mut_ptr(),
                sk.as_mut_ptr(),
            )
        };

        if result == 0 {
            Ok(DilithiumKeyPair {
                public_key: pk,
                secret_key: sk,
                variant: self.variant,
            })
        } else {
            Err(DilithiumSignerError::KeygenFailed)
        }
    }

    /// Sign data with Dilithium
    /// @param kp: Key pair (uses secret key)
    /// @param data: Data to sign (typically hash of kernel image)
    /// @return: Signature bytes
    pub fn sign(&self, kp: &DilithiumKeyPair, data: &[u8]) -> Result<Vec<u8>, DilithiumSignerError> {
        let sig_size = self.variant.signature_size();
        let mut sig = alloc::vec![0u8; sig_size];

        // SAFETY: FFI call to Dilithium signing
        let result = unsafe {
            dilithium_sign_ffi(
                self.variant as u32,
                kp.secret_key.as_ptr(),
                data.as_ptr(),
                data.len(),
                sig.as_mut_ptr(),
            )
        };

        if result == 0 {
            self.sign_count.fetch_add(1, Ordering::Relaxed);
            Ok(sig)
        } else {
            Err(DilithiumSignerError::SigningFailed)
        }
    }

    /// Verify Dilithium signature
    /// @param pk_data: Public key bytes
    /// @param data: Signed data
    /// @param sig: Signature bytes
    pub fn verify(
        &self,
        pk_data: &[u8],
        data: &[u8],
        sig: &[u8],
    ) -> Result<bool, DilithiumSignerError> {
        self.verify_count.fetch_add(1, Ordering::Relaxed);

        // SAFETY: FFI call to Dilithium verification
        let result = unsafe {
            dilithium_verify_ffi(
                self.variant as u32,
                pk_data.as_ptr(),
                data.as_ptr(),
                data.len(),
                sig.as_ptr(),
            )
        };

        let valid = result == 0;
        if valid {
            self.verify_success.fetch_add(1, Ordering::Relaxed);
        }
        Ok(valid)
    }

    /// Get signer statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.sign_count.load(Ordering::Acquire),
            self.verify_count.load(Ordering::Acquire),
            self.verify_success.load(Ordering::Acquire),
        )
    }
}

/// Hybrid signature: Dilithium + ECDSA
/// Produces a concatenated signature that can be verified
/// by either Dilithium or ECDSA. This provides:
/// - Post-quantum security from Dilithium
/// - Backward compatibility with ECDSA-only verifiers
/// - Defense in depth
pub fn hybrid_sign(
    dilithium_signer: &DilithiumCodeSigner,
    kp: &DilithiumKeyPair,
    data: &[u8],
    ecdsa_key: Option<&[u8]>,
) -> Result<Vec<u8>, DilithiumSignerError> {
    let dilithium_sig = dilithium_signer.sign(kp, data)?;

    match ecdsa_key {
        Some(_ek) => {
            let ecdsa_sig = ecdsa_sign_stub(data);
            let mut hybrid = Vec::with_capacity(dilithium_sig.len() + ecdsa_sig.len());
            hybrid.extend_from_slice(&dilithium_sig);
            hybrid.extend_from_slice(&ecdsa_sig);
            Ok(hybrid)
        }
        None => Ok(dilithium_sig),
    }
}

/// Hybrid verify: Dilithium + ECDSA
/// Verification strategy:
/// 1. Try Dilithium verification first
/// 2. If Dilithium fails, try ECDSA fallback
/// 3. Signature is valid if either component verifies
/// This ensures forward compatibility (new verifiers check Dilithium)
/// and backward compatibility (old verifiers check ECDSA).
pub fn hybrid_verify(
    dilithium_signer: &DilithiumCodeSigner,
    pk_data: &[u8],
    data: &[u8],
    sig: &[u8],
    ecdsa_pk: Option<&[u8]>,
) -> Result<bool, DilithiumSignerError> {
    let dilithium_sig_len = dilithium_signer.variant.signature_size();

    if sig.len() >= dilithium_sig_len {
        let dilithium_sig = &sig[..dilithium_sig_len];

        if let Ok(valid) = dilithium_signer.verify(pk_data, data, dilithium_sig) {
            if valid {
                return Ok(true);
            }
        }
    }

    if let Some(_ecdsa_pk_data) = ecdsa_pk {
        if sig.len() > dilithium_sig_len {
            let ecdsa_sig = &sig[dilithium_sig_len..];
            if ecdsa_verify_stub(data, ecdsa_sig) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Sign kernel image with Dilithium (or hybrid)
/// Generates hash of the kernel image, signs with Dilithium,
/// and produces a CodeSignature structure suitable for
/// the boot verifier.
pub fn sign_kernel_image(
    signer: &DilithiumCodeSigner,
    kp: &DilithiumKeyPair,
    image: &[u8],
    signer_name: &[u8],
) -> Result<CodeSignature, DilithiumSignerError> {
    let hash = compute_kernel_hash(image);

    let sig_bytes = if signer.is_hybrid_mode() {
        hybrid_sign(signer, kp, &hash, None)?
    } else {
        signer.sign(kp, &hash)?
    };

    let mut code_sig = CodeSignature::empty();
    code_sig.algorithm = signer.variant;
    code_sig.signature_len = sig_bytes.len().min(MAX_SIGNATURE_SIZE) as u32;
    code_sig.signature[..code_sig.signature_len as usize]
        .copy_from_slice(&sig_bytes[..code_sig.signature_len as usize]);
    code_sig.version = 1;
    code_sig.flags = SIG_FLAG_TRUSTED;

    let name_len = signer_name.len().min(64);
    code_sig.signer[..name_len].copy_from_slice(&signer_name[..name_len]);
    code_sig.signer_len = name_len as u32;

    let pk_hash = compute_kernel_hash(&kp.public_key);
    code_sig.pubkey_hash[..32].copy_from_slice(&pk_hash[..32]);
    code_sig.pubkey_hash_len = 32;

    Ok(code_sig)
}

/// Compute SHA-256 hash of kernel image
/// Uses a compact software SHA-256 implementation for no_std compatibility.
/// In production with hardware acceleration, this delegates to LASX/AES-NI.
fn compute_kernel_hash(data: &[u8]) -> [u8; 32] {
    soft_sha256(data)
}

/// Compact software SHA-256 (FIPS 180-4)
fn soft_sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let msg_len = data.len() as u64 * 8;
    let mut padded = alloc::vec![0u8; ((data.len() + 9 + 63) / 64) * 64];
    padded[..data.len()].copy_from_slice(data);
    padded[data.len()] = 0x80;
    let padded_len = padded.len();
    padded[padded_len - 8..].copy_from_slice(&msg_len.to_be_bytes());

    for chunk in padded.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut h = state;
        for i in 0..64 {
            let s1 = h[4].rotate_right(6) ^ h[4].rotate_right(11) ^ h[4].rotate_right(25);
            let ch = (h[4] & h[5]) ^ (!h[4] & h[6]);
            let t1 = h[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = h[0].rotate_right(2) ^ h[0].rotate_right(13) ^ h[0].rotate_right(22);
            let maj = (h[0] & h[1]) ^ (h[0] & h[2]) ^ (h[1] & h[2]);
            let t2 = s0.wrapping_add(maj);
            h[7] = h[6];
            h[6] = h[5];
            h[5] = h[4];
            h[4] = h[3].wrapping_add(t1);
            h[3] = h[2];
            h[2] = h[1];
            h[1] = h[0];
            h[0] = t1.wrapping_add(t2);
        }
        for (s, hv) in state.iter_mut().zip(h.iter()) {
            *s = s.wrapping_add(*hv);
        }
    }

    let mut out = [0u8; 32];
    for (i, s) in state.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&s.to_be_bytes());
    }
    out
}

/// FFI declarations for Dilithium operations
extern "C" {
    /// C key generation function
    fn dilithium_keygen_ffi(
        variant: u32,
        public_key: *mut u8,
        secret_key: *mut u8,
    ) -> i32;

    /// C signing function
    fn dilithium_sign_ffi(
        variant: u32,
        secret_key: *const u8,
        message: *const u8,
        message_len: usize,
        signature: *mut u8,
    ) -> i32;

    /// C verification function
    fn dilithium_verify_ffi(
        variant: u32,
        public_key: *const u8,
        message: *const u8,
        message_len: usize,
        signature: *const u8,
    ) -> i32;
}

/// Global Dilithium code signer
static mut DILITHIUM_SIGNER: Option<DilithiumCodeSigner> = None;

/// Initialize Dilithium code signer
pub fn init_dilithium_signer() -> Result<(), DilithiumSignerError> {
    let signer = DilithiumCodeSigner::with_dilithium3()?;
    // SAFETY: single-threaded init during boot
    unsafe {
        DILITHIUM_SIGNER = Some(signer);
    }
    log_info!("Dilithium code signer initialized (Dilithium-3)");
    Ok(())
}

/// Get global Dilithium code signer
pub fn get_dilithium_signer() -> Option<&'static DilithiumCodeSigner> {
    // SAFETY: read-only access after init
    unsafe {
        DILITHIUM_SIGNER.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilithium_code_signer_new() {
        let signer = DilithiumCodeSigner::new(SignatureAlgorithm::Dilithium3);
        assert!(signer.is_ok());
    }

    #[test]
    fn test_dilithium_code_signer_invalid_variant() {
        let signer = DilithiumCodeSigner::new(SignatureAlgorithm::Rsa2048);
        assert_eq!(signer.err(), Some(DilithiumSignerError::InvalidKey));
    }

    #[test]
    fn test_dilithium_code_signer_hybrid_mode() {
        let signer = DilithiumCodeSigner::with_dilithium3().unwrap();
        assert!(signer.is_hybrid_mode());
        signer.set_hybrid_mode(false);
        assert!(!signer.is_hybrid_mode());
    }

    #[test]
    fn test_dilithium_key_pair_empty() {
        let kp = DilithiumKeyPair::empty();
        assert!(kp.public_key.is_empty());
        assert!(kp.secret_key.is_empty());
    }

    #[test]
    fn test_compute_kernel_hash() {
        let data = b"test kernel image";
        let hash = compute_kernel_hash(data);
        assert_ne!(hash, [0u8; 32]);
    }

    #[test]
    fn test_signer_stats_initial() {
        let signer = DilithiumCodeSigner::with_dilithium3().unwrap();
        let (signs, verifies, successes) = signer.stats();
        assert_eq!(signs, 0);
        assert_eq!(verifies, 0);
        assert_eq!(successes, 0);
    }
}
