/*
 * Nuva OS - HAL - Power Regulator Core
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

//! Regulator Core
//! Reference-counted voltage/current regulator with over-temperature
//! and over-current protection that cannot be disabled.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use crate::{pr_info, pr_warn, pr_err, pr_debug};

// ============================================================================
// Regulator Protection
// ============================================================================

/// Protection flags for a regulator.
#[derive(Debug, Clone, Copy)]
pub struct RegulatorProtection {
    /// Over-temperature protection enabled (cannot be disabled).
    pub over_temp: bool,
    /// Over-current protection enabled (cannot be disabled).
    pub over_current: bool,
    /// Under-voltage lockout.
    pub under_voltage: bool,
}

impl RegulatorProtection {
    /// Default protection: over-temp and over-current always on.
    pub const fn new() -> Self {
        RegulatorProtection {
            over_temp: true,
            over_current: true,
            under_voltage: true,
        }
    }
}

// ============================================================================
// Regulator State
// ============================================================================

/// Regulator operational state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegulatorState {
    /// Off.
    Off = 0,
    /// On and regulating.
    On = 1,
    /// Error (fault detected).
    Error = 2,
    /// Bypass mode (pass-through).
    Bypass = 3,
}

// ============================================================================
// Regulator Core
// ============================================================================

/// Core regulator abstraction with reference counting and protection.
///
/// - enable_count: atomic reference count; regulator is on when > 0.
/// - Over-temperature and over-current protection CANNOT be disabled.
pub struct RegulatorCore {
    /// Regulator name.
    pub name: &'static str,
    /// Regulator ID.
    pub id: u32,
    /// Enable reference count.
    pub enable_count: AtomicU32,
    /// Current voltage in microvolts.
    pub voltage_uv: AtomicU32,
    /// Minimum voltage in microvolts.
    pub min_voltage_uv: u32,
    /// Maximum voltage in microvolts.
    pub max_voltage_uv: u32,
    /// Current load in microamps.
    pub current_ua: AtomicU32,
    /// Maximum current in microamps.
    pub max_current_ua: u32,
    /// Operational state.
    pub state: AtomicU32,
    /// Protection configuration.
    pub protection: RegulatorProtection,
    /// Over-temperature fault active.
    pub otemp_fault: AtomicBool,
    /// Over-current fault active.
    pub ocurr_fault: AtomicBool,
    /// Always-on flag (system-critical, cannot be disabled).
    pub always_on: bool,
    /// Total enable operations.
    pub enable_ops: AtomicU64,
    /// Total disable operations.
    pub disable_ops: AtomicU64,
}

impl RegulatorCore {
    /// Create a new regulator core.
    pub const fn new(
        name: &'static str,
        id: u32,
        min_voltage_uv: u32,
        max_voltage_uv: u32,
        max_current_ua: u32,
        always_on: bool,
    ) -> Self {
        RegulatorCore {
            name,
            id,
            enable_count: AtomicU32::new(0),
            voltage_uv: AtomicU32::new(0),
            min_voltage_uv,
            max_voltage_uv,
            current_ua: AtomicU32::new(0),
            max_current_ua,
            state: AtomicU32::new(RegulatorState::Off as u32),
            protection: RegulatorProtection::new(),
            otemp_fault: AtomicBool::new(false),
            ocurr_fault: AtomicBool::new(false),
            always_on,
            enable_ops: AtomicU64::new(0),
            disable_ops: AtomicU64::new(0),
        }
    }

    /// Enable the regulator (increment reference count).
    ///
    /// Returns the new reference count.
    pub fn enable(&self) -> u32 {
        let count = self.enable_count.fetch_add(1, Ordering::AcqRel) + 1;
        self.enable_ops.fetch_add(1, Ordering::Relaxed);
        if count == 1 {
            self.state.store(RegulatorState::On as u32, Ordering::Release);
            log_debug!("Regulator '{}' enabled (first ref)", self.name);
        }
        count
    }

    /// Disable the regulator (decrement reference count).
    ///
    /// Returns the new reference count. If lways_on is true or
    /// protection faults are active, the disable is refused and
    /// the current count is returned.
    pub fn disable(&self) -> u32 {
        // Cannot disable if always-on
        if self.always_on {
            log_warn!("Regulator '{}' is always-on, cannot disable", self.name);
            return self.enable_count.load(Ordering::Acquire);
        }
        // Cannot disable if protection faults are active
        if self.otemp_fault.load(Ordering::Acquire) || self.ocurr_fault.load(Ordering::Acquire) {
            log_warn!("Regulator '{}' has active fault, cannot disable", self.name);
            return self.enable_count.load(Ordering::Acquire);
        }
        let current = self.enable_count.load(Ordering::Acquire);
        if current == 0 {
            return 0;
        }
        let count = self.enable_count.fetch_sub(1, Ordering::AcqRel) - 1;
        self.disable_ops.fetch_add(1, Ordering::Relaxed);
        if count == 0 {
            self.state.store(RegulatorState::Off as u32, Ordering::Release);
            log_debug!("Regulator '{}' disabled (last ref)", self.name);
        }
        count
    }

    /// Check if the regulator is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enable_count.load(Ordering::Acquire) > 0
    }

    /// Set voltage in microvolts. Clamped to [min, max].
    pub fn set_voltage(&self, voltage_uv: u32) -> u32 {
        let clamped = voltage_uv.max(self.min_voltage_uv).min(self.max_voltage_uv);
        self.voltage_uv.store(clamped, Ordering::Release);
        clamped
    }

    /// Get voltage in microvolts.
    pub fn get_voltage(&self) -> u32 {
        self.voltage_uv.load(Ordering::Acquire)
    }

    /// Report over-temperature fault.
    ///
    /// Over-temperature protection CANNOT be disabled. When triggered,
    /// the regulator is forced off.
    pub fn report_over_temp(&self) {
        if self.protection.over_temp {
            self.otemp_fault.store(true, Ordering::Release);
            self.state.store(RegulatorState::Error as u32, Ordering::Release);
            log_err!("Regulator '{}' OVER-TEMPERATURE fault! Forced off.", self.name);
        }
    }

    /// Report over-current fault.
    ///
    /// Over-current protection CANNOT be disabled. When triggered,
    /// the regulator is forced off.
    pub fn report_over_current(&self) {
        if self.protection.over_current {
            self.ocurr_fault.store(true, Ordering::Release);
            self.state.store(RegulatorState::Error as u32, Ordering::Release);
            log_err!("Regulator '{}' OVER-CURRENT fault! Forced off.", self.name);
        }
    }

    /// Clear over-temperature fault (only after hardware reset).
    pub fn clear_over_temp(&self) {
        self.otemp_fault.store(false, Ordering::Release);
        if !self.ocurr_fault.load(Ordering::Acquire) {
            self.state.store(RegulatorState::Off as u32, Ordering::Release);
        }
    }

    /// Clear over-current fault (only after hardware reset).
    pub fn clear_over_current(&self) {
        self.ocurr_fault.store(false, Ordering::Release);
        if !self.otemp_fault.load(Ordering::Acquire) {
            self.state.store(RegulatorState::Off as u32, Ordering::Release);
        }
    }

    /// Get the operational state.
    pub fn get_state(&self) -> RegulatorState {
        match self.state.load(Ordering::Acquire) {
            0 => RegulatorState::Off,
            1 => RegulatorState::On,
            2 => RegulatorState::Error,
            3 => RegulatorState::Bypass,
            _ => RegulatorState::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regulator_state_values() {
        assert_eq!(RegulatorState::Off as u32, 0);
        assert_eq!(RegulatorState::On as u32, 1);
        assert_eq!(RegulatorState::Error as u32, 2);
        assert_eq!(RegulatorState::Bypass as u32, 3);
    }

    #[test]
    fn test_regulator_enable_disable() {
        let reg = RegulatorCore::new("vdd_cpu", 0, 700_000, 1_200_000, 5_000_000, false);
        assert!(!reg.is_enabled());
        reg.enable();
        assert!(reg.is_enabled());
        assert_eq!(reg.enable_count.load(Ordering::Acquire), 1);
        reg.disable();
        assert!(!reg.is_enabled());
    }

    #[test]
    fn test_regulator_ref_count() {
        let reg = RegulatorCore::new("vdd_gpu", 1, 700_000, 1_000_000, 3_000_000, false);
        reg.enable();
        reg.enable();
        assert_eq!(reg.enable_count.load(Ordering::Acquire), 2);
        reg.disable();
        assert!(reg.is_enabled()); // Still 1 ref
        reg.disable();
        assert!(!reg.is_enabled());
    }

    #[test]
    fn test_regulator_always_on() {
        let reg = RegulatorCore::new("vdd_sys", 2, 1_000_000, 1_800_000, 10_000_000, true);
        reg.enable();
        reg.disable(); // Should be refused
        assert!(reg.is_enabled());
    }

    #[test]
    fn test_regulator_set_voltage() {
        let reg = RegulatorCore::new("vdd_core", 3, 700_000, 1_200_000, 5_000_000, false);
        let v = reg.set_voltage(900_000);
        assert_eq!(v, 900_000);
        assert_eq!(reg.get_voltage(), 900_000);
    }

    #[test]
    fn test_regulator_voltage_clamp() {
        let reg = RegulatorCore::new("vdd_core", 3, 700_000, 1_200_000, 5_000_000, false);
        let v = reg.set_voltage(500_000); // Below min
        assert_eq!(v, 700_000);
        let v = reg.set_voltage(1_500_000); // Above max
        assert_eq!(v, 1_200_000);
    }

    #[test]
    fn test_regulator_over_temp_cannot_disable() {
        let reg = RegulatorCore::new("vdd_cpu", 0, 700_000, 1_200_000, 5_000_000, false);
        reg.enable();
        reg.report_over_temp();
        assert_eq!(reg.get_state(), RegulatorState::Error);
        reg.disable(); // Should be refused due to fault
        assert!(reg.is_enabled());
    }

    #[test]
    fn test_regulator_over_current_cannot_disable() {
        let reg = RegulatorCore::new("vdd_cpu", 0, 700_000, 1_200_000, 5_000_000, false);
        reg.enable();
        reg.report_over_current();
        assert_eq!(reg.get_state(), RegulatorState::Error);
        reg.disable(); // Should be refused due to fault
        assert!(reg.is_enabled());
    }

    #[test]
    fn test_regulator_clear_fault() {
        let reg = RegulatorCore::new("vdd_cpu", 0, 700_000, 1_200_000, 5_000_000, false);
        reg.enable();
        reg.report_over_temp();
        reg.clear_over_temp();
        assert_eq!(reg.get_state(), RegulatorState::Off);
    }
}
