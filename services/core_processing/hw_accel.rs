/*
 * Nuva OS - SystemService - CoreProcessing - Hardware Acceleration
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

//! Hardware acceleration with software fallback framework.
//! Provides generic execute_with_fallback logic: try hardware path first,
//! on failure automatically degrade to software path within 100ms.

use core::sync::atomic::{AtomicU32, Ordering};

use super::error::ServiceError;

/// Hardware acceleration state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwAccelState {
    /// Hardware acceleration active
    Active = 0,
    /// Software fallback active
    Fallback = 1,
    /// Hardware not available
    Unavailable = 2,
}

/// Hardware acceleration operation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwAccelResult {
    /// Operation succeeded via hardware
    HardwareSuccess,
    /// Operation succeeded via software fallback
    FallbackSuccess,
    /// Operation failed on both paths
    Failed(ServiceError),
}

/// Hardware acceleration manager - generic fallback logic
pub struct HwAccelManager {
    /// Current state
    state: AtomicU32,
    /// Consecutive hardware failure count
    hw_fail_count: AtomicU32,
    /// Maximum consecutive failures before marking unavailable
    max_fail_threshold: u32,
}

impl HwAccelManager {
    /// Create a new hardware acceleration manager
    pub const fn new(max_fail_threshold: u32) -> Self {
        HwAccelManager {
            state: AtomicU32::new(HwAccelState::Active as u32),
            hw_fail_count: AtomicU32::new(0),
            max_fail_threshold,
        }
    }

    /// Get current acceleration state
    pub fn get_state(&self) -> HwAccelState {
        match self.state.load(Ordering::Acquire) {
            0 => HwAccelState::Active,
            1 => HwAccelState::Fallback,
            2 => HwAccelState::Unavailable,
            _ => HwAccelState::Unavailable,
        }
    }

    /// Execute an operation with hardware/software fallback
    ///
    /// Tries hardware path first. If it fails, automatically degrades
    /// to software path. The fallback transition completes within 100ms.
    pub fn execute_with_fallback<H, S, R>(
        &self,
        hw_op: H,
        sw_op: S,
    ) -> HwAccelResult
    where
        H: FnOnce() -> Result<R, ServiceError>,
        S: FnOnce() -> Result<R, ServiceError>,
    {
        let current_state = self.get_state();

        if current_state == HwAccelState::Unavailable {
            return match sw_op() {
                Ok(_) => HwAccelResult::FallbackSuccess,
                Err(e) => HwAccelResult::Failed(e),
            };
        }

        if current_state == HwAccelState::Active {
            match hw_op() {
                Ok(_) => {
                    self.hw_fail_count.store(0, Ordering::Relaxed);
                    HwAccelResult::HardwareSuccess
                }
                Err(_) => {
                    let fails = self.hw_fail_count.fetch_add(1, Ordering::Relaxed) + 1;
                    if fails >= self.max_fail_threshold {
                        self.state.store(
                            HwAccelState::Unavailable as u32,
                            Ordering::Release,
                        );
                    } else {
                        self.state.store(
                            HwAccelState::Fallback as u32,
                            Ordering::Release,
                        );
                    }
                    pr_warn!("HW accel failed, falling back to software path");
                    match sw_op() {
                        Ok(_) => HwAccelResult::FallbackSuccess,
                        Err(e) => HwAccelResult::Failed(e),
                    }
                }
            }
        } else {
            match sw_op() {
                Ok(_) => HwAccelResult::FallbackSuccess,
                Err(e) => HwAccelResult::Failed(e),
            }
        }
    }

    /// Reset hardware acceleration state (e.g. after device re-initialization)
    pub fn reset(&self) {
        self.state.store(HwAccelState::Active as u32, Ordering::Release);
        self.hw_fail_count.store(0, Ordering::Relaxed);
    }

    /// Mark hardware as available again
    pub fn mark_available(&self) {
        self.state.store(HwAccelState::Active as u32, Ordering::Release);
        self.hw_fail_count.store(0, Ordering::Relaxed);
    }
}
