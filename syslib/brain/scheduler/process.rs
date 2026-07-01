/*
 * Nuva OS - SystemLibrary - Brain
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

//! Nuva Brain AI Process Scheduler
//!
//! Uses NPU inference for process scheduling decisions.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::hal::npu::{NpuHal, NpuModelId, NpuBuffer, NpuInferenceRequest, npu};
use crate::kernel::sched::task::{TaskStruct, TaskState, Prio};

/// AI Scheduler Configuration
pub struct AISchedulerConfig {
    /// Whether AI scheduling is enabled
    pub enabled: bool,

    /// Prediction model ID
    pub behavior_model_id: NpuModelId,

    /// Inference interval (ms)
    pub inference_interval_ms: u32,

    /// Confidence threshold
    pub confidence_threshold: f32,

    /// Maximum priority boost
    pub max_priority_boost: i32,

    /// Interactivity weight
    pub interactivity_weight: f32,

    /// Foreground application weight
    pub foreground_weight: f32,
}

impl Default for AISchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            behavior_model_id: 0,
            inference_interval_ms: 100,
            confidence_threshold: 0.7,
            max_priority_boost: 20,
            interactivity_weight: 0.4,
            foreground_weight: 0.3,
        }
    }
}

/// Application Behavior Prediction Result
#[derive(Debug, Clone)]
pub struct AppBehaviorPrediction {
    /// Application ID
    pub app_id: u64,

    /// CPU demand prediction
    pub cpu_demand: CpuDemandPrediction,

    /// Memory demand prediction
    pub memory_demand: MemoryDemandPrediction,

    /// I/O pattern prediction
    pub io_pattern: IoPatternPrediction,

    /// Interactivity prediction
    pub interactivity: InteractivityPrediction,

    /// Prediction confidence (0.0-1.0)
    pub confidence: f32,

    /// Prediction timestamp
    pub timestamp: u64,
}

/// CPU Demand Prediction
#[derive(Debug, Clone)]
pub struct CpuDemandPrediction {
    /// Big core demand (0.0-1.0)
    pub big_core_demand: f32,

    /// Little core demand (0.0-1.0)
    pub little_core_demand: f32,

    /// Predicted duration (ms)
    pub duration_ms: u32,

    /// Burst probability (0.0-1.0)
    pub burst_probability: f32,

    /// Average CPU usage prediction
    pub avg_usage: f32,

    /// Peak CPU usage prediction
    pub peak_usage: f32,
}

/// Memory Demand Prediction
#[derive(Debug, Clone)]
pub struct MemoryDemandPrediction {
    /// Predicted working set size (bytes)
    pub working_set_size: u64,

    /// Memory growth rate (bytes/second)
    pub growth_rate: i64,

    /// OOM risk (0.0-1.0)
    pub oom_risk: f32,

    /// Memory access pattern
    pub access_pattern: MemoryAccessPattern,
}

/// Memory Access Pattern
#[derive(Debug, Clone, Copy)]
pub enum MemoryAccessPattern {
    /// Sequential access
    Sequential,
    /// Random access
    Random,
    /// Mixed access
    Mixed,
    /// Locality-based access
    Locality,
}

/// I/O Pattern Prediction
#[derive(Debug, Clone)]
pub struct IoPatternPrediction {
    /// I/O intensity (0.0-1.0)
    pub intensity: f32,

    /// Read/write ratio (0.0-1.0, 0 = all writes, 1 = all reads)
    pub read_ratio: f32,

    /// Sequential/random ratio (0.0-1.0, 0 = all random, 1 = all sequential)
    pub sequential_ratio: f32,

    /// Average request size (bytes)
    pub avg_request_size: u32,
}

/// Interactivity Prediction
#[derive(Debug, Clone)]
pub struct InteractivityPrediction {
    /// Whether this is an interactive application
    pub is_interactive: bool,

    /// User interaction frequency (times/second)
    pub interaction_frequency: f32,

    /// Response sensitivity (0.0-1.0)
    pub response_sensitivity: f32,

    /// UI update rate (Hz)
    pub ui_update_rate: f32,
}

/// Scheduling Decision
#[derive(Debug, Clone)]
pub struct SchedulingDecision {
    /// Target priority
    pub priority: i32,

    /// CPU core affinity
    pub core_affinity: CoreAffinity,

    /// Time slice (ms)
    pub time_slice: u32,

    /// Whether to boost priority
    pub boost: bool,

    /// Boost reason
    pub boost_reason: Option<BoostReason>,

    /// Decision confidence
    pub confidence: f32,
}

/// CPU Core Affinity
#[derive(Debug, Clone, Copy)]
pub enum CoreAffinity {
    /// Any core
    Any,
    /// Big cores only
    BigCores,
    /// Little cores only
    LittleCores,
    /// Specific core
    Specific(u32),
}

/// Priority Boost Reason
#[derive(Debug, Clone, Copy)]
pub enum BoostReason {
    /// Interactive application
    Interactive,
    /// Foreground application
    Foreground,
    /// High-confidence prediction
    HighConfidence,
    /// User habit
    UserHabit,
    /// Urgent task
    Urgent,
}

/// AI Process Scheduler
pub struct AIProcessScheduler {
    /// Configuration
    config: AISchedulerConfig,

    /// Application behavior history
    app_history: [Option<AppHistory>; 64],

    /// History record count
    num_history: AtomicU32,

    /// Scheduling decision statistics
    stats: AISchedulerStats,

    /// Last inference time
    last_inference_time: AtomicU64,
}

/// Application Behavior History
#[derive(Debug, Clone)]
pub struct AppHistory {
    /// Application ID
    pub app_id: u64,

    /// CPU usage history
    pub cpu_usage_history: [f32; 16],

    /// Memory usage history
    pub memory_usage_history: [f32; 16],

    /// Runtime duration history
    pub duration_history: [u32; 16],

    /// Launch count
    pub launch_count: u32,

    /// Average interaction frequency
    pub avg_interaction_freq: f32,

    /// Time pattern (usage frequency across different hours of the day)
    pub time_pattern: [f32; 24],

    /// Last update time
    pub last_update: u64,
}

/// AI Scheduler Statistics
struct AISchedulerStats {
    /// Total decision count
    total_decisions: AtomicU64,

    /// AI decision count
    ai_decisions: AtomicU64,

    /// Boost count
    boost_count: AtomicU64,

    /// Average confidence (stored as fixed-point, 0-1000)
    avg_confidence: AtomicU32,
}

impl AIProcessScheduler {
    /// Create a new AI Scheduler
    pub const fn new() -> Self {
        Self {
            config: AISchedulerConfig::default(),
            app_history: [None; 64],
            num_history: AtomicU32::new(0),
            stats: AISchedulerStats {
                total_decisions: AtomicU64::new(0),
                ai_decisions: AtomicU64::new(0),
                boost_count: AtomicU64::new(0),
                avg_confidence: AtomicU32::new(0),
            },
            last_inference_time: AtomicU64::new(0),
        }
    }

    /// Predict application behavior
    pub async fn predict_behavior(&self, app_id: u64, context: &SchedulingContext) -> AppBehaviorPrediction {
        // 1. Extract features
        let features = self.extract_features(app_id, context);

        // 2. Run NPU inference
        let prediction = self.run_inference(&features).await;

        // 3. Post-process prediction
        self.post_process_prediction(prediction, app_id)
    }

    /// Extract features
    fn extract_features(&self, app_id: u64, context: &SchedulingContext) -> [f32; 128] {
        let mut features = [0.0f32; 128];

        // System state features (0-31)
        features[0] = context.cpu_load;
        features[1] = context.memory_pressure;
        features[2] = context.io_load;
        features[3] = context.temperature / 100.0;
        features[4] = context.battery_level as f32 / 100.0;

        // Application history features (32-63)
        if let Some(history) = self.get_app_history(app_id) {
            // Average CPU usage
            let avg_cpu: f32 = history.cpu_usage_history.iter().sum::<f32>() / 16.0;
            features[32] = avg_cpu;

            // Average memory usage
            let avg_mem: f32 = history.memory_usage_history.iter().sum::<f32>() / 16.0;
            features[33] = avg_mem;

            // Launch frequency
            features[34] = (history.launch_count as f32).ln() / 10.0;

            // Interaction frequency
            features[35] = history.avg_interaction_freq;

            // Current time slot usage probability
            let hour = (context.timestamp / 3600) % 24;
            features[36] = history.time_pattern[hour as usize];
        }

        // Time features (64-79)
        let hour = ((context.timestamp / 3600) % 24) as f32 / 24.0;
        features[64] = hour;
        features[65] = (hour * 2.0 * core::f32::consts::PI).sin();
        features[66] = (hour * 2.0 * core::f32::consts::PI).cos();

        features
    }

    /// Run NPU inference
    async fn run_inference(&self, features: &[f32; 128]) -> [f32; 32] {
        let mut output = [0.0f32; 32];

        // TODO: Use actual NPU for inference
        // Currently using simplified heuristic rules

        // CPU demand prediction
        output[0] = features[32] * 0.8 + features[0] * 0.2; // Big core demand
        output[1] = features[32] * 0.3; // Little core demand
        output[2] = 100.0; // Duration
        output[3] = if features[32] > 0.7 { 0.8 } else { 0.2 }; // Burst probability

        // Memory demand prediction
        output[4] = features[33] * 1024.0 * 1024.0 * 100.0; // Working set size
        output[5] = 0.0; // Growth rate
        output[6] = if features[33] > 0.8 { 0.7 } else { 0.1 }; // OOM risk

        // Interactivity prediction
        output[7] = features[35]; // Interaction frequency
        output[8] = if features[35] > 0.5 { 1.0 } else { 0.0 }; // Whether interactive

        // Confidence
        output[9] = 0.8; // Base confidence

        output
    }

    /// Post-process prediction result
    fn post_process_prediction(&self, output: [f32; 32], app_id: u64) -> AppBehaviorPrediction {
        AppBehaviorPrediction {
            app_id,
            cpu_demand: CpuDemandPrediction {
                big_core_demand: output[0].clamp(0.0, 1.0),
                little_core_demand: output[1].clamp(0.0, 1.0),
                duration_ms: output[2] as u32,
                burst_probability: output[3].clamp(0.0, 1.0),
                avg_usage: output[0],
                peak_usage: output[0] * 1.5,
            },
            memory_demand: MemoryDemandPrediction {
                working_set_size: output[4] as u64,
                growth_rate: output[5] as i64,
                oom_risk: output[6].clamp(0.0, 1.0),
                access_pattern: MemoryAccessPattern::Mixed,
            },
            io_pattern: IoPatternPrediction {
                intensity: 0.3,
                read_ratio: 0.7,
                sequential_ratio: 0.5,
                avg_request_size: 4096,
            },
            interactivity: InteractivityPrediction {
                is_interactive: output[8] > 0.5,
                interaction_frequency: output[7],
                response_sensitivity: if output[8] > 0.5 { 0.9 } else { 0.3 },
                ui_update_rate: 60.0,
            },
            confidence: output[9].clamp(0.0, 1.0),
            timestamp: 0, // TODO: Get current time
        }
    }

    /// Make a scheduling decision
    pub async fn make_scheduling_decision(
        &self,
        task: &TaskStruct,
        context: &SchedulingContext,
    ) -> SchedulingDecision {
        self.stats.total_decisions.fetch_add(1, Ordering::Relaxed);

        if !self.config.enabled {
            return self.default_decision(task);
        }

        // Get prediction
        let prediction = self.predict_behavior(task.pid as u64, context).await;

        // Calculate priority
        let priority = self.calculate_priority(task, &prediction);

        // Calculate core affinity
        let core_affinity = self.calculate_core_affinity(&prediction);

        // Calculate time slice
        let time_slice = self.calculate_time_slice(&prediction);

        // Check if boost is needed
        let (boost, boost_reason) = self.check_boost(task, &prediction);

        if boost {
            self.stats.boost_count.fetch_add(1, Ordering::Relaxed);
        }

        self.stats.ai_decisions.fetch_add(1, Ordering::Relaxed);

        SchedulingDecision {
            priority,
            core_affinity,
            time_slice,
            boost,
            boost_reason,
            confidence: prediction.confidence,
        }
    }

    /// Calculate priority
    fn calculate_priority(&self, task: &TaskStruct, prediction: &AppBehaviorPrediction) -> i32 {
        let base_priority = task.static_prio;

        let mut adjustment = 0;

        // Boost priority for interactive applications
        if prediction.interactivity.is_interactive {
            adjustment += (self.config.max_priority_boost as f32 * self.config.interactivity_weight) as i32;
        }

        // Boost priority for foreground applications
        if task.is_foreground() {
            adjustment += (self.config.max_priority_boost as f32 * self.config.foreground_weight) as i32;
        }

        // Give higher weight to high-confidence predictions
        if prediction.confidence > self.config.confidence_threshold {
            adjustment += 5;
        }

        // Clamp priority to valid range
        (base_priority - adjustment).clamp(100, 139)
    }

    /// Calculate core affinity
    fn calculate_core_affinity(&self, prediction: &AppBehaviorPrediction) -> CoreAffinity {
        // Assign high CPU demand applications to big cores
        if prediction.cpu_demand.big_core_demand > 0.7 {
            CoreAffinity::BigCores
        } else if prediction.cpu_demand.little_core_demand > 0.7 {
            CoreAffinity::LittleCores
        } else if prediction.interactivity.is_interactive {
            // Interactive applications prefer big cores
            CoreAffinity::BigCores
        } else {
            CoreAffinity::Any
        }
    }

    /// Calculate time slice
    fn calculate_time_slice(&self, prediction: &AppBehaviorPrediction) -> u32 {
        let base_slice = 100; // Base time slice 100ms

        // Interactive applications use shorter time slices for better responsiveness
        if prediction.interactivity.is_interactive {
            50
        } else if prediction.cpu_demand.burst_probability > 0.5 {
            // Bursty tasks use shorter time slices
            30
        } else {
            base_slice
        }
    }

    /// Check if boost is needed
    fn check_boost(&self, task: &TaskStruct, prediction: &AppBehaviorPrediction) -> (bool, Option<BoostReason>) {
        if prediction.interactivity.is_interactive {
            return (true, Some(BoostReason::Interactive));
        }

        if task.is_foreground() {
            return (true, Some(BoostReason::Foreground));
        }

        if prediction.confidence > 0.9 {
            return (true, Some(BoostReason::HighConfidence));
        }

        (false, None)
    }

    /// Default decision
    fn default_decision(&self, task: &TaskStruct) -> SchedulingDecision {
        SchedulingDecision {
            priority: task.static_prio,
            core_affinity: CoreAffinity::Any,
            time_slice: 100,
            boost: false,
            boost_reason: None,
            confidence: 0.0,
        }
    }

    /// Get application history
    fn get_app_history(&self, app_id: u64) -> Option<&AppHistory> {
        for history in &self.app_history {
            if let Some(h) = history {
                if h.app_id == app_id {
                    return Some(h);
                }
            }
        }
        None
    }

    /// Update application history
    pub fn update_history(&mut self, app_id: u64, cpu_usage: f32, memory_usage: f32) {
        // TODO: Implement history update
    }
}

/// Scheduling Context
#[derive(Debug, Clone)]
pub struct SchedulingContext {
    /// CPU load (0.0-1.0)
    pub cpu_load: f32,

    /// Memory pressure (0.0-1.0)
    pub memory_pressure: f32,

    /// I/O load (0.0-1.0)
    pub io_load: f32,

    /// Device temperature (degrees Celsius)
    pub temperature: f32,

    /// Battery level (%)
    pub battery_level: u32,

    /// Whether charging
    pub is_charging: bool,

    /// Current timestamp (seconds)
    pub timestamp: u64,

    /// Foreground application ID
    pub foreground_app: u64,
}

/// Global AI Scheduler
static mut AI_SCHEDULER: AIProcessScheduler = AIProcessScheduler::new();

/// Get the AI Scheduler
pub fn ai_scheduler() -> &'static mut AIProcessScheduler {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut AI_SCHEDULER }
}
