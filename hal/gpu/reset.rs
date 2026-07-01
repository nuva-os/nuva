/*
 * Nuva OS - HAL - Gpu - Reset Handler
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

/* GPU Hang Detection and Reset Handler - Plugin-level reset, NOT kernel panic.
 *
 * When a GPU hang is detected, this module performs a plugin-level reset
 * rather than crashing the kernel. This is critical for system stability:
 * a GPU hang should never take down the entire OS.
 *
 * Reset escalation path:
 * 1. Soft reset: Reset command processors only (fastest, least disruptive)
 * 2. Hard reset: Full GPU reset including shader cores (slower, more disruptive)
 * 3. Device unavailable: If hard reset fails, mark device as unavailable
 *    and notify userspace. The OS continues running without GPU acceleration.
 *
 * Hang detection heuristics:
 * - Fence timeout: A fence has not been signaled within expected time
 * - Ring buffer stall: Write pointer has not advanced for N checks
 * - Hardware status: GPU reports error via status register
 */

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, AtomicU8, Ordering};

use super::{GpuState, GpuError};

// ============================================================================
// Reset Constants
// ============================================================================

/// Maximum consecutive hang detections before escalation
const MAX_HANG_COUNT_BEFORE_ESCALATION: u32 = 3;

/// Maximum soft reset attempts before trying hard reset
const MAX_SOFT_RESET_ATTEMPTS: u32 = 3;

/// Maximum hard reset attempts before marking device unavailable
const MAX_HARD_RESET_ATTEMPTS: u32 = 2;

/// GPU status poll interval in microseconds
const HANG_POLL_INTERVAL_US: u32 = 1000;

/// Number of consecutive stalled polls to declare a hang
const HANG_STALL_THRESHOLD: u32 = 5;

// ============================================================================
// Reset Register Offsets (Maleoon GPU)
// ============================================================================

/// Soft reset register (per-engine)
const GPU_RESET_SOFT_REG: u64 = 0x0040;

/// Hard reset register (full GPU)
const GPU_RESET_HARD_REG: u64 = 0x0044;

/// Reset status register
const GPU_RESET_STATUS_REG: u64 = 0x0048;

/// Hang detect register (GPU self-reporting)
const GPU_HANG_DETECT_REG: u64 = 0x004C;

/// Engine status register (per-engine idle/busy)
const GPU_ENGINE_STATUS_REG: u64 = 0x0050;

// ============================================================================
// Reset Register Values
// ============================================================================

/// Trigger soft reset 
const RESET_SOFT_TRIGGER: u32 = 0x0000_0001;

/// Trigger hard reset 
const RESET_HARD_TRIGGER: u32 = 0x0000_0001;

/// Reset completed status 
const RESET_STATUS_DONE: u32 = 0x0000_0001;

/// Reset in progress status 
const RESET_STATUS_IN_PROGRESS: u32 = 0x0000_0002;

/// Reset failed status 
const RESET_STATUS_FAILED: u32 = 0x0000_0003;

/// Engine idle status 
const ENGINE_STATUS_IDLE: u32 = 0x0000_0000;

/// Engine busy status 
const ENGINE_STATUS_BUSY: u32 = 0x0000_0001;

/// Engine hung status 
const ENGINE_STATUS_HUNG: u32 = 0x0000_0002;

// ============================================================================
// Hang Detection Result
// ============================================================================

/// Hang detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangStatus {
    /// No hang detected, GPU operating normally
    Normal = 0,
    /// GPU may be stalled (watchdog warning)
    Stalled = 1,
    /// GPU hang confirmed (fence timeout or hardware report)
    Hung = 2,
    /// GPU device is unavailable (reset failed)
    Unavailable = 3,
}

// ============================================================================
// Reset Result
// ============================================================================

/// Reset operation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetResult {
    /// Reset succeeded, GPU is operational
    Success = 0,
    /// Reset partially succeeded (some engines recovered)
    Partial = 1,
    /// Reset failed, device marked unavailable
    Failed = 2,
    /// Reset not attempted (device already unavailable)
    NotAttempted = 3,
}

// ============================================================================
// GPU Reset Handler
// ============================================================================

/// GPU Reset Handler - hang detection and plugin-level reset
///
/// This handler performs GPU reset at the plugin/driver level.
/// It NEVER triggers a kernel panic. If all reset attempts fail,
/// the device is marked as unavailable and the OS continues running
/// without GPU acceleration.
struct GpuResetHandler {
    /// GPU control register base address
    ctrl_base: u64,
    /// Current hang status
    hang_status: AtomicU8,
    /// Consecutive hang detection count
    hang_count: AtomicU32,
    /// Consecutive stall detection count
    stall_count: AtomicU32,
    /// Last known ring buffer write pointer (for stall detection)
    last_wptr: AtomicU32,
    /// Number of soft resets performed
    soft_reset_count: AtomicU32,
    /// Number of hard resets performed
    hard_reset_count: AtomicU32,
    /// Total hang detections (cumulative, for diagnostics)
    total_hangs: AtomicU64,
    /// Device is marked unavailable (all resets failed)
    device_unavailable: AtomicBool,
    /// Reset is currently in progress
    reset_in_progress: AtomicBool,
    /// Current GPU state reference
    gpu_state: AtomicU32,
}
impl GpuResetHandler {
    /// Create a new reset handler
    pub const fn new(ctrl_base: u64) -> Self {
        GpuResetHandler {
            ctrl_base,
            hang_status: AtomicU8::new(HangStatus::Normal as u8),
            hang_count: AtomicU32::new(0),
            stall_count: AtomicU32::new(0),
            last_wptr: AtomicU32::new(0),
            soft_reset_count: AtomicU32::new(0),
            hard_reset_count: AtomicU32::new(0),
            total_hangs: AtomicU64::new(0),
            device_unavailable: AtomicBool::new(false),
            reset_in_progress: AtomicBool::new(false),
            gpu_state: AtomicU32::new(GpuState::Idle as u32),
        }
    }

    /// Get current hang status
    pub fn get_hang_status(&self) -> HangStatus {
        match self.hang_status.load(Ordering::Acquire) {
            0 => HangStatus::Normal,
            1 => HangStatus::Stalled,
            2 => HangStatus::Hung,
            _ => HangStatus::Unavailable,
        }
    }

    /// Check if device is marked unavailable
    pub fn is_device_unavailable(&self) -> bool {
        self.device_unavailable.load(Ordering::Acquire)
    }

    /// Detect GPU hang by checking hardware status and ring buffer progress
    ///
    /// This should be called periodically (e.g., from a watchdog timer or
    /// work queue). It does NOT trigger any reset itself - it only updates
    /// the hang status. Call handle_hang() to perform recovery.
    pub fn detect_hang(&self, current_wptr: u32, fence_timeout: bool) -> HangStatus {
        if self.device_unavailable.load(Ordering::Acquire) {
            return HangStatus::Unavailable;
        }
        if self.reset_in_progress.load(Ordering::Acquire) {
            return self.get_hang_status();
        }
        // Check 1: Hardware self-reported hang
        // SAFETY: reading GPU hang detect register
        let hw_hang = unsafe {
            let val = read_volatile((self.ctrl_base + GPU_HANG_DETECT_REG) as *const u32);
            val != 0
        };
        if hw_hang {
            self.report_hang();
            return HangStatus::Hung;
        }
        // Check 2: Fence timeout (caller reports this)
        if fence_timeout {
            self.report_hang();
            return HangStatus::Hung;
        }
        // Check 3: Ring buffer stall detection
        let last_wptr = self.last_wptr.load(Ordering::Acquire);
        if current_wptr == last_wptr && current_wptr != 0 {
            let stall = self.stall_count.fetch_add(1, Ordering::AcqRel) + 1;
            // SAFETY: reading GPU engine status register
            let engine_busy = unsafe {
                let val = read_volatile((self.ctrl_base + GPU_ENGINE_STATUS_REG) as *const u32);
                val == ENGINE_STATUS_BUSY || val == ENGINE_STATUS_HUNG
            };
            if engine_busy && stall >= HANG_STALL_THRESHOLD {
                self.report_hang();
                return HangStatus::Hung;
            } else if stall >= HANG_STALL_THRESHOLD {
                self.hang_status.store(HangStatus::Stalled as u8, Ordering::Release);
                return HangStatus::Stalled;
            }
        } else {
            self.stall_count.store(0, Ordering::Release);
        }
        self.last_wptr.store(current_wptr, Ordering::Release);
        HangStatus::Normal
    }

    /// Report a hang (internal)
    fn report_hang(&self) {
        self.hang_count.fetch_add(1, Ordering::AcqRel);
        self.total_hangs.fetch_add(1, Ordering::Release);
        self.hang_status.store(HangStatus::Hung as u8, Ordering::Release);
        self.gpu_state.store(GpuState::Error as u32, Ordering::Release);
        log_warn!("GPU: Hang detected (total={}, consecutive={})", self.total_hangs.load(Ordering::Acquire), self.hang_count.load(Ordering::Acquire));
    }

    /// Handle a detected hang by performing reset escalation
    ///
    /// Reset escalation path:
    /// 1. Soft reset (up to MAX_SOFT_RESET_ATTEMPTS)
    /// 2. Hard reset (up to MAX_HARD_RESET_ATTEMPTS)
    /// 3. Mark device unavailable (graceful degradation)
    ///
    /// This function NEVER panics the kernel.
    pub fn handle_hang(&self) -> ResetResult {
        if self.device_unavailable.load(Ordering::Acquire) {
            log_warn!("GPU: Device unavailable, reset not attempted");
            return ResetResult::NotAttempted;
        }
        if self.reset_in_progress.swap(true, Ordering::AcqRel) {
            log_debug!("GPU: Reset already in progress");
            return ResetResult::NotAttempted;
        }
        let hang_count = self.hang_count.load(Ordering::Acquire);
        let result = if hang_count <= MAX_HANG_COUNT_BEFORE_ESCALATION {
            self.soft_reset()
        } else {
            self.hard_reset()
        };
        self.reset_in_progress.store(false, Ordering::Release);
        if result == ResetResult::Success {
            self.hang_count.store(0, Ordering::Release);
            self.stall_count.store(0, Ordering::Release);
            self.hang_status.store(HangStatus::Normal as u8, Ordering::Release);
            self.gpu_state.store(GpuState::Idle as u32, Ordering::Release);
        }
        result
    }

    /// Perform a soft reset (reset command processors only)
    ///
    /// Soft reset is the least disruptive: it only resets the GPU command
    /// processing engines while keeping shader core state intact where
    /// possible. This is faster and less likely to cause visible glitches.
    pub fn soft_reset(&self) -> ResetResult {
        let attempts = self.soft_reset_count.load(Ordering::Acquire);
        if attempts >= MAX_SOFT_RESET_ATTEMPTS {
            log_warn!("GPU: Max soft reset attempts reached, escalating to hard reset");
            return self.hard_reset();
        }
        log_info!("GPU: Performing soft reset (attempt {}/{})", attempts + 1, MAX_SOFT_RESET_ATTEMPTS);
        // SAFETY: writing to GPU soft reset register
        unsafe {
            write_volatile((self.ctrl_base + GPU_RESET_SOFT_REG) as *mut u32, RESET_SOFT_TRIGGER);
            let mut timeout = 100_000;
            while timeout > 0 {
                let status = read_volatile((self.ctrl_base + GPU_RESET_STATUS_REG) as *const u32);
                if status == RESET_STATUS_DONE {
                    self.soft_reset_count.fetch_add(1, Ordering::Release);
                    log_info!("GPU: Soft reset completed successfully");
                    return ResetResult::Success;
                }
                if status == RESET_STATUS_FAILED { break; }
                core::hint::spin_loop();
                timeout -= 1;
            }
        }
        self.soft_reset_count.fetch_add(1, Ordering::Release);
        log_warn!("GPU: Soft reset timed out or failed");
        let attempts = self.soft_reset_count.load(Ordering::Acquire);
        if attempts >= MAX_SOFT_RESET_ATTEMPTS { self.hard_reset() } else { ResetResult::Failed }
    }

    /// Perform a hard reset (full GPU reset including shader cores)
    ///
    /// Hard reset is more disruptive: it resets the entire GPU including
    /// all shader cores, command processors, and internal state. This
    /// requires re-initialization of the GPU after reset.
    pub fn hard_reset(&self) -> ResetResult {
        let attempts = self.hard_reset_count.load(Ordering::Acquire);
        if attempts >= MAX_HARD_RESET_ATTEMPTS {
            log_warn!("GPU: Max hard reset attempts reached, marking device unavailable");
            self.mark_unavailable();
            return ResetResult::Failed;
        }
        log_info!("GPU: Performing hard reset (attempt {}/{})", attempts + 1, MAX_HARD_RESET_ATTEMPTS);
        // SAFETY: writing to GPU hard reset register
        unsafe {
            write_volatile((self.ctrl_base + GPU_RESET_HARD_REG) as *mut u32, RESET_HARD_TRIGGER);
            let mut timeout = 500_000;
            while timeout > 0 {
                let status = read_volatile((self.ctrl_base + GPU_RESET_STATUS_REG) as *const u32);
                if status == RESET_STATUS_DONE {
                    self.hard_reset_count.fetch_add(1, Ordering::Release);
                    log_info!("GPU: Hard reset completed successfully");
                    return ResetResult::Success;
                }
                if status == RESET_STATUS_FAILED { break; }
                core::hint::spin_loop();
                timeout -= 1;
            }
        }
        self.hard_reset_count.fetch_add(1, Ordering::Release);
        log_warn!("GPU: Hard reset timed out or failed");
        let attempts = self.hard_reset_count.load(Ordering::Acquire);
        if attempts >= MAX_HARD_RESET_ATTEMPTS { self.mark_unavailable(); ResetResult::Failed } else { ResetResult::Failed }
    }

    /// Mark the GPU device as unavailable (graceful degradation)
    ///
    /// When all reset attempts have failed, we mark the device as
    /// unavailable rather than panicking the kernel. Userspace will
    /// be notified and can fall back to software rendering.
    fn mark_unavailable(&self) {
        self.device_unavailable.store(true, Ordering::Release);
        self.hang_status.store(HangStatus::Unavailable as u8, Ordering::Release);
        self.gpu_state.store(GpuState::Error as u32, Ordering::Release);
        log_crit!("GPU: Device marked UNAVAILABLE after all reset attempts failed");
        log_crit!("GPU: System will continue without GPU acceleration");
        // NOTE: We do NOT panic. The OS continues running.
    }

    /// Get GPU state
    pub fn get_gpu_state(&self) -> GpuState {
        match self.gpu_state.load(Ordering::Acquire) {
            0 => GpuState::Idle,
            1 => GpuState::Running,
            2 => GpuState::Suspended,
            _ => GpuState::Error,
        }
    }

    /// Get diagnostic information
    pub fn diagnostics(&self) -> GpuResetDiagnostics {
        GpuResetDiagnostics {
            hang_status: self.get_hang_status(),
            consecutive_hangs: self.hang_count.load(Ordering::Acquire),
            total_hangs: self.total_hangs.load(Ordering::Acquire),
            soft_reset_count: self.soft_reset_count.load(Ordering::Acquire),
            hard_reset_count: self.hard_reset_count.load(Ordering::Acquire),
            stall_count: self.stall_count.load(Ordering::Acquire),
            device_unavailable: self.device_unavailable.load(Ordering::Acquire),
        }
    }

    /// Reset the reset handler state (e.g., after successful GPU re-initialization)
    pub fn reset_state(&self) {
        self.hang_count.store(0, Ordering::Release);
        self.stall_count.store(0, Ordering::Release);
        self.soft_reset_count.store(0, Ordering::Release);
        self.hard_reset_count.store(0, Ordering::Release);
        self.hang_status.store(HangStatus::Normal as u8, Ordering::Release);
        self.gpu_state.store(GpuState::Idle as u32, Ordering::Release);
        self.device_unavailable.store(false, Ordering::Release);
        log_info!("GPU: Reset handler state cleared");
    }
}

// ============================================================================
// Diagnostics
// ============================================================================

/// GPU reset diagnostic information
#[derive(Debug, Clone, Copy)]
pub struct GpuResetDiagnostics {
    /// Current hang status
    pub hang_status: HangStatus,
    /// Consecutive hang count
    pub consecutive_hangs: u32,
    /// Total hang count (cumulative)
    pub total_hangs: u64,
    /// Number of soft resets performed
    pub soft_reset_count: u32,
    /// Number of hard resets performed
    pub hard_reset_count: u32,
    /// Current stall count
    pub stall_count: u32,
    /// Device is unavailable
    pub device_unavailable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
use core::sync::atomic::AtomicU8;
    #[test]
    fn test_reset_handler_creation() {
        let handler = GpuResetHandler::new(0xF600_0000);
        assert_eq!(handler.get_hang_status(), HangStatus::Normal);
        assert!(!handler.is_device_unavailable());
    }
    #[test]
    fn test_hang_status_values() {
        assert_eq!(HangStatus::Normal as u8, 0);
        assert_eq!(HangStatus::Stalled as u8, 1);
        assert_eq!(HangStatus::Hung as u8, 2);
        assert_eq!(HangStatus::Unavailable as u8, 3);
    }
    #[test]
    fn test_reset_result_values() {
        assert_eq!(ResetResult::Success as u8, 0);
        assert_eq!(ResetResult::Partial as u8, 1);
        assert_eq!(ResetResult::Failed as u8, 2);
        assert_eq!(ResetResult::NotAttempted as u8, 3);
    }
    #[test]
    fn test_diagnostics() {
        let handler = GpuResetHandler::new(0xF600_0000);
        let diag = handler.diagnostics();
        assert_eq!(diag.hang_status, HangStatus::Normal);
        assert_eq!(diag.total_hangs, 0);
        assert!(!diag.device_unavailable);
    }
    #[test]
    fn test_reset_handler_state_reset() {
        let handler = GpuResetHandler::new(0xF600_0000);
        handler.reset_state();
        assert_eq!(handler.get_hang_status(), HangStatus::Normal);
        assert!(!handler.is_device_unavailable());
    }
}
