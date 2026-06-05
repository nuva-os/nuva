/*
 * Nuva OS - Hal - Npu - Traits
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
 * NPU (Neural Processing Unit) Hardware Abstraction Layer
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides hardware abstraction for neural processing units,
 * enabling AI/ML acceleration across different hardware platforms.
 */

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::sync::Arc;

/// NPU HAL trait - Hardware abstraction for neural processing units
/// Implementations support:
/// - ARM Ethos-N NPU
/// - Qualcomm Hexagon NPU
/// - Intel NPU
/// - Google Edge TPU
/// - Custom NPU implementations
pub trait NpuHal: Send + Sync {
    /// Initialize NPU
    fn initialize(&mut self) -> Result<(), NpuError>;

    /// Load model into NPU
    /// @param model: Model data (ONNX, TFLite, etc.)
    /// @return: Model ID for future reference
    fn load_model(&mut self, model: &ModelData) -> Result<ModelId, NpuError>;

    /// Unload model from NPU
    /// @param id: Model ID
    fn unload_model(&mut self, id: ModelId) -> Result<(), NpuError>;

    /// Create input/output buffer
    /// @param size: Buffer size in bytes
    /// @return: Buffer ID
    fn create_buffer(&mut self, size: usize) -> Result<BufferId, NpuError>;

    /// Destroy buffer
    /// @param id: Buffer ID
    fn destroy_buffer(&mut self, id: BufferId) -> Result<(), NpuError>;

    /// Write data to buffer
    /// @param id: Buffer ID
    /// @param data: Data to write
    fn write_buffer(&mut self, id: BufferId, data: &[u8]) -> Result<(), NpuError>;

    /// Read data from buffer
    /// @param id: Buffer ID
    /// @return: Buffer data
    fn read_buffer(&mut self, id: BufferId) -> Result<Vec<u8>, NpuError>;

    /// Execute inference
    /// @param request: Inference request
    /// @return: Inference result
    fn execute(&mut self, request: InferenceRequest) -> Result<InferenceResult, NpuError>;

    /// Execute asynchronous inference
    /// @param request: Inference request
    /// @return: Handle for async operation
    fn execute_async(&mut self, request: InferenceRequest) -> Result<InferenceHandle, NpuError>;

    /// Wait for async inference to complete
    /// @param handle: Async handle
    /// @return: Inference result
    fn wait(&mut self, handle: InferenceHandle) -> Result<InferenceResult, NpuError>;

    /// Get NPU capabilities
    fn capabilities(&self) -> NpuCapabilities;

    /// Get NPU statistics
    fn stats(&self) -> NpuStats;

    /// Shutdown NPU
    fn shutdown(&mut self) -> Result<(), NpuError>;

    /// Get NPU name
    fn name(&self) -> &str;
}

/// Model data
#[derive(Debug, Clone)]
pub struct ModelData {
    /// Model format
    pub format: ModelFormat,

    /// Model data
    pub data: Vec<u8>,

    /// Model name
    pub name: String,
}

/// Model format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFormat {
    /// ONNX format
    Onnx,

    /// TensorFlow Lite format
    TFLite,

    /// Custom format
    Custom,
}

/// Model ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelId(pub u64);

/// Buffer ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(pub u64);

/// Inference request
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// Model ID
    pub model_id: ModelId,

    /// Input buffer IDs
    pub input_buffers: Vec<BufferId>,

    /// Output buffer IDs
    pub output_buffers: Vec<BufferId>,

    /// Priority (0 = highest)
    pub priority: u32,

    /// Timeout in milliseconds (0 = no timeout)
    pub timeout_ms: u32,
}

/// Inference result
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Output buffer IDs
    pub output_buffers: Vec<BufferId>,

    /// Inference time in microseconds
    pub inference_time_us: u64,

    /// Was successful?
    pub success: bool,
}

/// Asynchronous inference handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InferenceHandle(pub u64);

/// NPU capabilities
#[derive(Debug, Clone)]
pub struct NpuCapabilities {
    /// Maximum model size
    pub max_model_size: usize,

    /// Maximum number of models
    pub max_models: usize,

    /// Maximum buffer size
    pub max_buffer_size: usize,

    /// Maximum number of buffers
    pub max_buffers: usize,

    /// Supported model formats
    pub supported_formats: Vec<ModelFormat>,

    /// Supports async execution
    pub async_execution: bool,

    /// Supports quantization
    pub quantization: bool,

    /// Number of NPU cores
    pub num_cores: u32,

    /// NPU frequency in MHz
    pub frequency_mhz: u32,

    /// Total NPU memory in bytes
    pub total_memory: usize,
}

/// NPU statistics
#[derive(Debug, Clone)]
pub struct NpuStats {
    /// Total inferences
    pub total_inferences: u64,

    /// Successful inferences
    pub successful_inferences: u64,

    /// Failed inferences
    pub failed_inferences: u64,

    /// Total inference time in microseconds
    pub total_inference_time_us: u64,

    /// Average inference time in microseconds
    pub avg_inference_time_us: u64,

    /// Current memory usage in bytes
    pub memory_usage: usize,

    /// Number of loaded models
    pub loaded_models: usize,

    /// NPU utilization (0-100%)
    pub utilization: u8,
}

/// NPU error type
#[derive(Debug, Clone)]
pub enum NpuError {
    /// NPU not initialized
    NotInitialized,

    /// NPU already initialized
    AlreadyInitialized,

    /// Model not found
    ModelNotFound(ModelId),

    /// Buffer not found
    BufferNotFound(BufferId),

    /// Out of memory
    OutOfMemory,

    /// Invalid model
    InvalidModel(String),

    /// Invalid buffer
    InvalidBuffer,

    /// Inference failed
    InferenceFailed(String),

    /// Hardware error
    HardwareError(String),

    /// Timeout
    Timeout,

    /// Not supported
    NotSupported,

    /// Invalid request
    InvalidRequest,
}

impl fmt::Display for NpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "NPU not initialized"),
            Self::AlreadyInitialized => write!(f, "NPU already initialized"),
            Self::ModelNotFound(id) => write!(f, "Model not found: {:?}", id),
            Self::BufferNotFound(id) => write!(f, "Buffer not found: {:?}", id),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::InvalidModel(msg) => write!(f, "Invalid model: {}", msg),
            Self::InvalidBuffer => write!(f, "Invalid buffer"),
            Self::InferenceFailed(msg) => write!(f, "Inference failed: {}", msg),
            Self::HardwareError(msg) => write!(f, "Hardware error: {}", msg),
            Self::Timeout => write!(f, "Timeout"),
            Self::NotSupported => write!(f, "Not supported"),
            Self::InvalidRequest => write!(f, "Invalid request"),
        }
    }
}
