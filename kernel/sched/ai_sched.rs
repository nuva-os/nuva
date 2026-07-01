/*
 * Nuva OS - Kernel - Sched - AiSched
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
 * AI Priority Scheduling Extension
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Extends Energy Aware Scheduling (EAS) with AI-task-aware
 * scheduling decisions:
 * - Priority boost for latency-sensitive AI inference
 * - Latency-aware CPU selection
 * - AI task classification (inference, training, preprocessing)
 * - Prefer performance cores for AI, efficiency cores for background
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

use crate::kernel::sched::eas::EasData;

/// Maximum AI priority boost (nice value reduction)
pub const AI_MAX_PRIORITY_BOOST: i32 = 5;

/// Default inference latency threshold (ms)
pub const DEFAULT_INFERENCE_LATENCY_MS: u32 = 10;

/// AI task classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTaskClass {
    /// Model inference (latency-sensitive)
    Inference = 0,
    /// Model training (throughput-oriented)
    Training = 1,
    /// Data preprocessing (moderate priority)
    Preprocessing = 2,
    /// Post-processing (e.g., NMS, decode)
    Postprocessing = 3,
    /// Not an AI task
    NonAi = 4,
}

/// AI scheduling extension
/// Augments EAS with AI-specific scheduling policies.
/// AI inference tasks receive priority boosts and are
/// preferentially placed on performance (big) cores.
pub struct AiSchedExt {
    /// Inference latency threshold (ms) - tasks below this get boosted
    pub inference_latency_threshold_ms: AtomicU32,
    /// AI task priority boost (nice value reduction, 0-5)
    pub ai_priority_boost: AtomicU32,
    /// Whether AI scheduling is enabled
    pub enabled: AtomicBool,
    /// Number of AI tasks boosted
    pub boost_count: AtomicU64,
    /// Number of latency-aware picks
    pub latency_pick_count: AtomicU64,
    /// Number of tasks classified as AI
    pub ai_task_count: AtomicU64,
    /// Minimum big core CPU index
    pub big_core_start: u32,
    /// Number of big cores
    pub num_big_cores: u32,
    /// Minimum LITTLE core CPU index
    pub little_core_start: u32,
    /// Number of LITTLE cores
    pub num_little_cores: u32,
}

impl AiSchedExt {
    /// Create new AI scheduling extension
    pub const fn new() -> Self {
        AiSchedExt {
            inference_latency_threshold_ms: AtomicU32::new(DEFAULT_INFERENCE_LATENCY_MS),
            ai_priority_boost: AtomicU32::new(3),
            enabled: AtomicBool::new(false),
            boost_count: AtomicU64::new(0),
            latency_pick_count: AtomicU64::new(0),
            ai_task_count: AtomicU64::new(0),
            big_core_start: 4,
            num_big_cores: 4,
            little_core_start: 0,
            num_little_cores: 4,
        }
    }

    /// Initialize AI scheduling extension
    pub fn init(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    /// AI task wakeup priority boost
    /// When an AI inference task wakes up, boost its priority
    /// to reduce scheduling latency. The boost amount depends
    /// on the task class and expected inference latency.
    /// @param task_class: Classification of the waking task
    /// @param expected_latency_ms: Expected inference latency
    /// @return: Priority boost amount (nice value reduction)
    pub fn ai_wakeup_boost(
        &self,
        task_class: AiTaskClass,
        expected_latency_ms: u32,
    ) -> i32 {
        if !self.enabled.load(Ordering::Acquire) {
            return 0;
        }

        let threshold = self.inference_latency_threshold_ms.load(Ordering::Acquire);
        let max_boost = self.ai_priority_boost.load(Ordering::Acquire);

        match task_class {
            AiTaskClass::Inference => {
                self.boost_count.fetch_add(1, Ordering::Relaxed);
                self.ai_task_count.fetch_add(1, Ordering::Relaxed);

                if expected_latency_ms <= threshold {
                    let boost = max_boost as i32;
                    boost.min(AI_MAX_PRIORITY_BOOST)
                } else if expected_latency_ms <= threshold * 2 {
                    (max_boost as i32) / 2
                } else {
                    (max_boost as i32) / 4
                }
            }
            AiTaskClass::Training => {
                self.ai_task_count.fetch_add(1, Ordering::Relaxed);
                0
            }
            AiTaskClass::Preprocessing => {
                self.ai_task_count.fetch_add(1, Ordering::Relaxed);
                1
            }
            AiTaskClass::Postprocessing => {
                self.ai_task_count.fetch_add(1, Ordering::Relaxed);
                (max_boost as i32) / 3
            }
            AiTaskClass::NonAi => 0,
        }
    }

    /// Latency-aware CPU selection
    /// For AI inference tasks, select the CPU that minimizes
    /// overall inference latency. Prefer big (performance) cores
    /// for inference, LITTLE (efficiency) cores for background.
    /// @param task_class: Classification of the task
    /// @param prev_cpu: Previous CPU the task ran on
    /// @param eas: EAS data for energy model access
    /// @return: Selected CPU
    pub fn ai_latency_aware_pick(
        &self,
        task_class: AiTaskClass,
        prev_cpu: usize,
        _eas: &EasData,
    ) -> usize {
        if !self.enabled.load(Ordering::Acquire) {
            return prev_cpu;
        }

        self.latency_pick_count.fetch_add(1, Ordering::Relaxed);

        match task_class {
            AiTaskClass::Inference | AiTaskClass::Training => {
                let big_start = self.big_core_start as usize;
                let num_big = self.num_big_cores as usize;

                if prev_cpu >= big_start && prev_cpu < big_start + num_big {
                    prev_cpu
                } else {
                    big_start
                }
            }
            AiTaskClass::Preprocessing | AiTaskClass::Postprocessing => {
                let little_start = self.little_core_start as usize;
                let num_little = self.num_little_cores as usize;

                if prev_cpu >= little_start && prev_cpu < little_start + num_little {
                    prev_cpu
                } else {
                    little_start
                }
            }
            AiTaskClass::NonAi => prev_cpu,
        }
    }

    /// Classify AI task based on heuristic signals
    /// Uses task name pattern, scheduling hints, and
    /// resource usage to classify the task type.
    /// @param task_name: Null-terminated task name
    /// @param compute_ratio: Compute/IO ratio (0.0-1.0, higher = more compute)
    /// @param memory_mb: Memory usage in MB
    /// @param has_npu_access: Whether task uses NPU
    /// @return: Task classification
    pub fn ai_task_classification(
        &self,
        task_name: &[u8],
        compute_ratio: u32,
        memory_mb: u32,
        has_npu_access: bool,
    ) -> AiTaskClass {
        if !has_npu_access && compute_ratio < 70 {
            return AiTaskClass::NonAi;
        }

        let name_str = core::str::from_utf8(task_name).unwrap_or("");

        if name_str.contains("infer") || name_str.contains("predict") || name_str.contains("detect") {
            return AiTaskClass::Inference;
        }

        if name_str.contains("train") || name_str.contains("learn") || name_str.contains("optim") {
            return AiTaskClass::Training;
        }

        if name_str.contains("preprocess") || name_str.contains("encode") || name_str.contains("tokeniz") {
            return AiTaskClass::Preprocessing;
        }

        if name_str.contains("postprocess") || name_str.contains("decode") || name_str.contains("nms") {
            return AiTaskClass::Postprocessing;
        }

        if has_npu_access && compute_ratio > 80 {
            return AiTaskClass::Inference;
        }

        if has_npu_access && memory_mb > 100 {
            return AiTaskClass::Training;
        }

        if has_npu_access {
            return AiTaskClass::Preprocessing;
        }

        AiTaskClass::NonAi
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.boost_count.load(Ordering::Acquire),
            self.latency_pick_count.load(Ordering::Acquire),
            self.ai_task_count.load(Ordering::Acquire),
        )
    }
}

/// Global AI scheduling extension
static AI_SCHED_EXT: crate::sync_oncelock::OnceLock<AiSchedExt> = crate::sync_oncelock::OnceLock::new();

/// Get global AI scheduling extension
pub fn get_ai_sched_ext() -> &'static AiSchedExt {
    AI_SCHED_EXT.get_or_init(AiSchedExt::new)
}

/// Initialize AI scheduling extension
pub fn init_ai_sched_ext() {
    get_ai_sched_ext().init();
}

/// External interface for HAL-layer AI scheduler integration
/// Maps u8 task class to AiTaskClass and calls ai_wakeup_boost
pub fn ai_wakeup_boost_external(task_class: u8, expected_latency_ms: u32) -> i32 {
    let ext = get_ai_sched_ext();
    let tc = match task_class {
        0 => AiTaskClass::Inference,
        1 => AiTaskClass::Training,
        2 => AiTaskClass::Preprocessing,
        3 => AiTaskClass::Postprocessing,
        _ => AiTaskClass::NonAi,
    };
    ext.ai_wakeup_boost(tc, expected_latency_ms)
}

/// External interface for HAL-layer AI scheduler integration
/// Maps u8 task class to AiTaskClass and calls ai_latency_aware_pick
pub fn ai_latency_aware_pick_external(task_class: u8, prev_cpu: usize) -> usize {
    let ext = get_ai_sched_ext();
    let tc = match task_class {
        0 => AiTaskClass::Inference,
        1 => AiTaskClass::Training,
        2 => AiTaskClass::Preprocessing,
        3 => AiTaskClass::Postprocessing,
        _ => AiTaskClass::NonAi,
    };
    let eas = crate::kernel::sched::eas::EasData::new();
    ext.ai_latency_aware_pick(tc, prev_cpu, &eas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::sched::eas::EasData;

    #[test]
    fn test_ai_sched_ext_new() {
        let ext = AiSchedExt::new();
        assert!(!ext.enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn test_ai_wakeup_boost_inference() {
        let ext = AiSchedExt::new();
        ext.init();

        let boost = ext.ai_wakeup_boost(AiTaskClass::Inference, 5);
        assert!(boost > 0);
    }

    #[test]
    fn test_ai_wakeup_boost_training() {
        let ext = AiSchedExt::new();
        ext.init();

        let boost = ext.ai_wakeup_boost(AiTaskClass::Training, 100);
        assert_eq!(boost, 0);
    }

    #[test]
    fn test_ai_wakeup_boost_non_ai() {
        let ext = AiSchedExt::new();
        ext.init();

        let boost = ext.ai_wakeup_boost(AiTaskClass::NonAi, 0);
        assert_eq!(boost, 0);
    }

    #[test]
    fn test_ai_wakeup_boost_disabled() {
        let ext = AiSchedExt::new();

        let boost = ext.ai_wakeup_boost(AiTaskClass::Inference, 1);
        assert_eq!(boost, 0);
    }

    #[test]
    fn test_ai_latency_aware_pick_inference() {
        let mut eas = EasData::new();
        eas.init();
        let ext = AiSchedExt::new();
        ext.init();

        let cpu = ext.ai_latency_aware_pick(AiTaskClass::Inference, 0, &eas);
        assert_eq!(cpu, 4);
    }

    #[test]
    fn test_ai_latency_aware_pick_preprocess() {
        let mut eas = EasData::new();
        eas.init();
        let ext = AiSchedExt::new();
        ext.init();

        let cpu = ext.ai_latency_aware_pick(AiTaskClass::Preprocessing, 4, &eas);
        assert_eq!(cpu, 0);
    }

    #[test]
    fn test_ai_task_classification_inference() {
        let ext = AiSchedExt::new();
        ext.init();

        let class = ext.ai_task_classification(b"inference_task", 90, 50, true);
        assert_eq!(class, AiTaskClass::Inference);
    }

    #[test]
    fn test_ai_task_classification_training() {
        let ext = AiSchedExt::new();
        ext.init();

        let class = ext.ai_task_classification(b"training_worker", 95, 200, true);
        assert_eq!(class, AiTaskClass::Training);
    }

    #[test]
    fn test_ai_task_classification_non_ai() {
        let ext = AiSchedExt::new();
        ext.init();

        let class = ext.ai_task_classification(b"web_server", 30, 10, false);
        assert_eq!(class, AiTaskClass::NonAi);
    }

    #[test]
    fn test_ai_task_classification_npu_heuristic() {
        let ext = AiSchedExt::new();
        ext.init();

        let class = ext.ai_task_classification(b"custom_task", 85, 50, true);
        assert_eq!(class, AiTaskClass::Inference);
    }
}
