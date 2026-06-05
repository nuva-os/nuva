/*
 * Nuva OS - HAL - Gpu - Ring Buffer
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

/* GPU Ring Buffer - Command submission via ring buffer with doorbell mechanism.
 *
 * The ring buffer is the primary interface for submitting GPU commands.
 * Commands are written into a circular buffer in system memory that is
 * shared between the CPU and GPU. After writing commands, the CPU rings
 * the doorbell register to notify the GPU that new work is available.
 *
 * Batch submission is supported to amortize doorbell ring overhead.
 */

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

use super::{GpuCommand, GpuError};

// ============================================================================
// Ring Buffer Constants
// ============================================================================

/// Default ring buffer size in bytes (64 KB)
pub const RING_BUFFER_DEFAULT_SIZE: u32 = 64 * 1024;

/// Maximum batch size for command submission
pub const RING_BUFFER_MAX_BATCH: usize = 64;

/// Command packet header size in bytes (type + size + fence_id)
pub const CMD_PKT_HEADER_SIZE: u32 = 12;

/// Maximum command payload size in bytes
pub const CMD_PKT_MAX_PAYLOAD: u32 = 4084;

/// Alignment requirement for ring buffer entries (256 bytes)
pub const RING_BUFFER_ALIGNMENT: u32 = 256;

// ============================================================================
// Command Packet Format
// ============================================================================

/// Command packet header (written into ring buffer)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CommandPacketHeader {
    /// Command type (render, compute, copy, etc.)
    pub cmd_type: u32,
    /// Total packet size in bytes (header + payload)
    pub total_size: u32,
    /// Fence ID for completion tracking
    pub fence_id: u64,
}

/// Command packet in ring buffer
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CommandPacket {
    /// Packet header
    pub header: CommandPacketHeader,
    /// Payload data (GPU command words)
    pub payload: [u32; 1024],
    /// Actual payload length in u32 words
    pub payload_len: u32,
}

impl CommandPacket {
    /// Create a new command packet from a GpuCommand
    pub fn from_gpu_command(cmd: &GpuCommand, fence_id: u64) -> Self {
        CommandPacket {
            header: CommandPacketHeader {
                cmd_type: cmd.cmd_type as u32,
                total_size: CMD_PKT_HEADER_SIZE + cmd.size as u32,
                fence_id,
            },
            payload: [0u32; 1024],
            payload_len: 0,
        }
    }
}

// ============================================================================
// Doorbell Mechanism
// ============================================================================

/// Doorbell register interface for notifying GPU of new work
pub struct Doorbell {
    /// MMIO base address for doorbell registers
    mmio_base: u64,
    /// Doorbell offset for this ring
    offset: u64,
    /// Last doorbell value written
    last_value: AtomicU32,
}

impl Doorbell {
    /// Create a new doorbell interface
    pub const fn new(mmio_base: u64, offset: u64) -> Self {
        Doorbell {
            mmio_base,
            offset,
            last_value: AtomicU32::new(0),
        }
    }

    /// Ring the doorbell to notify GPU of new commands
    #[inline]
    pub fn ring(&self, wptr: u32) {
        self.last_value.store(wptr, Ordering::Release);
        // SAFETY: writing to device MMIO register for doorbell notification
        unsafe {
            write_volatile((self.mmio_base + self.offset) as *mut u32, wptr);
        }
    }

    /// Read current doorbell value (GPU's read pointer)
    #[inline]
    pub fn read(&self) -> u32 {
        // SAFETY: reading from device MMIO register
        unsafe {
            read_volatile((self.mmio_base + self.offset + 4) as *const u32)
        }
    }

    /// Get last written doorbell value
    #[inline]
    pub fn last_written(&self) -> u32 {
        self.last_value.load(Ordering::Acquire)
    }
}

// ============================================================================
// Ring Buffer
// ============================================================================

/// Ring buffer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingBufferState {
    /// Not initialized
    Uninitialized = 0,
    /// Ready for command submission
    Ready = 1,
    /// Currently writing commands
    Writing = 2,
    /// Error state
    Error = 3,
}

/// GPU Ring Buffer - manages command submission via circular buffer
pub struct GpuRingBuffer {
    /// Ring buffer ID (for multi-ring GPUs)
    pub ring_id: u32,
    /// Base virtual address of the ring buffer in system memory
    buffer_base: u64,
    /// Size of the ring buffer in bytes
    buffer_size: u32,
    /// Current write pointer (offset in bytes from buffer_base)
    wptr: AtomicU32,
    /// Cached read pointer (last known GPU read position)
    rptr: AtomicU32,
    /// Ring buffer state
    state: AtomicU32,
    /// Doorbell mechanism
    doorbell: Doorbell,
    /// Number of commands submitted (cumulative)
    submitted_count: AtomicU64,
    /// Number of commands completed (from GPU fence updates)
    completed_count: AtomicU64,
    /// Batch accumulation counter
    batch_count: AtomicU32,
    /// Whether batch mode is enabled
    batch_enabled: AtomicBool,
    /// Initialized flag
    initialized: AtomicBool,
}

impl GpuRingBuffer {
    /// Create a new ring buffer instance
    pub const fn new(ring_id: u32, buffer_base: u64, buffer_size: u32,
                     doorbell_base: u64, doorbell_offset: u64) -> Self {
        GpuRingBuffer {
            ring_id,
            buffer_base,
            buffer_size,
            wptr: AtomicU32::new(0),
            rptr: AtomicU32::new(0),
            state: AtomicU32::new(RingBufferState::Uninitialized as u32),
            doorbell: Doorbell::new(doorbell_base, doorbell_offset),
            submitted_count: AtomicU64::new(0),
            completed_count: AtomicU64::new(0),
            batch_count: AtomicU32::new(0),
            batch_enabled: AtomicBool::new(true),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the ring buffer
    pub fn init(&self) -> Result<(), GpuError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }

        // Zero the ring buffer memory region
        // SAFETY: clearing the ring buffer region in system memory
        unsafe {
            let ptr = self.buffer_base as *mut u8;
            let size = self.buffer_size as usize;
            for i in 0..size {
                write_volatile(ptr.add(i), 0);
            }
        }

        self.wptr.store(0, Ordering::Release);
        self.rptr.store(0, Ordering::Release);
        self.batch_count.store(0, Ordering::Release);
        self.state.store(RingBufferState::Ready as u32, Ordering::Release);
        self.initialized.store(true, Ordering::Release);

        log_info!("Ring buffer {}: initialized (size={} KB, base=0x{:X})",
            self.ring_id, self.buffer_size / 1024, self.buffer_base);

        Ok(())
    }

    /// Get current ring buffer state
    pub fn get_state(&self) -> RingBufferState {
        match self.state.load(Ordering::Acquire) {
            0 => RingBufferState::Uninitialized,
            1 => RingBufferState::Ready,
            2 => RingBufferState::Writing,
            _ => RingBufferState::Error,
        }
    }

    /// Get current write pointer position
    pub fn get_wptr(&self) -> u32 {
        self.wptr.load(Ordering::Acquire)
    }

    /// Get current read pointer position (GPU's consumption point)
    pub fn get_rptr(&self) -> u32 {
        self.rptr.load(Ordering::Acquire)
    }

    /// Update the read pointer from GPU feedback (called by fence/interrupt handler)
    pub fn update_rptr(&self, new_rptr: u32) {
        let old_rptr = self.rptr.swap(new_rptr, Ordering::AcqRel);
        if new_rptr > old_rptr {
            let completed_bytes = new_rptr - old_rptr;
            // Estimate completed commands (rough: 1 cmd per alignment unit)
            let completed_cmds = completed_bytes / RING_BUFFER_ALIGNMENT;
            self.completed_count.fetch_add(completed_cmds as u64, Ordering::Release);
        }
    }

    /// Calculate available space in the ring buffer
    pub fn available_space(&self) -> u32 {
        let wptr = self.wptr.load(Ordering::Acquire);
        let rptr = self.rptr.load(Ordering::Acquire);

        if wptr >= rptr {
            // Write is ahead of read: space = buffer_size - (wptr - rptr) - 1
            self.buffer_size - (wptr - rptr) - 1
        } else {
            // Write has wrapped around: space = rptr - wptr - 1
            rptr - wptr - 1
        }
    }

    /// Submit a single command to the ring buffer
    pub fn submit_command(&self, cmd: &GpuCommand, fence_id: u64) -> Result<(), GpuError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpuError::NotInitialized);
        }

        let pkt = CommandPacket::from_gpu_command(cmd, fence_id);
        let pkt_size = CMD_PKT_HEADER_SIZE + pkt.header.total_size;

        // Align packet size
        let aligned_size = ((pkt_size + RING_BUFFER_ALIGNMENT - 1)
            / RING_BUFFER_ALIGNMENT) * RING_BUFFER_ALIGNMENT;

        // Check available space
        if aligned_size > self.available_space() {
            log_warn!("Ring buffer {}: insufficient space (need={}, avail={})",
                self.ring_id, aligned_size, self.available_space());
            return Err(GpuError::OutOfMemory);
        }

        // Write command packet into ring buffer
        self.write_packet(&pkt, aligned_size)?;

        // Update submission tracking
        self.submitted_count.fetch_add(1, Ordering::Release);

        // Ring doorbell (immediate if batch mode disabled)
        let batch = self.batch_enabled.load(Ordering::Acquire);
        if !batch {
            self.flush();
        } else {
            let bc = self.batch_count.fetch_add(1, Ordering::AcqRel) + 1;
            if bc as usize >= RING_BUFFER_MAX_BATCH {
                self.flush();
            }
        }

        Ok(())
    }

    /// Submit multiple commands as a batch (more efficient than individual submission)
    pub fn submit_commands_batch(&self, cmds: &[GpuCommand], fence_ids: &[u64]) -> Result<u32, GpuError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(GpuError::NotInitialized);
        }

        if cmds.len() != fence_ids.len() {
            return Err(GpuError::InvalidArg);
        }

        if cmds.is_empty() {
            return Ok(0);
        }

        // Set writing state
        self.state.store(RingBufferState::Writing as u32, Ordering::Release);

        let mut submitted: u32 = 0;
        for (i, cmd) in cmds.iter().enumerate() {
            let pkt = CommandPacket::from_gpu_command(cmd, fence_ids[i]);
            let pkt_size = CMD_PKT_HEADER_SIZE + pkt.header.total_size;
            let aligned_size = ((pkt_size + RING_BUFFER_ALIGNMENT - 1)
                / RING_BUFFER_ALIGNMENT) * RING_BUFFER_ALIGNMENT;

            if aligned_size > self.available_space() {
                // Flush what we have so far, then report partial submission
                self.flush();
                if submitted > 0 {
                    return Ok(submitted);
                }
                return Err(GpuError::OutOfMemory);
            }

            if self.write_packet(&pkt, aligned_size).is_err() {
                self.flush();
                if submitted > 0 {
                    return Ok(submitted);
                }
                return Err(GpuError::HardwareError);
            }

            submitted += 1;
        }

        self.submitted_count.fetch_add(submitted as u64, Ordering::Release);
        self.batch_count.fetch_add(submitted, Ordering::Release);

        // Single doorbell ring for entire batch
        self.flush();

        Ok(submitted)
    }

    /// Write a command packet into the ring buffer at the current wptr
    fn write_packet(&self, pkt: &CommandPacket, aligned_size: u32) -> Result<(), GpuError> {
        let wptr = self.wptr.load(Ordering::Acquire);

        // Check for wrap-around
        if wptr + aligned_size > self.buffer_size {
            // Need to wrap: write a NOP pad if there isn't enough room
            let remaining = self.buffer_size - wptr;
            if remaining >= RING_BUFFER_ALIGNMENT {
                // Write NOP packet to fill remaining space
                // SAFETY: writing NOP padding to ring buffer memory
                unsafe {
                    let nop_ptr = (self.buffer_base + wptr as u64) as *mut u32;
                    write_volatile(nop_ptr, 0); // NOP type
                    write_volatile(nop_ptr.add(1), remaining); // NOP size
                }
            }
            // Wrap write pointer to beginning
            self.wptr.store(0, Ordering::Release);
        }

        let wptr = self.wptr.load(Ordering::Acquire);

        // SAFETY: writing command packet into ring buffer memory
        unsafe {
            let base_ptr = (self.buffer_base + wptr as u64) as *mut u32;

            // Write header
            write_volatile(base_ptr, pkt.header.cmd_type);
            write_volatile(base_ptr.add(1), pkt.header.total_size);
            // Write fence_id (64-bit, split into two 32-bit writes)
            write_volatile(base_ptr.add(2), pkt.header.fence_id as u32);
            write_volatile(base_ptr.add(3), (pkt.header.fence_id >> 32) as u32);

            // Write payload
            for i in 0..pkt.payload_len as usize {
                write_volatile(base_ptr.add(4 + i), pkt.payload[i]);
            }
        }

        // Advance write pointer
        let new_wptr = wptr + aligned_size;
        self.wptr.store(new_wptr, Ordering::Release);
        self.state.store(RingBufferState::Writing as u32, Ordering::Release);

        Ok(())
    }

    /// Flush pending commands by ringing the doorbell
    pub fn flush(&self) {
        let wptr = self.wptr.load(Ordering::Acquire);
        self.doorbell.ring(wptr);
        self.batch_count.store(0, Ordering::Release);
        self.state.store(RingBufferState::Ready as u32, Ordering::Release);
    }

    /// Wait for the ring buffer to become idle (all submitted commands completed)
    pub fn wait_idle(&self, timeout_us: u64) -> Result<(), GpuError> {
        let wptr = self.wptr.load(Ordering::Acquire);
        let mut remaining = timeout_us;

        while remaining > 0 {
            // Update rptr from doorbell (GPU's read pointer)
            let gpu_rptr = self.doorbell.read();
            self.update_rptr(gpu_rptr);

            let rptr = self.rptr.load(Ordering::Acquire);
            if rptr == wptr {
                self.state.store(RingBufferState::Ready as u32, Ordering::Release);
                return Ok(());
            }

            core::hint::spin_loop();
            remaining = remaining.saturating_sub(1);
        }

        log_warn!("Ring buffer {}: wait_idle timeout (wptr={}, rptr={})",
            self.ring_id, wptr, self.rptr.load(Ordering::Acquire));
        Err(GpuError::Timeout)
    }

    /// Get current ring position (wptr, rptr)
    pub fn get_ring_position(&self) -> (u32, u32) {
        (self.wptr.load(Ordering::Acquire), self.rptr.load(Ordering::Acquire))
    }

    /// Get the number of submitted commands
    pub fn get_submitted_count(&self) -> u64 {
        self.submitted_count.load(Ordering::Acquire)
    }

    /// Get the number of completed commands
    pub fn get_completed_count(&self) -> u64 {
        self.completed_count.load(Ordering::Acquire)
    }

    /// Enable or disable batch mode
    pub fn set_batch_mode(&self, enabled: bool) {
        self.batch_enabled.store(enabled, Ordering::Release);
    }

    /// Reset the ring buffer (e.g., after GPU reset)
    pub fn reset(&self) {
        self.wptr.store(0, Ordering::Release);
        self.rptr.store(0, Ordering::Release);
        self.batch_count.store(0, Ordering::Release);
        self.submitted_count.store(0, Ordering::Release);
        self.completed_count.store(0, Ordering::Release);
        self.state.store(RingBufferState::Ready as u32, Ordering::Release);
        log_info!("Ring buffer {}: reset", self.ring_id);
    }
}

// ============================================================================
// Ring Buffer Manager (for multi-ring GPUs)
// ============================================================================

/// Maximum number of ring buffers per GPU
pub const MAX_RING_BUFFERS: usize = 8;

/// Ring buffer manager - manages multiple ring buffers for different GPU engines
pub struct RingBufferManager {
    /// Ring buffer slots
    rings: [Option<GpuRingBuffer>; MAX_RING_BUFFERS],
    /// Number of active rings
    num_rings: u32,
}

impl RingBufferManager {
    /// Create a new ring buffer manager
    pub const fn new() -> Self {
        RingBufferManager {
            rings: [None; MAX_RING_BUFFERS],
            num_rings: 0,
        }
    }

    /// Register a ring buffer
    pub fn register(&mut self, ring: GpuRingBuffer) -> Result<u32, GpuError> {
        for (i, slot) in self.rings.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(ring);
                self.num_rings += 1;
                return Ok(i as u32);
            }
        }
        Err(GpuError::OutOfMemory)
    }

    /// Get a ring buffer by ID
    pub fn get_ring(&self, ring_id: u32) -> Option<&GpuRingBuffer> {
        if (ring_id as usize) >= MAX_RING_BUFFERS {
            return None;
        }
        self.rings[ring_id as usize].as_ref()
    }

    /// Get a mutable reference to a ring buffer by ID
    pub fn get_ring_mut(&mut self, ring_id: u32) -> Option<&mut GpuRingBuffer> {
        if (ring_id as usize) >= MAX_RING_BUFFERS {
            return None;
        }
        self.rings[ring_id as usize].as_mut()
    }

    /// Flush all ring buffers
    pub fn flush_all(&self) {
        for slot in &self.rings {
            if let Some(ref ring) = slot {
                ring.flush();
            }
        }
    }

    /// Wait for all ring buffers to become idle
    pub fn wait_all_idle(&self, timeout_us: u64) -> Result<(), GpuError> {
        for slot in &self.rings {
            if let Some(ref ring) = slot {
                ring.wait_idle(timeout_us)?;
            }
        }
        Ok(())
    }

    /// Reset all ring buffers
    pub fn reset_all(&self) {
        for slot in &self.rings {
            if let Some(ref ring) = slot {
                ring.reset();
            }
        }
    }

    /// Get number of active rings
    pub fn num_rings(&self) -> u32 {
        self.num_rings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_creation() {
        let rb = GpuRingBuffer::new(0, 0x1000_0000, RING_BUFFER_DEFAULT_SIZE, 0xF610_0000, 0x0008);
        assert_eq!(rb.ring_id, 0);
        assert_eq!(rb.get_state(), RingBufferState::Uninitialized);
    }

    #[test]
    fn test_doorbell() {
        let db = Doorbell::new(0xF610_0000, 0x0008);
        assert_eq!(db.last_written(), 0);
    }

    #[test]
    fn test_ring_buffer_manager() {
        let mut mgr = RingBufferManager::new();
        assert_eq!(mgr.num_rings(), 0);

        let rb = GpuRingBuffer::new(0, 0x1000_0000, RING_BUFFER_DEFAULT_SIZE, 0xF610_0000, 0x0008);
        let result = mgr.register(rb);
        assert!(result.is_ok());
        assert_eq!(mgr.num_rings(), 1);
    }

    #[test]
    fn test_command_packet() {
        let cmd = GpuCommand {
            cmd_type: super::super::GpuCommandType::Render,
            data: 0x1000,
            size: 256,
            priority: 1,
            sync_obj: 0,
        };
        let pkt = CommandPacket::from_gpu_command(&cmd, 42);
        assert_eq!(pkt.header.fence_id, 42);
        assert_eq!(pkt.header.cmd_type, 0); // Render = 0
    }
}
