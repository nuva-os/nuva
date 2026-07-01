/*
 * Nuva OS - Kernel - Core - Random
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
use crate::{pr_info};
/*
 * Nuva OS - Kernel - Random Number Generation
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel random number generation and entropy pool.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Entropy Pool Size
pub const ENTROPY_POOL_SIZE: usize = 256;

/// Random State
pub struct RandomState {
    /// Entropy pool
    pub pool: [u8; ENTROPY_POOL_SIZE],
    /// Pool index
    pub index: AtomicU32,
    /// Entropy count (bits)
    pub entropy_count: AtomicU32,
    /// Initialized
    pub initialized: AtomicBool,
    /// Lock
    pub lock: AtomicU32,
}

impl RandomState {
    pub const fn new() -> Self {
        RandomState {
            pool: [0; ENTROPY_POOL_SIZE],
            index: AtomicU32::new(0),
            entropy_count: AtomicU32::new(0),
            initialized: AtomicBool::new(false),
            lock: AtomicU32::new(0),
        }
    }
    
    /// Lock
    fn lock(&self) {
        while self.lock.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_err() {
            core::hint::spin_loop();
        }
    }
    
    /// Unlock
    fn unlock(&self) {
        self.lock.store(0, Ordering::Release);
    }
    
    /// Add entropy
    pub fn add_entropy(&mut self, data: &[u8], entropy_bits: u32) {
        self.lock();
        
        // Mix data into pool
        for &byte in data {
            let idx = self.index.load(Ordering::Acquire) as usize;
            self.pool[idx % ENTROPY_POOL_SIZE] ^= byte;
            self.index.fetch_add(1, Ordering::AcqRel);
        }
        
        // Update entropy count
        self.entropy_count.fetch_add(entropy_bits, Ordering::AcqRel);
        
        // Mark as initialized if enough entropy
        if self.entropy_count.load(Ordering::Acquire) >= 128 {
            self.initialized.store(true, Ordering::Release);
        }
        
        self.unlock();
    }
    
    /// Get bytes
    pub fn get_bytes(&mut self, buf: &mut [u8]) {
        self.lock();
        
        if !self.initialized.load(Ordering::Acquire) {
            // Not enough entropy, use fallback
            self.unlock();
            self.fallback_random(buf);
            return;
        }
        
        // Generate random bytes
        for byte in buf.iter_mut() {
            let idx = self.index.load(Ordering::Acquire) as usize;
            
            // Mix pool
            let mut val = self.pool[idx % ENTROPY_POOL_SIZE];
            val = val.wrapping_add(self.pool[(idx + 1) % ENTROPY_POOL_SIZE]);
            val = val.wrapping_mul(0x5DEECE66D as u8);
            val ^= val >> 5;
            
            *byte = val;
            self.index.fetch_add(1, Ordering::AcqRel);
        }
        
        // Reduce entropy count
        let bits_used = buf.len() as u32 * 8;
        if self.entropy_count.load(Ordering::Acquire) > bits_used {
            self.entropy_count.fetch_sub(bits_used, Ordering::AcqRel);
        }
        
        self.unlock();
    }
    
    /// Fallback random (for early boot)
    fn fallback_random(&self, buf: &mut [u8]) {
        // Use simple LFSR for early boot
        static mut STATE: u64 = 0xDEADBEEFCAFEBABE;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            for byte in buf.iter_mut() {
                // LFSR step
                let bit = ((STATE >> 63) ^ (STATE >> 62) ^ (STATE >> 60) ^ (STATE >> 59)) & 1;
                STATE = (STATE << 1) | bit;
                *byte = STATE as u8;
            }
        }
    }
    
    /// Get random u32
    pub fn get_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.get_bytes(&mut buf);
        u32::from_le_bytes(buf)
    }
    
    /// Get random u64
    pub fn get_u64(&mut self) -> u64 {
        let mut buf = [0u8; 8];
        self.get_bytes(&mut buf);
        u64::from_le_bytes(buf)
    }
    
    /// Get random in range
    pub fn get_range(&mut self, min: u32, max: u32) -> u32 {
        if max <= min {
            return min;
        }
        
        let range = max - min;
        let val = self.get_u32();
        
        min + (val % range)
    }
}

/// PRNG State (for fast random)
pub struct PrngState {
    /// State
    pub state: AtomicU64,
    /// Increment
    pub inc: AtomicU64,
}

impl PrngState {
    pub const fn new() -> Self {
        PrngState {
            state: AtomicU64::new(0),
            inc: AtomicU64::new(0),
        }
    }
    
    /// Seed
    pub fn seed(&mut self, seed: u64) {
        self.state.store(seed, Ordering::Release);
        self.inc.store(0xDEADBEEFCAFEBABE, Ordering::Release);
    }
    
    /// Next random (PCG algorithm)
    pub fn next(&mut self) -> u32 {
        let oldstate = self.state.load(Ordering::Acquire);
        
        // Advance state
        let newstate = oldstate.wrapping_mul(6364136223846793005)
            .wrapping_add(self.inc.load(Ordering::Acquire));
        self.state.store(newstate, Ordering::Release);
        
        // Output function
        let xorshifted = ((oldstate >> 18) ^ oldstate) >> 27;
        let rot = (oldstate >> 59) as u32;
        
        (xorshifted as u32).rotate_right(rot)
    }
    
    /// Next random u64
    pub fn next_u64(&mut self) -> u64 {
        let high = self.next() as u64;
        let low = self.next() as u64;
        (high << 32) | low
    }
    
    /// Next random in range
    pub fn next_range(&mut self, min: u32, max: u32) -> u32 {
        if max <= min {
            return min;
        }
        min + (self.next() % (max - min))
    }
}

/// Random Manager
pub struct RandomManager {
    /// Main entropy pool
    pub entropy: RandomState,
    /// Fast PRNG
    pub prng: PrngState,
    /// Statistics
    pub stats: RandomStats,
}

/// Random Statistics
pub struct RandomStats {
    pub bytes_generated: AtomicU64,
    pub entropy_added: AtomicU64,
    pub reseed_count: AtomicU64,
}

impl RandomStats {
    pub const fn new() -> Self {
        RandomStats {
            bytes_generated: AtomicU64::new(0),
            entropy_added: AtomicU64::new(0),
            reseed_count: AtomicU64::new(0),
        }
    }
}

impl RandomManager {
    pub const fn new() -> Self {
        RandomManager {
            entropy: RandomState::new(),
            prng: PrngState::new(),
            stats: RandomStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Seed PRNG with initial entropy
        self.prng.seed(0xDEADBEEFCAFEBABE);
        
        // Add some initial entropy sources
        self.add_boot_entropy();
        
        log_info!("Random manager initialized");
    }
    
    /// Add boot entropy
    fn add_boot_entropy(&mut self) {
        // Add various boot-time entropy sources
        // In real implementation, would use:
        // - TSC/counter values
        // - Memory addresses
        // - Device timings
        // - CPU jitter
        
        let boot_data = [
            0xDE, 0xAD, 0xBE, 0xEF,
            0xCA, 0xFE, 0xBA, 0xBE,
            0x12, 0x34, 0x56, 0x78,
            0x9A, 0xBC, 0xDE, 0xF0,
        ];
        
        self.entropy.add_entropy(&boot_data, 64);
    }
    
    /// Add entropy
    pub fn add_entropy(&mut self, data: &[u8], bits: u32) {
        self.entropy.add_entropy(data, bits);
        self.stats.entropy_added.fetch_add(data.len() as u64, Ordering::AcqRel);
    }
    
    /// Get random bytes
    pub fn get_bytes(&mut self, buf: &mut [u8]) {
        self.entropy.get_bytes(buf);
        self.stats.bytes_generated.fetch_add(buf.len() as u64, Ordering::AcqRel);
    }
    
    /// Get random u32
    pub fn get_u32(&mut self) -> u32 {
        self.entropy.get_u32()
    }
    
    /// Get random u64
    pub fn get_u64(&mut self) -> u64 {
        self.entropy.get_u64()
    }
    
    /// Get random in range
    pub fn get_range(&mut self, min: u32, max: u32) -> u32 {
        self.entropy.get_range(min, max)
    }
    
    /// Fast random u32 (for non-crypto use)
    pub fn fast_u32(&mut self) -> u32 {
        self.prng.next()
    }
    
    /// Fast random u64 (for non-crypto use)
    pub fn fast_u64(&mut self) -> u64 {
        self.prng.next_u64()
    }
    
    /// Fast random in range
    pub fn fast_range(&mut self, min: u32, max: u32) -> u32 {
        self.prng.next_range(min, max)
    }
    
    /// Reseed PRNG from entropy pool
    pub fn reseed(&mut self) {
        let seed = self.entropy.get_u64();
        self.prng.seed(seed);
        self.stats.reseed_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.entropy.initialized.load(Ordering::Acquire)
    }
    
    /// Get entropy count
    pub fn entropy_count(&self) -> u32 {
        self.entropy.entropy_count.load(Ordering::Acquire)
    }
}

/// Global random manager
static RANDOM_MANAGER: crate::sync_oncelock::OnceLock<RandomManager> = crate::sync_oncelock::OnceLock::new();

/// Get random manager
pub fn random_manager() -> &'static RandomManager {
    RANDOM_MANAGER.get_or_init(RandomManager::new)
}

pub fn init_random_manager() -> &'static RandomManager {
    RANDOM_MANAGER.get_or_init(RandomManager::new)
}

/// Initialize random
pub fn init_random() {
    let mgr = random_manager();
    mgr.init();
}

// Convenience functions

/// Get random bytes
pub fn get_random_bytes(buf: &mut [u8]) {
    random_manager().get_bytes(buf);
}

/// Get random u32
pub fn get_random_u32() -> u32 {
    random_manager().get_u32()
}

/// Get random u64
pub fn get_random_u64() -> u64 {
    random_manager().get_u64()
}

/// Get random in range
pub fn get_random_range(min: u32, max: u32) -> u32 {
    random_manager().get_range(min, max)
}

/// Fast random u32 (non-crypto)
pub fn prandom_u32() -> u32 {
    random_manager().fast_u32()
}

/// Fast random u64 (non-crypto)
pub fn prandom_u64() -> u64 {
    random_manager().fast_u64()
}

/// Fast random in range
pub fn prandom_range(min: u32, max: u32) -> u32 {
    random_manager().fast_range(min, max)
}

/// Add entropy
pub fn add_device_randomness(data: &[u8]) {
    random_manager().add_entropy(data, (data.len() * 8 / 2) as u32);
}

/// Add interrupt randomness
pub fn add_interrupt_randomness(irq: u32, val: u32) {
    let data = [
        (irq & 0xFF) as u8,
        ((irq >> 8) & 0xFF) as u8,
        (val & 0xFF) as u8,
        ((val >> 8) & 0xFF) as u8,
    ];
    random_manager().add_entropy(&data, 4);
}

/// UUID Generation
#[repr(C)]
pub struct Uuid {
    pub data: [u8; 16],
}

impl Uuid {
    /// Generate random UUID (v4)
    pub fn generate() -> Self {
        let mut data = [0u8; 16];
        get_random_bytes(&mut data);
        
        // Set version (4) and variant (RFC 4122)
        data[6] = (data[6] & 0x0F) | 0x40;
        data[8] = (data[8] & 0x3F) | 0x80;
        
        Uuid { data }
    }
    
    /// Format to string
    pub fn format(&self) -> [u8; 37] {
        let mut buf = [0u8; 37];
        let hex = b"0123456789abcdef";
        
        let mut idx = 0;
        for (i, &byte) in self.data.iter().enumerate() {
            if i == 4 || i == 6 || i == 8 || i == 10 {
                buf[idx] = b'-';
                idx += 1;
            }
            buf[idx] = hex[(byte >> 4) as usize];
            buf[idx + 1] = hex[(byte & 0x0F) as usize];
            idx += 2;
        }
        buf[36] = 0;
        
        buf
    }
}

/// Generate UUID
pub fn generate_uuid() -> Uuid {
    Uuid::generate()
}

/// Simple hash for random mixing
pub fn simple_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    
    for (i, &byte) in data.iter().enumerate() {
        hash ^= (byte as u64).wrapping_mul(0x517CC1B727220A95);
        hash = hash.rotate_left(5);
        hash = hash.wrapping_add(i as u64);
    }
    
    hash
}
