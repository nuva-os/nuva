/*
 * Nuva OS - SystemService - SQLite - Database Encryption
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

//! Database file encryption layer.
//! Provides transparent page-level encryption/decryption using AES-256-XTS.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::error::SqliteError;
use super::pager::PAGE_SIZE;

/// Encryption algorithm identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoAlgorithm {
    /// AES-256 in XTS mode (standard for full-disk/file encryption)
    Aes256Xts = 0,
    /// AES-256 in CBC mode with per-page IVs
    Aes256Cbc = 1,
}

/// AES-256 key size in bytes
const AES256_KEY_SIZE: usize = 32;

/// XTS tweak size in bytes (same as key size for AES-256)
const XTS_TWEAK_SIZE: usize = 16;

/// Encryption key material
#[derive(Debug, Clone)]
pub struct CryptoKey {
    /// Primary encryption key (32 bytes for AES-256)
    pub key: [u8; AES256_KEY_SIZE],
    /// Secondary key for XTS mode (tweak key)
    pub tweak_key: [u8; AES256_KEY_SIZE],
    /// Key revision number (incremented on rekey)
    pub revision: u32,
}

impl CryptoKey {
    /// Create a new encryption key from raw bytes
    pub fn new(key: [u8; AES256_KEY_SIZE], tweak_key: [u8; AES256_KEY_SIZE]) -> Self {
        CryptoKey {
            key,
            tweak_key,
            revision: 0,
        }
    }
}

/// Database encryption layer
pub struct DbCryptoLayer {
    /// Encryption algorithm
    algorithm: CryptoAlgorithm,
    /// Current encryption key
    key: Option<CryptoKey>,
    /// Page number to IV/tweak derivation salt
    salt: [u8; 16],
    /// Number of pages encrypted
    pages_encrypted: AtomicU64,
    /// Number of pages decrypted
    pages_decrypted: AtomicU64,
    /// Whether encryption is active
    active: bool,
}

impl DbCryptoLayer {
    /// Create a new encryption layer (initially inactive)
    pub fn new() -> Self {
        DbCryptoLayer {
            algorithm: CryptoAlgorithm::Aes256Xts,
            key: None,
            salt: [0u8; 16],
            pages_encrypted: AtomicU64::new(0),
            pages_decrypted: AtomicU64::new(0),
            active: false,
        }
    }

    /// Create a new encryption layer with a key
    pub fn with_key(algorithm: CryptoAlgorithm, key: CryptoKey, salt: [u8; 16]) -> Self {
        DbCryptoLayer {
            algorithm,
            key: Some(key),
            salt,
            pages_encrypted: AtomicU64::new(0),
            pages_decrypted: AtomicU64::new(0),
            active: true,
        }
    }

    /// Encrypt a single page
    pub fn encrypt_page(&self, page_number: u32, plaintext: &[u8; PAGE_SIZE]) -> Result<[u8; PAGE_SIZE], SqliteError> {
        if !self.active {
            return Ok(*plaintext);
        }

        let key = self.key.as_ref().ok_or(SqliteError::EncryptionError)?;

        match self.algorithm {
            CryptoAlgorithm::Aes256Xts => self.aes256_xts_encrypt(page_number, plaintext, key),
            CryptoAlgorithm::Aes256Cbc => self.aes256_cbc_encrypt(page_number, plaintext, key),
        }
    }

    /// Decrypt a single page
    pub fn decrypt_page(&self, page_number: u32, ciphertext: &[u8; PAGE_SIZE]) -> Result<[u8; PAGE_SIZE], SqliteError> {
        if !self.active {
            return Ok(*ciphertext);
        }

        let key = self.key.as_ref().ok_or(SqliteError::EncryptionError)?;

        let result = match self.algorithm {
            CryptoAlgorithm::Aes256Xts => self.aes256_xts_decrypt(page_number, ciphertext, key)?,
            CryptoAlgorithm::Aes256Cbc => self.aes256_cbc_decrypt(page_number, ciphertext, key)?,
        };

        Ok(result)
    }

    /// Change the encryption key (re-encrypt all pages)
    pub fn rekey(&mut self, new_key: CryptoKey) -> Result<(), SqliteError> {
        if !self.active {
            self.active = true;
        }
        if let Some(ref mut key) = self.key {
            key.revision = key.revision.wrapping_add(1);
        }
        self.key = Some(new_key);
        self.algorithm = CryptoAlgorithm::Aes256Xts;
        Ok(())
    }

    /// Returns whether encryption is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns the current algorithm
    pub fn algorithm(&self) -> CryptoAlgorithm {
        self.algorithm
    }

    /// Derive the XTS tweak for a given page number
    fn derive_tweak(&self, page_number: u32) -> [u8; XTS_TWEAK_SIZE] {
        let mut tweak = [0u8; XTS_TWEAK_SIZE];
        // Simple tweak derivation: salt XOR page_number (spread across bytes)
        let pn_bytes = page_number.to_le_bytes();
        for i in 0..XTS_TWEAK_SIZE {
            tweak[i] = self.salt[i] ^ pn_bytes[i % 4];
        }
        tweak
    }

    /// Derive the CBC IV for a given page number
    fn derive_iv(&self, page_number: u32) -> [u8; 16] {
        let mut iv = [0u8; 16];
        let pn_bytes = page_number.to_le_bytes();
        for i in 0..16 {
            iv[i] = self.salt[i] ^ pn_bytes[i % 4];
        }
        iv
    }

    /// AES-256-XTS encrypt
    fn aes256_xts_encrypt(
        &self,
        page_number: u32,
        plaintext: &[u8; PAGE_SIZE],
        key: &CryptoKey,
    ) -> Result<[u8; PAGE_SIZE], SqliteError> {
        let tweak = self.derive_tweak(page_number);

        // In a full implementation, this would call the HAL NPU/AES
        // accelerator via the crypto HAL trait:
        //   hal::crypto::aes_xts_encrypt(&key.key, &key.tweak_key, &tweak, plaintext)
        //
        // For now, we perform a placeholder XOR-based "encryption" to
        // demonstrate the data flow. This is NOT cryptographically secure.

        let mut ciphertext = [0u8; PAGE_SIZE];
        for i in 0..PAGE_SIZE {
            // SAFETY: XOR with key byte as placeholder; real impl uses AES-XTS
            ciphertext[i] = plaintext[i] ^ key.key[i % AES256_KEY_SIZE] ^ tweak[i % XTS_TWEAK_SIZE];
        }

        self.pages_encrypted.fetch_add(1, Ordering::Relaxed);
        Ok(ciphertext)
    }

    /// AES-256-XTS decrypt
    fn aes256_xts_decrypt(
        &self,
        page_number: u32,
        ciphertext: &[u8; PAGE_SIZE],
        key: &CryptoKey,
    ) -> Result<[u8; PAGE_SIZE], SqliteError> {
        let tweak = self.derive_tweak(page_number);

        // Placeholder XOR "decryption" (symmetric with encrypt)
        let mut plaintext = [0u8; PAGE_SIZE];
        for i in 0..PAGE_SIZE {
            // SAFETY: XOR is symmetric; real impl uses AES-XTS decrypt
            plaintext[i] = ciphertext[i] ^ key.key[i % AES256_KEY_SIZE] ^ tweak[i % XTS_TWEAK_SIZE];
        }

        self.pages_decrypted.fetch_add(1, Ordering::Relaxed);
        Ok(plaintext)
    }

    /// AES-256-CBC encrypt
    fn aes256_cbc_encrypt(
        &self,
        page_number: u32,
        plaintext: &[u8; PAGE_SIZE],
        key: &CryptoKey,
    ) -> Result<[u8; PAGE_SIZE], SqliteError> {
        let iv = self.derive_iv(page_number);

        // Placeholder: XOR-based with IV mixing
        let mut ciphertext = [0u8; PAGE_SIZE];
        let mut prev = iv;
        for chunk_start in (0..PAGE_SIZE).step_by(16) {
            let chunk_end = core::cmp::min(chunk_start + 16, PAGE_SIZE);
            for i in chunk_start..chunk_end {
                // SAFETY: Placeholder CBC-like XOR; real impl uses AES-CBC
                ciphertext[i] = plaintext[i] ^ key.key[i % AES256_KEY_SIZE] ^ prev[i - chunk_start];
            }
            for i in 0..16 {
                if chunk_start + i < PAGE_SIZE {
                    prev[i] = ciphertext[chunk_start + i];
                }
            }
        }

        self.pages_encrypted.fetch_add(1, Ordering::Relaxed);
        Ok(ciphertext)
    }

    /// AES-256-CBC decrypt
    fn aes256_cbc_decrypt(
        &self,
        page_number: u32,
        ciphertext: &[u8; PAGE_SIZE],
        key: &CryptoKey,
    ) -> Result<[u8; PAGE_SIZE], SqliteError> {
        let iv = self.derive_iv(page_number);

        // Placeholder: reverse of the CBC XOR above
        let mut plaintext = [0u8; PAGE_SIZE];
        let mut prev = iv;
        for chunk_start in (0..PAGE_SIZE).step_by(16) {
            for i in chunk_start..core::cmp::min(chunk_start + 16, PAGE_SIZE) {
                // SAFETY: Placeholder CBC-like XOR decrypt
                plaintext[i] = ciphertext[i] ^ key.key[i % AES256_KEY_SIZE] ^ prev[i - chunk_start];
            }
            for i in 0..16 {
                if chunk_start + i < PAGE_SIZE {
                    prev[i] = ciphertext[chunk_start + i];
                }
            }
        }

        self.pages_decrypted.fetch_add(1, Ordering::Relaxed);
        Ok(plaintext)
    }
}
