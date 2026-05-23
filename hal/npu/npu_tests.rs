/*
 * Nuva OS - HAL - NPU Test Suite
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

//! NPU and AI Test Suite
/*!*/
//! Comprehensive tests for NPU HAL, ONNX, AI scheduler, and predictor.

use core::sync::atomic::Ordering;

/// Test result
#[derive(Debug)]
pub struct TestResult {
    pub name: &'static str,
    pub passed: bool,
    pub message: Option<&'static str>,
}

/// Test runner
pub struct TestRunner {
    results: [Option<TestResult>; 64],
    count: usize,
    passed: usize,
    failed: usize,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            results: core::array::from_fn(|_| None),
            count: 0,
            passed: 0,
            failed: 0,
        }
    }

    pub fn run_test(&mut self, name: &'static str, test: fn() -> bool) {
        let passed = test();
        let result = TestResult {
            name,
            passed,
            message: None,
        };

        if self.count < 64 {
            self.results[self.count] = Some(result);
            self.count += 1;

            if passed {
                self.passed += 1;
            } else {
                self.failed += 1;
            }
        }
    }

    pub fn print_summary(&self) {
        crate::log_info!("=== NPU/AI Test Summary ===");
        crate::log_info!("Total: {} tests", self.count);
        crate::log_info!("Passed: {}", self.passed);
        crate::log_info!("Failed: {}", self.failed);

        if self.failed > 0 {
            crate::log_info!("
Failed tests:");
            for i in 0..self.count {
                if let Some(ref result) = self.results[i] {
                    if !result.passed {
                        crate::log_info!("  - {}", result.name);
                    }
                }
            }
        }
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

// ============================================================================
// NPU Device Tests

fn test_data_type_size() -> bool {
    use super::device::DataType;

    assert_eq!(DataType::Float32.size(), 4);
    assert_eq!(DataType::Float16.size(), 2);
    assert_eq!(DataType::Int8.size(), 1);
    assert_eq!(DataType::Int64.size(), 8);
    assert_eq!(DataType::Bool.size(), 1);
    true
}

fn test_tensor_shape() -> bool {
    use super::device::{TensorShape, DataType};

    let shape = TensorShape::new(&[1, 3, 224, 224]);
    assert_eq!(shape.ndim, 4);
    assert_eq!(shape.dims[0], 1);
    assert_eq!(shape.dims[1], 3);
    assert_eq!(shape.dims[2], 224);
    assert_eq!(shape.dims[3], 224);
    assert_eq!(shape.elements(), 1 * 3 * 224 * 224);
    assert_eq!(shape.size_bytes(DataType::Float32), 1 * 3 * 224 * 224 * 4);
    true
}

fn test_model_handle() -> bool {
    use super::device::ModelHandle;

    let h1 = ModelHandle(1);
    let h2 = ModelHandle(2);
    let h3 = ModelHandle(1);

    assert_ne!(h1, h2);
    assert_eq!(h1, h3);
    true
}

fn test_tensor_handle() -> bool {
    use super::device::TensorHandle;

    let h1 = TensorHandle(1);
    let h2 = TensorHandle(2);

    assert_ne!(h1, h2);
    true
}

fn test_npu_vendor() -> bool {
    use super::device::NpuVendor;

    assert_eq!(NpuVendor::Huawei as i32, 0);
    assert_eq!(NpuVendor::Qualcomm as i32, 1);
    assert_eq!(NpuVendor::Intel as i32, 2);
    true
}

fn test_power_mode() -> bool {
    use super::device::PowerMode;

    assert_eq!(PowerMode::Performance as i32, 0);
    assert_eq!(PowerMode::Balanced as i32, 1);
    assert_eq!(PowerMode::PowerSave as i32, 2);
    true
}

// ============================================================================
// ONNX Runtime Tests

fn test_onnx_header() -> bool {
    use super::onnx::{OnnxHeader, ONNX_MAGIC};

    let header = OnnxHeader {
        magic: ONNX_MAGIC,
        version: 1,
        model_size: 1024,
        graph_offset: 0,
        graph_size: 512,
        metadata_offset: 512,
        metadata_size: 256,
    };

    assert_eq!(header.magic, ONNX_MAGIC);
    assert_eq!(header.version, 1);
    true
}

fn test_onnx_op_type() -> bool {
    use super::onnx::OnnxOpType;

    // Test some operator types
    let _ = OnnxOpType::Add;
    let _ = OnnxOpType::Conv;
    let _ = OnnxOpType::MatMul;
    let _ = OnnxOpType::Relu;
    true
}

fn test_onnx_session_options() -> bool {
    use super::onnx::{OnnxSessionOptions, GraphOptimizationLevel};

    let options = OnnxSessionOptions::new();
    assert!(options.enable_optimization);
    assert!(options.enable_memory_reuse);
    assert_eq!(options.optimization_level, GraphOptimizationLevel::All);
    true
}

fn test_onnx_memory_pool() -> bool {
    use super::onnx::OnnxMemoryPool;
    use super::device::BufferHandle;

    let mut pool = OnnxMemoryPool::new();
    let handle = pool.alloc(1024).unwrap();
    assert_eq!(handle, BufferHandle(0));

    pool.free(handle);
    true
}

fn test_onnx_session_stats() -> bool {
    use super::onnx::OnnxSessionStats;

    let stats = OnnxSessionStats::new();
    assert_eq!(stats.total_runs.load(Ordering::Relaxed), 0);
    assert_eq!(stats.successful_runs.load(Ordering::Relaxed), 0);
    true
}

// ============================================================================
// AI Scheduler Tests

fn test_task_priority() -> bool {
    use super::ai_scheduler::TaskPriority;

    assert!(TaskPriority::RealTime < TaskPriority::High);
    assert!(TaskPriority::High < TaskPriority::Normal);
    assert!(TaskPriority::Normal < TaskPriority::Low);
    assert!(TaskPriority::Low < TaskPriority::Background);
    true
}

fn test_task_state() -> bool {
    use super::ai_scheduler::TaskState;

    assert_eq!(TaskState::Pending as i32, 0);
    assert_eq!(TaskState::Ready as i32, 1);
    assert_eq!(TaskState::Running as i32, 2);
    assert_eq!(TaskState::Completed as i32, 3);
    true
}

fn test_task_type() -> bool {
    use super::ai_scheduler::TaskType;

    assert_eq!(TaskType::Inference as i32, 0);
    assert_eq!(TaskType::Training as i32, 1);
    assert_eq!(TaskType::Preprocessing as i32, 2);
    true
}

fn test_resource_type() -> bool {
    use super::ai_scheduler::ResourceType;

    assert_eq!(ResourceType::Cpu as i32, 0);
    assert_eq!(ResourceType::Gpu as i32, 1);
    assert_eq!(ResourceType::Npu as i32, 2);
    true
}

fn test_scheduling_reason() -> bool {
    use super::ai_scheduler::SchedulingReason;

    let _ = SchedulingReason::BestFit;
    let _ = SchedulingReason::LoadBalance;
    let _ = SchedulingReason::Deadline;
    let _ = SchedulingReason::Prediction;
    true
}

fn test_ai_scheduler_stats() -> bool {
    use super::ai_scheduler::AiSchedulerStats;

    let stats = AiSchedulerStats::new();
    assert_eq!(stats.tasks_scheduled.load(Ordering::Relaxed), 0);
    assert_eq!(stats.tasks_completed.load(Ordering::Relaxed), 0);
    true
}

// ============================================================================
// Performance Predictor Tests

fn test_feature_vector() -> bool {
    use super::predictor::FeatureVector;

    let mut features = FeatureVector::new();
    features.set(0, 1.0);
    features.set(1, 2.0);
    features.set(2, 3.0);

    assert_eq!(features.get(0), Some(1.0));
    assert_eq!(features.get(1), Some(2.0));
    assert_eq!(features.get(2), Some(3.0));
    assert_eq!(features.get(100), None); // Out of bounds
    true
}

fn test_prediction_type() -> bool {
    use super::predictor::PredictionType;

    assert_eq!(PredictionType::ExecutionTime as i32, 0);
    assert_eq!(PredictionType::MemoryUsage as i32, 1);
    assert_eq!(PredictionType::PowerConsumption as i32, 2);
    true
}

fn test_predictor_stats() -> bool {
    use super::predictor::PredictorStats;

    let stats = PredictorStats::new();
    assert_eq!(stats.predictions_made.load(Ordering::Relaxed), 0);
    assert_eq!(stats.training_iterations.load(Ordering::Relaxed), 0);
    true
}

fn test_feature_indices() -> bool {
    use super::predictor::feature_idx;

    assert_eq!(feature_idx::INPUT_SIZE, 0);
    assert_eq!(feature_idx::OUTPUT_SIZE, 1);
    assert_eq!(feature_idx::MODEL_SIZE, 2);
    assert_eq!(feature_idx::BATCH_SIZE, 3);
    true
}

// ============================================================================
// Integration Tests

fn test_npu_onnx_integration() -> bool {
    // Test that NPU and ONNX can work together
    use super::device::DataType;
    use super::onnx::OnnxSessionOptions;

    let dtype = DataType::Float32;
    let options = OnnxSessionOptions::new();

    dtype.size() == 4 && options.enable_optimization
}

fn test_scheduler_predictor_integration() -> bool {
    // Test that scheduler and predictor can work together
    use super::ai_scheduler::AiSchedulerStats;
    use super::predictor::PredictorStats;

    let sched_stats = AiSchedulerStats::new();
    let pred_stats = PredictorStats::new();

    sched_stats.tasks_scheduled.load(Ordering::Relaxed) == 0
        && pred_stats.predictions_made.load(Ordering::Relaxed) == 0
}

// ============================================================================
// Stress Tests

fn test_tensor_shape_stress() -> bool {
    use super::device::TensorShape;

    // Test various shapes
    let shapes = [
        TensorShape::new(&[1]),
        TensorShape::new(&[1, 1]),
        TensorShape::new(&[1, 3, 224, 224]),
        TensorShape::new(&[1, 64, 56, 56]),
        TensorShape::new(&[1, 128, 28, 28]),
        TensorShape::new(&[1, 256, 14, 14]),
        TensorShape::new(&[1, 512, 7, 7]),
        TensorShape::new(&[1, 1000]),
    ];

    // All shapes should have valid element counts
    shapes.iter().all(|s| s.elements() > 0)
}

fn test_feature_vector_stress() -> bool {
    use super::predictor::{FeatureVector, predictor_config};

    let mut features = FeatureVector::new();

    // Set all features
    for i in 0..predictor_config::NUM_FEATURES {
        features.set(i, i as f64);
    }

    // Verify all features
    for i in 0..predictor_config::NUM_FEATURES {
        if features.get(i) != Some(i as f64) {
            return false;
        }
    }

    true
}

// ============================================================================
// Performance Tests

fn test_tensor_shape_performance() -> bool {
    use super::device::{TensorShape, DataType};

    // Measure time for shape operations
    let start = 0u64; // TODO: Get actual time

    for _ in 0..1000 {
        let shape = TensorShape::new(&[1, 3, 224, 224]);
        let _ = shape.elements();
        let _ = shape.size_bytes(DataType::Float32);
    }

    let _end = 0u64; // TODO: Get actual time

    // For now, just verify the operations work
    true
}

fn test_prediction_performance() -> bool {
    use super::predictor::{FeatureVector, PredictorModel};

    let mut model = PredictorModel::new();
    model.init_random();

    let features = FeatureVector::new();

    // Measure time for predictions
    for _ in 0..100 {
        let _ = model.predict(&features);
    }

    true
}

// ============================================================================
// Main Test Runner

/// Run all NPU/AI tests
pub fn run_all_tests() -> bool {
    let mut runner = TestRunner::new();

    crate::log_info!("=== NPU/AI Test Suite ===
");

    // NPU device tests
    crate::log_info!("Running NPU Device tests...");
    runner.run_test("data_type_size", test_data_type_size);
    runner.run_test("tensor_shape", test_tensor_shape);
    runner.run_test("model_handle", test_model_handle);
    runner.run_test("tensor_handle", test_tensor_handle);
    runner.run_test("npu_vendor", test_npu_vendor);
    runner.run_test("power_mode", test_power_mode);

    // ONNX tests
    crate::log_info!("Running ONNX tests...");
    runner.run_test("onnx_header", test_onnx_header);
    runner.run_test("onnx_op_type", test_onnx_op_type);
    runner.run_test("onnx_session_options", test_onnx_session_options);
    runner.run_test("onnx_memory_pool", test_onnx_memory_pool);
    runner.run_test("onnx_session_stats", test_onnx_session_stats);

    // AI scheduler tests
    crate::log_info!("Running AI Scheduler tests...");
    runner.run_test("task_priority", test_task_priority);
    runner.run_test("task_state", test_task_state);
    runner.run_test("task_type", test_task_type);
    runner.run_test("resource_type", test_resource_type);
    runner.run_test("scheduling_reason", test_scheduling_reason);
    runner.run_test("ai_scheduler_stats", test_ai_scheduler_stats);

    // Performance predictor tests
    crate::log_info!("Running Performance Predictor tests...");
    runner.run_test("feature_vector", test_feature_vector);
    runner.run_test("prediction_type", test_prediction_type);
    runner.run_test("predictor_stats", test_predictor_stats);
    runner.run_test("feature_indices", test_feature_indices);

    // Integration tests
    crate::log_info!("Running Integration tests...");
    runner.run_test("npu_onnx_integration", test_npu_onnx_integration);
    runner.run_test("scheduler_predictor_integration", test_scheduler_predictor_integration);

    // Stress tests
    crate::log_info!("Running Stress tests...");
    runner.run_test("tensor_shape_stress", test_tensor_shape_stress);
    runner.run_test("feature_vector_stress", test_feature_vector_stress);

    // Performance tests
    crate::log_info!("Running Performance tests...");
    runner.run_test("tensor_shape_performance", test_tensor_shape_performance);
    runner.run_test("prediction_performance", test_prediction_performance);

    // Print summary
    runner.print_summary();

    runner.all_passed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_all() {
        assert!(run_all_tests());
    }
}
