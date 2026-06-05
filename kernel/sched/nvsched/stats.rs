/*
 * Nuva OS - Kernel - Sched - Nvsched - Stats
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
 * Nuva OS - Kernel - NvScheduler Statistics
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Comprehensive scheduling statistics for NvScheduler.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// NvSchedStats: comprehensive scheduling statistics
pub struct NvSchedStats {
    /// Total AI-driven decisions
    pub ai_decisions: AtomicU64,
    /// Total fallback decisions
    pub fallback_decisions: AtomicU64,
    /// Low-confidence events
    pub low_confidence_events: AtomicU64,
    /// Inference timeouts
    pub inference_timeouts: AtomicU64,
    /// Model version mismatches
    pub model_version_mismatches: AtomicU64,
    /// Total inference latency (microseconds)
    pub total_inference_latency_us: AtomicU64,
    /// Number of inference samples
    pub inference_samples: AtomicU64,
    /// Peak inference latency (microseconds)
    pub peak_inference_latency_us: AtomicU32,
    /// Task migrations executed
    pub migrations_executed: AtomicU64,
    /// Priority boosts applied
    pub priority_boosts: AtomicU64,
    /// Power-aware decisions
    pub power_aware_decisions: AtomicU64,
    /// Balancer-driven decisions
    pub balancer_driven_decisions: AtomicU64,
}

impl NvSchedStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        NvSchedStats {
            ai_decisions: AtomicU64::new(0),
            fallback_decisions: AtomicU64::new(0),
            low_confidence_events: AtomicU64::new(0),
            inference_timeouts: AtomicU64::new(0),
            model_version_mismatches: AtomicU64::new(0),
            total_inference_latency_us: AtomicU64::new(0),
            inference_samples: AtomicU64::new(0),
            peak_inference_latency_us: AtomicU32::new(0),
            migrations_executed: AtomicU64::new(0),
            priority_boosts: AtomicU64::new(0),
            power_aware_decisions: AtomicU64::new(0),
            balancer_driven_decisions: AtomicU64::new(0),
        }
    }

    /// Record an AI-driven decision
    pub fn record_ai_decision(&self, inference_latency_us: u32) {
        self.ai_decisions.fetch_add(1, Ordering::Relaxed);
        self.total_inference_latency_us.fetch_add(inference_latency_us as u64, Ordering::Relaxed);
        self.inference_samples.fetch_add(1, Ordering::Relaxed);

        let current_peak = self.peak_inference_latency_us.load(Ordering::Acquire);
        if inference_latency_us > current_peak {
            self.peak_inference_latency_us.store(inference_latency_us, Ordering::Release);
        }
    }

    /// Record a fallback decision
    pub fn record_fallback_decision(&self) {
        self.fallback_decisions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record low-confidence event
    pub fn record_low_confidence(&self) {
        self.low_confidence_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Record inference timeout
    pub fn record_inference_timeout(&self) {
        self.inference_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record model version mismatch
    pub fn record_model_mismatch(&self) {
        self.model_version_mismatches.fetch_add(1, Ordering::Relaxed);
    }

    /// Record task migration
    pub fn record_migration(&self) {
        self.migrations_executed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record priority boost
    pub fn record_priority_boost(&self) {
        self.priority_boosts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record power-aware decision
    pub fn record_power_aware_decision(&self) {
        self.power_aware_decisions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record balancer-driven decision
    pub fn record_balancer_driven_decision(&self) {
        self.balancer_driven_decisions.fetch_add(1, Ordering::Relaxed);
    }

    /// Get average inference latency in microseconds
    pub fn avg_inference_latency_us(&self) -> u32 {
        let samples = self.inference_samples.load(Ordering::Acquire);
        if samples == 0 {
            return 0;
        }
        let total = self.total_inference_latency_us.load(Ordering::Acquire);
        (total / samples) as u32
    }

    /// Get AI decision ratio (0-100 percentage)
    pub fn ai_decision_ratio_pct(&self) -> u32 {
        let ai = self.ai_decisions.load(Ordering::Acquire);
        let fallback = self.fallback_decisions.load(Ordering::Acquire);
        let total = ai + fallback;
        if total == 0 {
            return 100;
        }
        ((ai * 100) / total) as u32
    }
}

/// Global NvSchedStats instance
static NV_SCHED_STATS: NvSchedStats = NvSchedStats::new();

/// Get global scheduling statistics
pub fn get_nv_sched_stats() -> &'static NvSchedStats {
    &NV_SCHED_STATS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_ai_decision() {
        let stats = NvSchedStats::new();
        stats.record_ai_decision(50);
        stats.record_ai_decision(80);

        assert_eq!(stats.ai_decisions.load(Ordering::Relaxed), 2);
        assert_eq!(stats.avg_inference_latency_us(), 65);
    }

    #[test]
    fn test_stats_ai_ratio() {
        let stats = NvSchedStats::new();
        stats.record_ai_decision(50);
        stats.record_ai_decision(50);
        stats.record_fallback_decision();

        assert_eq!(stats.ai_decision_ratio_pct(), 66);
    }

    #[test]
    fn test_peak_latency() {
        let stats = NvSchedStats::new();
        stats.record_ai_decision(50);
        stats.record_ai_decision(120);
        stats.record_ai_decision(80);

        assert_eq!(stats.peak_inference_latency_us.load(Ordering::Relaxed), 120);
    }
}