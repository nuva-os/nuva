/*
 * Nuva OS - Kernel - Sched - Nvbalancer - Mod
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
/*
 * Nuva OS - Kernel - NvBalancer Heterogeneous Hardware Balancer
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * AI-driven heterogeneous hardware load balancer that
 * distributes workloads across GPU/NPU/CPU/Quantum devices
 * for optimal performance and power efficiency.
 */

pub mod device_types;
pub mod topology;
pub mod load_metrics;
pub mod load_collector;
pub mod optimizer;
pub mod migration_entry;
pub mod migrator;
pub mod oscillation;
pub mod hotplug;
pub mod stats;
pub mod api;
pub mod power_aware;

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

use crate::kernel::error::KernelResult;

/// Default imbalance trigger threshold (30%)
pub const DEFAULT_IMBALANCE_TRIGGER_PCT: u32 = 30;

/// Default balance target threshold (10%)
pub const DEFAULT_BALANCE_THRESHOLD_PCT: u32 = 10;

/// Default max convergence steps
pub const DEFAULT_MAX_CONVERGENCE_STEPS: u32 = 10;

/// Maximum number of heterogeneous devices
pub const MAX_HETERO_DEVICES: usize = 16;

/// NvBalancer: heterogeneous hardware load balancer
///
/// Integrates device topology management, load collection,
/// balance optimization, migration execution, and oscillation
/// detection. Triggered by NvScheduler when load imbalance
/// exceeds threshold (default 30%).
pub struct NvBalancer {
    /// Imbalance trigger threshold (percentage)
    imbalance_trigger_pct: AtomicU32,
    /// Balance target threshold (percentage)
    balance_threshold_pct: AtomicU32,
    /// Maximum convergence steps
    max_convergence_steps: AtomicU32,
    /// Whether balancer is enabled
    enabled: AtomicBool,
    /// Whether balancer is initialized
    initialized: AtomicBool,
}

impl NvBalancer {
    /// Create a new NvBalancer with default configuration
    pub const fn new() -> Self {
        NvBalancer {
            imbalance_trigger_pct: AtomicU32::new(DEFAULT_IMBALANCE_TRIGGER_PCT),
            balance_threshold_pct: AtomicU32::new(DEFAULT_BALANCE_THRESHOLD_PCT),
            max_convergence_steps: AtomicU32::new(DEFAULT_MAX_CONVERGENCE_STEPS),
            enabled: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize NvBalancer
    pub fn init(&self) {
        self.enabled.store(true, Ordering::Release);
        self.initialized.store(true, Ordering::Release);
    }

    /// Check if balancer is enabled
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Check if balancer is initialized
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Get imbalance trigger threshold
    #[inline(always)]
    pub fn imbalance_trigger_pct(&self) -> u32 {
        self.imbalance_trigger_pct.load(Ordering::Acquire)
    }

    /// Set imbalance trigger threshold
    pub fn set_imbalance_trigger_pct(&self, pct: u32) -> KernelResult<()> {
        if pct == 0 || pct > 100 {
            return Err(crate::kernel::error::KernelError::InvalidArgument);
        }
        self.imbalance_trigger_pct.store(pct, Ordering::Release);
        Ok(())
    }

    /// Get balance target threshold
    #[inline(always)]
    pub fn balance_threshold_pct(&self) -> u32 {
        self.balance_threshold_pct.load(Ordering::Acquire)
    }

    /// Set balance target threshold
    pub fn set_balance_threshold_pct(&self, pct: u32) -> KernelResult<()> {
        if pct > 100 {
            return Err(crate::kernel::error::KernelError::InvalidArgument);
        }
        self.balance_threshold_pct.store(pct, Ordering::Release);
        Ok(())
    }

    /// Get max convergence steps
    #[inline(always)]
    pub fn max_convergence_steps(&self) -> u32 {
        self.max_convergence_steps.load(Ordering::Acquire)
    }

    /// Set max convergence steps
    pub fn set_max_convergence_steps(&self, steps: u32) -> KernelResult<()> {
        if steps == 0 {
            return Err(crate::kernel::error::KernelError::InvalidArgument);
        }
        self.max_convergence_steps.store(steps, Ordering::Release);
        Ok(())
    }

    /// Check if load imbalance exceeds trigger threshold
    ///
    /// @param max_load: Maximum device utilization (0-100)
    /// @param min_load: Minimum device utilization (0-100)
    /// @return: true if imbalance exceeds threshold
    pub fn is_imbalanced(&self, max_load: u32, min_load: u32) -> bool {
        if max_load == 0 {
            return false;
        }
        let deviation = ((max_load - min_load) * 100) / max_load;
        deviation >= self.imbalance_trigger_pct()
    }
}

/// Global NvBalancer instance
static NV_BALANCER: core::sync::OnceLock<NvBalancer> = core::sync::OnceLock::new();

/// Get global NvBalancer instance
pub fn get_nv_balancer() -> &'static NvBalancer {
    NV_BALANCER.get_or_init(NvBalancer::new)
}

/// Initialize global NvBalancer
pub fn init_nv_balancer() {
    get_nv_balancer().init();
}