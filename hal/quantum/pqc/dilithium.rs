/*
 * Nuva OS - CRYSTALS-Dilithium Rust FFI Bindings
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

/// CRYSTALS-Dilithium Post-Quantum Digital Signatures
/// This module provides safe Rust bindings to the Dilithium C implementation,
/// offering EUF-CMA secure digital signatures resistant to quantum attacks.

/// Dilithium variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum DilithiumVariant {
    /// Dilithium2: 128-bit security level
    Dilithium2 = 0,
    /// Dilithium3: 192-bit security level (recommended)
    Dilithium3 = 1,
    /// Dilithium5: 256-bit security level
    Dilithium5 = 2,
}

/// Dilithium error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilithiumError {
    /// Key generation failed
    KeyGenerationFailed,
    /// Signing failed
    SigningFailed,
    /// Verification failed
    VerificationFailed,
    /// Invalid key size
    InvalidKeySize,
    /// Invalid signature size
    InvalidSignatureSize,
    /// Invalid variant
    InvalidVariant,
}

/// Dilithium public key
#[derive(Debug, Clone)]
pub struct PublicKey {
    /// Key data
    data: [u8; Self::MAX_SIZE],
    /// Actual length
    len: usize,
    /// Variant
    variant: DilithiumVariant,
}

impl PublicKey {
    /// Maximum public key size (Dilithium5)
    const MAX_SIZE: usize = 2592;

    /// Create new empty public key
    pub fn new(variant: DilithiumVariant) -> Self {
        let len = match variant {
            DilithiumVariant::Dilithium2 => 1312,
            DilithiumVariant::Dilithium3 => 1952,
            DilithiumVariant::Dilithium5 => 2592,
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
    pub fn variant(&self) -> DilithiumVariant {
        self.variant
    }
}

/// Dilithium secret key
#[derive(Debug, Clone)]
pub struct SecretKey {
    /// Key data
    data: [u8; Self::MAX_SIZE],
    /// Actual length
    len: usize,
    /// Variant
    variant: DilithiumVariant,
}

impl SecretKey {
    /// Maximum secret key size (Dilithium5)
    const MAX_SIZE: usize = 4864;

    /// Create new empty secret key
    pub fn new(variant: DilithiumVariant) -> Self {
        let len = match variant {
            DilithiumVariant::Dilithium2 => 2528,
            DilithiumVariant::Dilithium3 => 4000,
            DilithiumVariant::Dilithium5 => 4864,
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
    pub fn variant(&self) -> DilithiumVariant {
        self.variant
    }
}

/// Dilithium signature
#[derive(Debug, Clone)]
pub struct Signature {
    /// Signature data
    data: [u8; Self::MAX_SIZE],
    /// Actual length
    len: usize,
    /// Variant
    variant: DilithiumVariant,
}

impl Signature {
    /// Maximum signature size (Dilithium5)
    const MAX_SIZE: usize = 4595;

    /// Create new empty signature
    pub fn new(variant: DilithiumVariant) -> Self {
        let len = match variant {
            DilithiumVariant::Dilithium2 => 2420,
            DilithiumVariant::Dilithium3 => 3293,
            DilithiumVariant::Dilithium5 => 4595,
        };

        Signature {
            data: [0u8; Self::MAX_SIZE],
            len,
            variant,
        }
    }

    /// Get signature data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Get mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Get variant
    pub fn variant(&self) -> DilithiumVariant {
        self.variant
    }
}

/// Dilithium signature instance
pub struct Dilithium {
    variant: DilithiumVariant,
}

impl Dilithium {
    /// Create new Dilithium instance
    pub fn new(variant: DilithiumVariant) -> Self {
        Dilithium { variant }
    }

    /// Generate key pair
    pub fn keygen(&self) -> Result<(PublicKey, SecretKey), DilithiumError> {
        let mut pk = PublicKey::new(self.variant);
        let mut sk = SecretKey::new(self.variant);

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe {
            dilithium_keygen(
                self.variant as u32,
                pk.as_mut_ptr(),
                sk.as_mut_ptr(),
            )
        };

        if result == 0 {
            Ok((pk, sk))
        } else {
            Err(DilithiumError::KeyGenerationFailed)
        }
    }

    /// Sign a message
    pub fn sign(
        &self,
        secret_key: &SecretKey,
        message: &[u8],
    ) -> Result<Signature, DilithiumError> {
        let mut sig = Signature::new(self.variant);

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe {
            dilithium_sign(
                self.variant as u32,
                secret_key.as_bytes().as_ptr(),
                message.as_ptr(),
                message.len(),
                sig.as_mut_ptr(),
            )
        };

        if result == 0 {
            Ok(sig)
        } else {
            Err(DilithiumError::SigningFailed)
        }
    }

    /// Verify a signature
    pub fn verify(
        &self,
        public_key: &PublicKey,
        message: &[u8],
        signature: &Signature,
    ) -> Result<bool, DilithiumError> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe {
            dilithium_verify(
                self.variant as u32,
                public_key.as_bytes().as_ptr(),
                message.as_ptr(),
                message.len(),
                signature.as_bytes().as_ptr(),
            )
        };

        Ok(result == 0)
    }

    /// Get variant
    pub fn variant(&self) -> DilithiumVariant {
        self.variant
    }
}

// FFI declarations
extern "C" {
    /// C key generation function
    fn dilithium_keygen(
        variant: u32,
        public_key: *mut u8,
        secret_key: *mut u8,
    ) -> i32;

    /// C signing function
    fn dilithium_sign(
        variant: u32,
        secret_key: *const u8,
        message: *const u8,
        message_len: usize,
        signature: *mut u8,
    ) -> i32;

    /// C verification function
    fn dilithium_verify(
        variant: u32,
        public_key: *const u8,
        message: *const u8,
        message_len: usize,
        signature: *const u8,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilithium_keygen() {
        let dilithium = Dilithium::new(DilithiumVariant::Dilithium3);
        let result = dilithium.keygen();
        assert!(result.is_ok());
    }

    #[test]
    fn test_dilithium_sign_verify() {
        let dilithium = Dilithium::new(DilithiumVariant::Dilithium3);
        let message = b"Hello, Nuva OS!";

        // Generate key pair
        let (pk, sk) = dilithium.keygen().unwrap();

        // Sign message
        let signature = dilithium.sign(&sk, message).unwrap();

        // Verify signature
        let valid = dilithium.verify(&pk, message, &signature).unwrap();
        assert!(valid);
    }
}
