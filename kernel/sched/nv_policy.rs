/*
 * Nuva OS - Kernel - Sched - NvPolicy
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
 * Nuva OS - Kernel - NvSchedPolicy (Nuva Native Scheduling Policy)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native scheduling policies replacing POSIX SCHED_*.
 * Migrated from: POSIX SCHED_OTHER/SCHED_FIFO/SCHED_RR → NvSchedPolicy.
 *
 * INVARIANT: scheduler internal code does not use posix::errno::Errno.
 */

use core::fmt;
use crate::kernel::types::NvDuration;

/// Nuva native scheduling policy (replaces POSIX SCHED_*)
///
/// Migrated from: POSIX SCHED_OTHER/SCHED_FIFO/SCHED_RR → NvSchedPolicy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvSchedPolicy {
    /// Fair share scheduling (replaces SCHED_OTHER)
    NvFair = 0,
    /// Real-time FIFO (replaces SCHED_FIFO)
    NvRealtimeFifo = 1,
    /// Real-time round-robin (replaces SCHED_RR)
    NvRealtimeRr = 2,
    /// Hard realtime deadline scheduling (new, superior to POSIX)
    NvDeadline = 3,
    /// Energy-aware scheduling (new, for mobile/edge)
    NvEnergyAware = 4,
    /// AI-optimized scheduling (new, for AI workloads)
    NvAiOptimized = 5,
    /// Background best-effort scheduling (new)
    NvBackground = 6,
}

impl fmt::Display for NvSchedPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NvSchedPolicy::NvFair => write!(f, "NvFair"),
            NvSchedPolicy::NvRealtimeFifo => write!(f, "NvRealtimeFifo"),
            NvSchedPolicy::NvRealtimeRr => write!(f, "NvRealtimeRr"),
            NvSchedPolicy::NvDeadline => write!(f, "NvDeadline"),
            NvSchedPolicy::NvEnergyAware => write!(f, "NvEnergyAware"),
            NvSchedPolicy::NvAiOptimized => write!(f, "NvAiOptimized"),
            NvSchedPolicy::NvBackground => write!(f, "NvBackground"),
        }
    }
}

/// Deadline scheduling parameters for NvDeadline policy
///
/// INVARIANT: NvDeadline tasks deadline miss rate is 0 in steady state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvDeadlineParams {
    /// Worst-case execution time per period
    pub runtime: NvDuration,
    /// Absolute deadline relative to period start
    pub deadline: NvDuration,
    /// Scheduling period
    pub period: NvDuration,
}

impl NvDeadlineParams {
    /// Create new deadline parameters
    pub fn new(runtime: u64, deadline: u64, period: u64) -> Self {
        NvDeadlineParams {
            runtime: NvDuration::new(runtime),
            deadline: NvDuration::new(deadline),
            period: NvDuration::new(period),
        }
    }

    /// Check if deadline parameters are valid for admission control.
    /// runtime <= deadline <= period must hold.
    pub fn is_valid(&self) -> bool {
        self.runtime.as_u64() <= self.deadline.as_u64()
            && self.deadline.as_u64() <= self.period.as_u64()
    }

    /// Get utilization (runtime / period) as a fixed-point u32 (0..=0x10000)
    pub fn utilization_fp(&self) -> u32 {
        if self.period.as_u64() == 0 {
            return 0x10000;
        }
        ((self.runtime.as_u64() as u128 * 0x10000) / self.period.as_u64() as u128) as u32
    }
}

/// Energy-aware scheduling parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvEnergyAwareParams {
    /// Prefer idle CPUs for wake-up
    pub prefer_idle: bool,
    /// Maximum power budget in milliwatts (0 = unlimited)
    pub max_power_mw: u32,
}

/// AI-optimized scheduling parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvAiOptimizedParams {
    /// Batch window in nanoseconds for grouping AI inferences
    pub batch_window: NvDuration,
}

/// Nuva scheduling policy configuration (unified)
#[derive(Debug, Clone, Copy)]
pub enum NvSchedConfig {
    Fair,
    RealtimeFifo { priority: i32 },
    RealtimeRr { priority: i32, timeslice_ns: u64 },
    Deadline(NvDeadlineParams),
    EnergyAware(NvEnergyAwareParams),
    AiOptimized(NvAiOptimizedParams),
    Background,
}

impl NvSchedConfig {
    /// Get the policy type from this config
    pub fn policy(&self) -> NvSchedPolicy {
        match self {
            NvSchedConfig::Fair => NvSchedPolicy::NvFair,
            NvSchedConfig::RealtimeFifo { .. } => NvSchedPolicy::NvRealtimeFifo,
            NvSchedConfig::RealtimeRr { .. } => NvSchedPolicy::NvRealtimeRr,
            NvSchedConfig::Deadline(_) => NvSchedPolicy::NvDeadline,
            NvSchedConfig::EnergyAware(_) => NvSchedPolicy::NvEnergyAware,
            NvSchedConfig::AiOptimized(_) => NvSchedPolicy::NvAiOptimized,
            NvSchedConfig::Background => NvSchedPolicy::NvBackground,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sched_policy_no_posix_names() {
        let p = NvSchedPolicy::NvFair;
        assert_eq!(format!("{}", p), "NvFair");
        let p = NvSchedPolicy::NvDeadline;
        assert_eq!(format!("{}", p), "NvDeadline");
    }

    #[test]
    fn test_deadline_params_valid() {
        let params = NvDeadlineParams::new(5_000_000, 10_000_000, 20_000_000);
        assert!(params.is_valid());
    }

    #[test]
    fn test_deadline_params_invalid() {
        let params = NvDeadlineParams::new(15_000_000, 10_000_000, 20_000_000);
        assert!(!params.is_valid());
    }

    #[test]
    fn test_deadline_utilization() {
        let params = NvDeadlineParams::new(5_000_000, 10_000_000, 20_000_000);
        let util = params.utilization_fp();
        assert!(util > 0 && util < 0x10000);
    }

    #[test]
    fn test_sched_config_policy() {
        assert_eq!(NvSchedConfig::Fair.policy(), NvSchedPolicy::NvFair);
        assert_eq!(
            NvSchedConfig::Deadline(NvDeadlineParams::new(1, 2, 3)).policy(),
            NvSchedPolicy::NvDeadline
        );
    }
}
