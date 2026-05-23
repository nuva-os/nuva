/*
 * Qualcomm Hexagon DSP/NPU Driver
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Hardware driver for Qualcomm Hexagon DSP used as NPU
 * on Snapdragon platforms (e.g., Hexagon v68/v69/v73).
 * Provides firmware loading, compute submission, result
 * polling, and power/clock management.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, AtomicPtr, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::vec;

use crate::hal::npu::traits::{
    NpuHal, NpuError, NpuCapabilities, NpuStats,
    ModelId, BufferId, InferenceRequest, InferenceResult,
    InferenceHandle, ModelData, ModelFormat,
};

/// Hexagon DSP register offsets
pub mod hexagon_regs {
    /// DSP core control register
    pub const DSP_CTRL: u64 = 0x00;
    /// DSP status register
    pub const DSP_STATUS: u64 = 0x04;
    /// Firmware load address
    pub const FW_LOAD_ADDR: u64 = 0x08;
    /// Compute queue head pointer
    pub const COMPUTE_Q_HEAD: u64 = 0x10;
    /// Compute queue tail pointer
    pub const COMPUTE_Q_TAIL: u64 = 0x14;
    /// Result queue head pointer
    pub const RESULT_Q_HEAD: u64 = 0x18;
    /// Result queue tail pointer
    pub const RESULT_Q_TAIL: u64 = 0x1C;
    /// Interrupt status
    pub const IRQ_STATUS: u64 = 0x20;
    /// Interrupt mask
    pub const IRQ_MASK: u64 = 0x24;
    /// Power domain control
    pub const POWER_CTRL: u64 = 0x30;
    /// Clock control
    pub const CLOCK_CTRL: u64 = 0x34;
    /// Mailbox 0 (host->DSP)
    pub const MBOX_HOST_TO_DSP: u64 = 0x40;
    /// Mailbox 1 (DSP->host)
    pub const MBOX_DSP_TO_HOST: u64 = 0x44;
}

/// DSP control flags
pub mod dsp_ctrl {
    /// Start DSP
    pub const START: u32 = 1 << 0;
    /// Stop DSP
    pub const STOP: u32 = 1 << 1;
    /// Reset DSP
    pub const RESET: u32 = 1 << 2;
    /// Enable interrupts
    pub const IRQ_ENABLE: u32 = 1 << 3;
    /// Firmware loaded
    pub const FW_LOADED: u32 = 1 << 4;
}

/// Power domain states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexagonPowerState {
    /// Power off
    Off = 0,
    /// Retention (minimal power)
    Retention = 1,
    /// Active (full power)
    Active = 2,
    /// Turbo (overclocked)
    Turbo = 3,
}

/// Maximum compute contexts
pub const MAX_COMPUTE_CONTEXTS: usize = 16;

/// Maximum firmware size (8 MB)
pub const MAX_FIRMWARE_SIZE: usize = 8 * 1024 * 1024;

/// Maximum number of buffers
pub const MAX_BUFFERS: usize = 64;

/// DSP buffer descriptor
#[derive(Debug, Clone, Copy)]
pub struct DspBuffer {
    /// Buffer ID
    pub id: u64,
    /// Buffer size in bytes
    pub size: usize,
    /// DSP device address
    pub dsp_addr: u64,
    /// Whether buffer is active
    pub active: bool,
}

impl DspBuffer {
    pub const fn empty() -> Self {
        DspBuffer {
            id: 0,
            size: 0,
            dsp_addr: 0,
            active: false,
        }
    }
}

/// Hexagon compute context
pub struct HexagonComputeContext {
    /// Context ID
    pub id: u32,
    /// Model ID loaded in this context
    pub model_id: u64,
    /// Priority
    pub priority: u32,
    /// In use
    pub in_use: AtomicBool,
}

impl HexagonComputeContext {
    /// Create new context
    pub const fn new(id: u32) -> Self {
        HexagonComputeContext {
            id,
            model_id: 0,
            priority: 0,
            in_use: AtomicBool::new(false),
        }
    }
}

/// Hexagon DSP device structure
/// Represents a Hexagon DSP/NPU device with register access,
/// firmware management, compute contexts, and power control.
pub struct HexagonDsp {
    /// MMIO base address (virtual)
    mmio_base: AtomicU64,
    /// DSP generation (68, 69, 73)
    dsp_version: u32,
    /// Number of HVX threads
    num_hvx_threads: u32,
    /// L2 cache size in KB
    l2_cache_kb: u32,
    /// Power state
    power_state: AtomicU32,
    /// Clock frequency in Hz
    clock_hz: AtomicU64,
    /// Firmware loaded
    fw_loaded: AtomicBool,
    /// Compute contexts
    contexts: [HexagonComputeContext; MAX_COMPUTE_CONTEXTS],
    /// Next model ID
    next_model_id: AtomicU64,
    /// Next buffer ID
    next_buffer_id: AtomicU64,
    /// Buffer table
    buffers: [DspBuffer; MAX_BUFFERS],
    /// Number of active buffers
    num_buffers: AtomicU32,
    /// Total inferences
    total_inferences: AtomicU64,
    /// Successful inferences
    successful_inferences: AtomicU64,
    /// Initialized
    initialized: AtomicBool,
}

impl HexagonDsp {
    /// Create new Hexagon DSP instance
    pub const fn new() -> Self {
        HexagonDsp {
            mmio_base: AtomicU64::new(0),
            dsp_version: 68,
            num_hvx_threads: 4,
            l2_cache_kb: 512,
            power_state: AtomicU32::new(HexagonPowerState::Off as u32),
            clock_hz: AtomicU64::new(0),
            fw_loaded: AtomicBool::new(false),
            contexts: [
                HexagonComputeContext::new(0),
                HexagonComputeContext::new(1),
                HexagonComputeContext::new(2),
                HexagonComputeContext::new(3),
                HexagonComputeContext::new(4),
                HexagonComputeContext::new(5),
                HexagonComputeContext::new(6),
                HexagonComputeContext::new(7),
                HexagonComputeContext::new(8),
                HexagonComputeContext::new(9),
                HexagonComputeContext::new(10),
                HexagonComputeContext::new(11),
                HexagonComputeContext::new(12),
                HexagonComputeContext::new(13),
                HexagonComputeContext::new(14),
                HexagonComputeContext::new(15),
            ],
            next_model_id: AtomicU64::new(1),
            next_buffer_id: AtomicU64::new(1),
            buffers: [const { DspBuffer::empty() }; MAX_BUFFERS],
            num_buffers: AtomicU32::new(0),
            total_inferences: AtomicU64::new(0),
            successful_inferences: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Set MMIO base address
    /// Must be called before initialization, typically from
    /// device tree / ACPI resource parsing.
    pub fn set_mmio_base(&self, base: u64) {
        self.mmio_base.store(base, Ordering::Release);
    }

    /// Set DSP version and capabilities
    pub fn set_dsp_info(&mut self, version: u32, hvx_threads: u32, l2_kb: u32) {
        self.dsp_version = version;
        self.num_hvx_threads = hvx_threads;
        self.l2_cache_kb = l2_kb;
    }

    /// Get power state
    pub fn power_state(&self) -> HexagonPowerState {
        match self.power_state.load(Ordering::Acquire) {
            0 => HexagonPowerState::Off,
            1 => HexagonPowerState::Retention,
            2 => HexagonPowerState::Active,
            3 => HexagonPowerState::Turbo,
            _ => HexagonPowerState::Off,
        }
    }

    /// Load Hexagon microcode/firmware
    /// @param firmware: Firmware binary data
    /// @return: Ok on success, Err on failure
    pub fn hexagon_load_firmware(&self, firmware: &[u8]) -> Result<(), NpuError> {
        if firmware.len() > MAX_FIRMWARE_SIZE {
            return Err(NpuError::InvalidModel(
                String::from("Firmware too large"),
            ));
        }

        if firmware.is_empty() {
            return Err(NpuError::InvalidModel(
                String::from("Empty firmware"),
            ));
        }

        let base = self.mmio_base.load(Ordering::Acquire);
        if base == 0 {
            return Err(NpuError::HardwareError(
                String::from("MMIO not mapped"),
            ));
        }

        self.set_power_state(HexagonPowerState::Active);

        // SAFETY: FFI call to write firmware to DSP memory
        let result = unsafe {
            hexagon_write_firmware_ffi(
                base,
                firmware.as_ptr(),
                firmware.len(),
            )
        };

        if result == 0 {
            // SAFETY: FFI call to start DSP
            let start_result = unsafe {
                hexagon_dsp_start_ffi(base)
            };

            if start_result == 0 {
                self.fw_loaded.store(true, Ordering::Release);
                Ok(())
            } else {
                self.set_power_state(HexagonPowerState::Off);
                Err(NpuError::HardwareError(
                    String::from("DSP start failed"),
                ))
            }
        } else {
            self.set_power_state(HexagonPowerState::Off);
            Err(NpuError::HardwareError(
                String::from("Firmware write failed"),
            ))
        }
    }

    /// Submit compute task to DSP
    /// @param context_id: Compute context to use
    /// @param model_id: Model to execute
    /// @param input_buf: Input buffer address
    /// @param output_buf: Output buffer address
    /// @param priority: Task priority (0 = highest)
    /// @return: Handle for polling results
    pub fn hexagon_submit_compute(
        &mut self,
        context_id: u32,
        model_id: u64,
        input_buf: u64,
        output_buf: u64,
        priority: u32,
    ) -> Result<u64, NpuError> {
        if !self.fw_loaded.load(Ordering::Acquire) {
            return Err(NpuError::NotInitialized);
        }

        if context_id as usize >= MAX_COMPUTE_CONTEXTS {
            return Err(NpuError::InvalidRequest);
        }

        if !self.contexts[context_id as usize].in_use.load(Ordering::Acquire) {
            self.contexts[context_id as usize].in_use.store(true, Ordering::Release);
            self.contexts[context_id as usize].model_id = model_id;
            self.contexts[context_id as usize].priority = priority;
        }

        let handle = self.next_buffer_id.fetch_add(1, Ordering::AcqRel);

        let base = self.mmio_base.load(Ordering::Acquire);

        // SAFETY: FFI call to submit compute to DSP
        let result = unsafe {
            hexagon_submit_compute_ffi(
                base,
                context_id,
                model_id,
                input_buf,
                output_buf,
                priority,
            )
        };

        if result == 0 {
            self.total_inferences.fetch_add(1, Ordering::Relaxed);
            Ok(handle)
        } else {
            Err(NpuError::InferenceFailed(
                String::from("Compute submission failed"),
            ))
        }
    }

    /// Poll compute result
    /// @param handle: Handle from hexagon_submit_compute
    /// @param timeout_ms: Timeout in milliseconds
    /// @return: Inference result
    pub fn hexagon_poll_result(
        &self,
        handle: u64,
        timeout_ms: u32,
    ) -> Result<InferenceResult, NpuError> {
        if !self.fw_loaded.load(Ordering::Acquire) {
            return Err(NpuError::NotInitialized);
        }

        let base = self.mmio_base.load(Ordering::Acquire);

        // SAFETY: FFI call to poll DSP result
        let result = unsafe {
            hexagon_poll_result_ffi(base, handle, timeout_ms)
        };

        if result >= 0 {
            self.successful_inferences.fetch_add(1, Ordering::Relaxed);
            Ok(InferenceResult {
                output_buffers: Vec::new(),
                inference_time_us: result as u64,
                success: true,
            })
        } else {
            Err(NpuError::InferenceFailed(
                String::from("Compute failed or timed out"),
            ))
        }
    }

    /// Set power state
    fn set_power_state(&self, state: HexagonPowerState) {
        self.power_state.store(state as u32, Ordering::Release);

        let base = self.mmio_base.load(Ordering::Acquire);
        if base != 0 {
            // SAFETY: FFI call to set power domain
            unsafe {
                hexagon_set_power_ffi(base, state as u32);
            }
        }
    }

    /// Set clock frequency
    pub fn hexagon_set_clock(&self, freq_hz: u64) -> Result<(), NpuError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(NpuError::NotInitialized);
        }

        let base = self.mmio_base.load(Ordering::Acquire);

        // SAFETY: FFI call to set clock
        let result = unsafe {
            hexagon_set_clock_ffi(base, freq_hz)
        };

        if result == 0 {
            self.clock_hz.store(freq_hz, Ordering::Release);
            Ok(())
        } else {
            Err(NpuError::HardwareError(
                String::from("Clock set failed"),
            ))
        }
    }

    /// Get DSP statistics
    pub fn dsp_stats(&self) -> (u64, u64) {
        (
            self.total_inferences.load(Ordering::Acquire),
            self.successful_inferences.load(Ordering::Acquire),
        )
    }
}

impl NpuHal for HexagonDsp {
    fn initialize(&mut self) -> Result<(), NpuError> {
        if self.initialized.load(Ordering::Acquire) {
            return Err(NpuError::AlreadyInitialized);
        }

        let base = self.mmio_base.load(Ordering::Acquire);
        if base == 0 {
            return Err(NpuError::HardwareError(
                String::from("MMIO base not set"),
            ));
        }

        // SAFETY: FFI call to initialize DSP hardware
        let result = unsafe {
            hexagon_init_ffi(base)
        };

        if result == 0 {
            self.initialized.store(true, Ordering::Release);
            Ok(())
        } else {
            Err(NpuError::HardwareError(
                String::from("DSP init failed"),
            ))
        }
    }

    fn load_model(&mut self, model: &ModelData) -> Result<ModelId, NpuError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(NpuError::NotInitialized);
        }

        let id = ModelId(self.next_model_id.fetch_add(1, Ordering::AcqRel));

        let base = self.mmio_base.load(Ordering::Acquire);

        // SAFETY: FFI call to load model into DSP
        let result = unsafe {
            hexagon_load_model_ffi(
                base,
                id.0,
                model.data.as_ptr(),
                model.data.len(),
            )
        };

        if result == 0 {
            Ok(id)
        } else {
            Err(NpuError::InvalidModel(
                String::from("Model load failed on DSP"),
            ))
        }
    }

    fn unload_model(&mut self, id: ModelId) -> Result<(), NpuError> {
        let base = self.mmio_base.load(Ordering::Acquire);

        // SAFETY: FFI call to unload model from DSP
        let result = unsafe {
            hexagon_unload_model_ffi(base, id.0)
        };

        if result == 0 {
            Ok(())
        } else {
            Err(NpuError::ModelNotFound(id))
        }
    }

    fn create_buffer(&mut self, size: usize) -> Result<BufferId, NpuError> {
        if size == 0 {
            return Err(NpuError::InvalidBuffer);
        }

        let num = self.num_buffers.load(Ordering::Acquire);
        if num as usize >= MAX_BUFFERS {
            return Err(NpuError::OutOfMemory(
                String::from("DSP buffer table full"),
            ));
        }

        let base = self.mmio_base.load(Ordering::Acquire);
        // SAFETY: FFI call to allocate DSP device memory
        let dsp_addr = unsafe {
            hexagon_alloc_buffer_ffi(base, size)
        };

        if dsp_addr == 0 {
            return Err(NpuError::OutOfMemory(
                String::from("DSP device memory allocation failed"),
            ));
        }

        let id = self.next_buffer_id.fetch_add(1, Ordering::AcqRel);

        for i in 0..MAX_BUFFERS {
            if !self.buffers[i].active {
                self.buffers[i] = DspBuffer {
                    id,
                    size,
                    dsp_addr,
                    active: true,
                };
                self.num_buffers.fetch_add(1, Ordering::Release);
                return Ok(BufferId(id));
            }
        }

        Err(NpuError::OutOfMemory(String::from("No free buffer slot")))
    }

    fn destroy_buffer(&mut self, id: BufferId) -> Result<(), NpuError> {
        for i in 0..MAX_BUFFERS {
            if self.buffers[i].active && self.buffers[i].id == id.0 {
                let base = self.mmio_base.load(Ordering::Acquire);
                // SAFETY: FFI call to free DSP device memory
                unsafe {
                    hexagon_free_buffer_ffi(base, self.buffers[i].dsp_addr, self.buffers[i].size);
                }
                self.buffers[i].active = false;
                self.num_buffers.fetch_sub(1, Ordering::Release);
                return Ok(());
            }
        }
        Err(NpuError::InvalidBuffer)
    }

    fn write_buffer(&mut self, id: BufferId, data: &[u8]) -> Result<(), NpuError> {
        let base = self.mmio_base.load(Ordering::Acquire);
        // SAFETY: FFI call to write buffer to DSP memory
        let result = unsafe {
            hexagon_write_buffer_ffi(base, id.0, data.as_ptr(), data.len())
        };
        if result == 0 { Ok(()) } else { Err(NpuError::InvalidBuffer) }
    }

    fn read_buffer(&mut self, id: BufferId) -> Result<Vec<u8>, NpuError> {
        let base = self.mmio_base.load(Ordering::Acquire);
        let mut size: usize = 0;

        for i in 0..MAX_BUFFERS {
            if self.buffers[i].active && self.buffers[i].id == id.0 {
                size = self.buffers[i].size;
                break;
            }
        }

        if size == 0 {
            return Err(NpuError::InvalidBuffer);
        }

        let mut data = alloc::vec![0u8; size];
        // SAFETY: FFI call to read buffer from DSP memory
        let result = unsafe {
            hexagon_read_buffer_ffi(base, id.0, data.as_mut_ptr(), size)
        };

        if result == 0 {
            Ok(data)
        } else {
            Err(NpuError::InvalidBuffer)
        }
    }

    fn execute(&mut self, request: InferenceRequest) -> Result<InferenceResult, NpuError> {
        let handle = self.execute_async(request)?;
        self.wait(handle)
    }

    fn execute_async(&mut self, request: InferenceRequest) -> Result<InferenceHandle, NpuError> {
        if !self.fw_loaded.load(Ordering::Acquire) {
            return Err(NpuError::NotInitialized);
        }

        let handle = InferenceHandle(
            self.next_buffer_id.fetch_add(1, Ordering::AcqRel),
        );

        let base = self.mmio_base.load(Ordering::Acquire);

        // SAFETY: FFI call to submit async inference
        let result = unsafe {
            hexagon_submit_async_ffi(
                base,
                handle.0,
                request.model_id.0,
                request.priority,
            )
        };

        if result == 0 {
            self.total_inferences.fetch_add(1, Ordering::Relaxed);
            Ok(handle)
        } else {
            Err(NpuError::InferenceFailed(
                String::from("Async submit failed"),
            ))
        }
    }

    fn wait(&mut self, handle: InferenceHandle) -> Result<InferenceResult, NpuError> {
        self.hexagon_poll_result(handle.0, 5000)
    }

    fn capabilities(&self) -> NpuCapabilities {
        NpuCapabilities {
            max_model_size: 64 * 1024 * 1024,
            max_models: MAX_COMPUTE_CONTEXTS,
            max_buffer_size: 16 * 1024 * 1024,
            max_buffers: 256,
            supported_formats: vec![ModelFormat::Onnx, ModelFormat::TFLite],
            async_execution: true,
            quantization: true,
            num_cores: self.num_hvx_threads,
            frequency_mhz: (self.clock_hz.load(Ordering::Acquire) / 1_000_000) as u32,
            total_memory: self.l2_cache_kb as usize * 1024,
        }
    }

    fn stats(&self) -> NpuStats {
        NpuStats {
            total_inferences: self.total_inferences.load(Ordering::Acquire),
            successful_inferences: self.successful_inferences.load(Ordering::Acquire),
            failed_inferences: self.total_inferences.load(Ordering::Acquire)
                - self.successful_inferences.load(Ordering::Acquire),
            total_inference_time_us: 0,
            avg_inference_time_us: 0,
            memory_usage: 0,
            loaded_models: 0,
            utilization: 0,
        }
    }

    fn shutdown(&mut self) -> Result<(), NpuError> {
        self.set_power_state(HexagonPowerState::Off);
        self.fw_loaded.store(false, Ordering::Release);
        self.initialized.store(false, Ordering::Release);
        Ok(())
    }

    fn name(&self) -> &str {
        "HexagonDSP"
    }
}

/// FFI declarations for Hexagon DSP hardware
extern "C" {
    fn hexagon_init_ffi(mmio_base: u64) -> i32;
    fn hexagon_write_firmware_ffi(mmio_base: u64, fw: *const u8, fw_len: usize) -> i32;
    fn hexagon_dsp_start_ffi(mmio_base: u64) -> i32;
    fn hexagon_submit_compute_ffi(
        mmio_base: u64, ctx_id: u32, model_id: u64,
        input_buf: u64, output_buf: u64, priority: u32,
    ) -> i32;
    fn hexagon_poll_result_ffi(mmio_base: u64, handle: u64, timeout_ms: u32) -> i64;
    fn hexagon_submit_async_ffi(mmio_base: u64, handle: u64, model_id: u64, priority: u32) -> i32;
    fn hexagon_set_power_ffi(mmio_base: u64, state: u32);
    fn hexagon_set_clock_ffi(mmio_base: u64, freq_hz: u64) -> i32;
    fn hexagon_load_model_ffi(mmio_base: u64, model_id: u64, data: *const u8, len: usize) -> i32;
    fn hexagon_unload_model_ffi(mmio_base: u64, model_id: u64) -> i32;
    fn hexagon_write_buffer_ffi(mmio_base: u64, buf_id: u64, data: *const u8, len: usize) -> i32;
    fn hexagon_alloc_buffer_ffi(mmio_base: u64, size: usize) -> u64;
    fn hexagon_free_buffer_ffi(mmio_base: u64, dsp_addr: u64, size: usize);
    fn hexagon_read_buffer_ffi(mmio_base: u64, buf_id: u64, data: *mut u8, size: usize) -> i32;
}

/// Global Hexagon DSP instance
static mut HEXAGON_DSP: HexagonDsp = HexagonDsp::new();

/// Get global Hexagon DSP
pub fn get_hexagon_dsp() -> &'static mut HexagonDsp {
    // SAFETY: singleton access
    unsafe { &mut HEXAGON_DSP }
}

/// Initialize Hexagon DSP driver
pub fn init_hexagon_dsp() -> Result<(), NpuError> {
    get_hexagon_dsp().initialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexagon_dsp_new() {
        let dsp = HexagonDsp::new();
        assert_eq!(dsp.dsp_version, 68);
        assert_eq!(dsp.num_hvx_threads, 4);
    }

    #[test]
    fn test_hexagon_power_state() {
        let dsp = HexagonDsp::new();
        assert_eq!(dsp.power_state(), HexagonPowerState::Off);
    }

    #[test]
    fn test_compute_context() {
        let ctx = HexagonComputeContext::new(5);
        assert_eq!(ctx.id, 5);
        assert!(!ctx.in_use.load(Ordering::Relaxed));
    }

    #[test]
    fn test_hexagon_name() {
        let dsp = HexagonDsp::new();
        assert_eq!(dsp.name(), "HexagonDSP");
    }
}
