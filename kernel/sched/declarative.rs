/*
 * Nuva OS - Kernel - Declarative Scheduler Policy Configuration
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

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::kernel::error::{KernelError, KernelResult};

/**
 * Declarative scheduler policy configuration.
 *
 * Allows scheduler parameters to be defined declaratively
 * and applied at runtime without system restart. Supports
 * hot-update notification for dynamic tuning.
 *
 * # Example
 * ```rust
 * declare_sched_policy! {
 *     CFS_POLICY {
 *         min_granularity_ns: 1000000,
 *         latency_ns: 5000000,
 *         wakeup_granularity_ns: 1000000,
 *         load_avg_period_ms: 1024,
 *     }
 * }
 * ```
 */
#[repr(C, align(64))]
pub struct SchedPolicyConfig {
    /** Policy name */
    pub name: &'static str,

    /** Minimum scheduling granularity (nanoseconds) */
    pub min_granularity_ns: AtomicU32,

    /** Scheduler latency target (nanoseconds) */
    pub latency_ns: AtomicU32,

    /** Wakeup preemption granularity (nanoseconds) */
    pub wakeup_granularity_ns: AtomicU32,

    /** Load average period (milliseconds) */
    pub load_avg_period_ms: AtomicU32,

    /** Time slice for RT tasks (milliseconds) */
    pub rt_time_slice_ms: AtomicU32,

    /** Load balance threshold (percentage) */
    pub lb_threshold_pct: AtomicU32,

    /** Configuration generation number for hot-update tracking */
    pub generation: AtomicU64,
}

impl SchedPolicyConfig {
    /** Create a new scheduler policy configuration */
    pub const fn new(
        name: &'static str,
        min_granularity_ns: u32,
        latency_ns: u32,
        wakeup_granularity_ns: u32,
        load_avg_period_ms: u32,
        rt_time_slice_ms: u32,
        lb_threshold_pct: u32,
    ) -> Self {
        SchedPolicyConfig {
            name,
            min_granularity_ns: AtomicU32::new(min_granularity_ns),
            latency_ns: AtomicU32::new(latency_ns),
            wakeup_granularity_ns: AtomicU32::new(wakeup_granularity_ns),
            load_avg_period_ms: AtomicU32::new(load_avg_period_ms),
            rt_time_slice_ms: AtomicU32::new(rt_time_slice_ms),
            lb_threshold_pct: AtomicU32::new(lb_threshold_pct),
            generation: AtomicU64::new(0),
        }
    }

    /**
     * Hot-update policy parameters.
     *
     * Atomically applies new parameters and increments the
     * generation counter. Returns Err if any parameter is
     * invalid (e.g., zero granularity).
     */
    pub fn update(
        &self,
        min_granularity_ns: u32,
        latency_ns: u32,
        wakeup_granularity_ns: u32,
        load_avg_period_ms: u32,
        rt_time_slice_ms: u32,
        lb_threshold_pct: u32,
    ) -> KernelResult<()> {
        if min_granularity_ns == 0 || latency_ns == 0 {
            return Err(KernelError::InvalidArgument);
        }
        if lb_threshold_pct > 100 {
            return Err(KernelError::InvalidArgument);
        }

        self.min_granularity_ns.store(min_granularity_ns, Ordering::Release);
        self.latency_ns.store(latency_ns, Ordering::Release);
        self.wakeup_granularity_ns.store(wakeup_granularity_ns, Ordering::Release);
        self.load_avg_period_ms.store(load_avg_period_ms, Ordering::Release);
        self.rt_time_slice_ms.store(rt_time_slice_ms, Ordering::Release);
        self.lb_threshold_pct.store(lb_threshold_pct, Ordering::Release);
        self.generation.fetch_add(1, Ordering::AcqRel);

        Ok(())
    }

    /** Get the current configuration generation */
    #[inline(always)]
    pub fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /** Get minimum granularity in nanoseconds */
    #[inline(always)]
    pub fn min_granularity(&self) -> u32 {
        self.min_granularity_ns.load(Ordering::Acquire)
    }

    /** Get scheduler latency target in nanoseconds */
    #[inline(always)]
    pub fn latency(&self) -> u32 {
        self.latency_ns.load(Ordering::Acquire)
    }
}

/**
 * Default CFS policy configuration.
 *
 * Calibrated for typical mobile workloads:
 * - 1ms minimum granularity for smooth interactive response
 * - 5ms latency target for fair scheduling
 * - 1ms wakeup granularity to prevent excessive preemption
 */
pub static CFS_POLICY_DEFAULT: SchedPolicyConfig = SchedPolicyConfig::new(
    "cfs_default",
    1_000_000,   // min_granularity_ns: 1ms
    5_000_000,   // latency_ns: 5ms
    1_000_000,   // wakeup_granularity_ns: 1ms
    1024,        // load_avg_period_ms
    100,         // rt_time_slice_ms
    25,          // lb_threshold_pct: 25%
);

/**
 * Macro to define a declarative scheduler policy configuration.
 */
#[macro_export]
macro_rules! declare_sched_policy {
    (
        $name:ident {
            min_granularity_ns: $min_g:expr,
            latency_ns: $lat:expr,
            wakeup_granularity_ns: $wkup:expr,
            load_avg_period_ms: $load:expr,
        }
    ) => {
        static $name: $crate::kernel::sched::declarative::SchedPolicyConfig =
            $crate::kernel::sched::declarative::SchedPolicyConfig::new(
                stringify!($name),
                $min_g,
                $lat,
                $wkup,
                $load,
                100,  // rt_time_slice_ms default
                25,   // lb_threshold_pct default
            );
    };
}
