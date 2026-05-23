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


/// GPU State
// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuState {
    /// emptyidle
    Idle = 0,
    /// runinfix
    Running = 1,
    /// suspend
    Suspended = 2,
    /// Error
    Error = 3,
}

/// GPU Info
pub struct GpuInfo {
    /// GPU ID
    pub gpu_id: u32,
    /// GPU Name
    pub name: &'static str,
    /// CurrentState
    pub state: GpuState,
    /// CurrentFrequency
    pub current_freq: u64,
    /// MinFrequency
    pub min_freq: u64,
    /// MaxFrequency
    pub max_freq: u64,
    /// explicitexistSize
    pub vram_size: u64,
    /// interestuserate
    pub utilization: u32,
    /// tempDegree
    pub temperature: i32,
}

/// GPU commandType
#[derive(Debug, Clone, Copy)]
pub enum GpuCommandType {
    /// Rendercommand
    Render = 0,
    /// calculatecommand
    Compute = 1,
    /// Copycommand
    Copy = 2,
    /// Clearcommand
    Clear = 3,
    /// Synchronouscommand
    Sync = 4,
}

/// GPU command
#[derive(Debug, Clone, Copy)]
pub struct GpuCommand {
    /// commandType
    pub cmd_type: GpuCommandType,
    /// commandDatapointer
    pub data: u64,
    /// DataSize
    pub size: u64,
    /// Priority
    pub priority: u32,
    /// SynchronousObject
    pub sync_obj: u64,
}

// ============================================================================
// GPU Device Trait
// ============================================================================

/// GPU device error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuError {
    /// Device not initialized
    NotInitialized = 0,
    /// Out of memory
    OutOfMemory = 1,
    /// Device busy
    Busy = 2,
    /// Invalid argument
    InvalidArg = 3,
    /// Hardware error
    HardwareError = 4,
    /// Timeout
    Timeout = 5,
    /// Not supported
    NotSupported = 6,
}

/// GPU device trait - abstract interface for GPU drivers
pub trait GpuDevice: Send + Sync {
    /// Initialize the GPU device
    fn initialize(&mut self) -> Result<(), GpuError>;

    /// Submit a command buffer for execution
    fn submit_command_buffer(&mut self, cmd_buf: &GpuCommandBufferRef) -> Result<u64, GpuError>;

    /// Wait for GPU to become idle
    fn wait_idle(&mut self, timeout_us: u64) -> Result<(), GpuError>;

    /// Get GPU info
    fn get_info(&self) -> &GpuInfo;

    /// Get GPU state
    fn get_state(&self) -> GpuState;
}

// ============================================================================
// GPU Command Buffer
// ============================================================================

/// Command buffer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBufferState {
    /// Available for reuse
    Free = 0,
    /// Being recorded
    Recording = 1,
    /// Ready for submission
    Ready = 2,
    /// Submitted to GPU
    Submitted = 3,
    /// Execution completed
    Completed = 4,
}

/// Maximum commands per buffer
pub const GPU_CMD_BUF_SIZE: usize = 256;

/// GPU command buffer
pub struct GpuCommandBuffer {
    /// Buffer ID
    pub id: u32,
    /// Buffer state
    pub state: AtomicU32,
    /// Commands in buffer
    pub commands: [GpuCommand; GPU_CMD_BUF_SIZE],
    /// Number of commands
    pub count: u32,
    /// Associated fence
    pub fence_id: u64,
}

impl GpuCommandBuffer {
    /// Create a new command buffer
    pub fn new(id: u32) -> Self {
        let default_cmd = GpuCommand {
            cmd_type: GpuCommandType::Sync,
            data: 0,
            size: 0,
            priority: 0,
            sync_obj: 0,
        };
        GpuCommandBuffer {
            id,
            state: AtomicU32::new(CommandBufferState::Free as u32),
            commands: [default_cmd; GPU_CMD_BUF_SIZE],
            count: 0,
            fence_id: 0,
        }
    }

    /// Get buffer state
    pub fn get_state(&self) -> CommandBufferState {
        match self.state.load(Ordering::Acquire) {
            0 => CommandBufferState::Free,
            1 => CommandBufferState::Recording,
            2 => CommandBufferState::Ready,
            3 => CommandBufferState::Submitted,
            4 => CommandBufferState::Completed,
            _ => CommandBufferState::Free,
        }
    }

    /// Begin recording commands
    pub fn begin(&mut self) -> Result<(), GpuError> {
        let state = self.get_state();
        if state != CommandBufferState::Free && state != CommandBufferState::Completed {
            return Err(GpuError::Busy);
        }
        self.count = 0;
        self.state.store(CommandBufferState::Recording as u32, Ordering::Release);
        Ok(())
    }

    /// Push a command into the buffer
    pub fn push(&mut self, cmd: GpuCommand) -> Result<(), GpuError> {
        if self.get_state() != CommandBufferState::Recording {
            return Err(GpuError::InvalidArg);
        }
        if self.count as usize >= GPU_CMD_BUF_SIZE {
            return Err(GpuError::OutOfMemory);
        }
        self.commands[self.count as usize] = cmd;
        self.count += 1;
        Ok(())
    }

    /// Finish recording and mark ready
    pub fn finish(&mut self) -> Result<(), GpuError> {
        if self.get_state() != CommandBufferState::Recording {
            return Err(GpuError::InvalidArg);
        }
        self.state.store(CommandBufferState::Ready as u32, Ordering::Release);
        Ok(())
    }

    /// Reset buffer for reuse
    pub fn reset(&mut self) {
        self.count = 0;
        self.fence_id = 0;
        self.state.store(CommandBufferState::Free as u32, Ordering::Release);
    }
}

/// Reference to a command buffer (for submission)
pub struct GpuCommandBufferRef {
    /// Buffer ID
    pub id: u32,
    /// Command count
    pub count: u32,
}

// ============================================================================
// GPU Memory Management (GpuHeap + GART)
// ============================================================================

/// GPU heap allocation result
#[derive(Debug, Clone, Copy)]
pub struct GpuAllocation {
    /// GPU virtual address
    pub gpu_addr: u64,
    /// Size in bytes
    pub size: u64,
    /// GART entry index
    pub gart_index: u32,
}

/// GART (Graphics Address Remapping Table) entry
#[derive(Debug, Clone, Copy)]
pub struct GartEntry {
    /// GPU virtual address
    pub gpu_addr: u64,
    /// System physical address
    pub sys_addr: u64,
    /// Size in bytes
    pub size: u64,
    /// Entry is valid
    pub valid: bool,
}

/// Maximum GART entries
pub const MAX_GART_ENTRIES: usize = 1024;

/// GPU heap - manages GPU memory allocations
pub struct GpuHeap {
    /// Base address of GPU VRAM
    pub vram_base: u64,
    /// Total VRAM size
    pub vram_size: u64,
    /// Current allocation offset
    pub alloc_offset: AtomicU64,
    /// GART entries
    pub gart: [GartEntry; MAX_GART_ENTRIES],
    /// Number of used GART entries
    pub gart_used: AtomicU32,
    /// Total allocated bytes
    pub allocated: AtomicU64,
}

impl GpuHeap {
    /// Create a new GPU heap
    pub fn new(vram_base: u64, vram_size: u64) -> Self {
        let default_gart = GartEntry {
            gpu_addr: 0,
            sys_addr: 0,
            size: 0,
            valid: false,
        };
        GpuHeap {
            vram_base,
            vram_size,
            alloc_offset: AtomicU64::new(0),
            gart: [default_gart; MAX_GART_ENTRIES],
            gart_used: AtomicU32::new(0),
            allocated: AtomicU64::new(0),
        }
    }

    /// Allocate GPU memory
    pub fn allocate(&mut self, size: u64, align: u64) -> Result<GpuAllocation, GpuError> {
        let effective_align = if align == 0 { 4096 } else { align };
        let current = self.alloc_offset.load(Ordering::Acquire);
        let aligned = ((current + effective_align - 1) / effective_align) * effective_align;
        let new_offset = aligned + size;

        if new_offset > self.vram_size {
            return Err(GpuError::OutOfMemory);
        }

        self.alloc_offset.store(new_offset, Ordering::Release);
        self.allocated.fetch_add(size, Ordering::AcqRel);

        let gpu_addr = self.vram_base + aligned;

        let gart_index = self.map_gart(gpu_addr, 0, size)?;

        Ok(GpuAllocation {
            gpu_addr,
            size,
            gart_index,
        })
    }

    /// Free GPU memory (simple bump allocator - cannot free individual)
    pub fn free(&mut self, _alloc: &GpuAllocation) -> Result<(), GpuError> {
        self.allocated.fetch_sub(_alloc.size, Ordering::AcqRel);
        if (_alloc.gart_index as usize) < MAX_GART_ENTRIES {
            self.gart[_alloc.gart_index as usize].valid = false;
        }
        Ok(())
    }

    /// Map a GART entry (GPU VA -> System PA)
    pub fn map_gart(&mut self, gpu_addr: u64, sys_addr: u64, size: u64) -> Result<u32, GpuError> {
        let used = self.gart_used.load(Ordering::Acquire);
        if used as usize >= MAX_GART_ENTRIES {
            return Err(GpuError::OutOfMemory);
        }

        let idx = used;
        self.gart[idx as usize] = GartEntry {
            gpu_addr,
            sys_addr,
            size,
            valid: true,
        };
        self.gart_used.fetch_add(1, Ordering::Release);
        Ok(idx)
    }

    /// Unmap a GART entry
    pub fn unmap_gart(&mut self, index: u32) -> Result<(), GpuError> {
        if index as usize >= MAX_GART_ENTRIES {
            return Err(GpuError::InvalidArg);
        }
        self.gart[index as usize].valid = false;
        Ok(())
    }

    /// Get total allocated bytes
    pub fn get_allocated(&self) -> u64 {
        self.allocated.load(Ordering::Acquire)
    }

    /// Get free bytes
    pub fn get_free(&self) -> u64 {
        self.vram_size.saturating_sub(self.allocated.load(Ordering::Acquire))
    }
}

// ============================================================================
// GPU Fence (Synchronization)
// ============================================================================

/// GPU fence state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceState {
    /// Not yet signaled
    Pending = 0,
    /// Signaled (operation complete)
    Signaled = 1,
    /// Error occurred
    Error = 2,
}

/// Maximum concurrent fences
pub const MAX_GPU_FENCES: usize = 128;

/// GPU fence for synchronization
pub struct GpuFence {
    /// Fence ID
    pub id: u64,
    /// Fence state
    pub state: AtomicU32,
    /// Associated command buffer
    pub cmd_buf_id: u32,
    /// Timestamp when signaled
    pub timestamp: AtomicU64,
}

impl GpuFence {
    /// Create a new fence
    pub const fn new(id: u64) -> Self {
        GpuFence {
            id,
            state: AtomicU32::new(FenceState::Pending as u32),
            cmd_buf_id: 0,
            timestamp: AtomicU64::new(0),
        }
    }

    /// Get fence state
    pub fn get_state(&self) -> FenceState {
        match self.state.load(Ordering::Acquire) {
            0 => FenceState::Pending,
            1 => FenceState::Signaled,
            _ => FenceState::Error,
        }
    }

    /// Check if fence is signaled
    pub fn is_signaled(&self) -> bool {
        self.get_state() == FenceState::Signaled
    }

    /// Signal the fence
    pub fn signal(&self, timestamp: u64) {
        self.timestamp.store(timestamp, Ordering::Release);
        self.state.store(FenceState::Signaled as u32, Ordering::Release);
    }

    /// Set fence to error state
    pub fn set_error(&self) {
        self.state.store(FenceState::Error as u32, Ordering::Release);
    }
}

/// GPU fence manager
pub struct GpuFenceManager {
    /// Fence pool
    pub fences: [GpuFence; MAX_GPU_FENCES],
    /// Next fence ID
    pub next_id: AtomicU64,
    /// Number of active fences
    pub active_count: AtomicU32,
}

impl GpuFenceManager {
    /// Create new fence manager
    pub fn new() -> Self {
        // SAFETY: GpuFence contains AtomicU32/AtomicU64 which are zero-initializable.
        // AtomicU32::new(0) and AtomicU64::new(0) are valid zero states.
        let fences: [GpuFence; MAX_GPU_FENCES] = unsafe {
            core::mem::zeroed()
        };
        GpuFenceManager {
            fences,
            next_id: AtomicU64::new(1),
            active_count: AtomicU32::new(0),
        }
    }

    /// Allocate a new fence
    pub fn create_fence(&mut self) -> Result<&GpuFence, GpuError> {
        let fence_id = self.next_id.fetch_add(1, Ordering::AcqRel);

        for i in 0..MAX_GPU_FENCES {
            let state = self.fences[i].get_state();
            if state == FenceState::Signaled || state == FenceState::Error {
                self.fences[i] = GpuFence::new(fence_id);
                self.active_count.fetch_add(1, Ordering::AcqRel);
                return Ok(&self.fences[i]);
            }
            if self.fences[i].id == 0 {
                self.fences[i] = GpuFence::new(fence_id);
                self.active_count.fetch_add(1, Ordering::AcqRel);
                return Ok(&self.fences[i]);
            }
        }

        Err(GpuError::OutOfMemory)
    }

    /// Wait for a fence to be signaled
    pub fn wait_fence(&self, fence_id: u64, timeout_us: u64) -> Result<(), GpuError> {
        let mut remaining = timeout_us;
        loop {
            for i in 0..MAX_GPU_FENCES {
                if self.fences[i].id == fence_id && self.fences[i].is_signaled() {
                    self.active_count.fetch_sub(1, Ordering::AcqRel);
                    return Ok(());
                }
            }
            if remaining == 0 {
                return Err(GpuError::Timeout);
            }
            remaining = remaining.saturating_sub(1);
            core::hint::spin_loop();
        }
    }
}

/// GPU HAL Operation
pub struct GpuHalOps {
    /// Initialize
    pub init: fn() -> i32,
    /// Get GPU Info
    pub get_gpu_info: fn() -> GpuInfo,
    /// Commitcommand
    pub submit_command: fn(cmd: &GpuCommand) -> i32,
    /// Waitcommandcomplete
    pub wait_command: fn(sync_obj: u64, timeout: u64) -> i32,
    /// SetFrequency
    pub set_frequency: fn(freq: u64) -> i32,
    /// GetFrequency
    pub get_frequency: fn() -> u64,
    /// EnteremptyidleState
    pub enter_idle: fn() -> i32,
    /// ExitemptyidleState
    pub exit_idle: fn() -> i32,
    /// suspend
    pub suspend: fn() -> i32,
    /// Recovery
    pub resume: fn() -> i32,
}

/// GPU HAL Device
pub struct GpuHalDevice {
    /// GPU Info
    pub info: GpuInfo,
    /// HAL Operation
    pub ops: &'static GpuHalOps,
    /// commandQueuecount
    pub num_queues: u32,
}

impl GpuHalDevice {
    pub const fn new() -> Self {
        GpuHalDevice {
            info: GpuInfo {
                gpu_id: 0,
                name: "Unknown",
                state: GpuState::Idle,
                current_freq: 0,
                min_freq: 0,
                max_freq: 0,
                vram_size: 0,
                utilization: 0,
                temperature: 0,
            },
            ops: &GPU_HAL_OPS_NONE,
            num_queues: 0,
        }
    }
    
    /// Initialize
    pub fn init(&mut self) -> i32 {
        (self.ops.init)()
    }
    
    /// Get GPU Info
    pub fn get_info(&self) -> &GpuInfo {
        &self.info
    }
    
    /// Commitcommand
    pub fn submit_command(&self, cmd: &GpuCommand) -> i32 {
        (self.ops.submit_command)(cmd)
    }
    
    /// Waitcommandcomplete
    pub fn wait_command(&self, sync_obj: u64, timeout: u64) -> i32 {
        (self.ops.wait_command)(sync_obj, timeout)
    }
    
    /// SetFrequency
    pub fn set_frequency(&mut self, freq: u64) -> i32 {
        (self.ops.set_frequency)(freq)
    }
    
    /// GetFrequency
    pub fn get_frequency(&self) -> u64 {
        (self.ops.get_frequency)()
    }
    
    /// EnteremptyidleState
    pub fn enter_idle(&mut self) -> i32 {
        (self.ops.enter_idle)()
    }
    
    /// ExitemptyidleState
    pub fn exit_idle(&mut self) -> i32 {
        (self.ops.exit_idle)()
    }
    
    /// suspend
    pub fn suspend(&mut self) -> i32 {
        (self.ops.suspend)()
    }
    
    /// Recovery
    pub fn resume(&mut self) -> i32 {
        (self.ops.resume)()
    }
}

/// empty  GPU HAL Operation
static GPU_HAL_OPS_NONE: GpuHalOps = GpuHalOps {
    init: || -1,
    get_gpu_info: || GpuInfo {
        gpu_id: 0,
        name: "None",
        state: GpuState::Error,
        current_freq: 0,
        min_freq: 0,
        max_freq: 0,
        vram_size: 0,
        utilization: 0,
        temperature: 0,
    },
    submit_command: |_cmd| -1,
    wait_command: |_sync_obj, _timeout| -1,
    set_frequency: |_freq| -1,
    get_frequency: || 0,
    enter_idle: || -1,
    exit_idle: || -1,
    suspend: || -1,
    resume: || -1,
};

/// Global GPU HAL Device
static mut GPU_HAL_DEVICE: GpuHalDevice = GpuHalDevice::new();

pub fn get_gpu_hal() -> &'static mut GpuHalDevice {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut GPU_HAL_DEVICE }
}

pub fn init_gpu_hal() {
    log_info!("GPU HAL initialized");
}

/// Initialize GPU subsystem (FFI-compatible wrapper)
pub fn init_gpu() {
    init_gpu_hal();
}

/// Shutdown GPU subsystem
pub fn shutdown_gpu() {
    log_info!("GPU HAL shutdown");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_hal() {
        let hal = get_gpu_hal();
        assert_eq!(hal.info.gpu_id, 0);
    }

    #[test]
    fn test_gpu_state() {
        assert_eq!(GpuState::Idle as i32, 0);
        assert_eq!(GpuState::Running as i32, 1);
        assert_eq!(GpuState::Suspended as i32, 2);
        assert_eq!(GpuState::Error as i32, 3);
    }

    #[test]
    fn test_gpu_command_type() {
        assert_eq!(GpuCommandType::Render as i32, 0);
        assert_eq!(GpuCommandType::Compute as i32, 1);
        assert_eq!(GpuCommandType::Copy as i32, 2);
        assert_eq!(GpuCommandType::Clear as i32, 3);
        assert_eq!(GpuCommandType::Sync as i32, 4);
    }

    #[test]
    fn test_gpu_info() {
        let info = GpuInfo {
            gpu_id: 0,
            name: "Maleoon 910",
            state: GpuState::Running,
            current_freq: 750_000_000,
            min_freq: 300_000_000,
            max_freq: 900_000_000,
            vram_size: 8 * 1024 * 1024 * 1024,  // 8GB
            utilization: 75,
            temperature: 65000,  // 65°C
        };

        assert_eq!(info.gpu_id, 0);
        assert_eq!(info.name, "Maleoon 910");
        assert_eq!(info.state, GpuState::Running);
        assert_eq!(info.vram_size, 8 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_gpu_command() {
        let cmd = GpuCommand {
            cmd_type: GpuCommandType::Render,
            data: 0x1000,
            size: 1024,
            priority: 1,
            sync_obj: 0,
        };

        assert_eq!(cmd.cmd_type, GpuCommandType::Render);
        assert_eq!(cmd.size, 1024);
        assert_eq!(cmd.priority, 1);
    }

    #[test]
    fn test_gpu_hal_device_new() {
        let device = GpuHalDevice::new();
        assert_eq!(device.info.gpu_id, 0);
        assert_eq!(device.num_queues, 0);
    }
}