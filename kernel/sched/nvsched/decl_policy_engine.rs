/*
 * Nuva OS - Kernel - Sched - Nvsched - DeclPolicyEngine
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
 * Nuva OS - Kernel - NvScheduler Declarative Policy Engine
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Enhanced declarative policy engine with AI-aware
 * configuration fields and runtime hot-update support.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

use crate::kernel::error::{KernelError, KernelResult};

/// DeclPolicyEngine: Enhanced declarative scheduling policy
///
/// Extends the base SchedPolicyConfig with AI-specific
/// fields: confidence threshold, inference budget,
/// power-aware flag, and balancer-driven flag.
/// Supports runtime hot-update via generation counter.
#[repr(C, align(64))]
pub struct DeclPolicyEngine {
    /// Policy name
    pub name: &'static str,
    /// Minimum scheduling granularity (nanoseconds)
    pub min_granularity_ns: AtomicU32,
    /// Scheduler latency target (nanoseconds)
    pub latency_ns: AtomicU32,
    /// Wakeup preemption granularity (nanoseconds)
    pub wakeup_granularity_ns: AtomicU32,
    /// Load average period (milliseconds)
    pub load_avg_period_ms: AtomicU32,
    /// RT time slice (milliseconds)
    pub rt_time_slice_ms: AtomicU32,
    /// Load balance threshold (percentage)
    pub lb_threshold_pct: AtomicU32,
    /// AI confidence threshold (0-100 percentage)
    pub ai_confidence_threshold: AtomicU32,
    /// Inference budget (microseconds)
    pub inference_budget_us: AtomicU32,
    /// Power-aware scheduling enabled
    pub power_aware_enabled: AtomicBool,
    /// Balancer-driven scheduling enabled
    pub balancer_driven: AtomicBool,
    /// Configuration generation counter
    pub generation: AtomicU64,
}

impl DeclPolicyEngine {
    /// Create a new declarative policy engine
    pub const fn new(
        name: &'static str,
        min_granularity_ns: u32,
        latency_ns: u32,
        wakeup_granularity_ns: u32,
        load_avg_period_ms: u32,
        rt_time_slice_ms: u32,
        lb_threshold_pct: u32,
        ai_confidence_threshold: u32,
        inference_budget_us: u32,
        power_aware_enabled: bool,
        balancer_driven: bool,
    ) -> Self {
        DeclPolicyEngine {
            name,
            min_granularity_ns: AtomicU32::new(min_granularity_ns),
            latency_ns: AtomicU32::new(latency_ns),
            wakeup_granularity_ns: AtomicU32::new(wakeup_granularity_ns),
            load_avg_period_ms: AtomicU32::new(load_avg_period_ms),
            rt_time_slice_ms: AtomicU32::new(rt_time_slice_ms),
            lb_threshold_pct: AtomicU32::new(lb_threshold_pct),
            ai_confidence_threshold: AtomicU32::new(ai_confidence_threshold),
            inference_budget_us: AtomicU32::new(inference_budget_us),
            power_aware_enabled: AtomicBool::new(power_aware_enabled),
            balancer_driven: AtomicBool::new(balancer_driven),
            generation: AtomicU64::new(0),
        }
    }

    /// Hot-update all policy parameters atomically
    pub fn update(
        &self,
        min_granularity_ns: u32,
        latency_ns: u32,
        wakeup_granularity_ns: u32,
        load_avg_period_ms: u32,
        rt_time_slice_ms: u32,
        lb_threshold_pct: u32,
        ai_confidence_threshold: u32,
        inference_budget_us: u32,
        power_aware_enabled: bool,
        balancer_driven: bool,
    ) -> KernelResult<()> {
        if min_granularity_ns == 0 || latency_ns == 0 {
            return Err(KernelError::InvalidArgument);
        }
        if lb_threshold_pct > 100 || ai_confidence_threshold > 100 {
            return Err(KernelError::InvalidArgument);
        }
        if inference_budget_us == 0 {
            return Err(KernelError::InvalidArgument);
        }

        self.min_granularity_ns.store(min_granularity_ns, Ordering::Release);
        self.latency_ns.store(latency_ns, Ordering::Release);
        self.wakeup_granularity_ns.store(wakeup_granularity_ns, Ordering::Release);
        self.load_avg_period_ms.store(load_avg_period_ms, Ordering::Release);
        self.rt_time_slice_ms.store(rt_time_slice_ms, Ordering::Release);
        self.lb_threshold_pct.store(lb_threshold_pct, Ordering::Release);
        self.ai_confidence_threshold.store(ai_confidence_threshold, Ordering::Release);
        self.inference_budget_us.store(inference_budget_us, Ordering::Release);
        self.power_aware_enabled.store(power_aware_enabled, Ordering::Release);
        self.balancer_driven.store(balancer_driven, Ordering::Release);
        self.generation.fetch_add(1, Ordering::AcqRel);

        Ok(())
    }

    /// Get current generation
    #[inline(always)]
    pub fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /// Get AI confidence threshold
    #[inline(always)]
    pub fn confidence_threshold(&self) -> u32 {
        self.ai_confidence_threshold.load(Ordering::Acquire)
    }

    /// Get inference budget in microseconds
    #[inline(always)]
    pub fn inference_budget(&self) -> u32 {
        self.inference_budget_us.load(Ordering::Acquire)
    }

    /// Check if power-aware is enabled
    #[inline(always)]
    pub fn power_aware(&self) -> bool {
        self.power_aware_enabled.load(Ordering::Acquire)
    }

    /// Check if balancer-driven is enabled
    #[inline(always)]
    pub fn balancer_driven(&self) -> bool {
        self.balancer_driven.load(Ordering::Acquire)
    }
}

/// Default AI-optimized policy
pub static DEFAULT_AI_POLICY: DeclPolicyEngine = DeclPolicyEngine::new(
    "ai_default",
    1_000_000,   // min_granularity_ns: 1ms
    5_000_000,   // latency_ns: 5ms
    1_000_000,   // wakeup_granularity_ns: 1ms
    1024,        // load_avg_period_ms
    100,         // rt_time_slice_ms
    25,          // lb_threshold_pct: 25%
    50,          // ai_confidence_threshold: 50%
    100,         // inference_budget_us: 100us
    true,        // power_aware_enabled
    true,        // balancer_driven
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        assert_eq!(DEFAULT_AI_POLICY.confidence_threshold(), 50);
        assert_eq!(DEFAULT_AI_POLICY.inference_budget(), 100);
        assert!(DEFAULT_AI_POLICY.power_aware());
        assert!(DEFAULT_AI_POLICY.balancer_driven());
    }

    #[test]
    fn test_hot_update() {
        let policy = DeclPolicyEngine::new(
            "test",
            1_000_000, 5_000_000, 1_000_000, 1024, 100, 25,
            50, 100, true, true,
        );

        let result = policy.update(
            2_000_000, 10_000_000, 2_000_000, 2048, 200, 30,
            70, 200, false, false,
        );
        assert!(result.is_ok());
        assert_eq!(policy.confidence_threshold(), 70);
        assert_eq!(policy.inference_budget(), 200);
        assert!(!policy.power_aware());
        assert_eq!(policy.current_generation(), 1);
    }

    #[test]
    fn test_hot_update_invalid() {
        let policy = DeclPolicyEngine::new(
            "test",
            1_000_000, 5_000_000, 1_000_000, 1024, 100, 25,
            50, 100, true, true,
        );

        let result = policy.update(0, 5_000_000, 1_000_000, 1024, 100, 25, 50, 100, true, true);
        assert!(result.is_err());
    }
}