/*
 * Nuva OS - HAL - Npu
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

//! NPU (Neural Processing Unit) HAL
/*!*/
//! Complete NPU support with:
//! - Device abstraction
//! - ONNX runtime
//! - AI scheduler
//! - Performance predictor

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod device;
pub mod onnx;
pub mod ai_scheduler;
pub mod predictor;
pub mod npu_tests;
pub mod davinci;
pub mod traits;
pub mod hexagon;

// Re-export key types
pub use device::{
    NpuDevice, NpuManager, NpuInfo, NpuError, NpuStats,
    ModelHandle, TensorHandle, BufferHandle, DataType,
    TensorShape, TensorDesc, ModelFormat, ModelInfo,
    InferenceResult, PowerMode, NpuFeatures, NpuVendor,
    init_npu, get_npu_manager,
};

// Re-export NpuInfo, ModelFormat, ModelInfo from device module
// (local definitions removed to avoid name collisions)
pub use onnx::{
    OnnxRuntime, OnnxSession, OnnxGraph, OnnxNode, OnnxOpType,
    OnnxSessionOptions, GraphOptimizationLevel,
    init_onnx, get_onnx_runtime,
};
pub use ai_scheduler::{
    AiScheduler, TaskDesc, TaskPriority, TaskType, TaskState,
    ResourceDesc, ResourceType, SchedulingDecision,
    PerformancePrediction, init_ai_scheduler, get_ai_scheduler,
};
pub use predictor::{
    PerformancePredictor, PredictionResult, PredictionType,
    FeatureVector, TaskInfo, init_predictor, get_predictor,
};

/// NPU state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpuState {
    /// Idle
    Idle = 0,
    /// Running
    Running = 1,
    /// Suspended
    Suspended = 2,
    /// Error
    Error = 3,
}

/// Compute task type
#[derive(Debug, Clone, Copy)]
pub enum ComputeTaskType {
    /// Inference
    Inference = 0,
    /// Training
    Training = 1,
    /// Quantization
    Quantization = 2,
    /// Compilation
    Compilation = 3,
}

/// Compute task
pub struct ComputeTask {
    /// Task type
    pub task_type: ComputeTaskType,
    /// Model ID
    pub model_id: u32,
    /// Input data pointer
    pub input_data: u64,
    /// Input size
    pub input_size: u64,
    /// Output data pointer
    pub output_data: u64,
    /// Output size
    pub output_size: u64,
    /// Priority
    pub priority: u32,
    /// Synchronization object
    pub sync_obj: u64,
    /// Input address
    pub input_addr: u64,
    /// Output address
    pub output_addr: u64,
}

/// NPU HAL operations
pub struct NpuHalOps {
    /// Initialize
    pub init: fn() -> i32,
    /// Get NPU info
    pub get_npu_info: fn() -> NpuInfo,
    /// Load model
    pub load_model: fn(data: &[u8], format: ModelFormat) -> i32,
    /// Unload model
    pub unload_model: fn(model_id: u32) -> i32,
    /// Submit compute task
    pub submit_task: fn(task: &ComputeTask) -> i32,
    /// Wait for task completion
    pub wait_task: fn(sync_obj: u64, timeout: u64) -> i32,
    /// Set frequency
    pub set_frequency: fn(freq: u64) -> i32,
    /// Get frequency
    pub get_frequency: fn() -> u64,
    /// Suspend
    pub suspend: fn() -> i32,
    /// Resume
    pub resume: fn() -> i32,
}

/// NPU HAL device
pub struct NpuHalDevice {
    /// NPU info
    pub info: NpuInfo,
    /// HAL operations
    pub ops: &'static NpuHalOps,
    /// Number of loaded models
    pub num_models: u32,
}

impl NpuHalDevice {
    pub const fn new() -> Self {
        NpuHalDevice {
            info: NpuInfo {
                npu_id: 0,
                name: "Unknown",
                vendor: NpuVendor::Huawei,
                version: "",
                frequency_mhz: 0,
                state: NpuState::Idle as u32,
                current_freq: 0,
                min_freq: 0,
                max_freq: 0,
                num_cores: 0,
                memory_size: 0,
                memory_bandwidth_gbps: 0,
                supported_dtypes: 0,
                max_batch_size: 0,
                features: NpuFeatures {
                    async_execution: false,
                    dynamic_shapes: false,
                    quantization: false,
                    sparse: false,
                    mixed_precision: false,
                    onnx: false,
                    tflite: false,
                    pytorch: false,
                },
                utilization: 0,
                temperature: 0,
            },
            ops: &NPU_HAL_OPS_NONE,
            num_models: 0,
        }
    }

    /// Initialize
    pub fn init(&mut self) -> i32 {
        (self.ops.init)()
    }

    /// Load model
    pub fn load_model(&mut self, data: &[u8], format: ModelFormat) -> i32 {
        let result = (self.ops.load_model)(data, format);
        if result >= 0 {
            self.num_models += 1;
        }
        result
    }

    /// Unload model
    pub fn unload_model(&mut self, model_id: u32) -> i32 {
        let result = (self.ops.unload_model)(model_id);
        if result >= 0 {
            self.num_models -= 1;
        }
        result
    }

    /// Submit compute task
    pub fn submit_task(&self, task: &ComputeTask) -> i32 {
        (self.ops.submit_task)(task)
    }

    /// Wait for task completion
    pub fn wait_task(&self, sync_obj: u64, timeout: u64) -> i32 {
        (self.ops.wait_task)(sync_obj, timeout)
    }
}

/// Empty NPU HAL operations
static NPU_HAL_OPS_NONE: NpuHalOps = NpuHalOps {
    init: || -1,
    get_npu_info: || NpuInfo {
        npu_id: 0,
        name: "None",
        vendor: NpuVendor::Huawei,
        version: "",
        frequency_mhz: 0,
        state: NpuState::Error as u32,
        current_freq: 0,
        min_freq: 0,
        max_freq: 0,
        num_cores: 0,
        memory_size: 0,
        memory_bandwidth_gbps: 0,
        supported_dtypes: 0,
        max_batch_size: 0,
        features: NpuFeatures::empty(),
        utilization: 0,
        temperature: 0,
    },
    load_model: |_data, _format| -1,
    unload_model: |_model_id| -1,
    submit_task: |_task| -1,
    wait_task: |_sync_obj, _timeout| -1,
    set_frequency: |_freq| -1,
    get_frequency: || 0,
    suspend: || -1,
    resume: || -1,
};

/// Global NPU HAL device
static mut NPU_HAL_DEVICE: NpuHalDevice = NpuHalDevice::new();

pub fn get_npu_hal() -> &'static mut NpuHalDevice {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut NPU_HAL_DEVICE }
}

pub fn init_npu_hal() {
    log_info!("NPU HAL initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npu_hal() {
        let hal = get_npu_hal();
        assert_eq!(hal.info.npu_id, 0);
    }

    #[test]
    fn test_npu_state() {
        assert_eq!(NpuState::Idle as i32, 0);
        assert_eq!(NpuState::Running as i32, 1);
        assert_eq!(NpuState::Suspended as i32, 2);
        assert_eq!(NpuState::Error as i32, 3);
    }

    #[test]
    fn test_model_format() {
        assert_eq!(ModelFormat::Onnx as i32, 0);
        assert_eq!(ModelFormat::TFLite as i32, 1);
        assert_eq!(ModelFormat::Caffe as i32, 2);
        assert_eq!(ModelFormat::Custom as i32, 3);
    }

    #[test]
    fn test_compute_task_type() {
        assert_eq!(ComputeTaskType::Inference as i32, 0);
        assert_eq!(ComputeTaskType::Training as i32, 1);
        assert_eq!(ComputeTaskType::Quantization as i32, 2);
        assert_eq!(ComputeTaskType::Compilation as i32, 3);
    }

    #[test]
    fn test_npu_info() {
        let info = NpuInfo {
            npu_id: 0,
            name: "Da Vinci",
            vendor: NpuVendor::Huawei,
            version: "",
            frequency_mhz: 1000,
            state: NpuState::Running,
            current_freq: 1_000_000_000,
            min_freq: 500_000_000,
            max_freq: 1_500_000_000,
            num_cores: 8,
            memory_size: 4 * 1024 * 1024 * 1024,  // 4GB
            memory_bandwidth_gbps: 0,
            supported_dtypes: 0,
            max_batch_size: 0,
            features: NpuFeatures::empty(),
            utilization: 80,
            temperature: 55000,  // 55°C
        };

        assert_eq!(info.npu_id, 0);
        assert_eq!(info.name, "Da Vinci");
        assert_eq!(info.num_cores, 8);
        assert_eq!(info.memory_size, 4 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_model_info() {
        let info = ModelInfo {
            model_id: 1,
            name: "resnet50",
            format: ModelFormat::Onnx,
            size: 100 * 1024 * 1024,  // 100MB
            num_inputs: 1,
            num_outputs: 1,
        };

        assert_eq!(info.model_id, 1);
        assert_eq!(info.format, ModelFormat::Onnx);
        assert_eq!(info.num_inputs, 1);
    }

    #[test]
    fn test_compute_task() {
        let task = ComputeTask {
            task_type: ComputeTaskType::Inference,
            model_id: 1,
            input_data: 0x1000,
            input_size: 1024,
            output_data: 0x2000,
            output_size: 2048,
            priority: 1,
            sync_obj: 0,
            input_addr: 0x1000,
            output_addr: 0x2000,
        };

        assert_eq!(task.task_type, ComputeTaskType::Inference);
        assert_eq!(task.model_id, 1);
        assert_eq!(task.input_size, 1024);
        assert_eq!(task.output_size, 2048);
    }

    #[test]
    fn test_npu_hal_device_new() {
        let device = NpuHalDevice::new();
        assert_eq!(device.info.npu_id, 0);
        assert_eq!(device.num_models, 0);
    }
}
