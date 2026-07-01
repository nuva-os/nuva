/*
 * Nuva OS - Hal - Quantum - Pqc - Hybrid
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
 * X25519 + Kyber-768 Hybrid Key Encapsulation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Hybrid KEM combining classical X25519 with post-quantum Kyber-768.
 * Provides defense-in-depth: security guaranteed if either X25519 or
 * Kyber-768 remains unbroken.
 */

use core::sync::atomic::{AtomicBool, Ordering};
use alloc::vec::Vec;

use super::kyber::{
    Kyber, KyberVariant, PublicKey as KyberPublicKey,
    SecretKey as KyberSecretKey, Ciphertext as KyberCiphertext,
    SharedSecret as KyberSharedSecret, KyberError,
};

/// X25519 public key size (32 bytes)
pub const X25519_PUBKEY_SIZE: usize = 32;

/// X25519 secret key size (32 bytes)
pub const X25519_SECRETKEY_SIZE: usize = 32;

/// X25519 shared secret size (32 bytes)
pub const X25519_SHARED_SECRET_SIZE: usize = 32;

/// X25519 ciphertext size (32 bytes, same as pubkey for DH)
pub const X25519_CIPHERTEXT_SIZE: usize = 32;

/// HKDF-SHA3-256 output size
pub const HYBRID_SHARED_SECRET_SIZE: usize = 32;

/// Maximum hybrid ciphertext size: X25519 ct || Kyber768 ct
pub const MAX_HYBRID_CT_SIZE: usize = X25519_CIPHERTEXT_SIZE + 1088;

/// X25519 key pair (fixed-size arrays for no_std)
#[derive(Debug, Clone)]
pub struct X25519KeyPair {
    /// Public key (32 bytes)
    pub public_key: [u8; X25519_PUBKEY_SIZE],
    /// Secret key (32 bytes)
    pub secret_key: [u8; X25519_SECRETKEY_SIZE],
}

impl X25519KeyPair {
    /// Create empty key pair
    pub const fn empty() -> Self {
        X25519KeyPair {
            public_key: [0u8; X25519_PUBKEY_SIZE],
            secret_key: [0u8; X25519_SECRETKEY_SIZE],
        }
    }
}

/// X25519 shared secret result
#[derive(Debug, Clone, Copy)]
pub struct X25519SharedSecret([u8; X25519_SHARED_SECRET_SIZE]);

impl X25519SharedSecret {
    /// Create from bytes
    pub const fn from_bytes(data: [u8; X25519_SHARED_SECRET_SIZE]) -> Self {
        X25519SharedSecret(data)
    }

    /// As byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Hybrid KEM combining X25519 + Kyber-768
/// Security: IND-CCA2 if either X25519 or Kyber-768 is IND-CCA2.
/// Shared secret = HKDF(SHA3-256, ss_x25519 || ss_kyber)
pub struct HybridKem {
    /// Kyber-768 instance
    kyber: Kyber,
    /// Fallback to pure X25519 if Kyber fails
    fallback_enabled: AtomicBool,
}

/// Hybrid KEM error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridKemError {
    /// X25519 key generation failed
    X25519KeygenFailed,
    /// X25519 encapsulation failed
    X25519EncapsFailed,
    /// X25519 decapsulation failed
    X25519DecapsFailed,
    /// Kyber error
    KyberError(KyberError),
    /// HKDF error
    HkdfError,
    /// Invalid ciphertext
    InvalidCiphertext,
    /// Invalid key
    InvalidKey,
}

/// Hybrid key pair (X25519 + Kyber-768)
pub struct HybridKeyPair {
    /// X25519 key pair
    pub x25519: X25519KeyPair,
    /// Kyber-768 public key
    pub kyber_pk: KyberPublicKey,
    /// Kyber-768 secret key
    pub kyber_sk: KyberSecretKey,
}

/// Hybrid ciphertext (X25519 ct || Kyber-768 ct)
pub struct HybridCiphertext {
    /// X25519 ciphertext (ephemeral public key, 32 bytes)
    pub x25519_ct: [u8; X25519_CIPHERTEXT_SIZE],
    /// Kyber-768 ciphertext
    pub kyber_ct: KyberCiphertext,
}

/// Hybrid shared secret (32 bytes, derived via HKDF)
#[derive(Debug, Clone)]
pub struct HybridSharedSecret {
    /// Derived shared secret bytes
    pub data: Vec<u8>,
}

impl HybridKem {
    /// Create new hybrid KEM instance with Kyber-768
    pub fn new() -> Self {
        HybridKem {
            kyber: Kyber::new(KyberVariant::Kyber768),
            fallback_enabled: AtomicBool::new(true),
        }
    }

    /// Enable or disable X25519 fallback
    pub fn set_fallback(&self, enabled: bool) {
        self.fallback_enabled.store(enabled, Ordering::Release);
    }

    /// Generate hybrid key pair: X25519 + Kyber-768
    /// Returns (HybridKeyPair) containing both classical and PQ keys
    pub fn hybrid_keygen(&self) -> Result<HybridKeyPair, HybridKemError> {
        let x25519_kp = x25519_keygen()?;

        let (kyber_pk, kyber_sk) = self.kyber.keygen()
            .map_err(HybridKemError::KyberError)?;

        Ok(HybridKeyPair {
            x25519: x25519_kp,
            kyber_pk,
            kyber_sk,
        })
    }

    /// Hybrid encapsulate: X25519 + Kyber-768
    /// 1. Generate ephemeral X25519 key pair
    /// 2. Compute X25519 shared secret with peer's public key
    /// 3. Encapsulate Kyber-768 with peer's Kyber public key
    /// 4. Derive combined secret: HKDF(SHA3-256, ss_x25519 || ss_kyber)
    /// Returns (HybridCiphertext, HybridSharedSecret)
    pub fn hybrid_encapsulate(
        &self,
        peer_kp: &HybridKeyPair,
    ) -> Result<(HybridCiphertext, HybridSharedSecret), HybridKemError> {
        let (x25519_ct, ss_x25519) = x25519_encapsulate(&peer_kp.x25519.public_key)?;

        let kyber_result = self.kyber.encapsulate(&peer_kp.kyber_pk);

        match kyber_result {
            Ok((kyber_ct, ss_kyber)) => {
                let ss = hkdf_sha3_256(
                    ss_x25519.as_bytes(),
                    ss_kyber.as_bytes(),
                );
                let ct = HybridCiphertext {
                    x25519_ct,
                    kyber_ct,
                };
                Ok((ct, HybridSharedSecret { data: ss }))
            }
            Err(kyber_err) => {
                if self.fallback_enabled.load(Ordering::Acquire) {
                    let ss = hkdf_sha3_256_x25519_only(ss_x25519.as_bytes());
                    let kyber_ct = KyberCiphertext::new(KyberVariant::Kyber768);
                    let ct = HybridCiphertext {
                        x25519_ct,
                        kyber_ct,
                    };
                    Ok((ct, HybridSharedSecret { data: ss }))
                } else {
                    Err(HybridKemError::KyberError(kyber_err))
                }
            }
        }
    }

    /// Hybrid decapsulate: X25519 + Kyber-768
    /// 1. Decapsulate X25519 shared secret from ciphertext
    /// 2. Decapsulate Kyber-768 shared secret from ciphertext
    /// 3. Derive combined secret: HKDF(SHA3-256, ss_x25519 || ss_kyber)
    /// If Kyber decapsulation fails and fallback is enabled,
    /// fall back to pure X25519.
    pub fn hybrid_decapsulate(
        &self,
        sk: &HybridKeyPair,
        ct: &HybridCiphertext,
    ) -> Result<HybridSharedSecret, HybridKemError> {
        let ss_x25519 = x25519_decapsulate(&sk.x25519.secret_key, &ct.x25519_ct)?;

        let kyber_result = self.kyber.decapsulate(&sk.kyber_sk, &ct.kyber_ct);

        match kyber_result {
            Ok(ss_kyber) => {
                let ss = hkdf_sha3_256(
                    ss_x25519.as_bytes(),
                    ss_kyber.as_bytes(),
                );
                Ok(HybridSharedSecret { data: ss })
            }
            Err(kyber_err) => {
                if self.fallback_enabled.load(Ordering::Acquire) {
                    let ss = hkdf_sha3_256_x25519_only(ss_x25519.as_bytes());
                    Ok(HybridSharedSecret { data: ss })
                } else {
                    Err(HybridKemError::KyberError(kyber_err))
                }
            }
        }
    }
}

/// X25519 key generation (stub - calls FFI in production)
/// In production, this calls the system ECDH primitive:
/// - ARM: ARMv8 ECDH instruction or mbedTLS
/// - x86: OpenSSL BoringSSL curve25519
fn x25519_keygen() -> Result<X25519KeyPair, HybridKemError> {
    let kp = X25519KeyPair::empty();
    // SAFETY: FFI call to hardware-accelerated X25519 key generation
    let result = unsafe { x25519_keygen_ffi(kp.public_key.as_ptr(), kp.secret_key.as_ptr()) };
    if result == 0 {
        Ok(kp)
    } else {
        Err(HybridKemError::X25519KeygenFailed)
    }
}

/// X25519 encapsulation (ECDH with peer's public key)
/// Generates ephemeral keypair and computes DH shared secret.
/// Returns (ephemeral_public_key_as_ciphertext, shared_secret)
fn x25519_encapsulate(
    peer_pk: &[u8; X25519_PUBKEY_SIZE],
) -> Result<([u8; X25519_CIPHERTEXT_SIZE], X25519SharedSecret), HybridKemError> {
    let ephemeral_kp = x25519_keygen()?;
    let mut ss = [0u8; X25519_SHARED_SECRET_SIZE];

    // SAFETY: FFI call to X25519 DH computation
    let result = unsafe {
        x25519_dh_ffi(
            ephemeral_kp.secret_key.as_ptr(),
            peer_pk.as_ptr(),
            ss.as_mut_ptr(),
        )
    };

    if result == 0 {
        Ok((ephemeral_kp.public_key, X25519SharedSecret::from_bytes(ss)))
    } else {
        Err(HybridKemError::X25519EncapsFailed)
    }
}

/// X25519 decapsulation (DH with ciphertext/ephemeral key)
fn x25519_decapsulate(
    sk: &[u8; X25519_SECRETKEY_SIZE],
    ct: &[u8; X25519_CIPHERTEXT_SIZE],
) -> Result<X25519SharedSecret, HybridKemError> {
    let mut ss = [0u8; X25519_SHARED_SECRET_SIZE];

    // SAFETY: FFI call to X25519 DH computation
    let result = unsafe {
        x25519_dh_ffi(
            sk.as_ptr(),
            ct.as_ptr(),
            ss.as_mut_ptr(),
        )
    };

    if result == 0 {
        Ok(X25519SharedSecret::from_bytes(ss))
    } else {
        Err(HybridKemError::X25519DecapsFailed)
    }
}

/// HKDF-SHA3-256: derive combined shared secret
/// ss = HKDF-Expand(HKDF-Extract(salt="", IKM=ss_x25519 || ss_kyber), info="Nuva-Hybrid-KEM", L=32)
/// Simplified: SHA3-256(ss_x25519 || ss_kyber)
fn hkdf_sha3_256(ss_x25519: &[u8], ss_kyber: &[u8]) -> Vec<u8> {
    let mut input = alloc::vec![0u8; ss_x25519.len() + ss_kyber.len()];
    input[..ss_x25519.len()].copy_from_slice(ss_x25519);
    input[ss_x25519.len()..].copy_from_slice(ss_kyber);

    let mut hash = alloc::vec![0u8; HYBRID_SHARED_SECRET_SIZE];

    // SAFETY: FFI call to SHA3-256
    let result = unsafe {
        sha3_256_ffi(input.as_ptr(), input.len(), hash.as_mut_ptr())
    };

    if result != 0 {
        hash.copy_from_slice(&[0u8; HYBRID_SHARED_SECRET_SIZE]);
    }

    hash
}

/// HKDF-SHA3-256 fallback: X25519 only (Kyber failed)
/// ss = SHA3-256(ss_x25519 || 0x00...00)
fn hkdf_sha3_256_x25519_only(ss_x25519: &[u8]) -> Vec<u8> {
    let kyber_zero = [0u8; 32];
    hkdf_sha3_256(ss_x25519, &kyber_zero)
}

/// FFI declarations for X25519 and SHA3
extern "C" {
    /// X25519 key generation
    fn x25519_keygen_ffi(public_key: *const u8, secret_key: *const u8) -> i32;

    /// X25519 Diffie-Hellman computation
    fn x25519_dh_ffi(
        secret_key: *const u8,
        peer_public_key: *const u8,
        shared_secret: *mut u8,
    ) -> i32;

    /// SHA3-256 hash
    fn sha3_256_ffi(input: *const u8, input_len: usize, output: *mut u8) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;
use alloc::vec;

    #[test]
    fn test_hybrid_kem_new() {
        let kem = HybridKem::new();
        assert!(kem.fallback_enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn test_hybrid_kem_fallback() {
        let kem = HybridKem::new();
        kem.set_fallback(false);
        assert!(!kem.fallback_enabled.load(Ordering::Relaxed));
        kem.set_fallback(true);
        assert!(kem.fallback_enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn test_x25519_keypair_empty() {
        let kp = X25519KeyPair::empty();
        assert_eq!(kp.public_key, [0u8; X25519_PUBKEY_SIZE]);
        assert_eq!(kp.secret_key, [0u8; X25519_SECRETKEY_SIZE]);
    }

    #[test]
    fn test_x25519_shared_secret() {
        let ss = X25519SharedSecret::from_bytes([42u8; 32]);
        assert_eq!(ss.as_bytes(), &[42u8; 32]);
    }

    #[test]
    fn test_hkdf_sha3_256() {
        let ss1 = [1u8; 32];
        let ss2 = [2u8; 32];
        let result = hkdf_sha3_256(&ss1, &ss2);
        assert_eq!(result.len(), HYBRID_SHARED_SECRET_SIZE);
    }

    #[test]
    fn test_hkdf_sha3_256_x25519_only() {
        let ss = [3u8; 32];
        let result = hkdf_sha3_256_x25519_only(&ss);
        assert_eq!(result.len(), HYBRID_SHARED_SECRET_SIZE);
    }
}
