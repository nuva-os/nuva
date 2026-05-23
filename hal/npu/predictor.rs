/*
 * Nuva OS - HAL - Performance Predictor
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

//! Performance Predictor
/*!*/
//! ML-based performance prediction for execution time and resource usage.

// TODO: AtomicF32 does not exist in core::sync::atomic; using AtomicU32 as a workaround
// (use f32::to_bits() / f32::from_bits() to convert)
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use crate::{pr_info};

/// Predictor configuration
pub mod predictor_config {
    /// Number of features
    pub const NUM_FEATURES: usize = 32;

    /// Hidden layer size
    pub const HIDDEN_SIZE: usize = 64;

    /// Number of training samples
    pub const TRAINING_SAMPLES: usize = 1000;

    /// Prediction window
    pub const PREDICTION_WINDOW: usize = 100;

    /// Model update interval
    pub const UPDATE_INTERVAL_MS: u64 = 1000;
}

/// Prediction type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictionType {
    ExecutionTime = 0,
    MemoryUsage = 1,
    PowerConsumption = 2,
    Throughput = 3,
    Latency = 4,
}

/// Prediction result
#[derive(Debug, Clone)]
pub struct PredictionResult {
    pub prediction_type: PredictionType,
    pub value: f64,
    pub confidence: f32,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub features_used: u32,
}

/// Feature vector
#[derive(Debug, Clone)]
pub struct FeatureVector {
    pub features: [f64; predictor_config::NUM_FEATURES],
    pub valid: [bool; predictor_config::NUM_FEATURES],
}

impl FeatureVector {
    pub fn new() -> Self {
        Self {
            features: [0.0; predictor_config::NUM_FEATURES],
            valid: [false; predictor_config::NUM_FEATURES],
        }
    }

    pub fn set(&mut self, idx: usize, value: f64) {
        if idx < predictor_config::NUM_FEATURES {
            self.features[idx] = value;
            self.valid[idx] = true;
        }
    }

    pub fn get(&self, idx: usize) -> Option<f64> {
        if idx < predictor_config::NUM_FEATURES && self.valid[idx] {
            Some(self.features[idx])
        } else {
            None
        }
    }
}

/// Feature indices
pub mod feature_idx {
    pub const INPUT_SIZE: usize = 0;
    pub const OUTPUT_SIZE: usize = 1;
    pub const MODEL_SIZE: usize = 2;
    pub const BATCH_SIZE: usize = 3;
    pub const NUM_LAYERS: usize = 4;
    pub const NUM_PARAMS: usize = 5;
    pub const NUM_OPS: usize = 6;
    pub const MEMORY_BANDWIDTH: usize = 7;
    pub const COMPUTE_FLOPS: usize = 8;
    pub const RESOURCE_UTIL: usize = 9;
    pub const TEMPERATURE: usize = 10;
    pub const POWER: usize = 11;
    pub const HISTORICAL_AVG: usize = 12;
    pub const HISTORICAL_STD: usize = 13;
    pub const HISTORICAL_MIN: usize = 14;
    pub const HISTORICAL_MAX: usize = 15;
    pub const TIME_OF_DAY: usize = 16;
    pub const DAY_OF_WEEK: usize = 17;
    pub const QUEUE_DEPTH: usize = 18;
    pub const NUM_CONCURRENT: usize = 19;
    pub const CACHE_HIT_RATE: usize = 20;
    pub const MEMORY_FRAGMENTATION: usize = 21;
}

/// Neural network layer
pub struct NeuralLayer {
    pub weights: [[f32; predictor_config::NUM_FEATURES]; predictor_config::HIDDEN_SIZE],
    pub biases: [f32; predictor_config::HIDDEN_SIZE],
    pub activations: [f32; predictor_config::HIDDEN_SIZE],
}

impl NeuralLayer {
    pub fn new() -> Self {
        Self {
            weights: [[0.0; predictor_config::NUM_FEATURES]; predictor_config::HIDDEN_SIZE],
            biases: [0.0; predictor_config::HIDDEN_SIZE],
            activations: [0.0; predictor_config::HIDDEN_SIZE],
        }
    }

    pub fn forward(&mut self, input: &[f64; predictor_config::NUM_FEATURES]) -> [f32; predictor_config::HIDDEN_SIZE] {
        for i in 0..predictor_config::HIDDEN_SIZE {
            let mut sum = self.biases[i];
            for j in 0..predictor_config::NUM_FEATURES {
                sum += self.weights[i][j] * input[j] as f32;
            }
            // ReLU activation
            self.activations[i] = if sum > 0.0 { sum } else { 0.0 };
        }
        self.activations
    }
}

/// Performance predictor model
pub struct PredictorModel {
    /// Input layer
    input_layer: NeuralLayer,

    /// Hidden layer
    hidden_layer: NeuralLayer,

    /// Output weights
    output_weights: [f32; predictor_config::HIDDEN_SIZE],

    /// Output bias
    output_bias: f32,

    /// Training samples
    training_data: Vec<TrainingSample>,

    /// Model statistics
    stats: PredictorStats,
}

/// Training sample
#[derive(Debug, Clone)]
pub struct TrainingSample {
    pub features: FeatureVector,
    pub target: f64,
    pub weight: f32,
}

/// Predictor statistics
pub struct PredictorStats {
    pub predictions_made: AtomicU64,
    pub predictions_correct: AtomicU64,
    // Using AtomicU32 to represent f32 bits (use f32::to_bits/from_bits)
    pub total_error: AtomicU32,
    pub avg_error: AtomicU32,
    pub training_iterations: AtomicU64,
}

impl PredictorStats {
    pub fn new() -> Self {
        Self {
            predictions_made: AtomicU64::new(0),
            predictions_correct: AtomicU64::new(0),
            total_error: AtomicU32::new(0.0f32.to_bits()),
            avg_error: AtomicU32::new(0.0f32.to_bits()),
            training_iterations: AtomicU64::new(0),
        }
    }
}

impl PredictorModel {
    pub fn new() -> Self {
        Self {
            input_layer: NeuralLayer::new(),
            hidden_layer: NeuralLayer::new(),
            output_weights: [0.0; predictor_config::HIDDEN_SIZE],
            output_bias: 0.0,
            training_data: Vec::new(),
            stats: PredictorStats::new(),
        }
    }

    /// Initialize with random weights
    pub fn init_random(&mut self) {
        // Simple initialization
        for i in 0..predictor_config::HIDDEN_SIZE {
            for j in 0..predictor_config::NUM_FEATURES {
                self.input_layer.weights[i][j] = (i as f32 * 0.01) - 0.5;
                self.hidden_layer.weights[i][j] = (j as f32 * 0.01) - 0.5;
            }
            self.input_layer.biases[i] = 0.0;
            self.hidden_layer.biases[i] = 0.0;
            self.output_weights[i] = (i as f32 * 0.01) - 0.5;
        }
        self.output_bias = 0.0;
    }

    /// Predict
    pub fn predict(&mut self, features: &FeatureVector) -> PredictionResult {
        // Forward pass
        let input = features.features;
        let hidden1 = self.input_layer.forward(&input);
        let hidden2 = self.hidden_layer.forward(&input);

        // Output
        let mut output = self.output_bias;
        for i in 0..predictor_config::HIDDEN_SIZE {
            output += self.output_weights[i] * hidden2[i];
        }

        // Calculate confidence
        let valid_count = features.valid.iter().filter(|&&v| v).count() as f32;
        let confidence = valid_count / predictor_config::NUM_FEATURES as f32;

        // Calculate bounds (simple: ±10%)
        let lower = output * 0.9;
        let upper = output * 1.1;

        // Update statistics
        self.stats.predictions_made.fetch_add(1, Ordering::Relaxed);

        PredictionResult {
            prediction_type: PredictionType::ExecutionTime,
            value: output as f64,
            confidence,
            lower_bound: lower as f64,
            upper_bound: upper as f64,
            features_used: valid_count as u32,
        }
    }

    /// Add training sample
    pub fn add_sample(&mut self, sample: TrainingSample) {
        self.training_data.push(sample);

        // Limit training data size
        if self.training_data.len() > predictor_config::TRAINING_SAMPLES {
            self.training_data.remove(0);
        }
    }

    /// Train model
    pub fn train(&mut self, epochs: usize, learning_rate: f32) -> Result<(), PredictorError> {
        if self.training_data.is_empty() {
            return Err(PredictorError::NoTrainingData);
        }

        for _ in 0..epochs {
            for sample_idx in 0..self.training_data.len() {
                let features = self.training_data[sample_idx].features.clone();
                let target = self.training_data[sample_idx].target;
                let weight = self.training_data[sample_idx].weight;
                // Forward pass
                let pred = self.predict(&features);

                // Calculate error
                let error = target - pred.value;
                let weighted_error = error as f32 * weight;

                // Update output weights (gradient descent)
                for i in 0..predictor_config::HIDDEN_SIZE {
                    self.output_weights[i] += learning_rate * weighted_error * self.hidden_layer.activations[i];
                }
                self.output_bias += learning_rate * weighted_error;

                // Update statistics
                let current = f32::from_bits(self.stats.total_error.load(Ordering::Relaxed));
                self.stats.total_error.store((current + error.abs() as f32).to_bits(), Ordering::Relaxed);
            }
        }

        // Update average error
        let total = self.stats.predictions_made.load(Ordering::Relaxed);
        if total > 0 {
            let avg = f32::from_bits(self.stats.total_error.load(Ordering::Relaxed)) / total as f32;
            self.stats.avg_error.store(avg.to_bits(), Ordering::Release);
        }

        self.stats.training_iterations.fetch_add(epochs as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> &PredictorStats {
        &self.stats
    }

    /// Get prediction accuracy
    pub fn accuracy(&self) -> f32 {
        let total = self.stats.predictions_made.load(Ordering::Relaxed);
        let correct = self.stats.predictions_correct.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            correct as f32 / total as f32
        }
    }
}

/// Performance predictor
pub struct PerformancePredictor {
    /// Models for each prediction type
    models: [PredictorModel; 5],

    /// Feature extractor
    feature_extractor: FeatureExtractor,

    /// Historical data
    history: Vec<HistoricalEntry>,

    /// Enabled
    enabled: AtomicBool,
}

/// Historical entry
#[derive(Debug, Clone)]
pub struct HistoricalEntry {
    pub timestamp: u64,
    pub features: FeatureVector,
    pub actual: f64,
    pub predicted: f64,
}

/// Feature extractor
pub struct FeatureExtractor {
    /// Historical statistics
    historical_avg: [f64; predictor_config::NUM_FEATURES],
    historical_std: [f64; predictor_config::NUM_FEATURES],
    historical_min: [f64; predictor_config::NUM_FEATURES],
    historical_max: [f64; predictor_config::NUM_FEATURES],
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            historical_avg: [0.0; predictor_config::NUM_FEATURES],
            historical_std: [0.0; predictor_config::NUM_FEATURES],
            historical_min: [f64::MAX; predictor_config::NUM_FEATURES],
            historical_max: [f64::MIN; predictor_config::NUM_FEATURES],
        }
    }

    /// Extract features from task
    pub fn extract(&self, task: &TaskInfo) -> FeatureVector {
        let mut features = FeatureVector::new();

        features.set(feature_idx::INPUT_SIZE, task.input_size as f64);
        features.set(feature_idx::OUTPUT_SIZE, task.output_size as f64);
        features.set(feature_idx::MODEL_SIZE, task.model_size as f64);
        features.set(feature_idx::BATCH_SIZE, task.batch_size as f64);
        features.set(feature_idx::NUM_LAYERS, task.num_layers as f64);
        features.set(feature_idx::NUM_PARAMS, task.num_params as f64);
        features.set(feature_idx::RESOURCE_UTIL, task.resource_util as f64);
        features.set(feature_idx::TEMPERATURE, task.temperature as f64);
        features.set(feature_idx::POWER, task.power_mw as f64);

        // Add historical features
        features.set(feature_idx::HISTORICAL_AVG, self.historical_avg[feature_idx::INPUT_SIZE]);
        features.set(feature_idx::HISTORICAL_STD, self.historical_std[feature_idx::INPUT_SIZE]);
        features.set(feature_idx::HISTORICAL_MIN, self.historical_min[feature_idx::INPUT_SIZE]);
        features.set(feature_idx::HISTORICAL_MAX, self.historical_max[feature_idx::INPUT_SIZE]);

        features
    }

    /// Update historical statistics
    pub fn update(&mut self, features: &FeatureVector) {
        for i in 0..predictor_config::NUM_FEATURES {
            if let Some(value) = features.get(i) {
                // Update min/max
                self.historical_min[i] = self.historical_min[i].min(value);
                self.historical_max[i] = self.historical_max[i].max(value);

                // Update average (simple moving average)
                let alpha = 0.01;
                self.historical_avg[i] = alpha * value + (1.0 - alpha) * self.historical_avg[i];
            }
        }
    }
}

/// Task information for feature extraction
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub input_size: u64,
    pub output_size: u64,
    pub model_size: u64,
    pub batch_size: u32,
    pub num_layers: u32,
    pub num_params: u64,
    pub num_ops: u32,
    pub resource_util: f32,
    pub temperature: f32,
    pub power_mw: u32,
}

impl PerformancePredictor {
    pub fn new() -> Self {
        let mut predictor = Self {
            models: [
                PredictorModel::new(),
                PredictorModel::new(),
                PredictorModel::new(),
                PredictorModel::new(),
                PredictorModel::new(),
            ],
            feature_extractor: FeatureExtractor::new(),
            history: Vec::new(),
            enabled: AtomicBool::new(true),
        };

        // Initialize models
        for model in &mut predictor.models {
            model.init_random();
        }

        predictor
    }

    /// Initialize predictor
    pub fn init(&mut self) -> Result<(), PredictorError> {
        log_info!("Performance Predictor initialized");
        self.enabled.store(true, Ordering::Release);
        Ok(())
    }

    /// Predict execution time
    pub fn predict_time(&mut self, task: &TaskInfo) -> PredictionResult {
        let features = self.feature_extractor.extract(task);
        self.models[PredictionType::ExecutionTime as usize].predict(&features)
    }

    /// Predict memory usage
    pub fn predict_memory(&mut self, task: &TaskInfo) -> PredictionResult {
        let features = self.feature_extractor.extract(task);
        self.models[PredictionType::MemoryUsage as usize].predict(&features)
    }

    /// Predict power consumption
    pub fn predict_power(&mut self, task: &TaskInfo) -> PredictionResult {
        let features = self.feature_extractor.extract(task);
        self.models[PredictionType::PowerConsumption as usize].predict(&features)
    }

    /// Record actual result
    pub fn record_result(&mut self, task: &TaskInfo, actual_time: u64) {
        let features = self.feature_extractor.extract(task);
        let predicted = self.predict_time(task);

        // Add to history
        self.history.push(HistoricalEntry {
            timestamp: crate::hal::cpu::read_cycle_counter() / 1000,
            features: features.clone(),
            actual: actual_time as f64,
            predicted: predicted.value,
        });

        // Limit history size
        if self.history.len() > predictor_config::PREDICTION_WINDOW {
            self.history.remove(0);
        }

        // Add training sample
        let sample = TrainingSample {
            features,
            target: actual_time as f64,
            weight: 1.0,
        };
        self.models[PredictionType::ExecutionTime as usize].add_sample(sample);

        // Update feature extractor
        self.feature_extractor.update(&self.history.last().unwrap().features);
    }

    /// Train models
    pub fn train(&mut self, epochs: usize, learning_rate: f32) -> Result<(), PredictorError> {
        for model in &mut self.models {
            model.train(epochs, learning_rate)?;
        }
        Ok(())
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Enable/disable
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
    }
}

/// Predictor error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredictorError {
    NoTrainingData,
    InvalidFeatures,
    ModelNotTrained,
    PredictionFailed,
}

/// Global performance predictor
static mut PERFORMANCE_PREDICTOR: core::mem::MaybeUninit<PerformancePredictor> = core::mem::MaybeUninit::uninit();

/// Get performance predictor
pub fn get_predictor() -> &'static mut PerformancePredictor {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { PERFORMANCE_PREDICTOR.assume_init_mut() }
}

/// Initialize performance predictor
pub fn init_predictor() -> Result<(), PredictorError> {
    // SAFETY: PERFORMANCE_PREDICTOR is only written here during init
    unsafe { PERFORMANCE_PREDICTOR.write(PerformancePredictor::new()); }
    get_predictor().init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector() {
        let mut features = FeatureVector::new();
        features.set(0, 1.0);
        features.set(1, 2.0);

        assert_eq!(features.get(0), Some(1.0));
        assert_eq!(features.get(1), Some(2.0));
        assert_eq!(features.get(2), None);
    }

    #[test]
    fn test_neural_layer() {
        let mut layer = NeuralLayer::new();
        let input = [1.0; predictor_config::NUM_FEATURES];
        let output = layer.forward(&input);

        assert_eq!(output.len(), predictor_config::HIDDEN_SIZE);
    }

    #[test]
    fn test_predictor_model() {
        let mut model = PredictorModel::new();
        model.init_random();

        let features = FeatureVector::new();
        let result = model.predict(&features);

        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    }

    #[test]
    fn test_predictor_stats() {
        let stats = PredictorStats::new();
        assert_eq!(stats.predictions_made.load(Ordering::Relaxed), 0);
    }
}
