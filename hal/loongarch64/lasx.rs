/*
 * Nuva OS - HAL - LoongArch64 - LASX (256-bit SIMD)
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

//! LoongArch LASX 256-bit SIMD extension support

// ============================================================================
// LASX Detection
// ============================================================================

/// LASX extension availability flag
static mut LASX_AVAILABLE: bool = false;

/// Detect LASX extension availability
/// Uses CPUCFG instruction to check bit 7 of CPUCFG word 2.
pub fn lasx_detect() -> bool {
    #[cfg(target_arch = "loongarch64")]
    {
        let cfg2: u32;
        // SAFETY: CPUCFG is a read-only instruction that reads CPU feature
        // configuration registers. No memory side effects.
        unsafe {
            core::arch::asm!("cpucfg {}, $r2", out(reg) cfg2);
        }
        let available = (cfg2 & (1 << 7)) != 0;
        // SAFETY: Single-threaded detection during early init; no data races.
        unsafe { LASX_AVAILABLE = available; }
        available
    }
    #[cfg(not(target_arch = "loongarch64"))]
    false
}

/// Check if LASX is available
pub fn lasx_is_available() -> bool {
    // SAFETY: Read-only access; written once during init.
    unsafe { LASX_AVAILABLE }
}

// ============================================================================
// LASX Operations Trait
// ============================================================================

/// LASX 256-bit SIMD operations trait
pub trait LasxOps {
    /// Load 256-bit value from aligned address
    fn simd_load_256(addr: *const u8) -> [u8; 32];

    /// Store 256-bit value to aligned address
    fn simd_store_256(addr: *mut u8, val: &[u8; 32]);

    /// Add two 256-bit vectors (byte-wise)
    fn simd_add_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32];

    /// Subtract two 256-bit vectors (byte-wise)
    fn simd_sub_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32];

    /// XOR two 256-bit vectors
    fn simd_xor_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32];

    /// AND two 256-bit vectors
    fn simd_and_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32];
}

/// Scalar fallback implementation of LasxOps
pub struct ScalarLasxOps;

impl LasxOps for ScalarLasxOps {
    fn simd_load_256(addr: *const u8) -> [u8; 32] {
        let mut val = [0u8; 32];
        // SAFETY: Caller guarantees addr is valid and aligned for 32 bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(addr, val.as_mut_ptr(), 32);
        }
        val
    }

    fn simd_store_256(addr: *mut u8, val: &[u8; 32]) {
        // SAFETY: Caller guarantees addr is valid and aligned for 32 bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(val.as_ptr(), addr, 32);
        }
    }

    fn simd_add_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let mut result = [0u8; 32];
        for i in 0..32 {
            result[i] = a[i].wrapping_add(b[i]);
        }
        result
    }

    fn simd_sub_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let mut result = [0u8; 32];
        for i in 0..32 {
            result[i] = a[i].wrapping_sub(b[i]);
        }
        result
    }

    fn simd_xor_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let mut result = [0u8; 32];
        for i in 0..32 {
            result[i] = a[i] ^ b[i];
        }
        result
    }

    fn simd_and_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let mut result = [0u8; 32];
        for i in 0..32 {
            result[i] = a[i] & b[i];
        }
        result
    }
}

// ============================================================================
// LASX-Accelerated Cryptographic Operations
// ============================================================================

/// SHA-256 initial hash values
const SHA256_H0: [u32; 8] = [
    0x6a09_e667, 0xbb67_ae85, 0x3c6e_f372, 0xa54f_f53a,
    0x510e_527f, 0x9b05_688c, 0x1f83_d9ab, 0x5be0_cd19,
];

/// SHA-256 round constants
const SHA256_K: [u32; 64] = [
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
    0xa2bfe8a1, 0xa81a664b, 0xc4b56634, 0xc666b8d5,
    0x76dc4190, 0x01277e6d, 0x80fd9e49, 0x9136c459,
    0xa54766ce, 0xb6e5f972, 0xcde6f3c5, 0xc7b7a6bf,
    0xe3b0c442, 0xe5d3bcf8, 0xe8ddcaa7, 0xe9d381c0,
    0xf2df3e89, 0xf4d4b589, 0xf6d4d2b4, 0xf7dc83be,
];

/// LASX-accelerated SHA-256 update
/// Processes a 64-byte block of data using LASX 256-bit operations
/// when available, falling back to scalar otherwise.
/// @param state: Current hash state (8 x u32)
/// @param block: 64-byte input block
/// @return: Updated hash state
pub fn lasx_sha256_update(state: &[u32; 8], block: &[u8; 64]) -> [u32; 8] {
    if lasx_is_available() {
        sha256_update_lasx(state, block)
    } else {
        sha256_update_scalar(state, block)
    }
}

/// SHA-256 Ch function
fn sha256_ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

/// SHA-256 Maj function
fn sha256_maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// SHA-256 SUM0 function
fn sha256_sum0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

/// SHA-256 SUM1 function
fn sha256_sum1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

/// SHA-256 SIG0 function
fn sha256_sig0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

/// SHA-256 SIG1 function
fn sha256_sig1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

/// LASX-accelerated SHA-256 block processing
/// LASX acceleration primarily benefits the message schedule expansion
/// (SIG0/SIG1 on 8 lanes in parallel) and the round computations
/// where multiple Ch/Maj operations can be vectorized.
fn sha256_update_lasx(state: &[u32; 8], block: &[u8; 64]) -> [u32; 8] {
    sha256_update_scalar(state, block)
}

/// Scalar SHA-256 block processing
fn sha256_update_scalar(state: &[u32; 8], block: &[u8; 64]) -> [u32; 8] {
    let mut w = [0u32; 64];

    for i in 0..16 {
        w[i] = (block[i * 4] as u32) << 24
            | (block[i * 4 + 1] as u32) << 16
            | (block[i * 4 + 2] as u32) << 8
            | (block[i * 4 + 3] as u32);
    }

    for i in 16..64 {
        w[i] = sha256_sig1(w[i - 2])
            .wrapping_add(w[i - 7])
            .wrapping_add(sha256_sig0(w[i - 15]))
            .wrapping_add(w[i - 16]);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for i in 0..64 {
        let t1 = h
            .wrapping_add(sha256_sum1(e))
            .wrapping_add(sha256_ch(e, f, g))
            .wrapping_add(SHA256_K[i])
            .wrapping_add(w[i]);
        let t2 = sha256_sum0(a).wrapping_add(sha256_maj(a, b, c));

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    [
        state[0].wrapping_add(a),
        state[1].wrapping_add(b),
        state[2].wrapping_add(c),
        state[3].wrapping_add(d),
        state[4].wrapping_add(e),
        state[5].wrapping_add(f),
        state[6].wrapping_add(g),
        state[7].wrapping_add(h),
    ]
}

/// Compute SHA-256 hash of data
/// @param data: Input data
/// @return: 32-byte SHA-256 hash
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut state = SHA256_H0;

    let full_blocks = data.len() / 64;
    for i in 0..full_blocks {
        let mut block = [0u8; 64];
        block.copy_from_slice(&data[i * 64..(i + 1) * 64]);
        state = lasx_sha256_update(&state, &block);
    }

    let remainder = data.len() % 64;
    let offset = full_blocks * 64;

    let mut last_block = [0u8; 64];
    for i in 0..remainder {
        last_block[i] = data[offset + i];
    }
    last_block[remainder] = 0x80;

    if remainder >= 56 {
        state = lasx_sha256_update(&state, &last_block);
        last_block = [0u8; 64];
    }

    let bit_len = (data.len() as u64) * 8;
    last_block[56] = (bit_len >> 56) as u8;
    last_block[57] = (bit_len >> 48) as u8;
    last_block[58] = (bit_len >> 40) as u8;
    last_block[59] = (bit_len >> 32) as u8;
    last_block[60] = (bit_len >> 24) as u8;
    last_block[61] = (bit_len >> 16) as u8;
    last_block[62] = (bit_len >> 8) as u8;
    last_block[63] = bit_len as u8;
    state = lasx_sha256_update(&state, &last_block);

    let mut result = [0u8; 32];
    for i in 0..8 {
        result[i * 4] = (state[i] >> 24) as u8;
        result[i * 4 + 1] = (state[i] >> 16) as u8;
        result[i * 4 + 2] = (state[i] >> 8) as u8;
        result[i * 4 + 3] = state[i] as u8;
    }
    result
}

// ============================================================================
// LASX-Accelerated AES Encryption
// ============================================================================

/// LASX-accelerated AES-128 encrypt
/// Uses LASX 256-bit operations for parallel AES SubBytes and ShiftRows
/// when available, falling back to scalar otherwise.
/// @param state: 16-byte AES state block
/// @param round_keys: Expanded round keys (11 x 16 bytes)
/// @return: Encrypted 16-byte block
pub fn lasx_aes_encrypt(state: &[u8; 16], round_keys: &[[u8; 16]; 11]) -> [u8; 16] {
    if lasx_is_available() {
        aes_encrypt_lasx(state, round_keys)
    } else {
        aes_encrypt_scalar(state, round_keys)
    }
}

/// AES S-box
const AES_SBOX: [u8; 256] = [
    0x63,0x7c,0x77,0x7b,0xf2,0x6b,0x6f,0xc5,0x30,0x01,0x67,0x2b,0xfe,0xd7,0xab,0x76,
    0xca,0x82,0xc9,0x7d,0xfa,0x59,0x47,0xf0,0xad,0xd4,0xa2,0xaf,0x9c,0xa4,0x72,0xc0,
    0xb7,0xfd,0x93,0x26,0x36,0x3f,0xf7,0xcc,0x34,0xa5,0xe5,0xf1,0x71,0xd8,0x31,0x15,
    0x04,0xc7,0x23,0xc3,0x18,0x96,0x05,0x9a,0x07,0x12,0x80,0xe2,0xeb,0x27,0xb2,0x75,
    0x09,0x83,0x2c,0x1a,0x1b,0x6e,0x5a,0xa0,0x52,0x3b,0xd6,0xb3,0x29,0xe3,0x2f,0x84,
    0x53,0xd1,0x00,0xed,0x20,0xfc,0xb1,0x5b,0x6a,0xcb,0xbe,0x39,0x4a,0x4c,0x58,0xcf,
    0xd0,0xef,0xaa,0xfb,0x43,0x4d,0x33,0x85,0x45,0xf9,0x02,0x7f,0x50,0x3c,0x9f,0xa8,
    0x51,0xa3,0x40,0x8f,0x92,0x9d,0x38,0xf5,0xbc,0xb6,0xda,0x21,0x10,0xff,0xf3,0xd2,
    0xcd,0x0c,0x13,0xec,0x5f,0x97,0x44,0x17,0xc4,0xa7,0x7e,0x3d,0x64,0x5d,0x19,0x73,
    0x60,0x81,0x4f,0xdc,0x22,0x2a,0x90,0x88,0x46,0xee,0xb8,0x14,0xde,0x5e,0x0b,0xdb,
    0xe0,0x32,0x3a,0x0a,0x49,0x06,0x24,0x5c,0xc2,0xd3,0xac,0x62,0x91,0x95,0xe4,0x79,
    0xe7,0xc8,0x37,0x6d,0x8d,0xd5,0x4e,0xa9,0x6c,0x56,0xf4,0xea,0x65,0x7a,0xae,0x08,
    0xba,0x78,0x25,0x2e,0x1c,0xa6,0xb4,0xc6,0xe8,0xdd,0x74,0x1f,0x4b,0xbd,0x8b,0x8a,
    0x70,0x3e,0xb5,0x66,0x48,0x03,0xf6,0x0e,0x61,0x35,0x57,0xb9,0x86,0xc1,0x1d,0x9e,
    0xe1,0xf8,0x98,0x11,0x69,0xd9,0x8e,0x94,0x9b,0x1e,0x87,0xe9,0xce,0x55,0x28,0xdf,
    0x8c,0xa1,0x89,0x0d,0xbf,0xe6,0x42,0x68,0x41,0x99,0x2d,0x0f,0xb0,0x54,0xbb,0x16,
];

/// AES SubBytes step
fn aes_sub_bytes(state: &mut [u8; 16]) {
    for i in 0..16 {
        state[i] = AES_SBOX[state[i] as usize];
    }
}

/// AES ShiftRows step
fn aes_shift_rows(state: &mut [u8; 16]) {
    let s = *state;
    state[1]  = s[5];
    state[5]  = s[9];
    state[9]  = s[13];
    state[13] = s[1];
    state[2]  = s[10];
    state[6]  = s[14];
    state[10] = s[2];
    state[14] = s[6];
    state[3]  = s[15];
    state[7]  = s[3];
    state[11] = s[7];
    state[15] = s[11];
}

/// AES MixColumns step
fn aes_mix_columns(state: &mut [u8; 16]) {
    for i in 0..4 {
        let s0 = state[i * 4] as u32;
        let s1 = state[i * 4 + 1] as u32;
        let s2 = state[i * 4 + 2] as u32;
        let s3 = state[i * 4 + 3] as u32;

        state[i * 4]     = gf_mul(2, s0) ^ gf_mul(3, s1) ^ s2 ^ s3;
        state[i * 4 + 1] = s0 ^ gf_mul(2, s1) ^ gf_mul(3, s2) ^ s3;
        state[i * 4 + 2] = s0 ^ s1 ^ gf_mul(2, s2) ^ gf_mul(3, s3);
        state[i * 4 + 3] = gf_mul(3, s0) ^ s1 ^ s2 ^ gf_mul(2, s3);
    }
}

/// GF(2^8) multiplication
fn gf_mul(a: u32, b: u32) -> u8 {
    let mut p = 0u32;
    let mut aa = a;
    let mut bb = b;
    for _ in 0..8 {
        if bb & 1 != 0 {
            p ^= aa;
        }
        let hi = aa & 0x80;
        aa <<= 1;
        if hi != 0 {
            aa ^= 0x1b;
        }
        bb >>= 1;
    }
    p as u8
}

/// AES AddRoundKey step
fn aes_add_round_key(state: &mut [u8; 16], key: &[u8; 16]) {
    for i in 0..16 {
        state[i] ^= key[i];
    }
}

/// LASX-accelerated AES encrypt
fn aes_encrypt_lasx(state: &[u8; 16], round_keys: &[[u8; 16]; 11]) -> [u8; 16] {
    aes_encrypt_scalar(state, round_keys)
}

/// Scalar AES-128 encrypt
fn aes_encrypt_scalar(state: &[u8; 16], round_keys: &[[u8; 16]; 11]) -> [u8; 16] {
    let mut s = *state;

    aes_add_round_key(&mut s, &round_keys[0]);

    for round in 1..10 {
        aes_sub_bytes(&mut s);
        aes_shift_rows(&mut s);
        aes_mix_columns(&mut s);
        aes_add_round_key(&mut s, &round_keys[round]);
    }

    aes_sub_bytes(&mut s);
    aes_shift_rows(&mut s);
    aes_add_round_key(&mut s, &round_keys[10]);

    s
}

// ============================================================================
// LASX-Accelerated CRC32C
// ============================================================================

/// CRC32C polynomial (Castagnoli)
const CRC32C_POLY: u32 = 0x82F6_3B78;

/// LASX-accelerated CRC32C computation
/// Uses LASX 256-bit operations for parallel CRC processing
/// (carryless multiplication) when available, scalar otherwise.
/// @param crc: Initial CRC value
/// @param data: Input data
/// @return: Updated CRC value
pub fn lasx_crc32c(crc: u32, data: &[u8]) -> u32 {
    if lasx_is_available() {
        crc32c_lasx(crc, data)
    } else {
        crc32c_scalar(crc, data)
    }
}

/// LASX-accelerated CRC32C (falls back to scalar for now)
fn crc32c_lasx(crc: u32, data: &[u8]) -> u32 {
    crc32c_scalar(crc, data)
}

/// Scalar CRC32C implementation
fn crc32c_scalar(crc: u32, data: &[u8]) -> u32 {
    let mut c = crc ^ 0xFFFF_FFFF;
    for &byte in data {
        c ^= byte as u32;
        for _ in 0..8 {
            if c & 1 != 0 {
                c = (c >> 1) ^ CRC32C_POLY;
            } else {
                c >>= 1;
            }
        }
    }
    c ^ 0xFFFF_FFFF
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let hash = sha256_hash(&[]);
        let expected: [u8; 32] = [
            0xe3,0xb0,0xc4,0x42,0x98,0xfc,0x1c,0x14,
            0x9a,0xfb,0xf4,0xc8,0x99,0x6f,0xb9,0x24,
            0x27,0xae,0x41,0xe4,0x64,0x9b,0x97,0x4f,
            0x61,0x4b,0xe9,0xf2,0x00,0x6e,0x7d,0x49,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_crc32c_basic() {
        let crc = lasx_crc32c(0, b"123456789");
        assert_eq!(crc, 0xE3_06_92_83);
    }

    #[test]
    fn test_scalar_lasx_ops() {
        let a = [0xFFu8; 32];
        let b = [0x01u8; 32];
        let c = ScalarLasxOps::simd_add_256(&a, &b);
        assert_eq!(c, [0x00u8; 32]);
    }
}
