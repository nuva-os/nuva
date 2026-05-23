/*
 * Post-Quantum Cryptography (PQC) Provider
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides hardware abstraction for post-quantum
 * cryptographic algorithms, including CRYSTALS-Kyber and
 * CRYSTALS-Dilithium as specified by NIST PQC standardization.
 */

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;

pub mod kyber;
pub mod dilithium;
pub mod hybrid;
pub mod tls_kem;

/// PQC Provider trait - Hardware abstraction for post-quantum crypto
/// All PQC hardware implementations must implement this trait.
/// This enables support for:
/// - Hardware accelerators (NPU, crypto engines)
/// - Software implementations (reference implementations)
/// - Hybrid approaches (hardware + software fallback)
pub trait PqcProvider: Send + Sync {
    // ===== CRYSTALS-Kyber (Key Encapsulation) =====

    /// Generate Kyber key pair
    /// @param variant: Kyber variant (Kyber512, Kyber768, Kyber1024)
    /// @return: (public_key, secret_key)
    fn kyber_keygen(&self, variant: KyberVariant) -> Result<(PublicKey, SecretKey), PqcError>;

    /// Encapsulate shared secret using Kyber
    /// @param pk: Public key
    /// @return: (shared_secret, ciphertext)
    fn kyber_encapsulate(&self, pk: &PublicKey) -> Result<(SharedSecret, Ciphertext), PqcError>;

    /// Decapsulate shared secret using Kyber
    /// @param sk: Secret key
    /// @param ct: Ciphertext
    /// @return: shared_secret
    fn kyber_decapsulate(&self, sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, PqcError>;

    // ===== CRYSTALS-Dilithium (Digital Signatures) =====

    /// Generate Dilithium key pair
    /// @param variant: Dilithium variant (Dilithium2, Dilithium3, Dilithium5)
    /// @return: (public_key, secret_key)
    fn dilithium_keygen(&self, variant: DilithiumVariant) -> Result<(PublicKey, SecretKey), PqcError>;

    /// Sign message using Dilithium
    /// @param sk: Secret key
    /// @param msg: Message to sign
    /// @return: signature
    fn dilithium_sign(&self, sk: &SecretKey, msg: &[u8]) -> Result<Signature, PqcError>;

    /// Verify signature using Dilithium
    /// @param pk: Public key
    /// @param msg: Message
    /// @param sig: Signature
    /// @return: true if valid
    fn dilithium_verify(&self, pk: &PublicKey, msg: &[u8], sig: &Signature) -> Result<bool, PqcError>;

    // ===== Provider Information =====

    /// Get provider name
    fn name(&self) -> &str;

    /// Get supported algorithms
    fn supported_algorithms(&self) -> Vec<PqcAlgorithm>;

    /// Check if algorithm is supported
    fn is_supported(&self, algo: PqcAlgorithm) -> bool;
}

/// Kyber variant enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KyberVariant {
    /// Kyber512 - 512-bit security
    Kyber512,

    /// Kyber768 - 768-bit security (recommended)
    Kyber768,

    /// Kyber1024 - 1024-bit security
    Kyber1024,
}

impl KyberVariant {
    /// Get public key size in bytes
    pub const fn public_key_size(&self) -> usize {
        match self {
            Self::Kyber512 => 800,
            Self::Kyber768 => 1184,
            Self::Kyber1024 => 1568,
        }
    }

    /// Get secret key size in bytes
    pub const fn secret_key_size(&self) -> usize {
        match self {
            Self::Kyber512 => 1632,
            Self::Kyber768 => 2400,
            Self::Kyber1024 => 3168,
        }
    }

    /// Get ciphertext size in bytes
    pub const fn ciphertext_size(&self) -> usize {
        match self {
            Self::Kyber512 => 768,
            Self::Kyber768 => 1088,
            Self::Kyber1024 => 1568,
        }
    }

    /// Get shared secret size in bytes
    pub const fn shared_secret_size(&self) -> usize {
        32 // Always 32 bytes for all variants
    }
}

/// Dilithium variant enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilithiumVariant {
    /// Dilithium2 - Level 2 security
    Dilithium2,

    /// Dilithium3 - Level 3 security
    Dilithium3,

    /// Dilithium5 - Level 5 security (recommended)
    Dilithium5,
}

impl DilithiumVariant {
    /// Get public key size in bytes
    pub const fn public_key_size(&self) -> usize {
        match self {
            Self::Dilithium2 => 1312,
            Self::Dilithium3 => 1952,
            Self::Dilithium5 => 2592,
        }
    }

    /// Get secret key size in bytes
    pub const fn secret_key_size(&self) -> usize {
        match self {
            Self::Dilithium2 => 2560,
            Self::Dilithium3 => 4032,
            Self::Dilithium5 => 4864,
        }
    }

    /// Get signature size in bytes
    pub const fn signature_size(&self) -> usize {
        match self {
            Self::Dilithium2 => 2420,
            Self::Dilithium3 => 3293,
            Self::Dilithium5 => 4595,
        }
    }
}

/// PQC algorithm enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcAlgorithm {
    Kyber512,
    Kyber768,
    Kyber1024,
    Dilithium2,
    Dilithium3,
    Dilithium5,
}

/// Public key
#[derive(Debug, Clone)]
pub struct PublicKey {
    /// Key data
    pub data: Vec<u8>,

    /// Algorithm
    pub algorithm: PqcAlgorithm,
}

/// Secret key
#[derive(Debug, Clone)]
pub struct SecretKey {
    /// Key data (sensitive)
    pub data: Vec<u8>,

    /// Algorithm
    pub algorithm: PqcAlgorithm,
}

impl SecretKey {
    /// Securely zeroize secret key
    pub fn zeroize(&mut self) {
        // Use volatile write to prevent compiler optimization
        for byte in self.data.iter_mut() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Shared secret
#[derive(Debug, Clone)]
pub struct SharedSecret {
    /// Secret data (32 bytes)
    pub data: Vec<u8>,
}

impl SharedSecret {
    /// Securely zeroize shared secret
    pub fn zeroize(&mut self) {
        for byte in self.data.iter_mut() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl Drop for SharedSecret {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Ciphertext
#[derive(Debug, Clone)]
pub struct Ciphertext {
    /// Ciphertext data
    pub data: Vec<u8>,
}

/// Signature
#[derive(Debug, Clone)]
pub struct Signature {
    /// Signature data
    pub data: Vec<u8>,
}

/// PQC error type
#[derive(Debug, Clone)]
pub enum PqcError {
    /// Invalid key
    InvalidKey,

    /// Invalid signature
    InvalidSignature,

    /// Invalid ciphertext
    InvalidCiphertext,

    /// Algorithm not supported
    AlgorithmNotSupported(PqcAlgorithm),

    /// Hardware error
    HardwareError(String),

    /// Out of memory
    OutOfMemory,

    /// Random number generation failed
    RngFailed,

    /// Verification failed
    VerificationFailed,
}

impl fmt::Display for PqcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "Invalid key"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidCiphertext => write!(f, "Invalid ciphertext"),
            Self::AlgorithmNotSupported(algo) => write!(f, "Algorithm not supported: {:?}", algo),
            Self::HardwareError(msg) => write!(f, "Hardware error: {}", msg),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::RngFailed => write!(f, "Random number generation failed"),
            Self::VerificationFailed => write!(f, "Verification failed"),
        }
    }
}
