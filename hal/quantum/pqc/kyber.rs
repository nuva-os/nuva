/*
 * Nuva OS - CRYSTALS-Kyber Rust FFI Bindings
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

/// CRYSTALS-Kyber Post-Quantum Key Encapsulation
/// This module provides safe Rust bindings to the Kyber C implementation,
/// offering IND-CCA2 secure key exchange resistant to quantum attacks.

use core::mem::MaybeUninit;

/// Kyber variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KyberVariant {
    /// Kyber-512: 512-bit security level
    Kyber512 = 0,
    /// Kyber-768: 768-bit security level (recommended)
    Kyber768 = 1,
    /// Kyber-1024: 1024-bit security level
    Kyber1024 = 2,
}

/// Kyber error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KyberError {
    /// Key generation failed
    KeyGenerationFailed,
    /// Encapsulation failed
    EncapsulationFailed,
    /// Decapsulation failed
    DecapsulationFailed,
    /// Invalid key size
    InvalidKeySize,
    /// Invalid ciphertext size
    InvalidCiphertextSize,
    /// Invalid variant
    InvalidVariant,
}

/// Kyber public key
#[derive(Debug, Clone)]
pub struct PublicKey {
    /// Key data
    data: [u8; Self::MAX_SIZE],
    /// Actual length
    len: usize,
    /// Variant
    variant: KyberVariant,
}

impl PublicKey {
    /// Maximum public key size (Kyber-1024)
    const MAX_SIZE: usize = 1568;

    /// Create new empty public key
    pub fn new(variant: KyberVariant) -> Self {
        let len = match variant {
            KyberVariant::Kyber512 => 800,
            KyberVariant::Kyber768 => 1184,
            KyberVariant::Kyber1024 => 1568,
        };

        PublicKey {
            data: [0u8; Self::MAX_SIZE],
            len,
            variant,
        }
    }

    /// Get key data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Get mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Get variant
    pub fn variant(&self) -> KyberVariant {
        self.variant
    }
}

/// Kyber secret key
#[derive(Debug, Clone)]
pub struct SecretKey {
    /// Key data
    data: [u8; Self::MAX_SIZE],
    /// Actual length
    len: usize,
    /// Variant
    variant: KyberVariant,
}

impl SecretKey {
    /// Maximum secret key size (Kyber-1024)
    const MAX_SIZE: usize = 3168;

    /// Create new empty secret key
    pub fn new(variant: KyberVariant) -> Self {
        let len = match variant {
            KyberVariant::Kyber512 => 1632,
            KyberVariant::Kyber768 => 2400,
            KyberVariant::Kyber1024 => 3168,
        };

        SecretKey {
            data: [0u8; Self::MAX_SIZE],
            len,
            variant,
        }
    }

    /// Get key data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Get mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Get variant
    pub fn variant(&self) -> KyberVariant {
        self.variant
    }
}

/// Kyber ciphertext
#[derive(Debug, Clone)]
pub struct Ciphertext {
    /// Ciphertext data
    data: [u8; Self::MAX_SIZE],
    /// Actual length
    len: usize,
    /// Variant
    variant: KyberVariant,
}

impl Ciphertext {
    /// Maximum ciphertext size (Kyber-1024)
    const MAX_SIZE: usize = 1568;

    /// Create new empty ciphertext
    pub fn new(variant: KyberVariant) -> Self {
        let len = match variant {
            KyberVariant::Kyber512 => 768,
            KyberVariant::Kyber768 => 1088,
            KyberVariant::Kyber1024 => 1568,
        };

        Ciphertext {
            data: [0u8; Self::MAX_SIZE],
            len,
            variant,
        }
    }

    /// Get ciphertext data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Get mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Get variant
    pub fn variant(&self) -> KyberVariant {
        self.variant
    }
}

/// Shared secret (32 bytes)
#[derive(Debug, Clone, Copy)]
pub struct SharedSecret([u8; 32]);

impl SharedSecret {
    /// Create new shared secret
    pub fn new() -> Self {
        SharedSecret([0u8; 32])
    }

    /// Get shared secret data
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }
}

/// Kyber KEM instance
pub struct Kyber {
    variant: KyberVariant,
}

impl Kyber {
    /// Create new Kyber instance
    pub fn new(variant: KyberVariant) -> Self {
        Kyber { variant }
    }

    /// Generate key pair
    pub fn keygen(&self) -> Result<(PublicKey, SecretKey), KyberError> {
        let mut pk = PublicKey::new(self.variant);
        let mut sk = SecretKey::new(self.variant);

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe {
            kyber_keygen(
                self.variant as u32,
                pk.as_mut_ptr(),
                sk.as_mut_ptr(),
            )
        };

        if result == 0 {
            Ok((pk, sk))
        } else {
            Err(KyberError::KeyGenerationFailed)
        }
    }

    /// Encapsulate shared secret
    pub fn encapsulate(
        &self,
        public_key: &PublicKey,
    ) -> Result<(Ciphertext, SharedSecret), KyberError> {
        let mut ct = Ciphertext::new(self.variant);
        let mut ss = SharedSecret::new();

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe {
            kyber_encapsulate(
                self.variant as u32,
                public_key.as_bytes().as_ptr(),
                ct.as_mut_ptr(),
                ss.as_mut_ptr(),
            )
        };

        if result == 0 {
            Ok((ct, ss))
        } else {
            Err(KyberError::EncapsulationFailed)
        }
    }

    /// Decapsulate shared secret
    pub fn decapsulate(
        &self,
        secret_key: &SecretKey,
        ciphertext: &Ciphertext,
    ) -> Result<SharedSecret, KyberError> {
        let mut ss = SharedSecret::new();

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe {
            kyber_decapsulate(
                self.variant as u32,
                secret_key.as_bytes().as_ptr(),
                ciphertext.as_bytes().as_ptr(),
                ss.as_mut_ptr(),
            )
        };

        if result == 0 {
            Ok(ss)
        } else {
            Err(KyberError::DecapsulationFailed)
        }
    }

    /// Get variant
    pub fn variant(&self) -> KyberVariant {
        self.variant
    }
}

// FFI declarations
extern "C" {
    /// C key generation function
    fn kyber_keygen(
        variant: u32,
        public_key: *mut u8,
        secret_key: *mut u8,
    ) -> i32;

    /// C encapsulation function
    fn kyber_encapsulate(
        variant: u32,
        public_key: *const u8,
        ciphertext: *mut u8,
        shared_secret: *mut u8,
    ) -> i32;

    /// C decapsulation function
    fn kyber_decapsulate(
        variant: u32,
        secret_key: *const u8,
        ciphertext: *const u8,
        shared_secret: *mut u8,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber_keygen() {
        let kyber = Kyber::new(KyberVariant::Kyber768);
        let result = kyber.keygen();
        assert!(result.is_ok());
    }

    #[test]
    fn test_kyber_encapsulate_decapsulate() {
        let kyber = Kyber::new(KyberVariant::Kyber768);

        // Generate key pair
        let (pk, sk) = kyber.keygen().unwrap();

        // Encapsulate
        let (ct, ss1) = kyber.encapsulate(&pk).unwrap();

        // Decapsulate
        let ss2 = kyber.decapsulate(&sk, &ct).unwrap();

        // Verify shared secrets match
        assert_eq!(ss1.as_bytes(), ss2.as_bytes());
    }
}
