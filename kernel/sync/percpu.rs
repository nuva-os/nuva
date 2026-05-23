/*
 * Nuva OS - Kernel - Per-CPU Data Structures
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

//! Per-CPU data structures for lock-free access on the local CPU.
//! Each CPU gets its own copy of data, eliminating lock contention.

/// Maximum number of CPUs
pub const MAX_CPUS: usize = 256;

/// Cache line size for alignment
pub const CACHE_LINE_SIZE: usize = 64;

/// Per-CPU data container.
/// Each CPU gets its own copy of data, eliminating lock contention.
#[repr(C, align(64))]
pub struct PerCpu<T> {
    data: [T; MAX_CPUS],
}

impl<T: Default + Copy> PerCpu<T> {
    /// Create a new PerCpu with default values
    pub const fn new(default: T) -> Self {
        Self {
            data: [default; MAX_CPUS],
        }
    }

    /// Get reference to current CPU's data (O(1) via TLS)
    #[inline(always)]
    pub fn current(&self) -> &T {
        let cpu_id = Self::current_cpu_id();
        &self.data[cpu_id]
    }

    /// Get mutable reference to current CPU's data
    #[inline(always)]
    pub fn current_mut(&mut self) -> &mut T {
        let cpu_id = Self::current_cpu_id();
        &mut self.data[cpu_id]
    }

    /// Get reference to specific CPU's data
    #[inline]
    pub fn for_cpu(&self, cpu_id: usize) -> Option<&T> {
        if cpu_id < MAX_CPUS {
            Some(&self.data[cpu_id])
        } else {
            None
        }
    }

    /// Get mutable reference to specific CPU's data
    #[inline]
    pub fn for_cpu_mut(&mut self, cpu_id: usize) -> Option<&mut T> {
        if cpu_id < MAX_CPUS {
            Some(&mut self.data[cpu_id])
        } else {
            None
        }
    }

    /// Get reference to specific CPU's data without bounds checking
    /// # Safety
    /// Caller must ensure cpu_id < MAX_CPUS
    #[inline(always)]
    pub unsafe fn for_cpu_unchecked(&self, cpu_id: usize) -> &T {
        self.data.get_unchecked(cpu_id)
    }

    /// Get mutable reference to specific CPU's data without bounds checking
    /// # Safety
    /// Caller must ensure cpu_id < MAX_CPUS
    #[inline(always)]
    pub unsafe fn for_cpu_mut_unchecked(&mut self, cpu_id: usize) -> &mut T {
        self.data.get_unchecked_mut(cpu_id)
    }

    /// Get current CPU ID via TLS register
    #[inline(always)]
    pub fn current_cpu_id() -> usize {
        #[cfg(target_arch = "aarch64")]
        {
            let cpu_id: u64;
            // SAFETY: Reading TPIDR_EL1 is safe, it contains the current CPU ID
            unsafe { core::arch::asm!("mrs {}, tpidr_el1", out(reg) cpu_id); }
            cpu_id as usize
        }
        #[cfg(target_arch = "x86_64")]
        {
            let cpu_id: u64;
            // SAFETY: Reading GS base offset is safe, it contains the current CPU ID
            unsafe { core::arch::asm!("movq %gs:0, {}", out(reg) cpu_id); }
            cpu_id as usize
        }
        #[cfg(target_arch = "loongarch64")]
        {
            let cpu_id: u64;
            // SAFETY: Reading $tp register is safe, it contains the current CPU ID
            unsafe { core::arch::asm!("move {}, $tp", out(reg) cpu_id); }
            cpu_id as usize
        }
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "loongarch64")))]
        {
            0
        }
    }
}

/// Macro to define a static Per-CPU variable
#[macro_export]
macro_rules! define_percpu {
    ($name:ident, $type:ty, $default:expr) => {
        static mut $name: $crate::kernel::sync::percpu::PerCpu<$type> =
            $crate::kernel::sync::percpu::PerCpu::new($default);
    };
}
