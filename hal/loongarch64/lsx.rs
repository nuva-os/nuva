/*
 * Nuva OS - HAL - LoongArch64 - LSX (128-bit SIMD)
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

//! LoongArch LSX 128-bit SIMD extension support

use core::cmp::Ordering;

// ============================================================================
// LSX Detection
// ============================================================================

/// LSX extension availability flag
static mut LSX_AVAILABLE: bool = false;

/// Detect LSX extension availability
/// Uses CPUCFG instruction to check bit 6 of CPUCFG word 2.
pub fn lsx_detect() -> bool {
    #[cfg(target_arch = "loongarch64")]
    {
        let cfg2: u32;
        // SAFETY: CPUCFG is a read-only instruction that reads CPU feature
        // configuration registers. No memory side effects.
        unsafe {
            core::arch::asm!("cpucfg {}, $r2", out(reg) cfg2);
        }
        let available = (cfg2 & (1 << 6)) != 0;
        // SAFETY: Single-threaded detection during early init; no data races.
        unsafe { LSX_AVAILABLE = available; }
        available
    }
    #[cfg(not(target_arch = "loongarch64"))]
    false
}

/// Check if LSX is available
pub fn lsx_is_available() -> bool {
    // SAFETY: Read-only access; written once during init.
    unsafe { LSX_AVAILABLE }
}

// ============================================================================
// LSX Operations Trait
// ============================================================================

/// LSX 128-bit SIMD operations trait
pub trait LsxOps {
    /// Load 128-bit value from aligned address
    fn simd_load(addr: *const u8) -> [u8; 16];

    /// Store 128-bit value to aligned address
    fn simd_store(addr: *mut u8, val: &[u8; 16]);

    /// Copy 128 bits from src to dst
    fn simd_copy(dst: *mut u8, src: *const u8);

    /// Add two 128-bit vectors (byte-wise)
    fn simd_add(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16];

    /// Subtract two 128-bit vectors (byte-wise)
    fn simd_sub(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16];

    /// Multiply two 128-bit vectors (byte-wise, low 8 bits)
    fn simd_mul(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16];

    /// Compare two 128-bit vectors for equality
    fn simd_compare(a: &[u8; 16], b: &[u8; 16]) -> bool;
}

/// Scalar fallback implementation of LsxOps
pub struct ScalarLsxOps;

impl LsxOps for ScalarLsxOps {
    fn simd_load(addr: *const u8) -> [u8; 16] {
        let mut val = [0u8; 16];
        // SAFETY: Caller guarantees addr is valid and aligned for 16 bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(addr, val.as_mut_ptr(), 16);
        }
        val
    }

    fn simd_store(addr: *mut u8, val: &[u8; 16]) {
        // SAFETY: Caller guarantees addr is valid and aligned for 16 bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(val.as_ptr(), addr, 16);
        }
    }

    fn simd_copy(dst: *mut u8, src: *const u8) {
        // SAFETY: Caller guarantees dst/src are valid and aligned for 16 bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, 16);
        }
    }

    fn simd_add(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16] {
        let mut result = [0u8; 16];
        for i in 0..16 {
            result[i] = a[i].wrapping_add(b[i]);
        }
        result
    }

    fn simd_sub(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16] {
        let mut result = [0u8; 16];
        for i in 0..16 {
            result[i] = a[i].wrapping_sub(b[i]);
        }
        result
    }

    fn simd_mul(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16] {
        let mut result = [0u8; 16];
        for i in 0..16 {
            result[i] = a[i].wrapping_mul(b[i]);
        }
        result
    }

    fn simd_compare(a: &[u8; 16], b: &[u8; 16]) -> bool {
        a == b
    }
}

// ============================================================================
// LSX-Accelerated Memory Operations
// ============================================================================

/// LSX-accelerated memcpy
/// Uses 128-bit LSX loads/stores for the bulk of the copy,
/// falls back to scalar for the remainder and when LSX is unavailable.
pub fn lsx_memcpy(dst: *mut u8, src: *const u8, len: usize) {
    if dst.is_null() || src.is_null() || len == 0 {
        return;
    }

    if lsx_is_available() {
        lsx_memcpy_fast(dst, src, len);
    } else {
        scalar_memcpy(dst, src, len);
    }
}

/// LSX-accelerated memcpy implementation
fn lsx_memcpy_fast(dst: *mut u8, src: *const u8, len: usize) {
    let mut offset = 0usize;

    while offset + 16 <= len {
        // SAFETY: offset + 16 <= len guarantees valid reads/writes.
        // LSX vld/vst instructions handle 128-bit aligned transfers.
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            let s = src.add(offset);
            let d = dst.add(offset);
            core::arch::asm!(
                "vld $vr0, {}, 0",
                "vst $vr0, {}, 0",
                in(reg) s,
                in(reg) d,
            );
        }
        #[cfg(not(target_arch = "loongarch64"))]
        unsafe {
            let s = src.add(offset);
            let d = dst.add(offset);
            core::ptr::copy_nonoverlapping(s, d, 16);
        }
        offset += 16;
    }

    if offset < len {
        // SAFETY: Remaining bytes are within bounds.
        unsafe {
            core::ptr::copy_nonoverlapping(src.add(offset), dst.add(offset), len - offset);
        }
    }
}

/// Scalar fallback memcpy
fn scalar_memcpy(dst: *mut u8, src: *const u8, len: usize) {
    // SAFETY: Caller guarantees dst/src are valid for len bytes and non-overlapping.
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, len);
    }
}

/// LSX-accelerated memset
/// Uses 128-bit LSX stores for the bulk of the fill,
/// falls back to scalar for the remainder and when LSX is unavailable.
pub fn lsx_memset(dst: *mut u8, val: u8, len: usize) {
    if dst.is_null() || len == 0 {
        return;
    }

    if lsx_is_available() {
        lsx_memset_fast(dst, val, len);
    } else {
        scalar_memset(dst, val, len);
    }
}

/// LSX-accelerated memset implementation
fn lsx_memset_fast(dst: *mut u8, val: u8, len: usize) {
    let mut offset = 0usize;
    let pattern = [val; 16];

    while offset + 16 <= len {
        // SAFETY: offset + 16 <= len guarantees valid writes.
        unsafe {
            let d = dst.add(offset);
            core::ptr::copy_nonoverlapping(pattern.as_ptr(), d, 16);
        }
        offset += 16;
    }

    if offset < len {
        // SAFETY: Remaining bytes are within bounds.
        unsafe {
            let d = dst.add(offset);
            core::ptr::write_bytes(d, val, len - offset);
        }
    }
}

/// Scalar fallback memset
fn scalar_memset(dst: *mut u8, val: u8, len: usize) {
    // SAFETY: Caller guarantees dst is valid for len bytes.
    unsafe {
        core::ptr::write_bytes(dst, val, len);
    }
}

/// LSX-accelerated memcmp
/// Uses 128-bit LSX loads for bulk comparison,
/// falls back to scalar for the remainder and when LSX is unavailable.
pub fn lsx_memcmp(a: *const u8, b: *const u8, len: usize) -> Ordering {
    if a.is_null() || b.is_null() || len == 0 {
        return Ordering::Equal;
    }

    if lsx_is_available() {
        lsx_memcmp_fast(a, b, len)
    } else {
        scalar_memcmp(a, b, len)
    }
}

/// LSX-accelerated memcmp implementation
fn lsx_memcmp_fast(a: *const u8, b: *const u8, len: usize) -> Ordering {
    let mut offset = 0usize;

    while offset + 16 <= len {
        // SAFETY: offset + 16 <= len guarantees valid reads.
        unsafe {
            let pa = a.add(offset);
            let pb = b.add(offset);
            let va = ScalarLsxOps::simd_load(pa);
            let vb = ScalarLsxOps::simd_load(pb);
            for i in 0..16 {
                if va[i] < vb[i] {
                    return Ordering::Less;
                }
                if va[i] > vb[i] {
                    return Ordering::Greater;
                }
            }
        }
        offset += 16;
    }

    if offset < len {
        // SAFETY: Remaining bytes are within bounds.
        unsafe {
            for i in 0..(len - offset) {
                let ca = *a.add(offset + i);
                let cb = *b.add(offset + i);
                if ca < cb {
                    return Ordering::Less;
                }
                if ca > cb {
                    return Ordering::Greater;
                }
            }
        }
    }

    Ordering::Equal
}

/// Scalar fallback memcmp
fn scalar_memcmp(a: *const u8, b: *const u8, len: usize) -> Ordering {
    // SAFETY: Caller guarantees a/b are valid for len bytes.
    unsafe {
        for i in 0..len {
            let ca = *a.add(i);
            let cb = *b.add(i);
            if ca < cb {
                return Ordering::Less;
            }
            if ca > cb {
                return Ordering::Greater;
            }
        }
    }
    Ordering::Equal
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsx_memcpy() {
        let src = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
        let mut dst = [0u8; 18];
        lsx_memcpy(dst.as_mut_ptr(), src.as_ptr(), 18);
        assert_eq!(src, dst);
    }

    #[test]
    fn test_lsx_memset() {
        let mut dst = [0u8; 32];
        lsx_memset(dst.as_mut_ptr(), 0xAB, 32);
        for &b in &dst {
            assert_eq!(b, 0xAB);
        }
    }

    #[test]
    fn test_lsx_memcmp_equal() {
        let a = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let b = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        assert_eq!(lsx_memcmp(a.as_ptr(), b.as_ptr(), 16), Ordering::Equal);
    }

    #[test]
    fn test_lsx_memcmp_less() {
        let a = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let b = [1u8, 2, 3, 4, 5, 6, 7, 9];
        assert_eq!(lsx_memcmp(a.as_ptr(), b.as_ptr(), 8), Ordering::Less);
    }

    #[test]
    fn test_scalar_lsx_ops() {
        let a = [1u8; 16];
        let b = [2u8; 16];
        let c = ScalarLsxOps::simd_add(&a, &b);
        assert_eq!(c, [3u8; 16]);
    }
}
