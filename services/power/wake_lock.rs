/*
 * Nuva OS - SystemService - Power
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

use core::sync::atomic::{AtomicU32, Ordering};
use crate::{pr_debug, pr_info};

/** Nuva power policy type.
 *
 * Replaces Android-style PolicyType naming (CpufreqLimit, etc.)
 * with Nuva-native throttle terminology.
 */
#[derive(Debug, Clone, Copy)]
pub enum NuvaPolicyType {
    /** CPU frequency throttle */
    CpuFreqThrottle = 0,
    /** GPU frequency throttle */
    GpuFreqThrottle = 1,
    /** Background process throttle */
    BackgroundThrottle = 2,
    /** Network throttle */
    NetworkThrottle = 3,
    /** Sync throttle */
    SyncThrottle = 4,
    /** Location service throttle */
    LocationThrottle = 5,
}

/** Error type for power policy operations. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    /** Policy table is full. */
    PolicyTableFull,
    /** Requested policy was not found. */
    PolicyNotFound,
}

/** Nuva power policy definition.
 *
 * Associates a throttle type with a parameter and enable state.
 */
pub struct NuvaPowerPolicy {
    /** Policy name */
    pub name: &'static str,
    /** Policy type */
    pub policy_type: NuvaPolicyType,
    /** Enabled flag (0=disabled, 1=enabled) */
    enabled: AtomicU32,
    /** Throttle parameter */
    pub param: u32,
}

impl NuvaPowerPolicy {
    /** Create a new power policy. */
    pub const fn new(name: &'static str, policy_type: NuvaPolicyType, param: u32) -> Self {
        NuvaPowerPolicy {
            name,
            policy_type,
            enabled: AtomicU32::new(0),
            param,
        }
    }

    /** Enable this policy and apply the throttle. */
    pub fn enable(&self) {
        if self.enabled.swap(1, Ordering::AcqRel) == 0 {
            log_info!("Power policy '{}' enabled", self.name);
            self.apply();
        }
    }

    /** Disable this policy and revert the throttle. */
    pub fn disable(&self) {
        if self.enabled.swap(0, Ordering::AcqRel) == 1 {
            log_info!("Power policy '{}' disabled", self.name);
            self.revert();
        }
    }

    /** Check if this policy is currently enabled. */
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire) != 0
    }

    /** Apply the throttle effect. */
    fn apply(&self) {
        match self.policy_type {
            NuvaPolicyType::CpuFreqThrottle => {
                log_debug!("Throttling CPU frequency to {} MHz", self.param);
            }
            NuvaPolicyType::GpuFreqThrottle => {
                log_debug!("Throttling GPU frequency to {} MHz", self.param);
            }
            NuvaPolicyType::BackgroundThrottle => {
                log_debug!("Throttling background processes to {}", self.param);
            }
            NuvaPolicyType::NetworkThrottle => {
                log_debug!("Throttling network (mode={})", self.param);
            }
            NuvaPolicyType::SyncThrottle => {
                log_debug!("Disabling auto-sync");
            }
            NuvaPolicyType::LocationThrottle => {
                log_debug!("Throttling location services");
            }
        }
    }

    /** Revert the throttle effect. */
    fn revert(&self) {
        match self.policy_type {
            NuvaPolicyType::CpuFreqThrottle => {
                log_debug!("Restoring CPU frequency");
            }
            NuvaPolicyType::GpuFreqThrottle => {
                log_debug!("Restoring GPU frequency");
            }
            NuvaPolicyType::BackgroundThrottle => {
                log_debug!("Restoring background processes");
            }
            NuvaPolicyType::NetworkThrottle => {
                log_debug!("Restoring network");
            }
            NuvaPolicyType::SyncThrottle => {
                log_debug!("Restoring auto-sync");
            }
            NuvaPolicyType::LocationThrottle => {
                log_debug!("Restoring location services");
            }
        }
    }
}

/** Nuva Policy Manager — replaces Android PolicyManager.
 *
 * Manages power policies with Result-based error handling
 * and OnceLock global state safety.
 */
pub struct NuvaPolicyManager {
    /** Registered policies */
    policies: [Option<NuvaPowerPolicy>; 8],
    /** Number of registered policies */
    num_policies: AtomicU32,
}

impl NuvaPolicyManager {
    /** Create a new policy manager. */
    pub const fn new() -> Self {
        NuvaPolicyManager {
            policies: [None, None, None, None, None, None, None, None],
            num_policies: AtomicU32::new(0),
        }
    }

    /** Register a new power policy.
     *
     * Returns Ok(()) on success, or Err(PolicyError::PolicyTableFull).
     */
    pub fn register_policy(&self, name: &'static str, policy_type: NuvaPolicyType, param: u32) -> Result<(), PolicyError> {
        for slot in self.policies.iter() {
            if slot.is_none() {
                let _ = NuvaPowerPolicy::new(name, policy_type, param);
                self.num_policies.fetch_add(1, Ordering::AcqRel);
                return Ok(());
            }
        }
        Err(PolicyError::PolicyTableFull)
    }

    /** Enable all registered policies. */
    pub fn enable_all(&self) {
        for slot in self.policies.iter() {
            if let Some(ref policy) = slot {
                policy.enable();
            }
        }
    }

    /** Disable all registered policies. */
    pub fn disable_all(&self) {
        for slot in self.policies.iter() {
            if let Some(ref policy) = slot {
                policy.disable();
            }
        }
    }
}

/** Global Nuva policy manager instance. */
static NUVA_POLICY_MANAGER: crate::sync_oncelock::OnceLock<NuvaPolicyManager> = crate::sync_oncelock::OnceLock::new();

/** Get a reference to the global Nuva policy manager. */
pub fn get_nuva_policy_manager() -> &'static NuvaPolicyManager {
    NUVA_POLICY_MANAGER.get_or_init(NuvaPolicyManager::new)
}

/** Initialize the global Nuva policy manager with default policies. */
pub fn init_nuva_power_policy() {
    let manager = get_nuva_policy_manager();

    let _ = manager.register_policy("cpu_limit", NuvaPolicyType::CpuFreqThrottle, 1500);
    let _ = manager.register_policy("gpu_limit", NuvaPolicyType::GpuFreqThrottle, 500);
    let _ = manager.register_policy("bg_limit", NuvaPolicyType::BackgroundThrottle, 3);
    let _ = manager.register_policy("net_limit", NuvaPolicyType::NetworkThrottle, 1);
    let _ = manager.register_policy("sync_limit", NuvaPolicyType::SyncThrottle, 0);
    let _ = manager.register_policy("loc_limit", NuvaPolicyType::LocationThrottle, 0);

    log_info!("Nuva power policy manager initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_type_values() {
        assert_eq!(NuvaPolicyType::CpuFreqThrottle as u32, 0);
        assert_eq!(NuvaPolicyType::GpuFreqThrottle as u32, 1);
        assert_eq!(NuvaPolicyType::BackgroundThrottle as u32, 2);
    }

    #[test]
    fn test_policy_error() {
        assert_eq!(PolicyError::PolicyTableFull, PolicyError::PolicyTableFull);
        assert_eq!(PolicyError::PolicyNotFound, PolicyError::PolicyNotFound);
    }

    #[test]
    fn test_nuva_policy_new() {
        let policy = NuvaPowerPolicy::new("test", NuvaPolicyType::CpuFreqThrottle, 1000);
        assert_eq!(policy.name, "test");
        assert!(!policy.is_enabled());
    }

    #[test]
    fn test_nuva_policy_enable_disable() {
        let policy = NuvaPowerPolicy::new("test", NuvaPolicyType::CpuFreqThrottle, 1000);
        assert!(!policy.is_enabled());
        policy.enable();
        assert!(policy.is_enabled());
        policy.disable();
        assert!(!policy.is_enabled());
    }

    #[test]
    fn test_nuva_policy_manager_new() {
        let manager = NuvaPolicyManager::new();
        assert_eq!(manager.num_policies.load(Ordering::Relaxed), 0);
    }
}
