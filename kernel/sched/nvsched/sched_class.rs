/*
 * Nuva OS - Kernel - Sched - Nvsched - SchedClass
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
 * Nuva OS - Kernel - NvScheduler Four-Level Scheduling Classes
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * AI-aware scheduling classes with power weight factors:
 * AI_REALTIME > AI_NORMAL > AI_BATCH > AI_IDLE
 */

use core::sync::atomic::{AtomicU64, Ordering};

/// Four-level AI scheduling class
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum NvAiSchedClass {
    /// Highest priority: latency-critical AI inference
    /// NPU -> Big core, performance-first, max boost 0-5
    AiRealtime = 0,
    /// Normal priority: standard AI workloads
    /// Big core -> NPU, balanced, boost 1-3
    AiNormal = 1,
    /// Batch priority: throughput-oriented AI training
    /// Little core, throughput-first, no boost
    AiBatch = 2,
    /// Lowest priority: idle/background AI tasks
    /// Little core, energy-first, no boost
    AiIdle = 3,
}

impl NvAiSchedClass {
    /// Convert from u8
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => NvAiSchedClass::AiRealtime,
            1 => NvAiSchedClass::AiNormal,
            2 => NvAiSchedClass::AiBatch,
            _ => NvAiSchedClass::AiIdle,
        }
    }

    /// Get priority boost range (min, max)
    pub fn boost_range(&self) -> (i8, i8) {
        match self {
            NvAiSchedClass::AiRealtime => (0, 5),
            NvAiSchedClass::AiNormal => (1, 3),
            NvAiSchedClass::AiBatch => (0, 0),
            NvAiSchedClass::AiIdle => (0, 0),
        }
    }

    /// Get power weight factor (higher = more power budget allowed)
    pub fn power_weight(&self) -> u32 {
        match self {
            NvAiSchedClass::AiRealtime => 100,
            NvAiSchedClass::AiNormal => 70,
            NvAiSchedClass::AiBatch => 40,
            NvAiSchedClass::AiIdle => 20,
        }
    }

    /// Get preferred device type as string
    pub fn preferred_device(&self) -> &'static str {
        match self {
            NvAiSchedClass::AiRealtime => "npu_or_big",
            NvAiSchedClass::AiNormal => "big_or_npu",
            NvAiSchedClass::AiBatch => "little",
            NvAiSchedClass::AiIdle => "little",
        }
    }

    /// Check if this class allows NPU access
    pub fn allows_npu(&self) -> bool {
        matches!(self, NvAiSchedClass::AiRealtime | NvAiSchedClass::AiNormal)
    }

    /// Check if this class prefers big cores
    pub fn prefers_big_core(&self) -> bool {
        matches!(self, NvAiSchedClass::AiRealtime | NvAiSchedClass::AiNormal)
    }
}

/// Per-class scheduling statistics
pub struct NvSchedClassStats {
    /// Tasks in AI_REALTIME class
    pub ai_realtime_count: AtomicU64,
    /// Tasks in AI_NORMAL class
    pub ai_normal_count: AtomicU64,
    /// Tasks in AI_BATCH class
    pub ai_batch_count: AtomicU64,
    /// Tasks in AI_IDLE class
    pub ai_idle_count: AtomicU64,
    /// Total class promotions
    pub promotions: AtomicU64,
    /// Total class demotions
    pub demotions: AtomicU64,
}

impl NvSchedClassStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        NvSchedClassStats {
            ai_realtime_count: AtomicU64::new(0),
            ai_normal_count: AtomicU64::new(0),
            ai_batch_count: AtomicU64::new(0),
            ai_idle_count: AtomicU64::new(0),
            promotions: AtomicU64::new(0),
            demotions: AtomicU64::new(0),
        }
    }

    /// Increment count for a scheduling class
    pub fn inc_class(&self, class: NvAiSchedClass) {
        match class {
            NvAiSchedClass::AiRealtime => self.ai_realtime_count.fetch_add(1, Ordering::Relaxed),
            NvAiSchedClass::AiNormal => self.ai_normal_count.fetch_add(1, Ordering::Relaxed),
            NvAiSchedClass::AiBatch => self.ai_batch_count.fetch_add(1, Ordering::Relaxed),
            NvAiSchedClass::AiIdle => self.ai_idle_count.fetch_add(1, Ordering::Relaxed),
        };
    }

    /// Decrement count for a scheduling class
    pub fn dec_class(&self, class: NvAiSchedClass) {
        match class {
            NvAiSchedClass::AiRealtime => self.ai_realtime_count.fetch_sub(1, Ordering::Relaxed),
            NvAiSchedClass::AiNormal => self.ai_normal_count.fetch_sub(1, Ordering::Relaxed),
            NvAiSchedClass::AiBatch => self.ai_batch_count.fetch_sub(1, Ordering::Relaxed),
            NvAiSchedClass::AiIdle => self.ai_idle_count.fetch_sub(1, Ordering::Relaxed),
        };
    }

    /// Record a class promotion
    pub fn record_promotion(&self) {
        self.promotions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a class demotion
    pub fn record_demotion(&self) {
        self.demotions.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sched_class_ordering() {
        assert!(NvAiSchedClass::AiRealtime < NvAiSchedClass::AiNormal);
        assert!(NvAiSchedClass::AiNormal < NvAiSchedClass::AiBatch);
        assert!(NvAiSchedClass::AiBatch < NvAiSchedClass::AiIdle);
    }

    #[test]
    fn test_boost_range() {
        assert_eq!(NvAiSchedClass::AiRealtime.boost_range(), (0, 5));
        assert_eq!(NvAiSchedClass::AiNormal.boost_range(), (1, 3));
        assert_eq!(NvAiSchedClass::AiBatch.boost_range(), (0, 0));
    }

    #[test]
    fn test_power_weight() {
        assert!(NvAiSchedClass::AiRealtime.power_weight() > NvAiSchedClass::AiNormal.power_weight());
        assert!(NvAiSchedClass::AiNormal.power_weight() > NvAiSchedClass::AiBatch.power_weight());
    }

    #[test]
    fn test_allows_npu() {
        assert!(NvAiSchedClass::AiRealtime.allows_npu());
        assert!(NvAiSchedClass::AiNormal.allows_npu());
        assert!(!NvAiSchedClass::AiBatch.allows_npu());
    }
}