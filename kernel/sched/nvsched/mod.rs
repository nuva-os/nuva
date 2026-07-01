/*
 * Nuva OS - Kernel - NvScheduler AI Intelligent Scheduler
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
 *
 * NvScheduler: AI-driven intelligent scheduling with multi-level
 * scheduling classes, NPU inference integration, three-tier fallback,
 * and power-aware/balancer-driven decision making.
 */

pub mod feature_vector;
pub mod inference_result;
pub mod npu_inference_engine;
pub mod sched_class;
pub mod task_classifier;
pub mod decision_maker;
pub mod decl_policy_engine;
pub mod fallback;
pub mod stats;
pub mod api;
pub mod power_aware;
pub mod balancer_coop;
pub mod coop_invariant;

use core::sync::atomic::{AtomicU8, AtomicBool, AtomicU32, Ordering};

use crate::kernel::error::KernelResult;

use crate::kernel::error::KernelError;

/// NvScheduler operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NvSchedMode {
    /// AI inference-driven scheduling (primary)
    AiInference = 0,
    /// Declarative policy-driven scheduling (fallback level 1)
    DeclPolicy = 1,
    /// Traditional CFS+RT scheduling (fallback level 2)
    Traditional = 2,
}

impl NvSchedMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => NvSchedMode::AiInference,
            1 => NvSchedMode::DeclPolicy,
            _ => NvSchedMode::Traditional,
        }
    }
}

/// NvScheduler: AI-driven intelligent scheduler
///
/// Integrates NPU inference engine, AI task classifier,
/// declarative policy engine, and fallback scheduler.
/// Supports three-tier fallback: AI inference -> declarative
/// policy -> CFS+RT traditional scheduling.
pub struct NvScheduler {
    /// Current scheduling mode
    mode: AtomicU8,
    /// Whether NPU is available for inference
    npu_available: AtomicBool,
    /// AI confidence threshold (0-100, scaled by 100)
    confidence_threshold_pct: AtomicU32,
    /// Inference budget in microseconds
    inference_budget_us: AtomicU32,
    /// Whether power-aware scheduling is enabled
    power_aware_enabled: AtomicBool,
    /// Whether balancer-driven scheduling is enabled
    balancer_driven: AtomicBool,
    /// Whether the scheduler is initialized
    initialized: AtomicBool,
}

impl NvScheduler {
    /// Create a new NvScheduler with default configuration
    pub const fn new() -> Self {
        NvScheduler {
            mode: AtomicU8::new(NvSchedMode::AiInference as u8),
            npu_available: AtomicBool::new(false),
            confidence_threshold_pct: AtomicU32::new(50),
            inference_budget_us: AtomicU32::new(100),
            power_aware_enabled: AtomicBool::new(true),
            balancer_driven: AtomicBool::new(true),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize NvScheduler
    pub fn init(&self, npu_available: bool) {
        self.npu_available.store(npu_available, Ordering::Release);
        if !npu_available {
            self.mode.store(NvSchedMode::DeclPolicy as u8, Ordering::Release);
        }
        self.initialized.store(true, Ordering::Release);
    }

    /// Get current scheduling mode
    #[inline(always)]
    pub fn mode(&self) -> NvSchedMode {
        NvSchedMode::from_u8(self.mode.load(Ordering::Acquire))
    }

    /// Set scheduling mode
    pub fn set_mode(&self, mode: NvSchedMode) -> KernelResult<()> {
        self.mode.store(mode as u8, Ordering::Release);
        Ok(())
    }

    /// Check if NPU is available
    #[inline(always)]
    pub fn npu_available(&self) -> bool {
        self.npu_available.load(Ordering::Acquire)
    }

    /// Set NPU availability (triggers mode change if needed)
    pub fn set_npu_available(&self, available: bool) {
        self.npu_available.store(available, Ordering::Release);
        if !available && self.mode() == NvSchedMode::AiInference {
            self.mode.store(NvSchedMode::DeclPolicy as u8, Ordering::Release);
        }
    }

    /// Get AI confidence threshold percentage (0-100)
    #[inline(always)]
    pub fn confidence_threshold_pct(&self) -> u32 {
        self.confidence_threshold_pct.load(Ordering::Acquire)
    }

    /// Set AI confidence threshold percentage
    pub fn set_confidence_threshold_pct(&self, pct: u32) -> KernelResult<()> {
        if pct > 100 {
            return Err(crate::kernel::error::KernelError::InvalidArgument);
        }
        self.confidence_threshold_pct.store(pct, Ordering::Release);
        Ok(())
    }

    /// Get inference budget in microseconds
    #[inline(always)]
    pub fn inference_budget_us(&self) -> u32 {
        self.inference_budget_us.load(Ordering::Acquire)
    }

    /// Set inference budget in microseconds
    pub fn set_inference_budget_us(&self, budget_us: u32) -> KernelResult<()> {
        if budget_us == 0 {
            return Err(crate::kernel::error::KernelError::InvalidArgument);
        }
        self.inference_budget_us.store(budget_us, Ordering::Release);
        Ok(())
    }

    /// Check if power-aware scheduling is enabled
    #[inline(always)]
    pub fn power_aware_enabled(&self) -> bool {
        self.power_aware_enabled.load(Ordering::Acquire)
    }

    /// Set power-aware scheduling enabled
    pub fn set_power_aware_enabled(&self, enabled: bool) {
        self.power_aware_enabled.store(enabled, Ordering::Release);
    }

    /// Check if balancer-driven scheduling is enabled
    #[inline(always)]
    pub fn balancer_driven(&self) -> bool {
        self.balancer_driven.load(Ordering::Acquire)
    }

    /// Set balancer-driven scheduling enabled
    pub fn set_balancer_driven(&self, enabled: bool) {
        self.balancer_driven.store(enabled, Ordering::Release);
    }

    /// Check if scheduler is initialized
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }
}

/// Global NvScheduler instance
static NV_SCHEDULER: crate::sync_oncelock::OnceLock<NvScheduler> = crate::sync_oncelock::OnceLock::new();

/// Get global NvScheduler instance
pub fn get_nv_scheduler() -> &'static NvScheduler {
    NV_SCHEDULER.get_or_init(NvScheduler::new)
}

/// Initialize global NvScheduler
pub fn init_nv_scheduler(npu_available: bool) {
    get_nv_scheduler().init(npu_available);
}