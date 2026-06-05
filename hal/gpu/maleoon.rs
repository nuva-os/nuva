/*
 * Nuva OS - HAL - Gpu
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



use super::{GpuInfo, GpuState, GpuCommand, GpuHalOps, GpuDevice, GpuError, GpuCommandBufferRef};
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

// ============================================================================
// Maleoon GPU Register Definitions
// ============================================================================

/// GPU control register base address
const GPU_CTRL_BASE: u64 = 0xF600_0000;

/// GPU command queue base address
const GPU_CMDQ_BASE: u64 = 0xF610_0000;

/// GPU VRAM base address
const GPU_VRAM_BASE: u64 = 0xF700_0000;

// Control register offsets
const GPU_CTRL_ENABLE: u64 = 0x0000;       // GPU enable
const GPU_CTRL_FREQ: u64 = 0x0004;         // Frequency setting
const GPU_CTRL_VOLTAGE: u64 = 0x0008;      // Voltage setting
const GPU_CTRL_STATE: u64 = 0x000C;        // State register
const GPU_CTRL_IDLE: u64 = 0x0010;         // Idle control
const GPU_CTRL_POWER: u64 = 0x0014;        // Power control
const GPU_CTRL_TEMP: u64 = 0x0018;         // Temperature reading
const GPU_CTRL_UTIL: u64 = 0x001C;         // Utilization

// Command queue register offsets
const GPU_CMDQ_HEAD: u64 = 0x0000;         // Queue head pointer
const GPU_CMDQ_TAIL: u64 = 0x0004;         // Queue tail pointer
const GPU_CMDQ_DOORBELL: u64 = 0x0008;     // Doorbell register
const GPU_CMDQ_STATUS: u64 = 0x000C;       // Queue state

// Command queue size
const GPU_CMDQ_SIZE: u32 = 256;

// GPU state bits
const GPU_STATE_BUSY: u32 = 0x0001;
const GPU_STATE_ERROR: u32 = 0x0002;
const GPU_STATE_IDLE: u32 = 0x0004;

// Delay constants
const GPU_CMD_DELAY_US: u32 = 10;
const GPU_POWER_DELAY_US: u32 = 1000;

// Maleoon-specific register offsets
const GPU_CTRL_MALEOON_ID: u64 = 0x0020;      // Maleoon chip ID
const GPU_CTRL_SHADER_CFG: u64 = 0x0024;      // Shader configuration
const GPU_CTRL_GART_BASE: u64 = 0x0028;       // GART base address
const GPU_CTRL_GART_SIZE: u64 = 0x002C;       // GART size
const GPU_CTRL_FENCE_BASE: u64 = 0x0030;      // Fence base address

// Maleoon 910 chip ID
const MALEOON_910_CHIP_ID: u32 = 0x9100_0001;

/// Maleoon GPU configuration
pub struct MaleoonConfig {
    /// GPU model
    pub model: &'static str,
    /// Number of cores
    pub num_cores: u32,
    /// Minimum frequency
    pub min_freq: u64,
    /// Maximum frequency
    pub max_freq: u64,
    /// VRAM size
    pub vram_size: u64,
}

impl MaleoonConfig {
    pub const fn new() -> Self {
        MaleoonConfig {
            model: "Maleoon 910",
            num_cores: 10,
            min_freq: 300_000_000,    // 300 MHz
            max_freq: 750_000_000,    // 750 MHz
            vram_size: 512 * 1024 * 1024,  // 512MB
        }
    }
}

/// Maleoon GPU HAL
pub struct MaleoonGpuHal {
    config: MaleoonConfig,
    current_freq: u64,
    state: GpuState,
    /// Command queue tail pointer
    cmdq_tail: AtomicU32,
    /// Synchronization object counter
    sync_counter: AtomicU64,
    /// Initialization flag
    initialized: AtomicBool,
    /// Cached GPU info
    gpu_info: GpuInfo,
}

impl MaleoonGpuHal {
    pub const fn new() -> Self {
        MaleoonGpuHal {
            config: MaleoonConfig::new(),
            current_freq: 0,
            state: GpuState::Idle,
            cmdq_tail: AtomicU32::new(0),
            sync_counter: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
            gpu_info: GpuInfo {
                gpu_id: 0,
                name: "Maleoon 910",
                state: GpuState::Idle,
                current_freq: 0,
                min_freq: 0,
                max_freq: 0,
                vram_size: 0,
                utilization: 0,
                temperature: 0,
            },
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

    /// Wait for GPU idle
    fn wait_gpu_idle(&self) -> bool {
        let mut timeout = 100_000; // 100ms
        while timeout > 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let state = Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_STATE);
                if (state & GPU_STATE_BUSY) == 0 {
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
        log_info!("Maleoon GPU HAL initialized");
        log_info!("  Model: {}", self.config.model);
        log_info!("  Cores: {}", self.config.num_cores);
        log_info!("  Frequency: {}-{} MHz",
            self.config.min_freq / 1_000_000,
            self.config.max_freq / 1_000_000);
        log_info!("  VRAM: {} MB", self.config.vram_size / (1024 * 1024));

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Enable GPU
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_ENABLE, 1);
            Self::udelay(GPU_POWER_DELAY_US);

            // Set initial frequency
            let init_freq = (self.config.max_freq / 1000) as u32;
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_FREQ, init_freq);

            // Initialize command queue
            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_HEAD, 0);
            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_TAIL, 0);
        }

        self.current_freq = self.config.max_freq;
        self.state = GpuState::Idle;

        0
    }

    /// Get GPU info
    pub fn get_gpu_info(&self) -> GpuInfo {
        // Read actual values from hardware
        // SAFETY: unsafe block required for low-level memory or hardware access
        let (utilization, temperature) = unsafe {
            let util = Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_UTIL);
            let temp = Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_TEMP);
            (util, temp as i32)
        };

        GpuInfo {
            gpu_id: 0,
            name: self.config.model,
            state: self.state,
            current_freq: self.current_freq,
            min_freq: self.config.min_freq,
            max_freq: self.config.max_freq,
            vram_size: self.config.vram_size,
            utilization,
            temperature,
        }
    }

    /// Maleoon-specific initialization sequence
    pub fn maleoon_init_sequence(&mut self) -> Result<(), GpuError> {
        log_info!("Maleoon GPU: Starting chip-specific initialization");

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Step 1: Verify chip ID
            let chip_id = Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_MALEOON_ID);
            if chip_id != MALEOON_910_CHIP_ID {
                log_warn!("Maleoon GPU: Unexpected chip ID 0x{:08X}", chip_id);
                return Err(GpuError::HardwareError);
            }
            log_info!("Maleoon GPU: Chip ID verified (0x{:08X})", chip_id);

            // Step 2: Configure shader cores
            let shader_cfg = (self.config.num_cores << 16) | 0x0001;
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_SHADER_CFG, shader_cfg);

            // Step 3: Configure GART
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_GART_BASE, GPU_VRAM_BASE as u32);
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_GART_SIZE,
                (self.config.vram_size / 4096) as u32);

            // Step 4: Configure fence mechanism
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_FENCE_BASE, 0);

            // Step 5: Power on sequence
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_POWER, 1);
            Self::udelay(GPU_POWER_DELAY_US);

            // Step 6: Enable GPU
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_ENABLE, 1);
            Self::udelay(GPU_POWER_DELAY_US);

            // Step 7: Set operating frequency
            let init_freq = (self.config.max_freq / 1000) as u32;
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_FREQ, init_freq);

            // Step 8: Initialize command queues
            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_HEAD, 0);
            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_TAIL, 0);
            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_STATUS, 1);

            // Step 9: Wait for GPU ready
            let mut timeout = 100_000;
            while timeout > 0 {
                let state = Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_STATE);
                if (state & GPU_STATE_IDLE) != 0 {
                    break;
                }
                Self::udelay(1);
                timeout -= 1;
            }
            if timeout == 0 {
                log_warn!("Maleoon GPU: Timeout waiting for ready");
                return Err(GpuError::HardwareError);
            }
        }

        self.current_freq = self.config.max_freq;
        self.state = GpuState::Idle;
        self.initialized.store(true, Ordering::Release);

        // Update cached info
        self.gpu_info = GpuInfo {
            gpu_id: 0,
            name: self.config.model,
            state: GpuState::Idle,
            current_freq: self.config.max_freq,
            min_freq: self.config.min_freq,
            max_freq: self.config.max_freq,
            vram_size: self.config.vram_size,
            utilization: 0,
            temperature: 0,
        };

        log_info!("Maleoon GPU: Initialization complete");
        log_info!("  Model: {}", self.config.model);
        log_info!("  Cores: {}", self.config.num_cores);
        log_info!("  Frequency: {}-{} MHz",
            self.config.min_freq / 1_000_000,
            self.config.max_freq / 1_000_000);
        log_info!("  VRAM: {} MB", self.config.vram_size / (1024 * 1024));

        Ok(())
    }

    /// Submit command
    pub fn submit_command(&mut self, cmd: &GpuCommand) -> i32 {
        if self.state == GpuState::Suspended {
            return -1;
        }

        log_debug!("GPU: Submit command type {:?}", cmd.cmd_type);

        // Get queue position
        let tail = self.cmdq_tail.load(Ordering::Acquire);
        let next_tail = (tail + 1) % GPU_CMDQ_SIZE;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Check if queue is full
            let head = Self::read_reg(GPU_CMDQ_BASE + GPU_CMDQ_HEAD);
            if next_tail == head {
                log_warn!("GPU: Command queue full");
                return -2;
            }

            // Write command to queue
            // Command format: [cmd_type | cmd_data | cmd_size | sync_obj]
            let cmdq_entry_addr = GPU_CMDQ_BASE + 0x1000 + (tail as u64 * 16);
            Self::write_reg(cmdq_entry_addr, cmd.cmd_type as u32);
            Self::write_reg(cmdq_entry_addr + 4, cmd.data as u32);
            Self::write_reg(cmdq_entry_addr + 8, cmd.size as u32);

            // Generate synchronization object
            let sync_obj = self.sync_counter.fetch_add(1, Ordering::AcqRel) + 1;
            Self::write_reg(cmdq_entry_addr + 12, sync_obj as u32);

            // Update tail pointer
            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_TAIL, next_tail);

            // Ring doorbell to notify GPU
            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_DOORBELL, 1);
        }

        self.state = GpuState::Running;
        0
    }

    /// Wait for command completion
    pub fn wait_command(&mut self, sync_obj: u64, timeout_us: u64) -> i32 {
        let mut remaining = timeout_us;

        while remaining > 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                // Check synchronization object state
                // In actual implementation, should check if synchronization object is completed
                let state = Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_STATE);
                if (state & GPU_STATE_BUSY) == 0 {
                    self.state = GpuState::Idle;
                    return 0;
                }
            }
            Self::udelay(1);
            remaining -= 1;
        }

        log_warn!("GPU: Command timeout");
        -1
    }

    /// Set frequency
    pub fn set_frequency(&mut self, freq: u64) -> i32 {
        if freq < self.config.min_freq || freq > self.config.max_freq {
            return -1;
        }

        log_debug!("GPU: Setting frequency to {} MHz", freq / 1_000_000);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Write frequency (kHz)
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_FREQ, (freq / 1000) as u32);
            Self::udelay(GPU_CMD_DELAY_US);
        }

        self.current_freq = freq;
        0
    }

    /// Get frequency
    pub fn get_frequency(&self) -> u64 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let freq_khz = Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_FREQ);
            (freq_khz as u64) * 1000
        }
    }

    /// Enter idle state
    pub fn enter_idle(&mut self) -> i32 {
        log_debug!("GPU: Entering idle state");

        // Wait for current command to complete
        if !self.wait_gpu_idle() {
            return -1;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Set idle mode
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_IDLE, 1);

            // Lower frequency to minimum
            let min_freq = (self.config.min_freq / 1000) as u32;
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_FREQ, min_freq);
        }

        self.state = GpuState::Idle;
        0
    }

    /// Exit idle state
    pub fn exit_idle(&mut self) -> i32 {
        log_debug!("GPU: Exiting idle state");

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Exit idle mode
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_IDLE, 0);

            // Restore frequency
            let freq = (self.current_freq / 1000) as u32;
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_FREQ, freq);
        }

        self.state = GpuState::Running;
        0
    }

    /// Suspend
    pub fn suspend(&mut self) -> i32 {
        log_info!("GPU: Suspending");

        // Wait for all commands to complete
        if !self.wait_gpu_idle() {
            log_warn!("GPU: Timeout waiting for idle before suspend");
            return -1;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Disable GPU
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_ENABLE, 0);

            // Power off
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_POWER, 0);
            Self::udelay(GPU_POWER_DELAY_US);
        }

        self.state = GpuState::Suspended;
        0
    }

    /// Resume
    pub fn resume(&mut self) -> i32 {
        log_info!("GPU: Resuming");

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Power on
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_POWER, 1);
            Self::udelay(GPU_POWER_DELAY_US);

            // Enable GPU
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_ENABLE, 1);
            Self::udelay(GPU_POWER_DELAY_US);

            // Restore frequency
            let freq = (self.current_freq / 1000) as u32;
            Self::write_reg(GPU_CTRL_BASE + GPU_CTRL_FREQ, freq);
        }

        self.state = GpuState::Idle;
        0
    }

    /// Get temperature
    pub fn get_temperature(&self) -> i32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_TEMP) as i32
        }
    }

    /// Get utilization
    pub fn get_utilization(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            Self::read_reg(GPU_CTRL_BASE + GPU_CTRL_UTIL)
        }
    }
}

fn gpu_ops_init() -> i32 { get_maleoon_hal().init() }
fn gpu_ops_get_info() -> GpuInfo { get_maleoon_hal().get_gpu_info() }
fn gpu_ops_submit(cmd: &GpuCommand) -> i32 { get_maleoon_hal().submit_command(cmd) }
fn gpu_ops_wait(sync_obj: u64, timeout: u64) -> i32 { get_maleoon_hal().wait_command(sync_obj, timeout) }
fn gpu_ops_set_freq(freq: u64) -> i32 { get_maleoon_hal().set_frequency(freq) }
fn gpu_ops_get_freq() -> u64 { get_maleoon_hal().current_freq }
fn gpu_ops_enter_idle() -> i32 { get_maleoon_hal().enter_idle() }
fn gpu_ops_exit_idle() -> i32 { get_maleoon_hal().exit_idle() }
fn gpu_ops_suspend() -> i32 { get_maleoon_hal().suspend() }
fn gpu_ops_resume() -> i32 { get_maleoon_hal().resume() }

/// Maleoon GPU HAL operations
pub static MALEOON_GPU_OPS: GpuHalOps = GpuHalOps {
    init: gpu_ops_init,
    get_gpu_info: gpu_ops_get_info,
    submit_command: gpu_ops_submit,
    wait_command: gpu_ops_wait,
    set_frequency: gpu_ops_set_freq,
    get_frequency: gpu_ops_get_freq,
    enter_idle: gpu_ops_enter_idle,
    exit_idle: gpu_ops_exit_idle,
    suspend: gpu_ops_suspend,
    resume: gpu_ops_resume,
};

static mut MALEOON_GPU_HAL: MaleoonGpuHal = MaleoonGpuHal::new();

pub fn get_maleoon_hal() -> &'static mut MaleoonGpuHal {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut MALEOON_GPU_HAL }
}

// ============================================================================
// GpuDevice trait implementation for Maleoon
// ============================================================================

impl GpuDevice for MaleoonGpuHal {
    fn initialize(&mut self) -> Result<(), GpuError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }
        self.maleoon_init_sequence()
    }

    fn submit_command_buffer(&mut self, cmd_buf: &GpuCommandBufferRef) -> Result<u64, GpuError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpuError::NotInitialized);
        }

        if self.state == GpuState::Suspended {
            return Err(GpuError::HardwareError);
        }

        let sync_obj = self.sync_counter.fetch_add(1, Ordering::AcqRel) + 1;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let tail = self.cmdq_tail.load(Ordering::Acquire);
            let next_tail = (tail + 1) % GPU_CMDQ_SIZE;

            let head = Self::read_reg(GPU_CMDQ_BASE + GPU_CMDQ_HEAD);
            if next_tail == head {
                return Err(GpuError::Busy);
            }

            let entry_addr = GPU_CMDQ_BASE + 0x1000 + (tail as u64 * 16);
            Self::write_reg(entry_addr, 1);
            Self::write_reg(entry_addr + 4, cmd_buf.id);
            Self::write_reg(entry_addr + 8, cmd_buf.count);
            Self::write_reg(entry_addr + 12, sync_obj as u32);

            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_TAIL, next_tail);
            Self::write_reg(GPU_CMDQ_BASE + GPU_CMDQ_DOORBELL, 1);

            self.cmdq_tail.store(next_tail, Ordering::Release);
        }

        self.state = GpuState::Running;
        Ok(sync_obj)
    }

    fn wait_idle(&mut self, timeout_us: u64) -> Result<(), GpuError> {
        let mut remaining = timeout_us;
        while remaining > 0 {
            if self.wait_gpu_idle() {
                self.state = GpuState::Idle;
                return Ok(());
            }
            remaining = remaining.saturating_sub(1);
        }
        Err(GpuError::Timeout)
    }

    fn get_info(&self) -> &GpuInfo {
        &self.gpu_info
    }

    fn get_state(&self) -> GpuState {
        self.state
    }
}

// ============================================================================
// GPU Interrupt Handler
// ============================================================================

const GPU_IRQ_FENCE: u32 = 0;
const GPU_IRQ_GART_FAULT: u32 = 1;
const GPU_IRQ_HANG: u32 = 2;
const GPU_IRQ_CMD_COMPLETE: u32 = 3;

pub fn maleoon_irq_handler(irq: u32) {
    let hal = get_maleoon_hal();
    match irq {
        _ if irq == GPU_IRQ_FENCE => {
            unsafe {
                let fence_val = MaleoonGpuHal::read_reg(GPU_CTRL_BASE + GPU_CTRL_FENCE_VALUE);
                hal.sync_counter.store(fence_val as u64, Ordering::Release);
            }
        }
        _ if irq == GPU_IRQ_GART_FAULT => {
            let fault = unsafe { MaleoonGpuHal::read_reg(GPU_CTRL_BASE + GPU_CTRL_GART_FAULT) };
            log_warn!("Maleoon GPU: GART fault at 0x{:X}", fault);
            unsafe { MaleoonGpuHal::write_reg(GPU_CTRL_BASE + GPU_CTRL_GART_TLB_INV, 1); }
        }
        _ if irq == GPU_IRQ_HANG => {
            log_error!("Maleoon GPU: Hang detected, attempting soft reset");
            unsafe {
                MaleoonGpuHal::write_reg(GPU_CTRL_BASE + GPU_CTRL_RESET_SOFT, RESET_SOFT_TRIGGER);
                let mut timeout = 100_000u32;
                while timeout > 0 {
                    let status = MaleoonGpuHal::read_reg(GPU_CTRL_BASE + GPU_CTRL_RESET_STATUS);
                    if status == RESET_STATUS_DONE { break; }
                    timeout -= 1;
                }
            }
            hal.state = GpuState::Error;
        }
        _ if irq == GPU_IRQ_CMD_COMPLETE => {
            let state = unsafe { MaleoonGpuHal::read_reg(GPU_CTRL_BASE + GPU_CTRL_STATE) };
            if (state & GPU_STATE_BUSY) == 0 {
                hal.state = GpuState::Idle;
            }
        }
        _ => {
            log_warn!("Maleoon GPU: Unknown IRQ {}", irq);
        }
    }
}

// ============================================================================
// VRAM Allocator
// ============================================================================

const VRAM_BLOCK_SIZE: u64 = 4096;

#[derive(Debug, Clone, Copy)]
pub struct VramRegion {
    pub offset: u64,
    pub size: u64,
    pub allocated: bool,
}

pub struct VramAllocator {
    regions: [VramRegion; 64],
    num_regions: u32,
    total_size: u64,
    used_size: u64,
}

impl VramAllocator {
    pub const fn new() -> Self {
        VramAllocator {
            regions: [VramRegion { offset: 0, size: 0, allocated: false }; 64],
            num_regions: 0,
            total_size: 0,
            used_size: 0,
        }
    }

    pub fn init(&mut self, vram_size: u64) {
        self.total_size = vram_size;
        self.used_size = 0;
        self.num_regions = 1;
        self.regions[0] = VramRegion {
            offset: 0,
            size: vram_size,
            allocated: false,
        };
    }

    pub fn alloc(&mut self, size: u64) -> Option<u64> {
        let aligned_size = ((size + VRAM_BLOCK_SIZE - 1) / VRAM_BLOCK_SIZE) * VRAM_BLOCK_SIZE;
        let mut best_idx: Option<usize> = None;
        let mut best_size: u64 = u64::MAX;

        for i in 0..self.num_regions as usize {
            let r = &self.regions[i];
            if !r.allocated && r.size >= aligned_size && r.size < best_size {
                best_idx = Some(i);
                best_size = r.size;
            }
        }

        let idx = best_idx?;
        let offset = self.regions[idx].offset;
        let remaining = self.regions[idx].size - aligned_size;

        self.regions[idx].size = aligned_size;
        self.regions[idx].allocated = true;

        if remaining > 0 && (self.num_regions as usize) < 64 {
            let nr = self.num_regions as usize;
            self.regions[nr] = VramRegion {
                offset: offset + aligned_size,
                size: remaining,
                allocated: false,
            };
            self.num_regions += 1;
        }

        self.used_size += aligned_size;
        Some(GPU_VRAM_BASE + offset)
    }

    pub fn free(&mut self, addr: u64) -> bool {
        if addr < GPU_VRAM_BASE { return false; }
        let offset = addr - GPU_VRAM_BASE;

        for i in 0..self.num_regions as usize {
            if self.regions[i].offset == offset && self.regions[i].allocated {
                self.used_size -= self.regions[i].size;
                self.regions[i].allocated = false;
                self.coalesce();
                return true;
            }
        }
        false
    }

    fn coalesce(&mut self) {
        if self.num_regions < 2 { return; }
        let n = self.num_regions as usize;
        let mut i = 0;
        while i < n - 1 {
            if !self.regions[i].allocated && !self.regions[i + 1].allocated {
                let end_i = self.regions[i].offset + self.regions[i].size;
                if end_i == self.regions[i + 1].offset {
                    self.regions[i].size += self.regions[i + 1].size;
                    for j in (i + 1)..n - 1 {
                        self.regions[j] = self.regions[j + 1];
                    }
                    self.num_regions -= 1;
                    continue;
                }
            }
            i += 1;
        }
    }

    pub fn used(&self) -> u64 { self.used_size }
    pub fn total(&self) -> u64 { self.total_size }
}

static mut VRAM_ALLOCATOR: VramAllocator = VramAllocator::new();

pub fn get_vram_allocator() -> &'static mut VramAllocator {
    unsafe { &mut VRAM_ALLOCATOR }
}

pub fn init_vram(vram_size: u64) {
    get_vram_allocator().init(vram_size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maleoon() {
        let hal = get_maleoon_hal();
        assert_eq!(hal.config.model, "Maleoon 910");
        assert_eq!(hal.config.num_cores, 10);
    }
}







// ============================================================================
// Additional Register Definitions (Reset, GART, Fence, Capability)
// ============================================================================

// Reset register offsets
const GPU_CTRL_RESET_SOFT: u64 = 0x0040;
const GPU_CTRL_RESET_HARD: u64 = 0x0044;
const GPU_CTRL_RESET_STATUS: u64 = 0x0048;
const GPU_CTRL_HANG_DETECT: u64 = 0x004C;
const GPU_CTRL_ENGINE_STATUS: u64 = 0x0050;

// GART register offsets
const GPU_CTRL_GART_CTRL: u64 = 0x0058;
const GPU_CTRL_GART_TLB_INV: u64 = 0x005C;
const GPU_CTRL_GART_FAULT: u64 = 0x0060;

// Fence register offsets
const GPU_CTRL_FENCE_CTRL: u64 = 0x0068;
const GPU_CTRL_FENCE_VALUE: u64 = 0x006C;
const GPU_CTRL_FENCE_IRQ: u64 = 0x0070;

// Capability register offsets
const GPU_CTRL_CAP_ADMIN: u64 = 0x0080;
const GPU_CTRL_CAP_RENDER: u64 = 0x0084;
const GPU_CTRL_CAP_COMPUTE: u64 = 0x0088;
const GPU_CTRL_CAP_COPY: u64 = 0x008C;

// Capability bit definitions
const CAP_GPU_ADMIN: u32 = 0x0000_0001;
const CAP_GPU_RENDER: u32 = 0x0000_0002;
const CAP_GPU_COMPUTE: u32 = 0x0000_0004;
const CAP_GPU_COPY: u32 = 0x0000_0008;
const CAP_GPU_IOMMU: u32 = 0x0000_0010;

// Reset register values
const RESET_SOFT_TRIGGER: u32 = 0x0000_0001;
const RESET_HARD_TRIGGER: u32 = 0x0000_0001;
const RESET_STATUS_DONE: u32 = 0x0000_0001;
const RESET_STATUS_IN_PROGRESS: u32 = 0x0000_0002;
const RESET_STATUS_FAILED: u32 = 0x0000_0003;

// Engine status values
const ENGINE_STATUS_IDLE: u32 = 0x0000_0000;
const ENGINE_STATUS_BUSY: u32 = 0x0000_0001;
const ENGINE_STATUS_HUNG: u32 = 0x0000_0002;

// GART control bits
const GART_CTRL_ENABLE: u32 = 0x0000_0001;
const GART_CTRL_IOMMU: u32 = 0x0000_0002;
const GART_CTRL_FAULT_IRQ: u32 = 0x0000_0004;

// Fence control bits
const FENCE_CTRL_ENABLE: u32 = 0x0000_0001;
const FENCE_CTRL_IRQ_EN: u32 = 0x0000_0002;

// ============================================================================
// Capability-Based GPU Access
// ============================================================================

/// GPU capability set - replaces DRM master/UID with capability-based access
///
/// In Nuva OS, GPU access is controlled by capabilities rather than
/// the traditional Linux DRM master/UID model. This provides finer-grained
/// access control and better security isolation between GPU contexts.
#[derive(Debug, Clone, Copy)]
pub struct GpuCapabilities {
    /// Raw capability bitmask
    caps: u32,
}

impl GpuCapabilities {
    /// Create a new capability set from a bitmask
    pub const fn from_bits(bits: u32) -> Self {
        GpuCapabilities { caps: bits }
    }

    /// No capabilities (unprivileged)
    pub const fn none() -> Self {
        GpuCapabilities { caps: 0 }
    }

    /// Admin capabilities (full access)
    pub const fn admin() -> Self {
        GpuCapabilities { caps: CAP_GPU_ADMIN | CAP_GPU_RENDER | CAP_GPU_COMPUTE | CAP_GPU_COPY | CAP_GPU_IOMMU }
    }

    /// Render capabilities (typical application)
    pub const fn render() -> Self {
        GpuCapabilities { caps: CAP_GPU_RENDER | CAP_GPU_COPY }
    }

    /// Compute capabilities (ML/compute workloads)
    pub const fn compute() -> Self {
        GpuCapabilities { caps: CAP_GPU_COMPUTE | CAP_GPU_COPY }
    }

    /// Check if a specific capability is present
    pub const fn has(&self, cap: u32) -> bool {
        (self.caps & cap) != 0
    }

    /// Check if admin capability is present
    pub const fn is_admin(&self) -> bool {
        (self.caps & CAP_GPU_ADMIN) != 0
    }

    /// Check if render capability is present
    pub const fn can_render(&self) -> bool {
        (self.caps & CAP_GPU_RENDER) != 0
    }

    /// Check if compute capability is present
    pub const fn can_compute(&self) -> bool {
        (self.caps & CAP_GPU_COMPUTE) != 0
    }

    /// Check if copy capability is present
    pub const fn can_copy(&self) -> bool {
        (self.caps & CAP_GPU_COPY) != 0
    }

    /// Check if IOMMU management capability is present
    pub const fn can_iommu(&self) -> bool {
        (self.caps & CAP_GPU_IOMMU) != 0
    }

    /// Merge with another capability set
    pub const fn union(&self, other: &GpuCapabilities) -> GpuCapabilities {
        GpuCapabilities { caps: self.caps | other.caps }
    }

    /// Get raw capability bitmask
    pub const fn bits(&self) -> u32 {
        self.caps
    }
}

/// Check GPU capabilities against hardware
///
/// Reads the capability registers from the GPU hardware and returns
/// the set of capabilities that are actually available.
pub fn read_gpu_capabilities() -> GpuCapabilities {
    // SAFETY: reading GPU capability registers
    unsafe {
        let admin = read_volatile((GPU_CTRL_BASE + GPU_CTRL_CAP_ADMIN) as *const u32);
        let render = read_volatile((GPU_CTRL_BASE + GPU_CTRL_CAP_RENDER) as *const u32);
        let compute = read_volatile((GPU_CTRL_BASE + GPU_CTRL_CAP_COMPUTE) as *const u32);
        let copy_cap = read_volatile((GPU_CTRL_BASE + GPU_CTRL_CAP_COPY) as *const u32);
        let caps = admin | render | compute | copy_cap;
        GpuCapabilities::from_bits(caps)
    }
}

/// Verify that the caller has the required capability
///
/// Returns Ok(()) if the capability is present, Err(GpuError) otherwise.
/// This replaces the traditional DRM master/UID check.
pub fn check_gpu_capability(caps: &GpuCapabilities, required: u32) -> Result<(), GpuError> {
    if caps.has(required) {
        Ok(())
    } else {
        log_warn!("GPU: Capability check failed (has=0x{:X}, need=0x{:X})", caps.bits(), required);
        Err(GpuError::NotSupported)
    }
}
