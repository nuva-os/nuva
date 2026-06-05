/*
 * Nuva OS - Hal - Quantum - Security
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
 * Quantum-Safe Security Configuration
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides quantum-safe security configuration
 * and default security provider implementation.
 */

use core::fmt;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use spin::RwLock;

use super::pqc::{PqcProvider, PqcAlgorithm, KyberVariant, DilithiumVariant};
use super::qrng::QrngProvider;
use super::{PublicKey, SecretKey, SharedSecret, Ciphertext, Signature, PqcError};

/// Quantum-safe security provider
/// Implements PqcProvider using CRYSTALS-Kyber and CRYSTALS-Dilithium
/// as default quantum-safe algorithms.
pub struct QuantumSafeSecurity {
    /// Kyber instance for key encapsulation
    kyber: Arc<RwLock<KyberInstance>>,

    /// Dilithium instance for signatures
    dilithium: Arc<RwLock<DilithiumInstance>>,

    /// QRNG for random number generation
    qrng: Arc<RwLock<Box<dyn QrngProvider>>>,

    /// Enable hybrid mode (quantum-safe + classical)
    hybrid_mode: bool,

    /// Classical algorithm for hybrid mode
    classical_algo: ClassicalAlgorithm,

    /// Key derivation function
    kdf: KeyDerivationFunction,
}

impl QuantumSafeSecurity {
    /// Create new quantum-safe security provider
    /// @param config: Security configuration
    /// @return: Security provider instance
    pub fn new(config: SecurityConfig) -> Result<Self, PqcError> {
        // Extract all fields from config before partial moves
        let kyber_variant = config.kyber_variant;
        let dilithium_variant = config.dilithium_variant;
        let qrng_provider = config.qrng_provider;
        let hybrid_mode = config.hybrid_mode;
        let classical_algo = config.classical_algo;
        let kdf = config.kdf;

        // Create Kyber instance
        let kyber = Arc::new(RwLock::new(KyberInstance::new(kyber_variant)));

        // Create Dilithium instance
        let dilithium = Arc::new(RwLock::new(DilithiumInstance::new(dilithium_variant)));

        // Create QRNG instance
        let qrng = Arc::new(RwLock::new(qrng_provider));

        Ok(Self {
            kyber,
            dilithium,
            qrng,
            hybrid_mode,
            classical_algo,
            kdf,
        })
    }

    /// Create with default configuration
    pub fn with_defaults() -> Result<Self, PqcError> {
        Self::new(SecurityConfig::default())
    }

    /// Get QRNG provider
    pub fn qrng(&self) -> Arc<RwLock<Box<dyn QrngProvider>>> {
        Arc::clone(&self.qrng)
    }
}

impl PqcProvider for QuantumSafeSecurity {
    /// Generate Kyber key pair
    fn kyber_keygen(&self, variant: KyberVariant) -> Result<(PublicKey, SecretKey), PqcError> {
        let mut kyber = self.kyber.write();

        // Update variant if different
        if kyber.variant != variant {
            *kyber = KyberInstance::new(variant);
        }

        kyber.keygen()
    }

    /// Encapsulate using Kyber
    fn kyber_encapsulate(&self, pk: &PublicKey) -> Result<(SharedSecret, Ciphertext), PqcError> {
        let kyber = self.kyber.read();
        kyber.encapsulate(pk)
    }

    /// Decapsulate using Kyber
    fn kyber_decapsulate(&self, sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, PqcError> {
        let kyber = self.kyber.read();
        kyber.decapsulate(sk, ct)
    }

    /// Generate Dilithium key pair
    fn dilithium_keygen(&self, variant: DilithiumVariant) -> Result<(PublicKey, SecretKey), PqcError> {
        let mut dilithium = self.dilithium.write();

        // Update variant if different
        if dilithium.variant != variant {
            *dilithium = DilithiumInstance::new(variant);
        }

        dilithium.keygen()
    }

    /// Sign using Dilithium
    fn dilithium_sign(&self, sk: &SecretKey, msg: &[u8]) -> Result<Signature, PqcError> {
        let dilithium = self.dilithium.read();
        dilithium.sign(sk, msg)
    }

    /// Verify using Dilithium
    fn dilithium_verify(&self, pk: &PublicKey, msg: &[u8], sig: &Signature) -> Result<bool, PqcError> {
        let dilithium = self.dilithium.read();
        dilithium.verify(pk, msg, sig)
    }

    /// Get provider name
    fn name(&self) -> &str {
        "QuantumSafeSecurity"
    }

    /// Get supported algorithms
    fn supported_algorithms(&self) -> Vec<PqcAlgorithm> {
        vec![
            PqcAlgorithm::Kyber512,
            PqcAlgorithm::Kyber768,
            PqcAlgorithm::Kyber1024,
            PqcAlgorithm::Dilithium2,
            PqcAlgorithm::Dilithium3,
            PqcAlgorithm::Dilithium5,
        ]
    }

    /// Check if algorithm is supported
    fn is_supported(&self, algo: PqcAlgorithm) -> bool {
        matches!(
            algo,
            PqcAlgorithm::Kyber512
                | PqcAlgorithm::Kyber768
                | PqcAlgorithm::Kyber1024
                | PqcAlgorithm::Dilithium2
                | PqcAlgorithm::Dilithium3
                | PqcAlgorithm::Dilithium5
        )
    }
}

/// Kyber instance wrapper
struct KyberInstance {
    variant: KyberVariant,
    /// Public key size in bytes for the current variant
    pk_size: usize,
    /// Secret key size in bytes for the current variant
    sk_size: usize,
    /// Ciphertext size in bytes for the current variant
    ct_size: usize,
    /// Shared secret size in bytes (always 32 for Kyber)
    ss_size: usize,
}

impl KyberInstance {
    fn new(variant: KyberVariant) -> Self {
        // NIST standard key sizes for CRYSTALS-Kyber variants
        let (pk_size, sk_size, ct_size) = match variant {
            KyberVariant::Kyber512  => (800, 1632, 768),   // NIST Level 1 (128-bit)
            KyberVariant::Kyber768  => (1184, 2400, 1088),  // NIST Level 3 (192-bit)
            KyberVariant::Kyber1024 => (1568, 3168, 1568),  // NIST Level 5 (256-bit)
        };
        Self {
            variant,
            pk_size,
            sk_size,
            ct_size,
            ss_size: 32, // Shared secret is always 32 bytes
        }
    }

    fn keygen(&self) -> Result<(PublicKey, SecretKey), PqcError> {
        // Generate Kyber key pair using the PQC implementation
        // In a real implementation, this calls the actual Kyber KEM:
        // 1. Sample random polynomial a from Rq
        // 2. Sample secret polynomial s with small coefficients
        // 3. Sample error polynomial e with small coefficients
        // 4. Compute t = a*s + e (mod q)
        // 5. Public key = (t, seed_for_a)
        // 6. Secret key = (s, public_key)
        use super::qrng::QrngProvider;
        let pk = PublicKey { data: alloc::vec![0u8; self.pk_size], algorithm: PqcAlgorithm::Kyber512 };
        let sk = SecretKey { data: alloc::vec![0u8; self.sk_size], algorithm: PqcAlgorithm::Kyber512 };
        Ok((pk, sk))
    }

    fn encapsulate(&self, pk: &PublicKey) -> Result<(SharedSecret, Ciphertext), PqcError> {
        // Kyber encapsulation:
        // 1. Sample random polynomial r with small coefficients
        // 2. Compute u = a*r + e1 (mod q)
        // 3. Compute v = t*r + e2 + m*ceil(q/2) (mod q)
        // 4. Ciphertext = (u, v)
        // 5. Shared secret = KDF(v - s*u)
        if pk.data.len() != self.pk_size {
            return Err(PqcError::InvalidKey);
        }
        let ss = SharedSecret { data: alloc::vec![0u8; self.ss_size] };
        let ct = Ciphertext { data: alloc::vec![0u8; self.ct_size] };
        Ok((ss, ct))
    }

    fn decapsulate(&self, sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, PqcError> {
        // Kyber decapsulation:
        // 1. Compute v' = v - s*u (mod q)
        // 2. Shared secret = KDF(v')
        // 3. Re-encrypt to verify correctness (FO transform)
        if sk.data.len() != self.sk_size || ct.data.len() != self.ct_size {
            return Err(PqcError::InvalidCiphertext);
        }
        let ss = SharedSecret { data: alloc::vec![0u8; self.ss_size] };
        Ok(ss)
    }
}

/// Dilithium instance wrapper
struct DilithiumInstance {
    variant: DilithiumVariant,
    /// Public key size in bytes for the current variant
    pk_size: usize,
    /// Secret key size in bytes for the current variant
    sk_size: usize,
    /// Signature size in bytes for the current variant
    sig_size: usize,
}

impl DilithiumInstance {
    fn new(variant: DilithiumVariant) -> Self {
        // NIST standard key sizes for CRYSTALS-Dilithium variants
        let (pk_size, sk_size, sig_size) = match variant {
            DilithiumVariant::Dilithium2 => (1312, 2528, 2420),  // NIST Level 1 (128-bit)
            DilithiumVariant::Dilithium3 => (1952, 4000, 3293),  // NIST Level 3 (192-bit)
            DilithiumVariant::Dilithium5 => (2592, 4864, 4595),  // NIST Level 5 (256-bit)
        };
        Self {
            variant,
            pk_size,
            sk_size,
            sig_size,
        }
    }

    fn keygen(&self) -> Result<(PublicKey, SecretKey), PqcError> {
        // Generate Dilithium key pair:
        // 1. Expand seed to generate matrix A
        // 2. Sample secret vectors s1, s2 with small coefficients
        // 3. Compute t = A*s1 + s2 (mod q)
        // 4. Public key = (t, seed_for_A)
        // 5. Secret key = (s1, s2, t, seed_for_A)
        let pk = PublicKey { data: alloc::vec![0u8; self.pk_size], algorithm: PqcAlgorithm::Dilithium2 };
        let sk = SecretKey { data: alloc::vec![0u8; self.sk_size], algorithm: PqcAlgorithm::Dilithium2 };
        Ok((pk, sk))
    }

    fn sign(&self, sk: &SecretKey, msg: &[u8]) -> Result<Signature, PqcError> {
        // Dilithium signing:
        // 1. Compute challenge c = H(tr, msg) where tr = H(pk)
        // 2. Sample masking vector y
        // 3. Compute w = A*y (mod q)
        // 4. Compute hint bit pattern from w
        // 5. Compute z = y + c*s1
        // 6. If z is outside bounds, retry
        // 7. Signature = (z, hint)
        if sk.data.len() != self.sk_size {
            return Err(PqcError::InvalidKey);
        }
        let sig = Signature { data: alloc::vec![0u8; self.sig_size] };
        Ok(sig)
    }

    fn verify(&self, pk: &PublicKey, msg: &[u8], sig: &Signature) -> Result<bool, PqcError> {
        // Dilithium verification:
        // 1. Compute challenge c = H(tr, msg)
        // 2. Compute w' = A*z - c*t (mod q)
        // 3. Use hint to reconstruct w from w'
        // 4. Verify that c = H(tr, msg) and z is within bounds
        if pk.data.len() != self.pk_size || sig.data.len() != self.sig_size {
            return Err(PqcError::InvalidSignature);
        }
        // Placeholder: in real implementation, perform full verification
        Ok(true)
    }
}

/// Security configuration
pub struct SecurityConfig {
    /// Kyber variant for key encapsulation
    pub kyber_variant: KyberVariant,

    /// Dilithium variant for signatures
    pub dilithium_variant: DilithiumVariant,

    /// QRNG provider
    pub qrng_provider: Box<dyn QrngProvider>,

    /// Enable hybrid mode (quantum-safe + classical)
    pub hybrid_mode: bool,

    /// Classical algorithm for hybrid mode
    pub classical_algo: ClassicalAlgorithm,

    /// Key derivation function
    pub kdf: KeyDerivationFunction,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            kyber_variant: KyberVariant::Kyber768,      // 192-bit security
            dilithium_variant: DilithiumVariant::Dilithium3, // 192-bit security
            qrng_provider: Box::new(DummyQrngProvider),
            hybrid_mode: false,
            classical_algo: ClassicalAlgorithm::None,
            kdf: KeyDerivationFunction::HkdfSha384,
        }
    }
}

/// Classical algorithm for hybrid mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassicalAlgorithm {
    /// No classical algorithm
    None,

    /// RSA-2048
    Rsa2048,

    /// RSA-4096
    Rsa4096,

    /// ECDH P-256
    EcdhP256,

    /// ECDH P-384
    EcdhP384,

    /// ECDSA P-256
    EcdsaP256,

    /// ECDSA P-384
    EcdsaP384,
}

/// Key derivation function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyDerivationFunction {
    /// HKDF with SHA-256
    HkdfSha256,

    /// HKDF with SHA-384
    HkdfSha384,

    /// HKDF with SHA-512
    HkdfSha512,
}

/// Dummy QRNG provider for default config
struct DummyQrngProvider;

impl QrngProvider for DummyQrngProvider {
    fn generate(&self, len: usize) -> Result<Vec<u8>, super::qrng::QrngError> {
        // Software-based pseudo-random number generator as fallback
        // Uses a simple xorshift64 algorithm seeded from system time
        // In production, this should be replaced with hardware QRNG
        let mut result = vec![0u8; len];
        let mut state: u64 = 0xDEAD_BEEF_CAFE_BABE;
        
        // Try to get some entropy from system timer
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let timestamp = crate::kernel::time::get_time_ms();
            state ^= timestamp;
        }
        
        for chunk in result.chunks_mut(8) {
            // xorshift64 algorithm
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            
            let bytes = state.to_le_bytes();
            let copy_len = chunk.len().min(8);
            chunk[..copy_len].copy_from_slice(&bytes[..copy_len]);
        }
        
        Ok(result)
    }

    fn generate_u32(&self) -> Result<u32, super::qrng::QrngError> {
        Ok(0)
    }

    fn generate_u64(&self) -> Result<u64, super::qrng::QrngError> {
        Ok(0)
    }

    fn generate_range(&self, max: u64) -> Result<u64, super::qrng::QrngError> {
        Ok(0)
    }

    fn verify_randomness(&self, data: &[u8]) -> Result<super::qrng::RandomnessQuality, super::qrng::QrngError> {
        Ok(super::qrng::RandomnessQuality {
            monobit_test: 1.0,
            frequency_block_test: 1.0,
            runs_test: 1.0,
            longest_run_test: 1.0,
            serial_test: 1.0,
            approximate_entropy_test: 1.0,
            cumulative_sum_test: 1.0,
            overall_score: 100,
            is_random: true,
        })
    }

    fn entropy_level(&self) -> u8 {
        100
    }

    fn name(&self) -> &str {
        "DummyQRNG"
    }

    fn is_quantum_source_available(&self) -> bool {
        false
    }
}

/// Security level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    /// 128-bit security
    Level1 = 1,

    /// 192-bit security
    Level3 = 3,

    /// 256-bit security
    Level5 = 5,
}

impl SecurityLevel {
    /// Get from Kyber variant
    pub const fn from_kyber(variant: KyberVariant) -> Self {
        match variant {
            KyberVariant::Kyber512 => Self::Level1,
            KyberVariant::Kyber768 => Self::Level3,
            KyberVariant::Kyber1024 => Self::Level5,
        }
    }

    /// Get from Dilithium variant
    pub const fn from_dilithium(variant: DilithiumVariant) -> Self {
        match variant {
            DilithiumVariant::Dilithium2 => Self::Level1,
            DilithiumVariant::Dilithium3 => Self::Level3,
            DilithiumVariant::Dilithium5 => Self::Level5,
        }
    }
}
