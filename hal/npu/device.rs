/*
 * Nuva OS - HAL - NPU Device Abstraction
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

//! NPU Device Abstraction
/*!*/
//! Complete NPU HAL interface with model loading, tensor management,
//! and execution capabilities.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, AtomicPtr, Ordering};
use crate::{pr_info};

/// NPU configuration
pub mod npu_config {
    /// Maximum models per NPU
    pub const MAX_MODELS: usize = 32;

    /// Maximum tensors per model
    pub const MAX_TENSORS: usize = 256;

    /// Maximum memory per NPU (4GB)
    pub const MAX_MEMORY: u64 = 4 * 1024 * 1024 * 1024;

    /// Default timeout (1 second)
    pub const DEFAULT_TIMEOUT_MS: u64 = 1000;

    /// Maximum queue depth
    pub const MAX_QUEUE_DEPTH: usize = 64;
}

/// NPU device ID
pub type NpuId = u32;

/// Model handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelHandle(pub u64);

/// Tensor handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TensorHandle(pub u64);

/// Buffer handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u64);

/// Event handle for synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventHandle(pub u64);

/// Data types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    /// 32-bit float
    Float32 = 0,
    /// 16-bit float
    Float16 = 1,
    /// 64-bit float
    Float64 = 2,
    /// 8-bit integer
    Int8 = 3,
    /// 16-bit integer
    Int16 = 4,
    /// 32-bit integer
    Int32 = 5,
    /// 64-bit integer
    Int64 = 6,
    /// 8-bit unsigned integer
    UInt8 = 7,
    /// 16-bit unsigned integer
    UInt16 = 8,
    /// 32-bit unsigned integer
    UInt32 = 9,
    /// 64-bit unsigned integer
    UInt64 = 10,
    /// Boolean
    Bool = 11,
    /// Brain float 16
    BFloat16 = 12,
    /// Complex 64
    Complex64 = 13,
    /// Complex 128
    Complex128 = 14,
}

impl DataType {
    /// Get size in bytes
    pub fn size(&self) -> usize {
        match self {
            DataType::Float32 => 4,
            DataType::Float16 => 2,
            DataType::Float64 => 8,
            DataType::Int8 => 1,
            DataType::Int16 => 2,
            DataType::Int32 => 4,
            DataType::Int64 => 8,
            DataType::UInt8 => 1,
            DataType::UInt16 => 2,
            DataType::UInt32 => 4,
            DataType::UInt64 => 8,
            DataType::Bool => 1,
            DataType::BFloat16 => 2,
            DataType::Complex64 => 8,
            DataType::Complex128 => 16,
        }
    }
}

/// Tensor shape
#[derive(Debug, Clone)]
pub struct TensorShape {
    pub dims: [u64; 8],
    pub ndim: usize,
}

impl TensorShape {
    pub fn new(dims: &[u64]) -> Self {
        let mut shape = Self {
            dims: [0; 8],
            ndim: dims.len().min(8),
        };
        for i in 0..shape.ndim {
            shape.dims[i] = dims[i];
        }
        shape
    }

    pub fn elements(&self) -> u64 {
        let mut count = 1u64;
        for i in 0..self.ndim {
            count *= self.dims[i];
        }
        count
    }

    pub fn size_bytes(&self, dtype: DataType) -> u64 {
        self.elements() * dtype.size() as u64
    }
}

/// Tensor descriptor
#[derive(Debug, Clone)]
pub struct TensorDesc {
    pub name: &'static str,
    pub shape: TensorShape,
    pub dtype: DataType,
    pub quant_param: Option<QuantParam>,
}

/// Quantization parameters
#[derive(Debug, Clone, Copy)]
pub struct QuantParam {
    pub scale: f32,
    pub zero_point: i32,
}

/// NPU device information
#[derive(Debug, Clone)]
pub struct NpuInfo {
    pub npu_id: NpuId,
    pub name: &'static str,
    pub vendor: NpuVendor,
    pub version: &'static str,
    pub num_cores: u32,
    pub frequency_mhz: u32,
    pub memory_size: u64,
    pub memory_bandwidth_gbps: u32,
    pub supported_dtypes: u32,
    pub max_batch_size: u32,
    pub features: NpuFeatures,
    /// NPU state
    pub state: u32,
    /// Current frequency
    pub current_freq: u32,
    /// Minimum frequency
    pub min_freq: u32,
    /// Maximum frequency
    pub max_freq: u32,
    /// Utilization percentage
    pub utilization: u32,
    /// Temperature in millidegrees
    pub temperature: u32,
}

/// NPU vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpuVendor {
    Huawei = 0,
    Qualcomm = 1,
    Intel = 2,
    Google = 3,
    Nvidia = 4,
    AMD = 5,
    Apple = 6,
    Custom = 7,
}

/// NPU features
#[derive(Debug, Clone, Copy)]
pub struct NpuFeatures {
    pub async_execution: bool,
    pub dynamic_shapes: bool,
    pub quantization: bool,
    pub sparse: bool,
    pub mixed_precision: bool,
    pub onnx: bool,
    pub tflite: bool,
    pub pytorch: bool,
}

impl NpuFeatures {
    pub fn empty() -> Self {
        NpuFeatures {
            async_execution: false,
            dynamic_shapes: false,
            quantization: false,
            sparse: false,
            mixed_precision: false,
            onnx: false,
            tflite: false,
            pytorch: false,
        }
    }
}

/// NPU device trait
pub trait NpuDevice: Send + Sync {
    /// Get device info
    fn info(&self) -> &NpuInfo;

    /// Initialize device
    fn initialize(&mut self) -> Result<(), NpuError>;

    /// Shutdown device
    fn shutdown(&mut self) -> Result<(), NpuError>;

    /// Load model
    fn load_model(&mut self, model: &[u8], format: ModelFormat) -> Result<ModelHandle, NpuError>;

    /// Unload model
    fn unload_model(&mut self, handle: ModelHandle) -> Result<(), NpuError>;

    /// Get model info
    fn model_info(&self, handle: ModelHandle) -> Option<&ModelInfo>;

    /// Create tensor
    fn create_tensor(
        &mut self,
        shape: &TensorShape,
        dtype: DataType,
    ) -> Result<TensorHandle, NpuError>;

    /// Create tensor with data
    fn create_tensor_with_data(
        &mut self,
        shape: &TensorShape,
        dtype: DataType,
        data: &[u8],
    ) -> Result<TensorHandle, NpuError>;

    /// Destroy tensor
    fn destroy_tensor(&mut self, handle: TensorHandle) -> Result<(), NpuError>;

    /// Get tensor data
    fn tensor_data(&self, handle: TensorHandle) -> Option<&[u8]>;

    /// Copy tensor data
    fn copy_tensor_data(&self, handle: TensorHandle, dst: &mut [u8]) -> Result<(), NpuError>;

    /// Create buffer
    fn create_buffer(&mut self, size: usize) -> Result<BufferHandle, NpuError>;

    /// Destroy buffer
    fn destroy_buffer(&mut self, handle: BufferHandle) -> Result<(), NpuError>;

    /// Write buffer
    fn write_buffer(&mut self, handle: BufferHandle, data: &[u8]) -> Result<(), NpuError>;

    /// Read buffer
    fn read_buffer(&self, handle: BufferHandle, dst: &mut [u8]) -> Result<(), NpuError>;

    /// Execute inference (synchronous)
    fn execute(
        &mut self,
        model: ModelHandle,
        inputs: &[TensorHandle],
        outputs: &mut [TensorHandle],
    ) -> Result<InferenceResult, NpuError>;

    /// Execute inference (asynchronous)
    fn execute_async(
        &mut self,
        model: ModelHandle,
        inputs: &[TensorHandle],
        outputs: &mut [TensorHandle],
    ) -> Result<EventHandle, NpuError>;

    /// Wait for event
    fn wait_event(&self, event: EventHandle, timeout_ms: u64) -> Result<(), NpuError>;

    /// Get statistics
    fn stats(&self) -> NpuStats;

    /// Set power mode
    fn set_power_mode(&mut self, mode: PowerMode) -> Result<(), NpuError>;

    /// Get power mode
    fn power_mode(&self) -> PowerMode;
}

/// Model format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFormat {
    Onnx = 0,
    TFLite = 1,
    Caffe = 2,
    PyTorch = 3,
    Custom = 4,
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub handle: ModelHandle,
    pub name: &'static str,
    pub format: ModelFormat,
    pub size: u64,
    pub inputs: Vec<TensorDesc>,
    pub outputs: Vec<TensorDesc>,
    pub metadata: ModelMetadata,
}

/// Model metadata
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub author: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub license: &'static str,
}

/// Inference result
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub outputs: Vec<TensorHandle>,
    pub inference_time_us: u64,
    pub preprocess_time_us: u64,
    pub postprocess_time_us: u64,
    pub success: bool,
}

/// NPU statistics
#[derive(Debug, Clone)]
pub struct NpuStats {
    pub total_inferences: u64,
    pub successful_inferences: u64,
    pub failed_inferences: u64,
    pub total_time_us: u64,
    pub avg_time_us: u64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub utilization: u32,
    pub temperature: i32,
    pub power_mw: u32,
}

/// Power mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    Performance = 0,
    Balanced = 1,
    PowerSave = 2,
    Custom = 3,
}

/// NPU error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpuError {
    NotInitialized,
    AlreadyInitialized,
    InvalidParam,
    InvalidModel,
    InvalidTensor,
    InvalidBuffer,
    InvalidHandle,
    OutOfMemory,
    OutOfResources,
    NotSupported,
    DeviceError,
    Timeout,
    ShapeMismatch,
    TypeMismatch,
    QueueFull,
    InternalError,
}

impl core::fmt::Display for NpuError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// NPU manager
pub struct NpuManager {
    /// Devices
    devices: [Option<*mut dyn NpuDevice>; 8],

    /// Number of devices
    num_devices: AtomicU32,

    /// Initialized
    initialized: AtomicBool,
}

impl NpuManager {
    pub const fn new() -> Self {
        Self {
            devices: [None; 8],
            num_devices: AtomicU32::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize NPU manager
    pub fn init(&mut self) -> Result<(), NpuError> {
        if self.initialized.load(Ordering::Acquire) {
            return Err(NpuError::AlreadyInitialized);
        }

        log_info!("NPU Manager initialized");
        self.initialized.store(true, Ordering::Release);
        Ok(())
    }

    /// Register device
    pub fn register_device(&mut self, device: *mut dyn NpuDevice) -> Result<NpuId, NpuError> {
        let id = self.num_devices.fetch_add(1, Ordering::AcqRel);
        if id as usize >= 8 {
            self.num_devices.fetch_sub(1, Ordering::AcqRel);
            return Err(NpuError::OutOfResources);
        }

        self.devices[id as usize] = Some(device);
        Ok(id)
    }

    /// Get device
    pub fn get_device(&self, id: NpuId) -> Option<&dyn NpuDevice> {
        if id as usize >= 8 {
            return None;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        self.devices[id as usize].map(|ptr| unsafe { &*ptr })
    }

    /// Get device mutable
    pub fn get_device_mut(&mut self, id: NpuId) -> Option<&mut dyn NpuDevice> {
        if id as usize >= 8 {
            return None;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        self.devices[id as usize].map(|ptr| unsafe { &mut *ptr })
    }

    /// Get number of devices
    pub fn num_devices(&self) -> u32 {
        self.num_devices.load(Ordering::Acquire)
    }
}

/// Global NPU manager
static NPU_MANAGER: core::sync::OnceLock<NpuManager> = core::sync::OnceLock::new();

/// Get NPU manager
pub fn get_npu_manager() -> &'static mut NpuManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut NPU_MANAGER }
}

/// Initialize NPU subsystem
pub fn init_npu() -> Result<(), NpuError> {
    get_npu_manager().init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_size() {
        assert_eq!(DataType::Float32.size(), 4);
        assert_eq!(DataType::Float16.size(), 2);
        assert_eq!(DataType::Int8.size(), 1);
        assert_eq!(DataType::Int64.size(), 8);
    }

    #[test]
    fn test_tensor_shape() {
        let shape = TensorShape::new(&[1, 3, 224, 224]);
        assert_eq!(shape.ndim, 4);
        assert_eq!(shape.elements(), 1 * 3 * 224 * 224);
    }

    #[test]
    fn test_tensor_shape_size() {
        let shape = TensorShape::new(&[1, 3, 224, 224]);
        let size = shape.size_bytes(DataType::Float32);
        assert_eq!(size, 1 * 3 * 224 * 224 * 4);
    }

    #[test]
    fn test_model_handle() {
        let h1 = ModelHandle(1);
        let h2 = ModelHandle(2);
        assert_ne!(h1, h2);
        assert_eq!(h1, ModelHandle(1));
    }

    #[test]
    fn test_npu_manager() {
        let manager = NpuManager::new();
        assert_eq!(manager.num_devices(), 0);
    }
}
