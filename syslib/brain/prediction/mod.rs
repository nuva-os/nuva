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

//! Nuva Brain Prediction Engine
//!
//! Implements application behavior prediction and user behavior prediction.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Prediction Engine Configuration
pub struct PredictionConfig {
    /// History window size
    pub history_window: u32,

    /// Prediction interval (ms)
    pub prediction_interval_ms: u32,

    /// Minimum sample count
    pub min_samples: u32,

    /// Confidence threshold
    pub confidence_threshold: f32,

    /// Feature dimension
    pub feature_dim: u32,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            history_window: 100,
            prediction_interval_ms: 1000,
            min_samples: 10,
            confidence_threshold: 0.7,
            feature_dim: 128,
        }
    }
}

/// Application Behavior Predictor
pub struct AppBehaviorPredictor {
    /// Configuration
    config: PredictionConfig,

    /// Application history data
    app_histories: [Option<AppBehaviorHistory>; 64],

    /// History count
    num_histories: AtomicU32,

    /// Prediction model parameters (simplified)
    model_weights: [f32; 128],

    /// Statistics info
    stats: PredictorStats,
}

/// Application Behavior History
#[derive(Debug, Clone)]
pub struct AppBehaviorHistory {
    /// Application ID
    pub app_id: u64,

    /// Application type
    pub app_type: AppType,

    /// CPU usage history
    pub cpu_usage_history: [f32; 100],

    /// Memory usage history
    pub memory_usage_history: [f32; 100],

    /// Runtime duration history (seconds)
    pub duration_history: [u32; 100],

    /// Launch time history (minutes since start of day)
    pub launch_time_history: [u32; 100],

    /// Interaction frequency history
    pub interaction_history: [f32; 100],

    /// Current index
    pub current_idx: u32,

    /// Sample count
    pub sample_count: u32,

    /// Last update time
    pub last_update: u64,
}

/// Application Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppType {
    /// Unknown
    Unknown,
    /// Game
    Game,
    /// Social
    Social,
    /// Video
    Video,
    /// Music
    Music,
    /// Reading
    Reading,
    /// Utility
    Utility,
    /// System
    System,
    /// Communication
    Communication,
    /// Shopping
    Shopping,
}

/// Predictor Statistics
struct PredictorStats {
    /// Total prediction count
    total_predictions: AtomicU64,

    /// Correct prediction count
    correct_predictions: AtomicU64,

    /// Average prediction latency (us)
    avg_latency_us: AtomicU32,
}

impl AppBehaviorPredictor {
    /// Create a new predictor
    pub const fn new() -> Self {
        Self {
            config: PredictionConfig::default(),
            app_histories: [None; 64],
            num_histories: AtomicU32::new(0),
            model_weights: [0.0; 128],
            stats: PredictorStats {
                total_predictions: AtomicU64::new(0),
                correct_predictions: AtomicU64::new(0),
                avg_latency_us: AtomicU32::new(0),
            },
        }
    }

    /// Record application behavior
    pub fn record_behavior(
        &mut self,
        app_id: u64,
        app_type: AppType,
        cpu_usage: f32,
        memory_usage: f32,
        duration: u32,
        interaction_freq: f32,
        timestamp: u64,
    ) {
        // Find or create history record
        let history = self.get_or_create_history(app_id, app_type);

        // Add new sample
        let idx = history.current_idx as usize % 100;
        history.cpu_usage_history[idx] = cpu_usage;
        history.memory_usage_history[idx] = memory_usage;
        history.duration_history[idx] = duration;
        history.interaction_history[idx] = interaction_freq;
        history.launch_time_history[idx] = ((timestamp % 86400) / 60) as u32;

        history.current_idx += 1;
        if history.sample_count < 100 {
            history.sample_count += 1;
        }
        history.last_update = timestamp;
    }

    /// Predict application behavior
    pub fn predict(&self, app_id: u64, context: &PredictionContext) -> BehaviorPrediction {
        self.stats.total_predictions.fetch_add(1, Ordering::Relaxed);

        // Get history data
        let history = match self.get_history(app_id) {
            Some(h) => h,
            None => return self.default_prediction(),
        };

        // Check if we have enough samples
        if history.sample_count < self.config.min_samples {
            return self.default_prediction();
        }

        // Extract features
        let features = self.extract_features(history, context);

        // Compute prediction (simplified linear model)
        let prediction = self.compute_prediction(&features);

        // Compute confidence
        let confidence = self.compute_confidence(history);

        BehaviorPrediction {
            app_id,
            predicted_cpu_usage: prediction.cpu_usage,
            predicted_memory_usage: prediction.memory_usage,
            predicted_duration: prediction.duration,
            predicted_interaction: prediction.interaction,
            app_type: history.app_type,
            confidence,
        }
    }

    /// Extract features
    fn extract_features(&self, history: &AppBehaviorHistory, context: &PredictionContext) -> [f32; 128] {
        let mut features = [0.0f32; 128];

        // CPU usage history statistics (0-15)
        let cpu_mean = history.cpu_usage_history[..history.sample_count as usize]
            .iter().sum::<f32>() / history.sample_count as f32;
        let cpu_var = self.compute_variance(&history.cpu_usage_history[..history.sample_count as usize], cpu_mean);
        features[0] = cpu_mean;
        features[1] = cpu_var.sqrt();
        features[2] = history.cpu_usage_history[(history.current_idx as usize - 1) % 100];

        // Memory usage history statistics (16-31)
        let mem_mean = history.memory_usage_history[..history.sample_count as usize]
            .iter().sum::<f32>() / history.sample_count as f32;
        features[16] = mem_mean;
        features[17] = history.memory_usage_history[(history.current_idx as usize - 1) % 100];

        // Runtime duration history statistics (32-47)
        let dur_mean = history.duration_history[..history.sample_count as usize]
            .iter().sum::<u32>() as f32 / history.sample_count as f32;
        features[32] = dur_mean / 3600.0; // Normalize to hours

        // Interaction frequency statistics (48-63)
        let int_mean = history.interaction_history[..history.sample_count as usize]
            .iter().sum::<f32>() / history.sample_count as f32;
        features[48] = int_mean;

        // Time pattern (64-87)
        let current_minute = ((context.timestamp % 86400) / 60) as usize;
        let hour = current_minute / 60;
        features[64 + hour] = 1.0;

        // Application type encoding (88-99)
        features[88 + history.app_type as usize] = 1.0;

        // Context features (100-127)
        features[100] = context.system_load;
        features[101] = context.memory_pressure;
        features[102] = context.battery_level as f32 / 100.0;
        features[103] = if context.is_charging { 1.0 } else { 0.0 };
        features[104] = context.temperature / 100.0;

        features
    }

    /// Compute prediction
    fn compute_prediction(&self, features: &[f32; 128]) -> PredictionResult {
        // Simplified: use weighted average
        let cpu_usage = features[0] * 0.6 + features[2] * 0.4;
        let memory_usage = features[16];
        let duration = features[32] * 3600.0;
        let interaction = features[48];

        PredictionResult {
            cpu_usage: cpu_usage.clamp(0.0, 1.0),
            memory_usage: memory_usage.clamp(0.0, 1.0),
            duration: duration as u32,
            interaction: interaction.clamp(0.0, 10.0),
        }
    }

    /// Compute confidence
    fn compute_confidence(&self, history: &AppBehaviorHistory) -> f32 {
        // Confidence based on sample count and variance
        let sample_factor = (history.sample_count as f32 / 100.0).min(1.0);

        let cpu_var = self.compute_variance(
            &history.cpu_usage_history[..history.sample_count as usize],
            history.cpu_usage_history[..history.sample_count as usize].iter().sum::<f32>() / history.sample_count as f32
        );

        // Lower variance means higher confidence
        let variance_factor = 1.0 - (cpu_var * 4.0).min(1.0);

        (sample_factor * 0.5 + variance_factor * 0.5).clamp(0.0, 1.0)
    }

    /// Compute variance
    fn compute_variance(&self, data: &[f32], mean: f32) -> f32 {
        if data.is_empty() {
            return 0.0;
        }
        data.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / data.len() as f32
    }

    /// Get or create a history record
    fn get_or_create_history(&mut self, app_id: u64, app_type: AppType) -> &mut AppBehaviorHistory {
        // Find existing record
        for i in 0..64 {
            if let Some(ref h) = self.app_histories[i] {
                if h.app_id == app_id {
                    return &mut self.app_histories[i].as_mut().unwrap();
                }
            }
        }

        // Create a new record
        let idx = self.num_histories.fetch_add(1, Ordering::Relaxed) as usize;
        if idx < 64 {
            self.app_histories[idx] = Some(AppBehaviorHistory {
                app_id,
                app_type,
                cpu_usage_history: [0.0; 100],
                memory_usage_history: [0.0; 100],
                duration_history: [0; 100],
                launch_time_history: [0; 100],
                interaction_history: [0.0; 100],
                current_idx: 0,
                sample_count: 0,
                last_update: 0,
            });
            return &mut self.app_histories[idx].as_mut().unwrap();
        }

        // If full, replace the oldest entry
        let mut oldest_idx = 0;
        let mut oldest_time = u64::MAX;
        for i in 0..64 {
            if let Some(ref h) = self.app_histories[i] {
                if h.last_update < oldest_time {
                    oldest_time = h.last_update;
                    oldest_idx = i;
                }
            }
        }

        self.app_histories[oldest_idx] = Some(AppBehaviorHistory {
            app_id,
            app_type,
            cpu_usage_history: [0.0; 100],
            memory_usage_history: [0.0; 100],
            duration_history: [0; 100],
            launch_time_history: [0; 100],
            interaction_history: [0.0; 100],
            current_idx: 0,
            sample_count: 0,
            last_update: 0,
        });

        &mut self.app_histories[oldest_idx].as_mut().unwrap()
    }

    /// Get a history record
    fn get_history(&self, app_id: u64) -> Option<&AppBehaviorHistory> {
        for i in 0..64 {
            if let Some(ref h) = self.app_histories[i] {
                if h.app_id == app_id {
                    return Some(h);
                }
            }
        }
        None
    }

    /// Default prediction
    fn default_prediction(&self) -> BehaviorPrediction {
        BehaviorPrediction {
            app_id: 0,
            predicted_cpu_usage: 0.3,
            predicted_memory_usage: 0.2,
            predicted_duration: 300,
            predicted_interaction: 0.5,
            app_type: AppType::Unknown,
            confidence: 0.0,
        }
    }
}

/// Prediction Result (internal)
struct PredictionResult {
    cpu_usage: f32,
    memory_usage: f32,
    duration: u32,
    interaction: f32,
}

/// Prediction Result (public)
#[derive(Debug, Clone)]
pub struct BehaviorPrediction {
    /// Application ID
    pub app_id: u64,

    /// Predicted CPU usage
    pub predicted_cpu_usage: f32,

    /// Predicted memory usage
    pub predicted_memory_usage: f32,

    /// Predicted runtime duration (seconds)
    pub predicted_duration: u32,

    /// Predicted interaction frequency
    pub predicted_interaction: f32,

    /// Application type
    pub app_type: AppType,

    /// Confidence
    pub confidence: f32,
}

/// Prediction Context
#[derive(Debug, Clone)]
pub struct PredictionContext {
    /// System load
    pub system_load: f32,

    /// Memory pressure
    pub memory_pressure: f32,

    /// Battery level
    pub battery_level: u32,

    /// Whether charging
    pub is_charging: bool,

    /// Temperature
    pub temperature: f32,

    /// Timestamp
    pub timestamp: u64,
}

/// User Habit Predictor
pub struct UserHabitPredictor {
    /// Configuration
    config: PredictionConfig,

    /// User habit data
    habits: UserHabits,

    /// Statistics info
    stats: PredictorStats,
}

/// User Habits
#[derive(Debug, Clone)]
pub struct UserHabits {
    /// Hourly app usage frequency (24 * 7 = 168 time slots)
    pub hourly_app_usage: [[u32; 64]; 168],

    /// App usage sequences
    pub app_sequences: [AppSequence; 32],

    /// Charging habits
    pub charging_habits: ChargingHabits,

    /// Idle periods
    pub idle_periods: [IdlePeriod; 16],
}

/// App Usage Sequence
#[derive(Debug, Clone)]
pub struct AppSequence {
    /// Previous application ID
    pub prev_app: u64,

    /// Next application ID
    pub next_app: u64,

    /// Occurrence count
    pub count: u32,
}

/// Charging Habits
#[derive(Debug, Clone)]
pub struct ChargingHabits {
    /// Hourly charging probability
    pub hourly_charging_prob: [f32; 24],

    /// Average charging duration (minutes)
    pub avg_charging_duration: u32,

    /// Average battery level when charging
    pub avg_charging_level: u32,
}

/// Idle Period
#[derive(Debug, Clone)]
pub struct IdlePeriod {
    /// Start time (minutes since start of day)
    pub start_minute: u32,

    /// End time
    pub end_minute: u32,

    /// Occurrence count
    pub count: u32,
}

impl UserHabitPredictor {
    /// Create a new user predictor
    pub const fn new() -> Self {
        Self {
            config: PredictionConfig::default(),
            habits: UserHabits {
                hourly_app_usage: [[0; 64]; 168],
                app_sequences: [AppSequence { prev_app: 0, next_app: 0, count: 0 }; 32],
                charging_habits: ChargingHabits {
                    hourly_charging_prob: [0.0; 24],
                    avg_charging_duration: 0,
                    avg_charging_level: 0,
                },
                idle_periods: [IdlePeriod { start_minute: 0, end_minute: 0, count: 0 }; 16],
            },
            stats: PredictorStats {
                total_predictions: AtomicU64::new(0),
                correct_predictions: AtomicU64::new(0),
                avg_latency_us: AtomicU32::new(0),
            },
        }
    }

    /// Record app usage
    pub fn record_app_usage(&mut self, app_id: u64, timestamp: u64) {
        let day_of_week = ((timestamp / 86400) % 7) as usize;
        let hour = ((timestamp % 86400) / 3600) as usize;
        let slot = day_of_week * 24 + hour;

        // Update hourly usage frequency
        let app_idx = (app_id % 64) as usize;
        self.habits.hourly_app_usage[slot][app_idx] += 1;
    }

    /// Predict the next application
    pub fn predict_next_app(&self, current_app: u64, timestamp: u64) -> Option<u64> {
        let day_of_week = ((timestamp / 86400) % 7) as usize;
        let hour = ((timestamp % 86400) / 3600) as usize;
        let slot = day_of_week * 24 + hour;

        // Find the most likely next application
        let mut best_app = None;
        let mut best_count = 0;

        for seq in &self.habits.app_sequences {
            if seq.prev_app == current_app && seq.count > best_count {
                best_count = seq.count;
                best_app = Some(seq.next_app);
            }
        }

        // If no sequence data found, use time slot statistics
        if best_app.is_none() {
            let usage = &self.habits.hourly_app_usage[slot];
            let mut max_usage = 0;
            for (i, &count) in usage.iter().enumerate() {
                if count > max_usage {
                    max_usage = count;
                    best_app = Some(i as u64);
                }
            }
        }

        best_app
    }

    /// Predict next charging time
    pub fn predict_next_charge(&self, current_level: u32, timestamp: u64) -> Option<u64> {
        let hour = ((timestamp % 86400) / 3600) as usize;

        // Find the next time slot with high charging probability
        for i in 0..24 {
            let check_hour = (hour + i) % 24;
            if self.habits.charging_habits.hourly_charging_prob[check_hour] > 0.5 {
                return Some(timestamp + (i as u64 * 3600));
            }
        }

        // Predict based on battery level
        if current_level < 20 {
            Some(timestamp + 3600) // Within 1 hour
        } else {
            None
        }
    }

    /// Predict idle period
    pub fn predict_idle_period(&self, timestamp: u64) -> Option<IdlePeriod> {
        let current_minute = ((timestamp % 86400) / 60) as u32;

        // Find current or upcoming idle period
        for period in &self.habits.idle_periods {
            if period.count > 5 { // At least 5 occurrences
                if period.start_minute >= current_minute {
                    return Some(period.clone());
                }
            }
        }

        None
    }
}

/// Global Prediction Engine
static mut APP_PREDICTOR: AppBehaviorPredictor = AppBehaviorPredictor::new();
static mut USER_PREDICTOR: UserHabitPredictor = UserHabitPredictor::new();

/// Get the application behavior predictor
pub fn app_predictor() -> &'static mut AppBehaviorPredictor {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut APP_PREDICTOR }
}

/// Get the user predictor
pub fn user_predictor() -> &'static mut UserHabitPredictor {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut USER_PREDICTOR }
}
