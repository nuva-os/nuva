/*
 * Nuva OS - Kernel - Stack Canary Protection
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

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Stack canary configuration
pub mod canary_config {
    /// Canary size in bytes
    pub const CANARY_SIZE: usize = 8;

    /// Canary position offset from stack top
    pub const CANARY_OFFSET: usize = 0;

    /// Enable kernel stack protection
    pub const KERNEL_STACK_PROTECTION: bool = true;

    /// Enable user stack protection
    pub const USER_STACK_PROTECTION: bool = true;
}

/// Stack canary value
pub struct StackCanary {
    /// Canary value
    pub value: AtomicU64,

    /// Original value (for verification)
    pub original: u64,

    /// Position in stack
    pub position: usize,

    /// Is valid
    pub valid: AtomicBool,
}

impl StackCanary {
    pub const fn new() -> Self {
        StackCanary {
            value: AtomicU64::new(0),
            original: 0,
            position: 0,
            valid: AtomicBool::new(false),
        }
    }

    /// Initialize canary with random value
    pub fn init(&self) {
        let random = Self::generate_canary();
        self.original = random;
        self.value.store(random, Ordering::Release);
        self.valid.store(true, Ordering::Release);
    }

    /// Generate a random canary value
    fn generate_canary() -> u64 {
        // Use a combination of random values
        // In production, this should use a proper CSPRNG

        // Placeholder: use xorshift64 PRNG
        static mut STATE: u64 = 0xCAFEBABEDEADBEEF;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut x = STATE;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            STATE = x;

            // Add some entropy
            x ^= 0x5A5A5A5A5A5A5A5A;
            x
        }
    }

    /// Verify canary value
    #[inline]
    pub fn verify(&self) -> bool {
        let current = self.value.load(Ordering::Acquire);
        current == self.original
    }

    /// Check and panic if canary is corrupted
    pub fn check(&self) {
        if !self.verify() {
            // Stack canary corrupted - stack overflow detected!
            Self::stack_overflow_handler();
        }
    }

    /// Stack overflow handler
    #[cold]
    fn stack_overflow_handler() {
        // In a real kernel, this would:
        // 1. Log the error
        // 2. Kill the current process
        // 3. Possibly panic the kernel

        // For now, just loop forever
        loop {
            core::hint::spin_loop();
        }
    }
}

/// Per-task stack canary
pub struct TaskStackCanary {
    /// Canary value for this task
    pub canary: StackCanary,

    /// Task ID
    pub task_id: u64,

    /// Stack base
    pub stack_base: *mut u8,

    /// Stack size
    pub stack_size: usize,
}

impl TaskStackCanary {
    pub const fn new() -> Self {
        TaskStackCanary {
            canary: StackCanary::new(),
            task_id: 0,
            stack_base: core::ptr::null_mut(),
            stack_size: 0,
        }
    }

    /// Initialize task stack canary
    pub fn init(&mut self, task_id: u64, stack_base: *mut u8, stack_size: usize) {
        self.task_id = task_id;
        self.stack_base = stack_base;
        self.stack_size = stack_size;
        self.canary.init();

        // Write canary to stack
        self.write_canary_to_stack();
    }

    /// Write canary value to stack
    fn write_canary_to_stack(&mut self) {
        if self.stack_base.is_null() {
            return;
        }

        // Canary is placed at the top of the stack
        // (stack grows down, so canary is at highest address)
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let canary_ptr = self
                .stack_base
                .add(self.stack_size - canary_config::CANARY_SIZE);
            *(canary_ptr as *mut u64) = self.canary.original;
        }
    }

    /// Verify stack canary
    pub fn verify(&self) -> bool {
        if self.stack_base.is_null() {
            return true;
        }

        // Read canary from stack
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let canary_ptr = self
                .stack_base
                .add(self.stack_size - canary_config::CANARY_SIZE);
            let stack_canary = *(canary_ptr as *const u64);

            stack_canary == self.canary.original
        }
    }

    /// Check stack canary and handle overflow
    pub fn check(&self) {
        if !self.verify() {
            StackCanary::stack_overflow_handler();
        }
    }
}

/// Stack canary manager
pub struct StackCanaryManager {
    /// Global canary (for kernel)
    pub global_canary: StackCanary,

    /// Manager enabled
    pub enabled: AtomicBool,

    /// Statistics
    pub stats: CanaryStats,
}

/// Canary statistics
pub struct CanaryStats {
    pub canaries_initialized: AtomicU64,
    pub canaries_verified: AtomicU64,
    pub overflows_detected: AtomicU64,
}

impl StackCanaryManager {
    pub const fn new() -> Self {
        StackCanaryManager {
            global_canary: StackCanary::new(),
            enabled: AtomicBool::new(true),
            stats: CanaryStats {
                canaries_initialized: AtomicU64::new(0),
                canaries_verified: AtomicU64::new(0),
                overflows_detected: AtomicU64::new(0),
            },
        }
    }

    /// Initialize the manager
    pub fn init(&self) {
        self.global_canary.init();
        self.enabled.store(true, Ordering::Release);
    }

    /// Create a new task canary
    pub fn create_task_canary(
        &self,
        task_id: u64,
        stack_base: *mut u8,
        stack_size: usize,
    ) -> TaskStackCanary {
        let mut canary = TaskStackCanary::new();
        canary.init(task_id, stack_base, stack_size);

        self.stats
            .canaries_initialized
            .fetch_add(1, Ordering::Relaxed);

        canary
    }

    /// Verify a task's stack canary
    pub fn verify_task(&self, canary: &TaskStackCanary) -> bool {
        let result = canary.verify();

        self.stats.canaries_verified.fetch_add(1, Ordering::Relaxed);

        if !result {
            self.stats
                .overflows_detected
                .fetch_add(1, Ordering::Relaxed);
        }

        result
    }
}

/// Global stack canary manager
static CANARY_MANAGER: core::sync::OnceLock<StackCanaryManager> = core::sync::OnceLock::new();

/// Get the canary manager
pub fn canary_manager() -> &'static StackCanaryManager {
    CANARY_MANAGER.get_or_init(StackCanaryManager::new)
}

pub fn init_canary_manager() -> &'static StackCanaryManager {
    CANARY_MANAGER.get_or_init(StackCanaryManager::new)
}

/// Initialize stack canary protection
pub fn init_stack_canary() {
    canary_manager().init();
}

/// Get global canary value
pub fn get_global_canary() -> u64 {
    canary_manager().global_canary.value.load(Ordering::Acquire)
}

/// Create task stack canary
pub fn create_task_canary(task_id: u64, stack_base: *mut u8, stack_size: usize) -> TaskStackCanary {
    canary_manager().create_task_canary(task_id, stack_base, stack_size)
}

/// Verify task stack canary
pub fn verify_task_canary(canary: &TaskStackCanary) -> bool {
    canary_manager().verify_task(canary)
}

/// Stack protection functions for compiler support
/// These functions are called by compiler-generated code
/// when -fstack-protector is enabled

/// Stack protector guard (for compiler)
/// This is the global canary value used by GCC/Clang
/// when -fstack-protector-strong is enabled
#[no_mangle]
// SAFETY: Returns the global canary value. The canary is read from an
// atomic variable, so concurrent access is safe.
pub unsafe fn __stack_chk_guard() -> usize {
    get_global_canary() as usize
}

/// Stack protector fail handler (for compiler)
/// Called when stack canary check fails
#[no_mangle]
#[cold]
// SAFETY: Called by compiler-generated code when the stack canary is
// corrupted. This is a fatal condition; the handler never returns.
pub unsafe fn __stack_chk_fail() {
    canary_manager()
        .stats
        .overflows_detected
        .fetch_add(1, Ordering::Relaxed);
    StackCanary::stack_overflow_handler();
}

/// Stack protector fail handler with location info
#[no_mangle]
#[cold]
// SAFETY: Called by compiler-generated code when a local canary check
// fails. The canary parameter is the value found on the stack.
pub unsafe fn __stack_chk_fail_local(canary: usize) {
    let expected = get_global_canary() as usize;
    if canary != expected {
        canary_manager()
            .stats
            .overflows_detected
            .fetch_add(1, Ordering::Relaxed);
        StackCanary::stack_overflow_handler();
    }
}
