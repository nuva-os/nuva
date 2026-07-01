/*
 * Nuva OS - Kernel - Sched - QuantSched
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
 * Quantization-Aware Scheduling
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Dynamically selects quantization mode (INT8, FP16, BF16, FP32)
 * based on NPU load, model accuracy requirements, and
 * performance-accuracy trade-off policies.
 *
 * Implements precision degradation strategy:
 * high load -> FP16 -> INT8 to maximize throughput
 * while maintaining accuracy bounds.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

use crate::{pr_info, pr_debug};

/// Quantization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuantMode {
    /// 8-bit integer (highest throughput, lowest precision)
    INT8 = 0,
    /// 16-bit floating point (good balance)
    FP16 = 1,
    /// Brain floating point 16 (wider range, lower precision than FP16)
    BF16 = 2,
    /// 32-bit floating point (highest precision, lowest throughput)
    FP32 = 3,
    /// Mixed precision (INT8 for weights, FP16 for accumulators)
    Mixed = 4,
}

impl QuantMode {
    /// Relative throughput (higher = faster)
    pub fn throughput_relative(&self) -> u32 {
        match self {
            QuantMode::INT8 => 4,
            QuantMode::FP16 => 2,
            QuantMode::BF16 => 2,
            QuantMode::FP32 => 1,
            QuantMode::Mixed => 3,
        }
    }

    /// Relative precision (higher = more accurate)
    pub fn precision_relative(&self) -> u32 {
        match self {
            QuantMode::INT8 => 1,
            QuantMode::FP16 => 3,
            QuantMode::BF16 => 2,
            QuantMode::FP32 => 4,
            QuantMode::Mixed => 2,
        }
    }

    /// Memory usage relative to FP32 (percentage)
    pub fn memory_ratio_pct(&self) -> u32 {
        match self {
            QuantMode::INT8 => 25,
            QuantMode::FP16 => 50,
            QuantMode::BF16 => 50,
            QuantMode::FP32 => 100,
            QuantMode::Mixed => 30,
        }
    }

    /// Try to degrade precision for higher throughput
    pub fn degrade(&self) -> Option<QuantMode> {
        match self {
            QuantMode::FP32 => Some(QuantMode::FP16),
            QuantMode::BF16 => Some(QuantMode::FP16),
            QuantMode::FP16 => Some(QuantMode::Mixed),
            QuantMode::Mixed => Some(QuantMode::INT8),
            QuantMode::INT8 => None,
        }
    }

    /// Try to upgrade precision for higher accuracy
    pub fn upgrade(&self) -> Option<QuantMode> {
        match self {
            QuantMode::INT8 => Some(QuantMode::Mixed),
            QuantMode::Mixed => Some(QuantMode::FP16),
            QuantMode::FP16 => Some(QuantMode::FP32),
            QuantMode::BF16 => Some(QuantMode::FP32),
            QuantMode::FP32 => None,
        }
    }
}

/// NPU utilization thresholds for quantization adjustment
pub mod thresholds {
    /// Low utilization: can upgrade precision
    pub const LOW: u32 = 30;
    /// Medium utilization: maintain current precision
    pub const MEDIUM: u32 = 60;
    /// High utilization: consider degrading precision
    pub const HIGH: u32 = 80;
    /// Critical utilization: force degradation
    pub const CRITICAL: u32 = 95;
}

/// Performance-accuracy trade-off policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantTradeoffPolicy {
    /// Maximize accuracy (prefer FP32/FP16)
    AccuracyFirst = 0,
    /// Maximize throughput (prefer INT8/Mixed)
    ThroughputFirst = 1,
    /// Balance accuracy and throughput
    Balanced = 2,
    /// Auto-adjust based on SLA targets
    AutoSla = 3,
}

/// Quantization scheduling policy
pub struct QuantSchedPolicy {
    /// Preferred quantization mode
    pub preferred_mode: AtomicU32,
    /// Current active quantization mode
    pub current_mode: AtomicU32,
    /// Trade-off policy
    pub tradeoff_policy: AtomicU32,
    /// NPU utilization at last adjustment
    pub last_npu_util: AtomicU32,
    /// Minimum acceptable precision (won't degrade below)
    pub min_precision: AtomicU32,
    /// Target inference latency (ms, 0 = no target)
    pub target_latency_ms: AtomicU32,
    /// Mode change count
    pub mode_changes: AtomicU64,
    /// Enabled
    pub enabled: AtomicBool,
}

impl QuantSchedPolicy {
    /// Create new policy with defaults
    pub const fn new() -> Self {
        QuantSchedPolicy {
            preferred_mode: AtomicU32::new(QuantMode::FP16 as u32),
            current_mode: AtomicU32::new(QuantMode::FP16 as u32),
            tradeoff_policy: AtomicU32::new(QuantTradeoffPolicy::Balanced as u32),
            last_npu_util: AtomicU32::new(0),
            min_precision: AtomicU32::new(QuantMode::INT8 as u32),
            target_latency_ms: AtomicU32::new(0),
            mode_changes: AtomicU64::new(0),
            enabled: AtomicBool::new(false),
        }
    }

    /// Initialize
    pub fn init(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    /// Set preferred mode
    pub fn set_preferred_mode(&self, mode: QuantMode) {
        self.preferred_mode.store(mode as u32, Ordering::Release);
    }

    /// Set trade-off policy
    pub fn set_tradeoff_policy(&self, policy: QuantTradeoffPolicy) {
        self.tradeoff_policy.store(policy as u32, Ordering::Release);
    }

    /// Set minimum precision floor
    pub fn set_min_precision(&self, mode: QuantMode) {
        self.min_precision.store(mode as u32, Ordering::Release);
    }

    /// Set target latency
    pub fn set_target_latency(&self, latency_ms: u32) {
        self.target_latency_ms.store(latency_ms, Ordering::Release);
    }

    /// Get current quantization mode
    pub fn current_mode(&self) -> QuantMode {
        match self.current_mode.load(Ordering::Acquire) {
            0 => QuantMode::INT8,
            1 => QuantMode::FP16,
            2 => QuantMode::BF16,
            3 => QuantMode::FP32,
            4 => QuantMode::Mixed,
            _ => QuantMode::FP16,
        }
    }

    /// Select quantization mode based on NPU load and requirements
    /// Decision matrix:
    /// - Low load + accuracy-first -> upgrade precision
    /// - High load + throughput-first -> degrade precision
    /// - Critical load -> force degradation regardless of policy
    /// @param npu_utilization: Current NPU utilization (0-100)
    /// @param current_latency_ms: Current inference latency
    /// @param accuracy_tolerance_pct: Acceptable accuracy loss (0-100)
    /// @return: Selected quantization mode
    pub fn quant_select_mode(
        &self,
        npu_utilization: u32,
        current_latency_ms: u32,
        accuracy_tolerance_pct: u32,
    ) -> QuantMode {
        if !self.enabled.load(Ordering::Acquire) {
            return self.current_mode();
        }

        let current = self.current_mode();
        let policy = match self.tradeoff_policy.load(Ordering::Acquire) {
            0 => QuantTradeoffPolicy::AccuracyFirst,
            1 => QuantTradeoffPolicy::ThroughputFirst,
            2 => QuantTradeoffPolicy::Balanced,
            3 => QuantTradeoffPolicy::AutoSla,
            _ => QuantTradeoffPolicy::Balanced,
        };

        let min_mode = match self.min_precision.load(Ordering::Acquire) {
            0 => QuantMode::INT8,
            1 => QuantMode::FP16,
            2 => QuantMode::BF16,
            3 => QuantMode::FP32,
            4 => QuantMode::Mixed,
            _ => QuantMode::INT8,
        };

        let target_lat = self.target_latency_ms.load(Ordering::Acquire);

        let new_mode = if npu_utilization >= thresholds::CRITICAL {
            self.degrade_to_min(current, min_mode)
        } else if npu_utilization >= thresholds::HIGH {
            match policy {
                QuantTradeoffPolicy::ThroughputFirst => {
                    self.degrade_to_min(current, min_mode)
                }
                QuantTradeoffPolicy::Balanced => {
                    if accuracy_tolerance_pct >= 5 {
                        current.degrade().unwrap_or(current)
                    } else {
                        current
                    }
                }
                _ => current,
            }
        } else if npu_utilization <= thresholds::LOW {
            match policy {
                QuantTradeoffPolicy::AccuracyFirst => {
                    current.upgrade().unwrap_or(current)
                }
                QuantTradeoffPolicy::Balanced => {
                    current.upgrade().unwrap_or(current)
                }
                _ => current,
            }
        } else if target_lat > 0 && current_latency_ms > target_lat {
            if accuracy_tolerance_pct >= 3 {
                self.degrade_to_min(current, min_mode)
            } else {
                current
            }
        } else {
            current
        };

        if new_mode != current {
            self.current_mode.store(new_mode as u32, Ordering::Release);
            self.mode_changes.fetch_add(1, Ordering::Relaxed);
            log_debug!("Quant mode changed: {:?} -> {:?} (util={}%, lat={}ms)",
                      current, new_mode, npu_utilization, current_latency_ms);
        }

        self.last_npu_util.store(npu_utilization, Ordering::Release);
        new_mode
    }

    /// Adjust scheduling parameters based on runtime feedback
    /// Monitors actual vs target performance and adjusts
    /// quantization mode dynamically.
    /// @param actual_latency_ms: Measured inference latency
    /// @param actual_accuracy_pct: Measured accuracy (0-100)
    /// @param npu_util: NPU utilization
    pub fn quant_adjust_schedule(
        &self,
        actual_latency_ms: u32,
        actual_accuracy_pct: u32,
        npu_util: u32,
    ) -> QuantMode {
        let target_lat = self.target_latency_ms.load(Ordering::Acquire);
        let current = self.current_mode();

        if target_lat > 0 && actual_latency_ms > target_lat * 2 {
            if let Some(lower) = current.degrade() {
                let min_mode = match self.min_precision.load(Ordering::Acquire) {
                    0 => QuantMode::INT8,
                    1 => QuantMode::FP16,
                    2 => QuantMode::BF16,
                    3 => QuantMode::FP32,
                    _ => QuantMode::INT8,
                };
                if lower as u32 >= min_mode as u32 {
                    self.current_mode.store(lower as u32, Ordering::Release);
                    self.mode_changes.fetch_add(1, Ordering::Relaxed);
                    return lower;
                }
            }
        }

        if actual_accuracy_pct < 90 && npu_util < thresholds::MEDIUM {
            if let Some(higher) = current.upgrade() {
                self.current_mode.store(higher as u32, Ordering::Release);
                self.mode_changes.fetch_add(1, Ordering::Relaxed);
                return higher;
            }
        }

        current
    }

    /// Degrade to minimum allowed mode
    fn degrade_to_min(&self, current: QuantMode, min_mode: QuantMode) -> QuantMode {
        if current as u32 > min_mode as u32 {
            current.degrade().unwrap_or(min_mode)
        } else {
            current
        }
    }

    /// Get mode change count
    pub fn mode_change_count(&self) -> u64 {
        self.mode_changes.load(Ordering::Acquire)
    }
}

/// Global quantization scheduling policy
static QUANT_SCHED_POLICY: crate::sync_oncelock::OnceLock<QuantSchedPolicy> = crate::sync_oncelock::OnceLock::new();

/// Get global quantization scheduling policy
pub fn get_quant_sched_policy() -> &'static QuantSchedPolicy {
    // SAFETY: singleton access
    unsafe { &QUANT_SCHED_POLICY }
}

/// Initialize quantization scheduling
pub fn init_quant_sched() {
    get_quant_sched_policy().init();
    log_info!("Quantization-aware scheduler initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quant_mode_throughput() {
        assert_eq!(QuantMode::INT8.throughput_relative(), 4);
        assert_eq!(QuantMode::FP16.throughput_relative(), 2);
        assert_eq!(QuantMode::FP32.throughput_relative(), 1);
    }

    #[test]
    fn test_quant_mode_precision() {
        assert_eq!(QuantMode::FP32.precision_relative(), 4);
        assert_eq!(QuantMode::INT8.precision_relative(), 1);
    }

    #[test]
    fn test_quant_mode_memory() {
        assert_eq!(QuantMode::FP32.memory_ratio_pct(), 100);
        assert_eq!(QuantMode::FP16.memory_ratio_pct(), 50);
        assert_eq!(QuantMode::INT8.memory_ratio_pct(), 25);
    }

    #[test]
    fn test_quant_mode_degrade() {
        assert_eq!(QuantMode::FP32.degrade(), Some(QuantMode::FP16));
        assert_eq!(QuantMode::FP16.degrade(), Some(QuantMode::Mixed));
        assert_eq!(QuantMode::Mixed.degrade(), Some(QuantMode::INT8));
        assert_eq!(QuantMode::INT8.degrade(), None);
    }

    #[test]
    fn test_quant_mode_upgrade() {
        assert_eq!(QuantMode::INT8.upgrade(), Some(QuantMode::Mixed));
        assert_eq!(QuantMode::Mixed.upgrade(), Some(QuantMode::FP16));
        assert_eq!(QuantMode::FP16.upgrade(), Some(QuantMode::FP32));
        assert_eq!(QuantMode::FP32.upgrade(), None);
    }

    #[test]
    fn test_quant_sched_policy_new() {
        let policy = QuantSchedPolicy::new();
        assert_eq!(policy.current_mode(), QuantMode::FP16);
    }

    #[test]
    fn test_quant_select_mode_low_util() {
        let policy = QuantSchedPolicy::new();
        policy.init();

        let mode = policy.quant_select_mode(20, 5, 5);
        assert!(mode as u32 >= QuantMode::FP16 as u32);
    }

    #[test]
    fn test_quant_select_mode_high_util() {
        let policy = QuantSchedPolicy::new();
        policy.init();
        policy.set_preferred_mode(QuantMode::FP16);
        policy.set_tradeoff_policy(QuantTradeoffPolicy::ThroughputFirst);

        let mode = policy.quant_select_mode(85, 5, 10);
        let _ = mode;
    }

    #[test]
    fn test_quant_select_mode_critical() {
        let policy = QuantSchedPolicy::new();
        policy.init();
        policy.set_preferred_mode(QuantMode::FP32);

        let mode = policy.quant_select_mode(96, 5, 10);
        assert!(mode as u32 <= QuantMode::FP16 as u32);
    }

    #[test]
    fn test_quant_adjust_schedule() {
        let policy = QuantSchedPolicy::new();
        policy.init();
        policy.set_target_latency(10);

        let mode = policy.quant_adjust_schedule(30, 95, 50);
        let _ = mode;
    }

    #[test]
    fn test_min_precision_floor() {
        let policy = QuantSchedPolicy::new();
        policy.init();
        policy.set_min_precision(QuantMode::FP16);
        policy.set_tradeoff_policy(QuantTradeoffPolicy::ThroughputFirst);

        let mode = policy.quant_select_mode(96, 5, 10);
        assert!(mode as u32 >= QuantMode::FP16 as u32);
    }
}
