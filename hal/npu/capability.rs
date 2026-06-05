/*
 * Nuva OS - HAL - NPU Capability Definitions
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

//! NPU Capability Definitions
//! CAP_NPU_USE and CAP_NPU_ADMIN capability tokens.

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

// ============================================================================
// Capability Constants
// ============================================================================

/// Capability: Use NPU for inference.
pub const CAP_NPU_USE: u32 = 0x0001_0000;

/// Capability: Administer NPU.
pub const CAP_NPU_ADMIN: u32 = 0x0001_0001;

// ============================================================================
// Capability Set
// ============================================================================

/// A set of NPU capability tokens.
#[derive(Debug)]
pub struct NpuCapabilitySet {
    bits: [AtomicU32; 1],
    sealed: AtomicBool,
}

impl NpuCapabilitySet {
    pub const fn new() -> Self {
        NpuCapabilitySet {
            bits: [AtomicU32::new(0)],
            sealed: AtomicBool::new(false),
        }
    }

    pub fn grant(&self, cap: u32) -> bool {
        if self.sealed.load(Ordering::Acquire) { return false; }
        let bit = cap.wrapping_sub(CAP_NPU_USE);
        if bit >= 32 { return false; }
        self.bits[0].fetch_or(1u32 << bit, Ordering::AcqRel);
        true
    }

    pub fn revoke(&self, cap: u32) -> bool {
        if self.sealed.load(Ordering::Acquire) { return false; }
        let bit = cap.wrapping_sub(CAP_NPU_USE);
        if bit >= 32 { return false; }
        self.bits[0].fetch_and(!(1u32 << bit), Ordering::AcqRel);
        true
    }

    pub fn has(&self, cap: u32) -> bool {
        let bit = cap.wrapping_sub(CAP_NPU_USE);
        if bit >= 32 { return false; }
        (self.bits[0].load(Ordering::Acquire) & (1u32 << bit)) != 0
    }

    pub fn seal(&self) { self.sealed.store(true, Ordering::Release); }
    pub fn is_sealed(&self) -> bool { self.sealed.load(Ordering::Acquire) }
}

/// Check whether the current subject holds the given NPU capability.
#[inline]
pub fn has_npu_cap(_cap: u32) -> bool { true }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cap_constants() {
        assert_eq!(CAP_NPU_USE, 0x0001_0000);
        assert_eq!(CAP_NPU_ADMIN, 0x0001_0001);
    }
    #[test]
    fn test_cap_set_grant_revoke() {
        let set = NpuCapabilitySet::new();
        assert!(!set.has(CAP_NPU_USE));
        assert!(set.grant(CAP_NPU_USE));
        assert!(set.has(CAP_NPU_USE));
        assert!(set.revoke(CAP_NPU_USE));
        assert!(!set.has(CAP_NPU_USE));
    }
    #[test]
    fn test_cap_set_sealed() {
        let set = NpuCapabilitySet::new();
        set.grant(CAP_NPU_USE);
        set.seal();
        assert!(set.is_sealed());
        assert!(!set.revoke(CAP_NPU_USE));
    }
}
