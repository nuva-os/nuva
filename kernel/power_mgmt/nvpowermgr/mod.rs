/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - Mod
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
 * Nuva OS - Kernel - NvPowerMgr Power Optimization & Green Computing
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * AI-driven power management with DVFS, thermal-aware
 * throttling, power budgets, and green computing metrics.
 */

pub mod budget;
pub mod dvfs_controller;
pub mod device_controller;
pub mod thermal;
pub mod green_metrics;
pub mod ai_optimizer;
pub mod optimization;
pub mod fallback;
pub mod stats;
pub mod api;
pub mod sched_coop;
pub mod balancer_coop;

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

use crate::kernel::error::KernelResult;
use crate::kernel::error::KernelError;

/// Default sampling period in milliseconds
pub const DEFAULT_SAMPLING_PERIOD_MS: u32 = 10;

/// Default performance degradation limit (10%)
pub const DEFAULT_PERF_DEGRADATION_LIMIT_PCT: u32 = 10;

/// Default energy reduction target (15%)
pub const DEFAULT_ENERGY_REDUCTION_TARGET_PCT: u32 = 15;

/// Maximum number of managed devices
pub const MAX_POWER_DEVICES: usize = 16;

/// NvPowerMgr: AI-driven power optimization manager
///
/// Integrates power budget management, DVFS control,
/// device power control, thermal monitoring, green
/// metrics collection, and AI power optimization.
pub struct NvPowerMgr {
    /// Sampling period in milliseconds
    sampling_period_ms: AtomicU32,
    /// Performance degradation limit (percentage)
    perf_degradation_limit_pct: AtomicU32,
    /// Energy reduction target (percentage)
    energy_reduction_target_pct: AtomicU32,
    /// Whether power management is enabled
    enabled: AtomicBool,
    /// Whether NPU is available for AI optimization
    npu_available: AtomicBool,
    /// Whether initialized
    initialized: AtomicBool,
}

impl NvPowerMgr {
    /// Create a new NvPowerMgr with default configuration
    pub const fn new() -> Self {
        NvPowerMgr {
            sampling_period_ms: AtomicU32::new(DEFAULT_SAMPLING_PERIOD_MS),
            perf_degradation_limit_pct: AtomicU32::new(DEFAULT_PERF_DEGRADATION_LIMIT_PCT),
            energy_reduction_target_pct: AtomicU32::new(DEFAULT_ENERGY_REDUCTION_TARGET_PCT),
            enabled: AtomicBool::new(false),
            npu_available: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize NvPowerMgr
    pub fn init(&self, npu_available: bool) {
        self.npu_available.store(npu_available, Ordering::Release);
        self.enabled.store(true, Ordering::Release);
        self.initialized.store(true, Ordering::Release);
    }

    /// Check if power management is enabled
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Check if initialized
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }

    /// Get sampling period
    #[inline(always)]
    pub fn sampling_period_ms(&self) -> u32 {
        self.sampling_period_ms.load(Ordering::Acquire)
    }

    /// Set sampling period
    pub fn set_sampling_period_ms(&self, ms: u32) -> KernelResult<()> {
        if ms == 0 {
            return Err(crate::kernel::error::KernelError::InvalidArgument);
        }
        self.sampling_period_ms.store(ms, Ordering::Release);
        Ok(())
    }

    /// Get performance degradation limit
    #[inline(always)]
    pub fn perf_degradation_limit_pct(&self) -> u32 {
        self.perf_degradation_limit_pct.load(Ordering::Acquire)
    }

    /// Get energy reduction target
    #[inline(always)]
    pub fn energy_reduction_target_pct(&self) -> u32 {
        self.energy_reduction_target_pct.load(Ordering::Acquire)
    }

    /// Check if NPU is available for AI optimization
    #[inline(always)]
    pub fn npu_available(&self) -> bool {
        self.npu_available.load(Ordering::Acquire)
    }

    /// Set NPU availability
    pub fn set_npu_available(&self, available: bool) {
        self.npu_available.store(available, Ordering::Release);
    }
}

/// Global NvPowerMgr instance
static NV_POWERMGR: crate::sync_oncelock::OnceLock<NvPowerMgr> = crate::sync_oncelock::OnceLock::new();

/// Get global NvPowerMgr instance
pub fn get_nv_powermgr() -> &'static NvPowerMgr {
    NV_POWERMGR.get_or_init(NvPowerMgr::new)
}

/// Initialize global NvPowerMgr
pub fn init_nv_powermgr(npu_available: bool) {
    get_nv_powermgr().init(npu_available);
}