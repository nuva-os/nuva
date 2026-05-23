/*
 * Plugin Signature Verification - Dilithium-based Plugin Signing
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

//! Plugin signature verification using CRYSTALS-Dilithium post-quantum signatures

use alloc::string::String;
use alloc::vec::Vec;

use crate::hal::quantum::pqc::dilithium::{
    Dilithium, DilithiumVariant, DilithiumError,
    PublicKey, Signature,
};
use super::core::{PluginId, PluginError};

// ============================================================================
// Signature Verification Strategy
// ============================================================================

/// Signature verification strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignaturePolicy {
    /// All plugins must have valid signatures; unsigned plugins are rejected
    Enforced,
    /// Signatures are verified but invalid signatures generate warnings only
    Permissive,
    /// Signature verification is disabled
    Disabled,
}

impl Default for SignaturePolicy {
    fn default() -> Self {
        SignaturePolicy::Enforced
    }
}

// ============================================================================
// Plugin Signature
// ============================================================================

/// Plugin signature wrapper
#[derive(Debug, Clone)]
pub struct PluginSignature {
    /// Dilithium signature data
    pub signature: Vec<u8>,
    /// Signer public key fingerprint (hash of public key)
    pub signer_fingerprint: [u8; 32],
    /// Dilithium variant used for signing
    pub variant: DilithiumVariant,
}

impl PluginSignature {
    /// Create a new plugin signature
    pub fn new(signature: Vec<u8>, fingerprint: [u8; 32], variant: DilithiumVariant) -> Self {
        PluginSignature {
            signature,
            signer_fingerprint: fingerprint,
            variant,
        }
    }

    /// Create from raw Dilithium Signature
    pub fn from_dilithium(sig: &Signature, fingerprint: [u8; 32]) -> Self {
        PluginSignature {
            signature: sig.as_bytes().to_vec(),
            signer_fingerprint: fingerprint,
            variant: sig.variant(),
        }
    }
}

// ============================================================================
// Signature Chain
// ============================================================================

/// Signature chain entry (for multi-level signing)
#[derive(Debug, Clone)]
pub struct SignatureChainEntry {
    /// Subject public key fingerprint
    pub subject: [u8; 32],
    /// Signer public key fingerprint
    pub signer: [u8; 32],
    /// Signature over subject's public key
    pub signature: PluginSignature,
}

/// Signature chain (certificate chain analog)
#[derive(Debug, Clone)]
pub struct SignatureChain {
    /// Chain entries from leaf to root
    pub entries: Vec<SignatureChainEntry>,
}

impl SignatureChain {
    /// Create empty signature chain
    pub fn new() -> Self {
        SignatureChain {
            entries: Vec::new(),
        }
    }

    /// Add entry to chain
    pub fn push(&mut self, entry: SignatureChainEntry) {
        self.entries.push(entry);
    }

    /// Get chain length
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for SignatureChain {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Signature Verification
// ============================================================================

/// Trusted root key store
pub struct TrustStore {
    /// Trusted root public keys (fingerprint -> public key)
    roots: Vec<(PublicKey, [u8; 32])>,
    /// Verification policy
    policy: SignaturePolicy,
    /// Dilithium variant
    variant: DilithiumVariant,
}

impl TrustStore {
    /// Create a new trust store
    pub fn new(policy: SignaturePolicy) -> Self {
        TrustStore {
            roots: Vec::new(),
            policy,
            variant: DilithiumVariant::Dilithium3,
        }
    }

    /// Add a trusted root key
    pub fn add_root(&mut self, key: PublicKey, fingerprint: [u8; 32]) {
        self.roots.push((key, fingerprint));
    }

    /// Set verification policy
    pub fn set_policy(&mut self, policy: SignaturePolicy) {
        self.policy = policy;
    }

    /// Get current policy
    pub fn policy(&self) -> SignaturePolicy {
        self.policy
    }

    /// Find root key by fingerprint
    pub fn find_root(&self, fingerprint: &[u8; 32]) -> Option<&PublicKey> {
        for (key, fp) in &self.roots {
            if fp == fingerprint {
                return Some(key);
            }
        }
        None
    }

    /// Verify plugin signature
    /// @param plugin_data: Raw plugin binary data
    /// @param signature: Plugin signature
    /// @return: Ok(true) if signature is valid, Ok(false) if invalid (Permissive),
    /// Err if verification fails (Enforced)
    pub fn verify_plugin_signature(
        &self,
        plugin_data: &[u8],
        signature: &PluginSignature,
    ) -> Result<bool, PluginError> {
        if self.policy == SignaturePolicy::Disabled {
            return Ok(true);
        }

        let pk = self.find_root(&signature.signer_fingerprint)
            .ok_or_else(|| PluginError::InvalidPlugin(
                String::from("Signer key not found in trust store")
            ))?;

        let dilithium = Dilithium::new(signature.variant);

        let sig = dilithium_signature_from_bytes(&signature.signature, signature.variant)
            .ok_or_else(|| PluginError::InvalidPlugin(
                String::from("Invalid signature data")
            ))?;

        match dilithium.verify(pk, plugin_data, &sig) {
            Ok(true) => Ok(true),
            Ok(false) => {
                if self.policy == SignaturePolicy::Permissive {
                    Ok(false)
                } else {
                    Err(PluginError::InvalidPlugin(
                        String::from("Plugin signature verification failed")
                    ))
                }
            }
            Err(DilithiumError::VerificationFailed) => {
                if self.policy == SignaturePolicy::Permissive {
                    Ok(false)
                } else {
                    Err(PluginError::InvalidPlugin(
                        String::from("Dilithium verification failed")
                    ))
                }
            }
            Err(_) => {
                Err(PluginError::InvalidPlugin(
                    String::from("Signature verification error")
                ))
            }
        }
    }

    /// Verify signature chain
    /// Validates each link in the chain from leaf to root,
    /// ensuring each certificate is signed by the next one,
    /// and the root is signed by a trusted key.
    pub fn verify_plugin_chain(
        &self,
        chain: &SignatureChain,
    ) -> Result<bool, PluginError> {
        if self.policy == SignaturePolicy::Disabled {
            return Ok(true);
        }

        if chain.is_empty() {
            return Err(PluginError::InvalidPlugin(
                String::from("Empty signature chain")
            ));
        }

        for entry in &chain.entries {
            let pk = self.find_root(&entry.signer)
                .ok_or_else(|| PluginError::InvalidPlugin(
                    String::from("Chain signer key not found in trust store")
                ))?;

            let dilithium = Dilithium::new(entry.signature.variant);

            let sig = dilithium_signature_from_bytes(
                &entry.signature.signature,
                entry.signature.variant,
            ).ok_or_else(|| PluginError::InvalidPlugin(
                String::from("Invalid chain signature data")
            ))?;

            let subject_bytes = entry.subject;

            match dilithium.verify(pk, &subject_bytes, &sig) {
                Ok(true) => continue,
                Ok(false) | Err(_) => {
                    if self.policy == SignaturePolicy::Permissive {
                        continue;
                    } else {
                        return Err(PluginError::InvalidPlugin(
                            String::from("Signature chain verification failed")
                        ));
                    }
                }
            }
        }

        Ok(true)
    }

    /// Check plugin integrity (hash verification)
    /// Computes SHA-256 of the plugin data and compares against expected hash.
    /// @param plugin_data: Raw plugin binary data
    /// @param expected_hash: Expected SHA-256 hash
    /// @return: Ok(true) if hash matches
    pub fn check_plugin_integrity(
        &self,
        plugin_data: &[u8],
        expected_hash: &[u8; 32],
    ) -> Result<bool, PluginError> {
        let computed = compute_plugin_hash(plugin_data);
        if &computed == expected_hash {
            Ok(true)
        } else {
            if self.policy == SignaturePolicy::Permissive {
                Ok(false)
            } else {
                Err(PluginError::InvalidPlugin(
                    String::from("Plugin integrity check failed: hash mismatch")
                ))
            }
        }
    }
}

/// Create a Dilithium Signature from raw bytes
/// Returns None if the byte slice has incorrect length for the variant.
fn dilithium_signature_from_bytes(bytes: &[u8], variant: DilithiumVariant) -> Option<Signature> {
    let expected_len = match variant {
        DilithiumVariant::Dilithium2 => 2420,
        DilithiumVariant::Dilithium3 => 3293,
        DilithiumVariant::Dilithium5 => 4595,
    };

    if bytes.len() != expected_len {
        return None;
    }

    let mut sig = Signature::new(variant);
    let sig_bytes = sig.as_mut_ptr();
    // SAFETY: sig_bytes points to a valid Signature buffer of expected_len bytes.
    // bytes.len() == expected_len guarantees no out-of-bounds.
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), sig_bytes, expected_len);
    }
    Some(sig)
}

/// Compute SHA-256 hash of plugin data
/// Uses platform-specific accelerated SHA-256 when available.
pub fn compute_plugin_hash(data: &[u8]) -> [u8; 32] {
    #[cfg(feature = "loongarch64")]
    {
        crate::hal::loongarch64::lasx::sha256_hash(data)
    }
    #[cfg(not(feature = "loongarch64"))]
    {
        scalar_sha256_hash(data)
    }
}

/// Scalar fallback SHA-256 hash
/// Full software implementation following FIPS 180-4.
#[cfg(not(feature = "loongarch64"))]
fn scalar_sha256_hash(data: &[u8]) -> [u8; 32] {
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
        0x391c0cb3, 0x4ed8aa4a, 0x5b9aca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut state: [u32; 8] = [
        0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_ff53,
        0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19,
    ];

    let bit_len = (data.len() as u64) * 8;

    let mut i = 0;
    while i + 64 <= data.len() {
        sha256_compress(&mut state, &data[i..i + 64], &K);
        i += 64;
    }

    let mut block = [0u8; 64];
    let remaining = data.len() - i;
    block[..remaining].copy_from_slice(&data[i..]);
    block[remaining] = 0x80;

    if remaining >= 56 {
        sha256_compress(&mut state, &block, &K);
        block = [0u8; 64];
    }

    block[56..64].copy_from_slice(&bit_len.to_be_bytes());
    sha256_compress(&mut state, &block, &K);

    let mut result = [0u8; 32];
    for j in 0..8 {
        result[j * 4..j * 4 + 4].copy_from_slice(&state[j].to_be_bytes());
    }
    result
}

/// SHA-256 compression function
#[cfg(not(feature = "loongarch64"))]
fn sha256_compress(state: &mut [u32; 8], block: &[u8], k: &[u32; 64]) {
    let mut w = [0u32; 64];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
    }

    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;

    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ (!e & g);
        let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

impl Default for TrustStore {
    fn default() -> Self {
        Self::new(SignaturePolicy::default())
    }
}
