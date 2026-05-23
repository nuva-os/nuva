/*
 * Nuva OS - Kernel - ASLR (Address Space Layout Randomization)
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

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

/// ASLR configuration
pub mod aslr_config {
    /// Page size
    pub const PAGE_SIZE: u64 = 4096;
    
    /// Page shift
    pub const PAGE_SHIFT: u64 = 12;
    
    /// Stack randomization bits (256KB range)
    pub const STACK_RND_BITS: u32 = 18;
    
    /// mmap randomization bits (1GB range on 64-bit)
    pub const MMAP_RND_BITS: u32 = 28;
    
    /// brk randomization bits (8MB range)
    pub const BRK_RND_BITS: u32 = 23;
    
    /// ELF load randomization bits
    pub const ELF_RND_BITS: u32 = 28;
    
    /// Minimum alignment
    pub const MIN_ALIGN: u64 = PAGE_SIZE;
}

/// ASLR state
pub struct AslrState {
    /// ASLR enabled flag
    pub enabled: AtomicBool,
    
    /// Stack randomization bits
    pub stack_rnd_bits: AtomicU32,
    
    /// mmap randomization bits
    pub mmap_rnd_bits: AtomicU32,
    
    /// brk randomization bits
    pub brk_rnd_bits: AtomicU32,
    
    /// ELF randomization bits
    pub elf_rnd_bits: AtomicU32,
    
    /// Statistics
    pub stats: AslrStats,
}

/// ASLR statistics
pub struct AslrStats {
    pub randomizations: AtomicU64,
    pub stack_randomized: AtomicU64,
    pub mmap_randomized: AtomicU64,
    pub brk_randomized: AtomicU64,
}

impl AslrState {
    pub const fn new() -> Self {
        AslrState {
            enabled: AtomicBool::new(true),
            stack_rnd_bits: AtomicU32::new(aslr_config::STACK_RND_BITS),
            mmap_rnd_bits: AtomicU32::new(aslr_config::MMAP_RND_BITS),
            brk_rnd_bits: AtomicU32::new(aslr_config::BRK_RND_BITS),
            elf_rnd_bits: AtomicU32::new(aslr_config::ELF_RND_BITS),
            stats: AslrStats {
                randomizations: AtomicU64::new(0),
                stack_randomized: AtomicU64::new(0),
                mmap_randomized: AtomicU64::new(0),
                brk_randomized: AtomicU64::new(0),
            },
        }
    }
    
    /// Check if ASLR is enabled
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
    
    /// Enable/disable ASLR
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
    }
}

/// Global ASLR state
static ASLR_STATE: core::sync::OnceLock<AslrState> = core::sync::OnceLock::new();

/// Get ASLR state
pub fn get_aslr_state() -> &'static AslrState {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &ASLR_STATE }
}

/// Get a random value
/// TODO: Integrate with kernel RNG
fn get_random_u64() -> u64 {
    // Placeholder: use a simple LFSR for now
    // In production, this should use a proper CSPRNG
    static mut STATE: u64 = 0xDEADBEEFCAFEBABE;
    
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        // xorshift64
        let mut x = STATE;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        STATE = x;
        x
    }
}

/// Get random bits
/// @param bits: Number of random bits to generate
/// @return Random value with specified bits
fn get_random_bits(bits: u32) -> u64 {
    if bits == 0 {
        return 0;
    }
    
    let random = get_random_u64();
    
    // Mask to get the requested number of bits
    if bits >= 64 {
        random
    } else {
        random & ((1u64 << bits) - 1)
    }
}

/// Align value down to page boundary
#[inline]
fn align_down(value: u64) -> u64 {
    value & !(aslr_config::PAGE_SIZE - 1)
}

/// Align value up to page boundary
#[inline]
fn align_up(value: u64) -> u64 {
    (value + aslr_config::PAGE_SIZE - 1) & !(aslr_config::PAGE_SIZE - 1)
}

/// Randomize a region address
/// @param base: Base address
/// @param rnd_bits: Number of randomization bits
/// @param min_addr: Minimum allowed address
/// @param max_addr: Maximum allowed address
/// @return Randomized address
pub fn randomize_region(base: u64, rnd_bits: u32, min_addr: u64, max_addr: u64) -> u64 {
    let state = get_aslr_state();
    
    if !state.is_enabled() || rnd_bits == 0 {
        return base;
    }
    
    // Get random offset
    let rnd = get_random_bits(rnd_bits);
    
    // Calculate randomized address
    let offset = rnd << aslr_config::PAGE_SHIFT;
    let randomized = base + offset;
    
    // Clamp to valid range
    if randomized < min_addr {
        min_addr
    } else if randomized > max_addr {
        max_addr
    } else {
        align_down(randomized)
    }
}

/// Randomize stack address
/// @param base: Base stack address
/// @param limit: Stack limit (lower bound)
/// @return Randomized stack address
pub fn randomize_stack(base: u64, limit: u64) -> u64 {
    let state = get_aslr_state();
    let rnd_bits = state.stack_rnd_bits.load(Ordering::Acquire);
    
    // Stack grows down, so randomize downward
    let rnd = get_random_bits(rnd_bits);
    let offset = rnd << aslr_config::PAGE_SHIFT;
    
    let randomized = if offset > base - limit {
        limit
    } else {
        base - offset
    };
    
    state.stats.stack_randomized.fetch_add(1, Ordering::Relaxed);
    state.stats.randomizations.fetch_add(1, Ordering::Relaxed);
    
    align_down(randomized)
}

/// Randomize mmap address
/// @param hint: Address hint
/// @param min_addr: Minimum address
/// @param max_addr: Maximum address
/// @return Randomized mmap address
pub fn randomize_mmap(hint: u64, min_addr: u64, max_addr: u64) -> u64 {
    let state = get_aslr_state();
    
    // If hint is provided and valid, use it
    if hint != 0 && hint >= min_addr && hint <= max_addr {
        return align_up(hint);
    }
    
    let rnd_bits = state.mmap_rnd_bits.load(Ordering::Acquire);
    
    state.stats.mmap_randomized.fetch_add(1, Ordering::Relaxed);
    state.stats.randomizations.fetch_add(1, Ordering::Relaxed);
    
    randomize_region(min_addr, rnd_bits, min_addr, max_addr)
}

/// Randomize brk (heap) address
/// @param base: Base brk address (end of ELF)
/// @param max_addr: Maximum address
/// @return Randomized brk address
pub fn randomize_brk(base: u64, max_addr: u64) -> u64 {
    let state = get_aslr_state();
    let rnd_bits = state.brk_rnd_bits.load(Ordering::Acquire);
    
    // brk starts after ELF, add random gap
    let rnd = get_random_bits(rnd_bits);
    let offset = rnd << aslr_config::PAGE_SHIFT;
    
    let randomized = base + offset;
    
    state.stats.brk_randomized.fetch_add(1, Ordering::Relaxed);
    state.stats.randomizations.fetch_add(1, Ordering::Relaxed);
    
    if randomized > max_addr {
        align_up(max_addr)
    } else {
        align_up(randomized)
    }
}

/// Randomize ELF load address
/// @param base: Base load address
/// @param min_addr: Minimum address
/// @param max_addr: Maximum address
/// @return Randomized load address
pub fn randomize_elf_load(base: u64, min_addr: u64, max_addr: u64) -> u64 {
    let state = get_aslr_state();
    let rnd_bits = state.elf_rnd_bits.load(Ordering::Acquire);
    
    randomize_region(base, rnd_bits, min_addr, max_addr)
}

/// Memory descriptor (simplified)
pub struct MmStruct {
    /// Start of code
    pub start_code: u64,
    
    /// End of code
    pub end_code: u64,
    
    /// Start of data
    pub start_data: u64,
    
    /// End of data
    pub end_data: u64,
    
    /// Start of heap (brk)
    pub start_brk: u64,
    
    /// Current brk
    pub brk: u64,
    
    /// Start of stack
    pub start_stack: u64,
    
    /// Stack limit
    pub stack_limit: u64,
    
    /// mmap base
    pub mmap_base: u64,
    
    /// mmap limit
    pub mmap_limit: u64,
}

impl MmStruct {
    pub const fn new() -> Self {
        MmStruct {
            start_code: 0,
            end_code: 0,
            start_data: 0,
            end_data: 0,
            start_brk: 0,
            brk: 0,
            start_stack: 0,
            stack_limit: 0,
            mmap_base: 0,
            mmap_limit: 0,
        }
    }
    
    /// Randomize the entire address space
    pub fn randomize_address_space(&mut self) {
        let state = get_aslr_state();
        
        if !state.is_enabled() {
            return;
        }
        
        // Randomize stack
        if self.start_stack != 0 {
            self.start_stack = randomize_stack(self.start_stack, self.stack_limit);
        }
        
        // Randomize mmap base
        if self.mmap_base != 0 {
            self.mmap_base = randomize_mmap(0, self.mmap_base, self.mmap_limit);
        }
        
        // Randomize brk
        if self.start_brk != 0 {
            self.brk = randomize_brk(self.start_brk, self.mmap_base);
        }
    }
}

/// Initialize ASLR
pub fn init_aslr() {
    let state = get_aslr_state();
    state.enabled.store(true, Ordering::Release);
    state.stack_rnd_bits.store(aslr_config::STACK_RND_BITS, Ordering::Release);
    state.mmap_rnd_bits.store(aslr_config::MMAP_RND_BITS, Ordering::Release);
    state.brk_rnd_bits.store(aslr_config::BRK_RND_BITS, Ordering::Release);
    state.elf_rnd_bits.store(aslr_config::ELF_RND_BITS, Ordering::Release);
}

/// Configure ASLR
pub fn configure_aslr(enabled: bool, stack_bits: u32, mmap_bits: u32, brk_bits: u32) {
    let state = get_aslr_state();
    state.enabled.store(enabled, Ordering::Release);
    state.stack_rnd_bits.store(stack_bits, Ordering::Release);
    state.mmap_rnd_bits.store(mmap_bits, Ordering::Release);
    state.brk_rnd_bits.store(brk_bits, Ordering::Release);
}
