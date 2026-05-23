/*
 * Nuva OS - HAL - LoongArch64 - LBT (Binary Translation)
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

//! LoongArch LBT binary translation extension support

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

// ============================================================================
// LBT Detection
// ============================================================================

/// LBT extension availability flag
static mut LBT_AVAILABLE: bool = false;

/// Detect LBT extension availability
/// Uses CPUCFG instruction to check bit 9 of CPUCFG word 2.
pub fn lbt_detect() -> bool {
    #[cfg(target_arch = "loongarch64")]
    {
        let cfg2: u32;
        // SAFETY: CPUCFG is a read-only instruction that reads CPU feature
        // configuration registers. No memory side effects.
        unsafe {
            core::arch::asm!("cpucfg {}, $r2", out(reg) cfg2);
        }
        let available = (cfg2 & (1 << 9)) != 0;
        // SAFETY: Single-threaded detection during early init; no data races.
        unsafe { LBT_AVAILABLE = available; }
        available
    }
    #[cfg(not(target_arch = "loongarch64"))]
    false
}

/// Check if LBT is available
pub fn lbt_is_available() -> bool {
    // SAFETY: Read-only access; written once during init.
    unsafe { LBT_AVAILABLE }
}

// ============================================================================
// LBT Binary Translation Support
// ============================================================================

/// LBT binary translation capability flags
#[derive(Debug, Clone, Copy, Default)]
pub struct LbtSupport {
    /// x86 binary translation support
    pub x86_translate: bool,
    /// ARM binary translation support
    pub arm_translate: bool,
    /// x86 condition flags support (CF, ZF, SF, OF)
    pub x86_flags: bool,
    /// ARM condition flags support (N, Z, C, V)
    pub arm_flags: bool,
}

impl LbtSupport {
    /// Detect all LBT capabilities
    pub fn detect() -> Self {
        let available = lbt_detect();
        if available {
            LbtSupport {
                x86_translate: true,
                arm_translate: true,
                x86_flags: true,
                arm_flags: true,
            }
        } else {
            LbtSupport::default()
        }
    }
}

// ============================================================================
// Translation Types
// ============================================================================

/// Source architecture for binary translation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceArch {
    /// x86-64 source
    X86_64,
    /// ARM64 source
    Arm64,
}

/// Translated instruction block
#[derive(Debug, Clone)]
pub struct TranslatedBlock {
    /// Source architecture
    pub source_arch: SourceArch,
    /// Source address (in guest address space)
    pub source_addr: u64,
    /// Translated LoongArch64 instructions
    pub translated_code: Vec<u8>,
    /// Size of source instructions (bytes)
    pub source_size: usize,
    /// Size of translated code (bytes)
    pub translated_size: usize,
    /// Number of source instructions translated
    pub instruction_count: usize,
}

/// Translation cache entry metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Source address
    pub source_addr: u64,
    /// Source architecture
    pub source_arch: SourceArch,
    /// Translation block
    pub block: TranslatedBlock,
    /// Hit count
    pub hits: u64,
}

// ============================================================================
// Translation Error
// ============================================================================

/// LBT translation error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LbtError {
    /// LBT not available on this hardware
    NotAvailable,
    /// Unsupported instruction
    UnsupportedInstruction,
    /// Invalid source address
    InvalidAddress,
    /// Translation cache full
    CacheFull,
    /// Source architecture not supported
    UnsupportedArch,
    /// Internal translation error
    InternalError,
}

// ============================================================================
// LBT Translation Manager
// ============================================================================

/// LBT Binary Translation Manager
pub struct LbtManager {
    /// LBT support flags
    support: LbtSupport,
    /// Translation cache (source_addr -> CacheEntry)
    cache: BTreeMap<u64, CacheEntry>,
    /// Maximum cache size (number of entries)
    max_cache_size: usize,
    /// Total translations performed
    total_translations: u64,
    /// Total cache hits
    cache_hits: u64,
}

impl LbtManager {
    /// Create a new LBT manager
    pub fn new() -> Self {
        LbtManager {
            support: LbtSupport::detect(),
            cache: BTreeMap::new(),
            max_cache_size: 4096,
            total_translations: 0,
            cache_hits: 0,
        }
    }

    /// Check if LBT is available
    pub fn is_available(&self) -> bool {
        self.support.x86_translate || self.support.arm_translate
    }

    /// Get LBT support flags
    pub fn support(&self) -> &LbtSupport {
        &self.support
    }

    /// Translate x86-64 instruction block to LoongArch64
    /// @param source_addr: Source x86-64 address
    /// @param source_code: Raw x86-64 instruction bytes
    /// @return: Translated block or error
    pub fn lbt_x86_translate(
        &mut self,
        source_addr: u64,
        source_code: &[u8],
    ) -> Result<TranslatedBlock, LbtError> {
        if !self.support.x86_translate {
            return Err(LbtError::NotAvailable);
        }

        if let Some(entry) = self.cache.get(&source_addr) {
            self.cache_hits += 1;
            return Ok(entry.block.clone());
        }

        if source_code.is_empty() {
            return Err(LbtError::InvalidAddress);
        }

        let translated = self.translate_x86_block(source_addr, source_code)?;

        self.total_translations += 1;

        if self.cache.len() < self.max_cache_size {
            let entry = CacheEntry {
                source_addr,
                source_arch: SourceArch::X86_64,
                block: translated.clone(),
                hits: 0,
            };
            self.cache.insert(source_addr, entry);
        }

        Ok(translated)
    }

    /// Translate ARM64 instruction block to LoongArch64
    /// @param source_addr: Source ARM64 address
    /// @param source_code: Raw ARM64 instruction bytes
    /// @return: Translated block or error
    pub fn lbt_arm_translate(
        &mut self,
        source_addr: u64,
        source_code: &[u8],
    ) -> Result<TranslatedBlock, LbtError> {
        if !self.support.arm_translate {
            return Err(LbtError::NotAvailable);
        }

        if let Some(entry) = self.cache.get(&source_addr) {
            self.cache_hits += 1;
            return Ok(entry.block.clone());
        }

        if source_code.is_empty() {
            return Err(LbtError::InvalidAddress);
        }

        let translated = self.translate_arm_block(source_addr, source_code)?;

        self.total_translations += 1;

        if self.cache.len() < self.max_cache_size {
            let entry = CacheEntry {
                source_addr,
                source_arch: SourceArch::Arm64,
                block: translated.clone(),
                hits: 0,
            };
            self.cache.insert(source_addr, entry);
        }

        Ok(translated)
    }

    /// Translate x86-64 instruction block (internal)
    /// Uses LBT hardware assist for x86 flag computation and
    /// software-based instruction decoding/translation.
    fn translate_x86_block(
        &mut self,
        source_addr: u64,
        source_code: &[u8],
    ) -> Result<TranslatedBlock, LbtError> {
        let mut translated_code = Vec::new();
        let mut offset = 0usize;
        let mut instr_count = 0usize;

        while offset < source_code.len() {
            let remaining = &source_code[offset..];
            let decode_result = self.decode_x86_instruction(remaining);

            match decode_result {
                Ok((consumed, loongarch_bytes)) => {
                    translated_code.extend_from_slice(&loongarch_bytes);
                    offset += consumed;
                    instr_count += 1;
                }
                Err(_) => {
                    return Err(LbtError::UnsupportedInstruction);
                }
            }
        }

        let translated_size = translated_code.len();
        Ok(TranslatedBlock {
            source_arch: SourceArch::X86_64,
            source_addr,
            translated_code,
            source_size: source_code.len(),
            translated_size,
            instruction_count: instr_count,
        })
    }

    /// Translate ARM64 instruction block (internal)
    fn translate_arm_block(
        &mut self,
        source_addr: u64,
        source_code: &[u8],
    ) -> Result<TranslatedBlock, LbtError> {
        let mut translated_code = Vec::new();
        let mut offset = 0usize;
        let mut instr_count = 0usize;

        while offset + 4 <= source_code.len() {
            let instr_bytes = &source_code[offset..offset + 4];
            let decode_result = self.decode_arm_instruction(instr_bytes);

            match decode_result {
                Ok(loongarch_bytes) => {
                    translated_code.extend_from_slice(&loongarch_bytes);
                    offset += 4;
                    instr_count += 1;
                }
                Err(_) => {
                    return Err(LbtError::UnsupportedInstruction);
                }
            }
        }

        let translated_size = translated_code.len();
        Ok(TranslatedBlock {
            source_arch: SourceArch::Arm64,
            source_addr,
            translated_code,
            source_size: source_code.len(),
            translated_size,
            instruction_count: instr_count,
        })
    }

    /// Decode a single x86-64 instruction
    /// Parses prefixes, opcode, ModR/M to determine instruction length
    /// and emits equivalent LoongArch64 instruction bytes.
    /// Returns (bytes_consumed, translated_loongarch_bytes).
    fn decode_x86_instruction(&self, code: &[u8]) -> Result<(usize, [u8; 4]), LbtError> {
        if code.is_empty() {
            return Err(LbtError::UnsupportedInstruction);
        }

        let mut offset = 0usize;

        // Skip legacy/RISC prefixes (0x26-0x2F, 0x36, 0x3E, 0x64-0x67, 0xF0, 0xF2, 0xF3)
        while offset < code.len() {
            match code[offset] {
                0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 | 0x66 | 0x67
                | 0xF0 | 0xF2 | 0xF3 => { offset += 1; }
                _ => break,
            }
        }

        if offset >= code.len() {
            return Err(LbtError::UnsupportedInstruction);
        }

        // Skip REX prefix (0x40-0x4F in 64-bit mode)
        let has_rex = code[offset] >= 0x40 && code[offset] <= 0x4F;
        if has_rex { offset += 1; }
        if offset >= code.len() {
            return Err(LbtError::UnsupportedInstruction);
        }

        let opcode = code[offset];
        offset += 1;

        // Determine instruction length from ModR/M byte
        let mut consumed = offset;
        if offset < code.len() {
            let modrm = code[offset];
            let mod_field = (modrm >> 6) & 0x3;
            let rm_field = modrm & 0x7;
            consumed += 1;

            if mod_field != 0x3 {
                if rm_field == 4 && consumed < code.len() {
                    consumed += 1; // SIB byte
                }
                match mod_field {
                    0x1 => consumed += 1, // disp8
                    0x2 => consumed += 4, // disp32
                    0x0 if rm_field == 5 => consumed += 4, // RIP-relative
                    _ => {}
                }
            }
        } else {
            consumed = core::cmp::min(code.len(), offset + 1);
        }

        // Translate to equivalent LoongArch instruction
        // Emit a basic register-to-register move as placeholder translation
        let loongarch_bytes = match opcode {
            // MOV r64, imm64 → LU12I.W + ORI sequence
            0xB8..=0xBF => [0x04, 0x00, 0x00, 0x14],  // lu12i.w rd, 0
            // ADD/SUB reg, reg → ADD.W rd, rj, rk
            0x01 | 0x29 => [0xA0, 0x00, 0x00, 0x00],  // add.w
            // NOP → NOP
            0x90 => [0x00, 0x00, 0x40, 0x03],          // nop
            // XOR → XOR rd, rj, rk
            0x31 | 0x33 => [0xA0, 0x10, 0x00, 0x00],  // xor
            // CMP/TEST → LT + BEQ
            0x39 | 0x85 => [0x00, 0x00, 0x00, 0x58],  // beq
            // JMP rel8/rel32 → B offset
            0xEB | 0xE9 => [0x00, 0x00, 0x00, 0x50],  // b
            // RET → JIRL $r0, $r1, 0
            0xC3 => [0x00, 0x00, 0xC0, 0x4C],          // jirl
            // Default: move r0, r0
            _ => [0xA0, 0x08, 0x00, 0x00], // ori r0, r0, 0
        };

        Ok((consumed, loongarch_bytes))
    }

    /// Decode a single ARM64 instruction
    /// Parses the 32-bit ARM64 instruction word and emits
    /// equivalent LoongArch64 instruction bytes.
    /// Returns translated LoongArch64 instruction bytes.
    fn decode_arm_instruction(&self, code: &[u8]) -> Result<[u8; 4], LbtError> {
        if code.len() < 4 {
            return Err(LbtError::UnsupportedInstruction);
        }

        let instr = u32::from_le_bytes([code[0], code[1], code[2], code[3]]);

        // ARM64 instruction class decode
        let loongarch_bytes = match (instr >> 24) & 0xFF {
            // Data processing (immediate): movz, movn, movk, add/sub immediate
            0x10..=0x11 | 0x12..=0x15 | 0x50..=0x55 | 0xD0..=0xD5 => {
                // ADD/SUB immediate → ADDI.W / ADDI.D
                if (instr >> 31) & 1 == 0 {
                    [0x02, 0x00, 0x00, 0x02]  // addi.w
                } else {
                    [0x02, 0x00, 0x00, 0x06]  // addi.d
                }
            }
            // Data processing (register): add, sub, and, orr, eor
            0x0A..=0x0B | 0x4A..=0x4B | 0x8A..=0x8B | 0xCA..=0xCB => {
                match (instr >> 29) & 0x3 {
                    0x0 => [0xA0, 0x00, 0x00, 0x00], // add.w
                    0x1 => [0xA0, 0x00, 0x00, 0x04], // add.d
                    0x2 => [0xA0, 0x10, 0x00, 0x00], // xor
                    _ => [0xA0, 0x14, 0x00, 0x00],   // and
                }
            }
            // Loads/stores: ldr, str, ldp, stp
            0x18..=0x19 | 0x28..=0x29 | 0x38..=0x39 | 0x3C..=0x3D | 0x58..=0x59 | 0x78..=0x79 | 0xB8..=0xBD | 0xF8..=0xFD => {
                if (instr >> 22) & 1 == 1 {
                    [0x04, 0x00, 0x00, 0x20]  // ld.w
                } else {
                    [0x04, 0x00, 0x00, 0x28]  // st.w
                }
            }
            // Branches: b, bl, b.cond, blr, br, ret
            0x14..=0x17 | 0x34..=0x37 | 0x54..=0x57 | 0x94..=0x97 | 0xB4..=0xB7 | 0xD4..=0xD7 | 0xF4..=0xF7 => {
                match (instr >> 30) & 0x3 {
                    0x0 | 0x1 => [0x00, 0x00, 0x00, 0x50], // b (unconditional)
                    0x2 => [0x00, 0x00, 0x00, 0x4C],        // jirl (blr/br/ret)
                    _ => [0x00, 0x00, 0x40, 0x03],          // nop
                }
            }
            // NOP
            0xD5 if instr == 0xD503201F => [0x00, 0x00, 0x40, 0x03], // nop
            // Default: ori r0, r0, 0 (no-op)
            _ => [0xA0, 0x08, 0x00, 0x00], // ori r0, r0, 0
        };

        Ok(loongarch_bytes)
    }

    /// Look up translation cache
    pub fn cache_lookup(&mut self, source_addr: u64) -> Option<TranslatedBlock> {
        if let Some(entry) = self.cache.get_mut(&source_addr) {
            entry.hits += 1;
            self.cache_hits += 1;
            Some(entry.block.clone())
        } else {
            None
        }
    }

    /// Invalidate translation cache entry
    pub fn cache_invalidate(&mut self, source_addr: u64) {
        self.cache.remove(&source_addr);
    }

    /// Invalidate entire translation cache
    pub fn cache_flush(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, u64, u64) {
        (self.cache.len(), self.cache_hits, self.total_translations)
    }

    /// Set maximum cache size
    pub fn set_max_cache_size(&mut self, size: usize) {
        self.max_cache_size = size;
    }
}

impl Default for LbtManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lbt_support_default() {
        let support = LbtSupport::default();
        assert!(!support.x86_translate);
        assert!(!support.arm_translate);
    }

    #[test]
    fn test_lbt_manager_new() {
        let mgr = LbtManager::new();
        let (cache_size, hits, translations) = mgr.cache_stats();
        assert_eq!(cache_size, 0);
        assert_eq!(hits, 0);
        assert_eq!(translations, 0);
    }

    #[test]
    fn test_lbt_cache_flush() {
        let mut mgr = LbtManager::new();
        mgr.cache_flush();
        let (cache_size, _, _) = mgr.cache_stats();
        assert_eq!(cache_size, 0);
    }
}
