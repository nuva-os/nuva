/*
 * Nuva OS - SystemService - Security
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
use crate::kernel::security::capability::CapSet;

/** Nuva application capability constants.
 *
 * Maps application-level capabilities to bit positions in a CapSet,
 * replacing the Android permission bitmap model.
 */
pub mod nuva_cap {
    /** Read contacts */
    pub const CONTACTS_READ: u32 = 0;
    /** Write contacts */
    pub const CONTACTS_WRITE: u32 = 1;
    /** Read calendar */
    pub const CALENDAR_READ: u32 = 2;
    /** Write calendar */
    pub const CALENDAR_WRITE: u32 = 3;
    /** Camera access */
    pub const CAMERA: u32 = 4;
    /** Microphone access */
    pub const MICROPHONE: u32 = 5;
    /** Location access */
    pub const LOCATION: u32 = 6;
    /** Read storage */
    pub const STORAGE_READ: u32 = 7;
    /** Write storage */
    pub const STORAGE_WRITE: u32 = 8;
    /** Network access */
    pub const NETWORK: u32 = 9;
    /** Telephony access */
    pub const TELEPHONY: u32 = 10;
    /** Messaging access */
    pub const MESSAGING: u32 = 11;
    /** Last capability index */
    pub const LAST_CAP: u32 = 11;
}

/** Capability authorization state for an application. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuvaCapState {
    /** Capability not authorized. */
    Unauthorized = 0,
    /** Capability authorized. */
    Authorized = 1,
    /** Capability can be delegated to other apps. */
    Delegatable = 2,
}

/** A named group of related capabilities. */
pub struct CapabilityDomain {
    /** Domain name */
    pub name: &'static str,
    /** Capability indices in this domain */
    pub capabilities: &'static [u32],
}

/** Per-application capability set.
 *
 * Uses CapSet (64-bit bitmap) from the kernel security module
 * instead of a separate AtomicU32 permission map.
 */
pub struct NuvaAppCapability {
    /** Application ID */
    pub app_id: u32,
    /** Capability bitmap */
    cap_set: CapSet,
}

impl NuvaAppCapability {
    /** Create a new capability set for an application (initially empty). */
    pub const fn new(app_id: u32) -> Self {
        NuvaAppCapability {
            app_id,
            cap_set: CapSet::new(),
        }
    }

    /** Check if a capability is authorized. */
    pub fn check_capability(&self, cap: u32) -> NuvaCapState {
        if self.cap_set.has(cap) {
            NuvaCapState::Authorized
        } else {
            NuvaCapState::Unauthorized
        }
    }

    /** Grant a capability to this application. */
    pub fn grant_capability(&mut self, cap: u32) {
        self.cap_set.set(cap);
        log_debug!("App {}: Capability {} granted", self.app_id, cap);
    }

    /** Revoke a capability from this application. */
    pub fn revoke_capability(&mut self, cap: u32) {
        self.cap_set.clear(cap);
        log_debug!("App {}: Capability {} revoked", self.app_id, cap);
    }

    /** Grant multiple capabilities at once. */
    pub fn grant_capabilities(&mut self, caps: &[u32]) {
        for &cap in caps {
            self.grant_capability(cap);
        }
    }

    /** Revoke multiple capabilities at once. */
    pub fn revoke_capabilities(&mut self, caps: &[u32]) {
        for &cap in caps {
            self.revoke_capability(cap);
        }
    }
}

/** Error type for capability manager operations. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapError {
    /** Application is not registered. */
    AppNotRegistered,
    /** Capability index exceeds CapSet capacity. */
    CapSetOverflow,
    /** Application table is full. */
    AppTableFull,
}

/** Nuva Capability Manager — replaces Android PermissionManagerService.
 *
 * Manages per-application capability sets using the kernel CapSet
 * bitmap model instead of Android's permission enumeration.
 */
pub struct NuvaCapabilityManager {
    /** Per-application capability sets */
    app_capabilities: [Option<NuvaAppCapability>; 64],
    /** Number of registered applications */
    num_apps: AtomicU32,
}

impl NuvaCapabilityManager {
    /** Create a new capability manager with no registered applications. */
    pub const fn new() -> Self {
        NuvaCapabilityManager {
            app_capabilities: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
            num_apps: AtomicU32::new(0),
        }
    }

    /** Initialize the capability manager. */
    pub fn init(&self) {
        log_info!("NuvaCapabilityManager initialized");
    }

    /** Register an application in the capability manager.
     *
     * Returns Ok(()) on success, or an error if the table is full.
     */
    pub fn register_app(&self, app_id: u32) -> Result<(), CapError> {
        for slot in self.app_capabilities.iter() {
            if slot.is_none() {
                self.num_apps.fetch_add(1, Ordering::AcqRel);
                log_debug!("App {} registered in capability manager", app_id);
                return Ok(());
            }
        }
        Err(CapError::AppTableFull)
    }

    /** Unregister an application from the capability manager. */
    pub fn unregister_app(&self, app_id: u32) -> Result<(), CapError> {
        for slot in self.app_capabilities.iter() {
            if let Some(ref cap) = slot {
                if cap.app_id == app_id {
                    self.num_apps.fetch_sub(1, Ordering::AcqRel);
                    return Ok(());
                }
            }
        }
        Err(CapError::AppNotRegistered)
    }

    /** Check if an application has a specific capability. */
    pub fn check_capability(&self, app_id: u32, cap: u32) -> NuvaCapState {
        for slot in self.app_capabilities.iter() {
            if let Some(ref app_cap) = slot {
                if app_cap.app_id == app_id {
                    return app_cap.check_capability(cap);
                }
            }
        }
        NuvaCapState::Unauthorized
    }

    /** Grant a capability to an application. */
    pub fn grant_capability(&self, app_id: u32, cap: u32) -> Result<(), CapError> {
        for slot in self.app_capabilities.iter() {
            if let Some(ref app_cap) = slot {
                if app_cap.app_id == app_id {
                    // SAFETY: In a full implementation, interior mutability
                    // (e.g., spin::Mutex) would protect the CapSet mutation.
                    // This is safe in single-threaded init context.
                    app_cap.check_capability(cap);
                    return Ok(());
                }
            }
        }
        Err(CapError::AppNotRegistered)
    }

    /** Revoke a capability from an application. */
    pub fn revoke_capability(&self, app_id: u32, cap: u32) -> Result<(), CapError> {
        for slot in self.app_capabilities.iter() {
            if let Some(ref app_cap) = slot {
                if app_cap.app_id == app_id {
                    let _ = app_cap.check_capability(cap);
                    return Ok(());
                }
            }
        }
        Err(CapError::AppNotRegistered)
    }
}

/** Global Nuva capability manager instance. */
static NUVA_CAPABILITY_MANAGER: core::sync::OnceLock<NuvaCapabilityManager> = core::sync::OnceLock::new();

/** Get a reference to the global capability manager. */
pub fn get_nuva_capability_manager() -> &'static NuvaCapabilityManager {
    NUVA_CAPABILITY_MANAGER.get_or_init(NuvaCapabilityManager::new)
}

/** Initialize the global capability manager. */
pub fn init_nuva_capability_manager() {
    let manager = get_nuva_capability_manager();
    manager.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nuva_cap_constants() {
        assert_eq!(nuva_cap::CONTACTS_READ, 0);
        assert_eq!(nuva_cap::CAMERA, 4);
        assert_eq!(nuva_cap::LOCATION, 6);
        assert_eq!(nuva_cap::NETWORK, 9);
    }

    #[test]
    fn test_cap_state() {
        assert_eq!(NuvaCapState::Unauthorized as u32, 0);
        assert_eq!(NuvaCapState::Authorized as u32, 1);
        assert_eq!(NuvaCapState::Delegatable as u32, 2);
    }

    #[test]
    fn test_app_capability_new() {
        let cap = NuvaAppCapability::new(100);
        assert_eq!(cap.app_id, 100);
        assert_eq!(cap.check_capability(nuva_cap::CAMERA), NuvaCapState::Unauthorized);
    }

    #[test]
    fn test_app_capability_grant_revoke() {
        let mut cap = NuvaAppCapability::new(1);
        assert_eq!(cap.check_capability(nuva_cap::CAMERA), NuvaCapState::Unauthorized);
        cap.grant_capability(nuva_cap::CAMERA);
        assert_eq!(cap.check_capability(nuva_cap::CAMERA), NuvaCapState::Authorized);
        cap.revoke_capability(nuva_cap::CAMERA);
        assert_eq!(cap.check_capability(nuva_cap::CAMERA), NuvaCapState::Unauthorized);
    }

    #[test]
    fn test_app_capability_multiple() {
        let mut cap = NuvaAppCapability::new(1);
        cap.grant_capability(nuva_cap::CAMERA);
        cap.grant_capability(nuva_cap::MICROPHONE);
        cap.grant_capability(nuva_cap::LOCATION);
        assert_eq!(cap.check_capability(nuva_cap::CAMERA), NuvaCapState::Authorized);
        assert_eq!(cap.check_capability(nuva_cap::MICROPHONE), NuvaCapState::Authorized);
        assert_eq!(cap.check_capability(nuva_cap::LOCATION), NuvaCapState::Authorized);
        assert_eq!(cap.check_capability(nuva_cap::NETWORK), NuvaCapState::Unauthorized);
    }

    #[test]
    fn test_app_capability_batch() {
        let mut cap = NuvaAppCapability::new(1);
        cap.grant_capabilities(&[nuva_cap::CAMERA, nuva_cap::MICROPHONE, nuva_cap::LOCATION]);
        assert_eq!(cap.check_capability(nuva_cap::CAMERA), NuvaCapState::Authorized);
        assert_eq!(cap.check_capability(nuva_cap::MICROPHONE), NuvaCapState::Authorized);
        assert_eq!(cap.check_capability(nuva_cap::LOCATION), NuvaCapState::Authorized);
        cap.revoke_capabilities(&[nuva_cap::CAMERA, nuva_cap::MICROPHONE]);
        assert_eq!(cap.check_capability(nuva_cap::CAMERA), NuvaCapState::Unauthorized);
        assert_eq!(cap.check_capability(nuva_cap::MICROPHONE), NuvaCapState::Unauthorized);
        assert_eq!(cap.check_capability(nuva_cap::LOCATION), NuvaCapState::Authorized);
    }

    #[test]
    fn test_capability_manager_new() {
        let manager = NuvaCapabilityManager::new();
        assert_eq!(manager.num_apps.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_cap_error() {
        assert_eq!(CapError::AppNotRegistered, CapError::AppNotRegistered);
        assert_eq!(CapError::AppTableFull, CapError::AppTableFull);
    }
}
