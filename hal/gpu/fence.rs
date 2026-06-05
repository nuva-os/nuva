/*
 * Nuva OS - HAL - Gpu - Fence Synchronization
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

/* GPU Fence Synchronization - Hybrid timeline + syncpoint fence mechanism.
 *
 * Two fence strategies are supported:
 * 1. Timeline fences: Monotonically increasing sequence numbers, allowing
 *    multiple operations to share a single fence with different points.
 *    Efficient for command streams with implicit ordering.
 * 2. Syncpoint fences: Individual per-operation fences for fine-grained
 *    synchronization when operations have no implicit ordering.
 *
 * The hybrid approach selects the optimal strategy based on usage pattern:
 * - Sequential command streams use timeline fences (lower overhead)
 * - Independent/parallel operations use syncpoint fences (more flexible)
 */

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, AtomicBool, Ordering};

use super::GpuError;

// ============================================================================
// Fence Constants
// ============================================================================

/// Maximum number of fence contexts (timeline fences)
pub const MAX_FENCE_CONTEXTS: usize = 32;

/// Maximum number of syncpoint fences
pub const MAX_SYNCPOINT_FENCES: usize = 256;

/// Fence wait spin iterations before yielding
pub const FENCE_SPIN_ITERATIONS: u32 = 100;

// ============================================================================
// Fence Type
// ============================================================================

/// Fence strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceType {
    /// Timeline fence - monotonically increasing sequence number
    Timeline = 0,
    /// Syncpoint fence - individual per-operation fence
    Syncpoint = 1,
}

// ============================================================================
// Timeline Fence
// ============================================================================

/// Timeline fence context - supports multiple wait points on a single sequence
pub struct TimelineFence {
    /// Context ID
    pub context_id: u32,
    /// Current signaled value (GPU has completed up to this point)
    signaled_value: AtomicU64,
    /// Last submitted value (CPU has submitted up to this point)
    submitted_value: AtomicU64,
    /// Hardware fence register address (MMIO)
    hw_reg_addr: u64,
    /// Context is active
    active: AtomicBool,
}
impl TimelineFence {
    /// Create a new timeline fence context
    pub const fn new(context_id: u32, hw_reg_addr: u64) -> Self {
        TimelineFence {
            context_id,
            signaled_value: AtomicU64::new(0),
            submitted_value: AtomicU64::new(0),
            hw_reg_addr,
            active: AtomicBool::new(false),
        }
    }

    /// Activate the timeline fence
    pub fn activate(&self) {
        self.signaled_value.store(0, Ordering::Release);
        self.submitted_value.store(0, Ordering::Release);
        self.active.store(true, Ordering::Release);
    }

    /// Deactivate the timeline fence
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
    }

    /// Get the next submission point (advances submitted_value)
    pub fn next_submission(&self) -> u64 {
        self.submitted_value.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// Signal the fence up to a given value (called from GPU interrupt/IRQ)
    pub fn signal(&self, value: u64) {
        let current = self.signaled_value.load(Ordering::Acquire);
        // Only advance, never go backward
        if value > current {
            self.signaled_value.store(value, Ordering::Release);
        }
    }

    /// Update signaled value from hardware register
    pub fn update_from_hw(&self) {
        // SAFETY: reading GPU fence value register
        let hw_value = unsafe {
            read_volatile(self.hw_reg_addr as *const u64)
        };
        self.signal(hw_value);
    }

    /// Check if a specific point has been signaled
    pub fn is_signaled(&self, value: u64) -> bool {
        self.update_from_hw();
        self.signaled_value.load(Ordering::Acquire) >= value
    }

    /// Wait for a specific point to be signaled
    pub fn wait(&self, value: u64, timeout_us: u64) -> Result<(), GpuError> {
        let mut remaining = timeout_us;
        loop {
            if self.is_signaled(value) {
                return Ok(());
            }
            if remaining == 0 {
                return Err(GpuError::Timeout);
            }
            for _ in 0..FENCE_SPIN_ITERATIONS {
                core::hint::spin_loop();
            }
            remaining = remaining.saturating_sub(1);
        }
    }

    /// Get current signaled value
    pub fn signaled_value(&self) -> u64 {
        self.signaled_value.load(Ordering::Acquire)
    }

    /// Get current submitted value
    pub fn submitted_value(&self) -> u64 {
        self.submitted_value.load(Ordering::Acquire)
    }
}

// ============================================================================
// Syncpoint Fence
// ============================================================================

/// Syncpoint fence state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncpointState {
    /// Not yet signaled
    Pending = 0,
    /// Signaled (operation complete)
    Signaled = 1,
    /// Error occurred
    Error = 2,
}

/// Syncpoint fence - individual per-operation fence
pub struct SyncpointFence {
    /// Fence ID
    pub id: u64,
    /// Fence state
    state: AtomicU8,
    /// Associated command buffer ID
    pub cmd_buf_id: u32,
    /// Ring buffer ID that this fence belongs to
    pub ring_id: u32,
    /// Timestamp when signaled (GPU clock)
    timestamp: AtomicU64,
    /// Hardware syncpoint register address
    hw_reg_addr: u64,
}
impl SyncpointFence {
    /// Create a new syncpoint fence
    pub const fn new(id: u64, cmd_buf_id: u32, ring_id: u32, hw_reg_addr: u64) -> Self {
        SyncpointFence {
            id,
            state: AtomicU8::new(SyncpointState::Pending as u8),
            cmd_buf_id,
            ring_id,
            timestamp: AtomicU64::new(0),
            hw_reg_addr,
        }
    }

    /// Get fence state
    pub fn get_state(&self) -> SyncpointState {
        match self.state.load(Ordering::Acquire) {
            0 => SyncpointState::Pending,
            1 => SyncpointState::Signaled,
            _ => SyncpointState::Error,
        }
    }

    /// Check if fence is signaled
    pub fn is_signaled(&self) -> bool {
        if self.get_state() == SyncpointState::Pending {
            self.update_from_hw();
        }
        self.get_state() == SyncpointState::Signaled
    }

    /// Signal the fence (called from GPU interrupt handler)
    pub fn signal(&self, timestamp: u64) {
        self.timestamp.store(timestamp, Ordering::Release);
        self.state.store(SyncpointState::Signaled as u8, Ordering::Release);
    }

    /// Set fence to error state
    pub fn set_error(&self) {
        self.state.store(SyncpointState::Error as u8, Ordering::Release);
    }

    /// Wait for the fence to be signaled
    pub fn wait(&self, timeout_us: u64) -> Result<(), GpuError> {
        let mut remaining = timeout_us;
        loop {
            if self.is_signaled() {
                return Ok(());
            }
            if self.get_state() == SyncpointState::Error {
                return Err(GpuError::HardwareError);
            }
            if remaining == 0 {
                return Err(GpuError::Timeout);
            }
            for _ in 0..FENCE_SPIN_ITERATIONS {
                core::hint::spin_loop();
            }
            remaining = remaining.saturating_sub(1);
        }
    }

    /// Update state from hardware register
    fn update_from_hw(&self) {
        if self.hw_reg_addr == 0 {
            return;
        }
        // SAFETY: reading GPU syncpoint status register
        let hw_val = unsafe {
            read_volatile(self.hw_reg_addr as *const u32)
        };
        if hw_val != 0 {
            self.signal(hw_val as u64);
        }
    }

    /// Get timestamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp.load(Ordering::Acquire)
    }

    /// Reset fence for reuse
    pub fn reset(&self) {
        self.state.store(SyncpointState::Pending as u8, Ordering::Release);
        self.timestamp.store(0, Ordering::Release);
    }
}

// ============================================================================
// Hybrid Fence (unified interface)
// ============================================================================

/// Hybrid fence - wraps either timeline or syncpoint fence
pub struct HybridFence {
    /// Fence type
    pub fence_type: FenceType,
    /// Timeline context ID (if FenceType::Timeline)
    pub timeline_context: u32,
    /// Timeline value (if FenceType::Timeline)
    pub timeline_value: u64,
    /// Syncpoint fence ID (if FenceType::Syncpoint)
    pub syncpoint_id: u64,
}
impl HybridFence {
    /// Create a timeline-based hybrid fence
    pub const fn timeline(context: u32, value: u64) -> Self {
        HybridFence {
            fence_type: FenceType::Timeline,
            timeline_context: context,
            timeline_value: value,
            syncpoint_id: 0,
        }
    }

    /// Create a syncpoint-based hybrid fence
    pub const fn syncpoint(id: u64) -> Self {
        HybridFence {
            fence_type: FenceType::Syncpoint,
            timeline_context: 0,
            timeline_value: 0,
            syncpoint_id: id,
        }
    }
}

// ============================================================================
// GPU Fence Manager (Hybrid)
// ============================================================================

/// GPU fence manager - manages both timeline and syncpoint fences
pub struct GpuFenceManager {
    /// Timeline fence contexts
    timelines: [Option<TimelineFence>; MAX_FENCE_CONTEXTS],
    /// Syncpoint fence pool
    syncpoints: [SyncpointFence; MAX_SYNCPOINT_FENCES],
    /// Next syncpoint fence ID
    next_syncpoint_id: AtomicU64,
    /// Number of active syncpoint fences
    active_syncpoints: AtomicU32,
    /// Number of active timeline contexts
    active_timelines: AtomicU32,
}
impl GpuFenceManager {
    /// Create a new fence manager
    pub fn new() -> Self {
        let syncpoints: [SyncpointFence; MAX_SYNCPOINT_FENCES] = unsafe {
            core::mem::zeroed()
        };
        GpuFenceManager {
            timelines: [None; MAX_FENCE_CONTEXTS],
            syncpoints,
            next_syncpoint_id: AtomicU64::new(1),
            active_syncpoints: AtomicU32::new(0),
            active_timelines: AtomicU32::new(0),
        }
    }

    /// Create a timeline fence context
    pub fn create_timeline(&mut self, hw_reg_addr: u64) -> Result<u32, GpuError> {
        for (i, slot) in self.timelines.iter_mut().enumerate() {
            if slot.is_none() {
                let timeline = TimelineFence::new(i as u32, hw_reg_addr);
                timeline.activate();
                *slot = Some(timeline);
                self.active_timelines.fetch_add(1, Ordering::Release);
                log_info!("Fence: created timeline context {} (hw=0x{:X})", i, hw_reg_addr);
                return Ok(i as u32);
            }
        }
        Err(GpuError::OutOfMemory)
    }

    /// Get a timeline fence context
    pub fn get_timeline(&self, context_id: u32) -> Option<&TimelineFence> {
        if (context_id as usize) >= MAX_FENCE_CONTEXTS {
            return None;
        }
        self.timelines[context_id as usize].as_ref()
    }

    /// Submit to a timeline fence (get next value)
    pub fn timeline_submit(&self, context_id: u32) -> Result<u64, GpuError> {
        let timeline = self.get_timeline(context_id).ok_or(GpuError::InvalidArg)?;
        Ok(timeline.next_submission())
    }

    /// Wait on a timeline fence
    pub fn timeline_wait(&self, context_id: u32, value: u64, timeout_us: u64) -> Result<(), GpuError> {
        let timeline = self.get_timeline(context_id).ok_or(GpuError::InvalidArg)?;
        timeline.wait(value, timeout_us)
    }

    /// Create a syncpoint fence
    pub fn create_syncpoint(&mut self, cmd_buf_id: u32, ring_id: u32, hw_reg_addr: u64) -> Result<u64, GpuError> {
        let fence_id = self.next_syncpoint_id.fetch_add(1, Ordering::AcqRel);
        for slot in &mut self.syncpoints {
            let state = slot.get_state();
            if state == SyncpointState::Signaled || state == SyncpointState::Error || slot.id == 0 {
                *slot = SyncpointFence::new(fence_id, cmd_buf_id, ring_id, hw_reg_addr);
                self.active_syncpoints.fetch_add(1, Ordering::Release);
                return Ok(fence_id);
            }
        }
        Err(GpuError::OutOfMemory)
    }

    /// Signal a syncpoint fence (called from GPU IRQ handler)
    pub fn signal_syncpoint(&self, fence_id: u64, timestamp: u64) -> Result<(), GpuError> {
        for slot in &self.syncpoints {
            if slot.id == fence_id {
                slot.signal(timestamp);
                self.active_syncpoints.fetch_sub(1, Ordering::Release);
                return Ok(());
            }
        }
        Err(GpuError::InvalidArg)
    }

    /// Wait on a syncpoint fence
    pub fn wait_syncpoint(&self, fence_id: u64, timeout_us: u64) -> Result<(), GpuError> {
        for slot in &self.syncpoints {
            if slot.id == fence_id {
                return slot.wait(timeout_us);
            }
        }
        Err(GpuError::InvalidArg)
    }

    /// Wait on a hybrid fence (dispatches to timeline or syncpoint)
    pub fn wait_fence(&self, fence: &HybridFence, timeout_us: u64) -> Result<(), GpuError> {
        match fence.fence_type {
            FenceType::Timeline => self.timeline_wait(fence.timeline_context, fence.timeline_value, timeout_us),
            FenceType::Syncpoint => self.wait_syncpoint(fence.syncpoint_id, timeout_us),
        }
    }

    /// Check if a hybrid fence is signaled
    pub fn is_fence_signaled(&self, fence: &HybridFence) -> bool {
        match fence.fence_type {
            FenceType::Timeline => {
                if let Some(tl) = self.get_timeline(fence.timeline_context) {
                    tl.is_signaled(fence.timeline_value)
                } else { false }
            }
            FenceType::Syncpoint => {
                for slot in &self.syncpoints {
                    if slot.id == fence.syncpoint_id { return slot.is_signaled(); }
                }
                false
            }
        }
    }

    /// Signal a timeline fence (called from GPU IRQ handler)
    pub fn signal_timeline(&self, context_id: u32, value: u64) -> Result<(), GpuError> {
        let timeline = self.get_timeline(context_id).ok_or(GpuError::InvalidArg)?;
        timeline.signal(value);
        Ok(())
    }

    /// Get number of active syncpoint fences
    pub fn active_syncpoint_count(&self) -> u32 { self.active_syncpoints.load(Ordering::Acquire) }

    /// Get number of active timeline contexts
    pub fn active_timeline_count(&self) -> u32 { self.active_timelines.load(Ordering::Acquire) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_timeline_fence() {
        let tl = TimelineFence::new(0, 0xF630_0000);
        assert_eq!(tl.context_id, 0);
        assert_eq!(tl.signaled_value(), 0);
        assert_eq!(tl.submitted_value(), 0);
    }
    #[test]
    fn test_timeline_fence_signal() {
        let tl = TimelineFence::new(0, 0);
        tl.activate();
        tl.signal(5);
        assert!(tl.is_signaled(3));
        assert!(tl.is_signaled(5));
        assert!(!tl.is_signaled(6));
    }
    #[test]
    fn test_syncpoint_fence() {
        let sp = SyncpointFence::new(1, 0, 0, 0);
        assert_eq!(sp.id, 1);
        assert_eq!(sp.get_state(), SyncpointState::Pending);
        assert!(!sp.is_signaled());
    }
    #[test]
    fn test_syncpoint_fence_signal() {
        let sp = SyncpointFence::new(1, 0, 0, 0);
        sp.signal(12345);
        assert!(sp.is_signaled());
        assert_eq!(sp.timestamp(), 12345);
    }
    #[test]
    fn test_hybrid_fence() {
        let tf = HybridFence::timeline(0, 5);
        assert_eq!(tf.fence_type, FenceType::Timeline);
        assert_eq!(tf.timeline_value, 5);
        let sf = HybridFence::syncpoint(42);
        assert_eq!(sf.fence_type, FenceType::Syncpoint);
        assert_eq!(sf.syncpoint_id, 42);
    }
    #[test]
    fn test_fence_manager() {
        let mut mgr = GpuFenceManager::new();
        assert_eq!(mgr.active_timeline_count(), 0);
        assert_eq!(mgr.active_syncpoint_count(), 0);
    }
}
