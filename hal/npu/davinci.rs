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



use super::{NpuInfo, NpuState, ModelFormat, ComputeTask, NpuHalOps};
use super::device::{NpuVendor, NpuFeatures, NpuStats};
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use crate::{pr_debug, pr_info, pr_warn};

// ============================================================================
// Da Vinci NPU Register Definitions
// ============================================================================

/// NPU control register base address
const NPU_CTRL_BASE: u64 = 0xF800_0000;

/// NPU memory base address
const NPU_MEM_BASE: u64 = 0xF810_0000;

/// NPU task queue base address
const NPU_TASKQ_BASE: u64 = 0xF820_0000;

// Control register offsets
const NPU_CTRL_ENABLE: u64 = 0x0000;       // NPU enable
const NPU_CTRL_FREQ: u64 = 0x0004;         // Frequency setting
const NPU_CTRL_STATE: u64 = 0x0008;        // State register
const NPU_CTRL_POWER: u64 = 0x000C;        // Power control
const NPU_CTRL_TEMP: u64 = 0x0010;         // Temperature reading
const NPU_CTRL_UTIL: u64 = 0x0014;         // Utilization
const NPU_CTRL_MODEL_COUNT: u64 = 0x0018;  // Number of loaded models

// Model management register offsets
const NPU_MODEL_LOAD: u64 = 0x0020;        // Model loading control
const NPU_MODEL_UNLOAD: u64 = 0x0024;      // Model unloading control
const NPU_MODEL_ADDR: u64 = 0x0028;        // Model address
const NPU_MODEL_SIZE: u64 = 0x002C;        // Model size
const NPU_MODEL_ID: u64 = 0x0030;          // Model ID

// Task queue register offsets
const NPU_TASKQ_HEAD: u64 = 0x0000;        // Queue head pointer
const NPU_TASKQ_TAIL: u64 = 0x0004;        // Queue tail pointer
const NPU_TASKQ_DOORBELL: u64 = 0x0008;    // Doorbell register
const NPU_TASKQ_STATUS: u64 = 0x000C;      // Queue state

// NPU state bits
const NPU_STATE_BUSY: u32 = 0x0001;
const NPU_STATE_ERROR: u32 = 0x0002;
const NPU_STATE_IDLE: u32 = 0x0004;

// Task queue size
const NPU_TASKQ_SIZE: u32 = 64;

// Maximum number of models
const NPU_MAX_MODELS: u32 = 16;

// Delay constants
const NPU_CMD_DELAY_US: u32 = 10;
const NPU_POWER_DELAY_US: u32 = 1000;
const NPU_MODEL_LOAD_DELAY_US: u32 = 10000;

// Da Vinci inference registers
const NPU_CTRL_INFERENCE_CTRL: u64 = 0x0040;  // Inference control
const NPU_CTRL_INFERENCE_STATUS: u64 = 0x0044; // Inference status
const NPU_CTRL_INPUT_BASE: u64 = 0x0048;      // Input buffer base
const NPU_CTRL_OUTPUT_BASE: u64 = 0x004C;     // Output buffer base
const NPU_CTRL_WEIGHT_BASE: u64 = 0x0050;     // Weight buffer base
const NPU_CTRL_BATCH_SIZE: u64 = 0x0054;      // Batch size

// Maximum inference queue depth
const NPU_MAX_INFERENCE_QUEUE: usize = 64;

// Inference task states
const INFERENCE_STATE_PENDING: u32 = 0;
const INFERENCE_STATE_RUNNING: u32 = 1;
const INFERENCE_STATE_COMPLETE: u32 = 2;
const INFERENCE_STATE_ERROR: u32 = 3;

/// Da Vinci NPU configuration


pub struct DaVinciConfig {
    /// NPU model
    pub model: &'static str,
    /// Number of compute units
    pub num_cores: u32,
    /// Minimum frequency
    pub min_freq: u64,
    /// Maximum frequency
    pub max_freq: u64,
    /// Memory size
    pub memory_size: u64,
}

impl DaVinciConfig {
    pub const fn new() -> Self {
        DaVinciConfig {
            model: "Da Vinci C310",
            num_cores: 1,
            min_freq: 500_000_000,    // 500 MHz
            max_freq: 1_000_000_000,  // 1 GHz
            memory_size: 8 * 1024 * 1024,  // 8MB
        }
    }
}

/// Inference task descriptor
#[derive(Debug, Clone, Copy)]
pub struct InferenceTask {
    /// Model ID
    pub model_id: u32,
    /// Input buffer address
    pub input_addr: u64,
    /// Output buffer address
    pub output_addr: u64,
    /// Task state
    pub state: u32,
    /// Submission timestamp
    pub submit_time: u64,
    /// Completion timestamp
    pub complete_time: u64,
}

impl InferenceTask {
    /// Create a new inference task
    pub const fn new() -> Self {
        InferenceTask {
            model_id: 0,
            input_addr: 0,
            output_addr: 0,
            state: INFERENCE_STATE_PENDING,
            submit_time: 0,
            complete_time: 0,
        }
    }
}

/// Performance statistics
pub struct DaVinciPerfStats {
    /// Total inferences
    pub total_inferences: AtomicU64,
    /// Successful inferences
    pub successful_inferences: AtomicU64,
    /// Failed inferences
    pub failed_inferences: AtomicU64,
    /// Total inference time (us)
    pub total_inference_time_us: AtomicU64,
    /// Peak inference time (us)
    pub peak_inference_time_us: AtomicU64,
    /// Total bytes of models loaded
    pub total_model_bytes: AtomicU64,
}

impl DaVinciPerfStats {
    /// Create zeroed stats
    pub const fn new() -> Self {
        DaVinciPerfStats {
            total_inferences: AtomicU64::new(0),
            successful_inferences: AtomicU64::new(0),
            failed_inferences: AtomicU64::new(0),
            total_inference_time_us: AtomicU64::new(0),
            peak_inference_time_us: AtomicU64::new(0),
            total_model_bytes: AtomicU64::new(0),
        }
    }

    /// Record a completed inference
    pub fn record_inference(&self, time_us: u64, success: bool) {
        self.total_inferences.fetch_add(1, Ordering::AcqRel);
        if success {
            self.successful_inferences.fetch_add(1, Ordering::AcqRel);
        } else {
            self.failed_inferences.fetch_add(1, Ordering::AcqRel);
        }
        self.total_inference_time_us.fetch_add(time_us, Ordering::AcqRel);
        let peak = self.peak_inference_time_us.load(Ordering::Acquire);
        if time_us > peak {
            self.peak_inference_time_us.store(time_us, Ordering::Release);
        }
    }

    /// Get average inference time
    pub fn avg_inference_time_us(&self) -> u64 {
        let total = self.total_inferences.load(Ordering::Acquire);
        if total == 0 {
            return 0;
        }
        self.total_inference_time_us.load(Ordering::Acquire) / total
    }
}

/// Da Vinci NPU HAL
pub struct DaVinciNpuHal {
    config: DaVinciConfig,
    current_freq: u64,
    state: NpuState,
    loaded_models: u32,
    /// Task queue tail pointer
    taskq_tail: AtomicU32,
    /// Synchronization object counter
    sync_counter: AtomicU64,
    /// Model memory offset
    model_mem_offset: AtomicU64,
    /// Initialization flag
    initialized: AtomicBool,
    /// Inference task queue
    inference_queue: [InferenceTask; NPU_MAX_INFERENCE_QUEUE],
    /// Inference queue head
    inference_head: AtomicU32,
    /// Inference queue tail
    inference_tail: AtomicU32,
    /// Performance statistics
    perf_stats: DaVinciPerfStats,
}

impl DaVinciNpuHal {
    pub const fn new() -> Self {
        DaVinciNpuHal {
            config: DaVinciConfig::new(),
            current_freq: 0,
            state: NpuState::Idle,
            loaded_models: 0,
            taskq_tail: AtomicU32::new(0),
            sync_counter: AtomicU64::new(0),
            model_mem_offset: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
            inference_queue: [InferenceTask::new(); NPU_MAX_INFERENCE_QUEUE],
            inference_head: AtomicU32::new(0),
            inference_tail: AtomicU32::new(0),
            perf_stats: DaVinciPerfStats::new(),
        }
    }

    // ========================================================================
    // Register operations
    // ========================================================================

    /// Read register
    #[inline]
    unsafe fn read_reg(addr: u64) -> u32 {
        read_volatile(addr as *const u32)
    }

    /// Write register
    #[inline]
    unsafe fn write_reg(addr: u64, value: u32) {
        write_volatile(addr as *mut u32, value);
    }

    /// Microsecond delay
    #[inline]
    fn udelay(us: u32) {
        let cycles = us * 100;
        let mut _dummy: u32 = 0;
        for _ in 0..cycles {
            core::hint::spin_loop();
            _dummy = _dummy.wrapping_add(1);
        }
    }

    /// Wait for NPU idle
    fn wait_npu_idle(&self) -> bool {
        let mut timeout = 1_000_000; // 1s
        while timeout > 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let state = Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_STATE);
                if (state & NPU_STATE_BUSY) == 0 {
                    return true;
                }
            }
            Self::udelay(1);
            timeout -= 1;
        }
        false
    }

    // ========================================================================
    // HAL interface implementation
    // ========================================================================

    /// Initialize
    pub fn init(&mut self) -> i32 {
        log_info!("Da Vinci NPU HAL initialized");
        log_info!("  Model: {}", self.config.model);
        log_info!("  Cores: {}", self.config.num_cores);
        log_info!("  Frequency: {}-{} MHz",
            self.config.min_freq / 1_000_000,
            self.config.max_freq / 1_000_000);
        log_info!("  Memory: {} MB", self.config.memory_size / (1024 * 1024));

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Enable NPU
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_ENABLE, 1);
            Self::udelay(NPU_POWER_DELAY_US);

            // Set initial frequency
            let init_freq = (self.config.max_freq / 1000) as u32;
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_FREQ, init_freq);

            // Initialize task queue
            Self::write_reg(NPU_TASKQ_BASE + NPU_TASKQ_HEAD, 0);
            Self::write_reg(NPU_TASKQ_BASE + NPU_TASKQ_TAIL, 0);

            // Clear model count
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_MODEL_COUNT, 0);
        }

        self.current_freq = self.config.max_freq;
        self.state = NpuState::Idle;

        0
    }

    /// Get NPU info
    pub fn get_npu_info(&self) -> NpuInfo {
        // Read actual values from hardware
        // SAFETY: unsafe block required for low-level memory or hardware access
        let (utilization, temperature) = unsafe {
            let util = Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_UTIL);
            let temp = Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_TEMP);
            (util, temp as u32)
        };

        NpuInfo {
            npu_id: 0,
            name: self.config.model,
            vendor: NpuVendor::Huawei,
            version: "",
            frequency_mhz: self.current_freq as u32,
            state: self.state as u32,
            current_freq: self.current_freq as u32,
            min_freq: self.config.min_freq as u32,
            max_freq: self.config.max_freq as u32,
            num_cores: self.config.num_cores,
            memory_size: self.config.memory_size,
            memory_bandwidth_gbps: 0,
            supported_dtypes: 0,
            max_batch_size: 0,
            features: NpuFeatures::empty(),
            utilization,
            temperature,
        }
    }

    /// Load model
    pub fn load_model(&mut self, data: &[u8], format: ModelFormat) -> i32 {
        if self.state == NpuState::Suspended {
            return -1;
        }

        if self.loaded_models >= NPU_MAX_MODELS {
            log_warn!("NPU: Maximum models reached");
            return -2;
        }

        log_debug!("NPU: Loading model (format: {:?}, size: {} bytes)", format, data.len());

        // Check memory space
        let model_size = data.len() as u64;
        let current_offset = self.model_mem_offset.load(Ordering::Acquire);
        if current_offset + model_size > self.config.memory_size {
            log_warn!("NPU: Not enough memory for model");
            return -3;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Write model data to NPU memory
            let model_addr = NPU_MEM_BASE + current_offset;
            for (i, &byte) in data.iter().enumerate() {
                write_volatile((model_addr + i as u64) as *mut u8, byte);
            }

            // Set model loading parameters
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_ADDR, current_offset as u32);
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_SIZE, model_size as u32);
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_ID, self.loaded_models);

            // Trigger model loading
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_LOAD, 1);

            // Wait for loading to complete
            Self::udelay(NPU_MODEL_LOAD_DELAY_US);

            // Check state
            let state = Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_STATE);
            if (state & NPU_STATE_ERROR) != 0 {
                log_warn!("NPU: Model load failed");
                return -4;
            }

            // Update model count
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_MODEL_COUNT, self.loaded_models + 1);
        }

        // Update memory offset
        self.model_mem_offset.fetch_add(model_size, Ordering::Release);

        let model_id = self.loaded_models;
        self.loaded_models += 1;

        log_debug!("NPU: Model {} loaded successfully", model_id);
        model_id as i32
    }

    /// Unload model
    pub fn unload_model(&mut self, model_id: u32) -> i32 {
        if model_id >= self.loaded_models {
            return -1;
        }

        log_debug!("NPU: Unloading model {}", model_id);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Set model unloading parameters
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_ID, model_id);

            // Trigger model unloading
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_UNLOAD, 1);

            // Wait for unloading to complete
            Self::udelay(NPU_CMD_DELAY_US);

            // Update model count
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_MODEL_COUNT, self.loaded_models - 1);
        }

        self.loaded_models -= 1;
        0
    }

    /// Submit compute task
    pub fn submit_task(&mut self, task: &ComputeTask) -> i32 {
        if self.state == NpuState::Suspended {
            return -1;
        }

        log_debug!("NPU: Submit task type {:?}", task.task_type);

        // Get queue position
        let tail = self.taskq_tail.load(Ordering::Acquire);
        let next_tail = (tail + 1) % NPU_TASKQ_SIZE;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Check if queue is full
            let head = Self::read_reg(NPU_TASKQ_BASE + NPU_TASKQ_HEAD);
            if next_tail == head {
                log_warn!("NPU: Task queue full");
                return -2;
            }

            // Write task to queue
            // Task format: [task_type | model_id | input_addr | output_addr | sync_obj]
            let taskq_entry_addr = NPU_TASKQ_BASE + 0x1000 + (tail as u64 * 20);
            Self::write_reg(taskq_entry_addr, task.task_type as u32);
            Self::write_reg(taskq_entry_addr + 4, task.model_id);
            Self::write_reg(taskq_entry_addr + 8, task.input_addr as u32);
            Self::write_reg(taskq_entry_addr + 12, task.output_addr as u32);

            // Generate synchronization object
            let sync_obj = self.sync_counter.fetch_add(1, Ordering::AcqRel) + 1;
            Self::write_reg(taskq_entry_addr + 16, sync_obj as u32);

            // Update tail pointer
            Self::write_reg(NPU_TASKQ_BASE + NPU_TASKQ_TAIL, next_tail);

            // Ring doorbell to notify NPU
            Self::write_reg(NPU_TASKQ_BASE + NPU_TASKQ_DOORBELL, 1);
        }

        self.state = NpuState::Running;
        0
    }

    /// Wait for task completion
    pub fn wait_task(&mut self, _sync_obj: u64, timeout_us: u64) -> i32 {
        let mut remaining = timeout_us;

        while remaining > 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let state = Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_STATE);
                if (state & NPU_STATE_BUSY) == 0 {
                    self.state = NpuState::Idle;
                    return 0;
                }
            }
            Self::udelay(1);
            remaining -= 1;
        }

        log_warn!("NPU: Task timeout");
        -1
    }

    /// Set frequency
    pub fn set_frequency(&mut self, freq: u64) -> i32 {
        if freq < self.config.min_freq || freq > self.config.max_freq {
            return -1;
        }

        log_debug!("NPU: Setting frequency to {} MHz", freq / 1_000_000);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_FREQ, (freq / 1000) as u32);
            Self::udelay(NPU_CMD_DELAY_US);
        }

        self.current_freq = freq;
        0
    }

    /// Get frequency
    pub fn get_frequency(&self) -> u64 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let freq_khz = Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_FREQ);
            (freq_khz as u64) * 1000
        }
    }

    /// Suspend
    pub fn suspend(&mut self) -> i32 {
        log_info!("NPU: Suspending");

        // Wait for all tasks to complete
        if !self.wait_npu_idle() {
            log_warn!("NPU: Timeout waiting for idle before suspend");
            return -1;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Disable NPU
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_ENABLE, 0);

            // Power off
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_POWER, 0);
            Self::udelay(NPU_POWER_DELAY_US);
        }

        self.state = NpuState::Suspended;
        0
    }

    /// Resume
    pub fn resume(&mut self) -> i32 {
        log_info!("NPU: Resuming");

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Power on
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_POWER, 1);
            Self::udelay(NPU_POWER_DELAY_US);

            // Enable NPU
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_ENABLE, 1);
            Self::udelay(NPU_POWER_DELAY_US);

            // Restore frequency
            let freq = (self.current_freq / 1000) as u32;
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_FREQ, freq);
        }

        self.state = NpuState::Idle;
        0
    }

    /// Get temperature
    pub fn get_temperature(&self) -> i32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_TEMP) as i32
        }
    }

    /// Get utilization
    pub fn get_utilization(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_UTIL)
        }
    }

    // ========================================================================
    // AI Model Loading and Inference
    // ========================================================================

    /// Load model weights into NPU memory
    pub fn load_model_weights(&mut self, model_id: u32, weights: &[u8]) -> Result<u64, i32> {
        if self.state == NpuState::Suspended {
            return Err(-1);
        }

        if model_id >= self.loaded_models {
            return Err(-2);
        }

        let weight_size = weights.len() as u64;
        let weight_offset = self.model_mem_offset.load(Ordering::Acquire);

        if weight_offset + weight_size > self.config.memory_size {
            log_warn!("NPU: Not enough memory for model weights");
            return Err(-3);
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let weight_addr = NPU_MEM_BASE + weight_offset;
            for (i, &byte) in weights.iter().enumerate() {
                write_volatile((weight_addr + i as u64) as *mut u8, byte);
            }

            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_WEIGHT_BASE, weight_offset as u32);
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_ADDR, weight_offset as u32);
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_SIZE, weight_size as u32);
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_ID, model_id);
        }

        self.model_mem_offset.fetch_add(weight_size, Ordering::Release);
        self.perf_stats.total_model_bytes.fetch_add(weight_size, Ordering::AcqRel);

        log_debug!("NPU: Model {} weights loaded ({} bytes)", model_id, weight_size);
        Ok(weight_offset)
    }

    /// Configure model input/output buffers
    pub fn configure_inference(
        &mut self,
        model_id: u32,
        input_addr: u64,
        output_addr: u64,
        batch_size: u32,
    ) -> Result<(), i32> {
        if model_id >= self.loaded_models {
            return Err(-1);
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            Self::write_reg(NPU_CTRL_BASE + NPU_MODEL_ID, model_id);
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_INPUT_BASE, input_addr as u32);
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_OUTPUT_BASE, output_addr as u32);
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_BATCH_SIZE, batch_size);
        }

        log_debug!("NPU: Model {} inference configured (batch={})", model_id, batch_size);
        Ok(())
    }

    /// Submit an inference task (asynchronous)
    pub fn submit_inference(
        &mut self,
        model_id: u32,
        input_addr: u64,
        output_addr: u64,
    ) -> Result<u64, i32> {
        if self.state == NpuState::Suspended {
            return Err(-1);
        }

        let tail = self.inference_tail.load(Ordering::Acquire);
        let next_tail = (tail + 1) % NPU_MAX_INFERENCE_QUEUE as u32;
        let head = self.inference_head.load(Ordering::Acquire);

        if next_tail == head {
            log_warn!("NPU: Inference queue full");
            return Err(-2);
        }

        let sync_obj = self.sync_counter.fetch_add(1, Ordering::AcqRel) + 1;

        self.inference_queue[tail as usize] = InferenceTask {
            model_id,
            input_addr,
            output_addr,
            state: INFERENCE_STATE_PENDING,
            submit_time: sync_obj,
            complete_time: 0,
        };

        self.inference_tail.store(next_tail, Ordering::Release);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            Self::write_reg(NPU_CTRL_BASE + NPU_CTRL_INFERENCE_CTRL, 1);
            Self::write_reg(NPU_TASKQ_BASE + NPU_TASKQ_DOORBELL, 1);
        }

        self.state = NpuState::Running;
        log_debug!("NPU: Inference submitted (sync_obj={})", sync_obj);
        Ok(sync_obj)
    }

    /// Poll for completed inference tasks
    pub fn poll_inference(&mut self) -> u32 {
        let mut completed = 0u32;
        let head = self.inference_head.load(Ordering::Acquire);
        let tail = self.inference_tail.load(Ordering::Acquire);

        let count = if tail >= head {
            tail - head
        } else {
            NPU_MAX_INFERENCE_QUEUE as u32 - head + tail
        };

        for i in 0..count {
            let idx = (head + i) % NPU_MAX_INFERENCE_QUEUE as u32;
            let task_state = self.inference_queue[idx as usize].state;

            if task_state == INFERENCE_STATE_COMPLETE || task_state == INFERENCE_STATE_ERROR {
                let success = task_state == INFERENCE_STATE_COMPLETE;
                let time_us = self.inference_queue[idx as usize].complete_time;
                self.perf_stats.record_inference(time_us, success);
                completed += 1;

                self.inference_head.fetch_add(1, Ordering::AcqRel);
            } else if task_state == INFERENCE_STATE_RUNNING {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let status = Self::read_reg(NPU_CTRL_BASE + NPU_CTRL_INFERENCE_STATUS);
                    if status == INFERENCE_STATE_COMPLETE {
                        self.inference_queue[idx as usize].state = INFERENCE_STATE_COMPLETE;
                        self.inference_queue[idx as usize].complete_time = 100;
                    } else if status == INFERENCE_STATE_ERROR {
                        self.inference_queue[idx as usize].state = INFERENCE_STATE_ERROR;
                    }
                }
            }
        }

        if completed > 0 {
            let new_head = self.inference_head.load(Ordering::Acquire);
            if new_head == self.inference_tail.load(Ordering::Acquire) {
                self.state = NpuState::Idle;
            }
        }

        completed
    }

    /// Get performance statistics
    pub fn get_perf_stats(&self) -> &DaVinciPerfStats {
        &self.perf_stats
    }

    /// Get NpuStats for device interface compatibility
    pub fn get_stats(&self) -> NpuStats {
        NpuStats {
            total_inferences: self.perf_stats.total_inferences.load(Ordering::Acquire),
            successful_inferences: self.perf_stats.successful_inferences.load(Ordering::Acquire),
            failed_inferences: self.perf_stats.failed_inferences.load(Ordering::Acquire),
            total_time_us: self.perf_stats.total_inference_time_us.load(Ordering::Acquire),
            avg_time_us: self.perf_stats.avg_inference_time_us(),
            memory_used: self.model_mem_offset.load(Ordering::Acquire),
            memory_total: self.config.memory_size,
            utilization: self.get_utilization(),
            temperature: self.get_temperature(),
            power_mw: 0,
        }
    }
}

/// Da Vinci NPU HAL operations (bridged to DaVinciNpuHal)
pub fn davinci_npu_ops() -> NpuHalOps {
    NpuHalOps {
        init: || {
            let hal = get_davinci_hal();
            hal.init() as i32
        },
        get_npu_info: || {
            let hal = get_davinci_hal();
            hal.get_npu_info()
        },
        load_model: |data, format| {
            let hal = get_davinci_hal();
            hal.load_model(data, format) as i32
        },
        unload_model: |model_id| {
            let hal = get_davinci_hal();
            hal.unload_model(model_id) as i32
        },
        submit_task: |task| {
            let hal = get_davinci_hal();
            hal.submit_task(task) as i32
        },
        wait_task: |sync_obj, timeout| {
            let hal = get_davinci_hal();
            hal.wait_task(sync_obj, timeout) as i32
        },
        set_frequency: |freq| {
            let hal = get_davinci_hal();
            hal.set_frequency(freq) as i32
        },
        get_frequency: || {
            let hal = get_davinci_hal();
            hal.get_frequency()
        },
        suspend: || {
            let hal = get_davinci_hal();
            hal.suspend() as i32
        },
        resume: || {
            let hal = get_davinci_hal();
            hal.resume() as i32
        },
    }
}

/// Static NPU ops instance (lazy-initialized via davinci_npu_ops())
pub static DAVINCI_NPU_OPS: NpuHalOps = NpuHalOps {
    init: || {
        let hal = get_davinci_hal();
        hal.init() as i32
    },
    get_npu_info: || {
        let hal = get_davinci_hal();
        hal.get_npu_info()
    },
    load_model: |data, format| {
        let hal = get_davinci_hal();
        hal.load_model(data, format) as i32
    },
    unload_model: |model_id| {
        let hal = get_davinci_hal();
        hal.unload_model(model_id) as i32
    },
    submit_task: |task| {
        let hal = get_davinci_hal();
        hal.submit_task(task) as i32
    },
    wait_task: |sync_obj, timeout| {
        let hal = get_davinci_hal();
        hal.wait_task(sync_obj, timeout) as i32
    },
    set_frequency: |freq| {
        let hal = get_davinci_hal();
        hal.set_frequency(freq) as i32
    },
    get_frequency: || {
        let hal = get_davinci_hal();
        hal.get_frequency()
    },
    suspend: || {
        let hal = get_davinci_hal();
        hal.suspend() as i32
    },
    resume: || {
        let hal = get_davinci_hal();
        hal.resume() as i32
    },
};

static mut DAVINCI_NPU_HAL: DaVinciNpuHal = DaVinciNpuHal::new();

pub fn get_davinci_hal() -> &'static mut DaVinciNpuHal {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut DAVINCI_NPU_HAL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_davinci() {
        let hal = get_davinci_hal();
        assert_eq!(hal.config.model, "Da Vinci C310");
        assert_eq!(hal.config.num_cores, 1);
    }
}
