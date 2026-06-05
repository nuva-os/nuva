/*
 * Nuva OS - Kernel - Security - SecurityHook
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
/*
 * Nuva OS - Kernel - Security Hook Trait
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva Security Module (NSM) — Composable security framework.
 * Refactored from Linux LSM imitation to Nuva native capability-based
 * security model.
 *
 * Migration note: Previously modeled after Linux LSM hook pattern
 * (*mut c_void, u32 mask) -> i32. Now uses NuvaCapability-based
 * capability tokens with Result<(), NuvaError> return types.
 */

use crate::types::{NuvaCapabilityId, NuvaAccessRight, NuvaResourceHandle, NuvaError};

/// Nuva native security hook trait for composable security modules.
/// Each method provides a default no-op implementation returning Ok(()).
///
/// This trait replaces the previous SecurityHook that imitated
/// Linux LSM hooks with raw pointers and errno returns.
pub trait NuvaSecurityHook: Send + Sync {
    /// Module name for identification and logging.
    fn name(&self) -> &'static str {
        "unknown"
    }

    /// Initialize the security module.
    /// Migrated from: fn init(&self) -> i32
    fn init(&self) -> Result<(), NuvaError> {
        Ok(())
    }

    /// Check access to a resource based on capability token.
    /// Migrated from: fn inode_permission(&self, *mut c_void, u32) -> i32
    fn check_access(
        &self,
        _capability: NuvaCapabilityId,
        _resource: NuvaResourceHandle,
        _access: NuvaAccessRight,
    ) -> Result<(), NuvaError> {
        Ok(())
    }

    /// Check process operation permission.
    /// Migrated from: fn task_alloc/task_free with *mut c_void
    fn check_process_op(
        &self,
        _capability: NuvaCapabilityId,
        _target_process: NuvaCapabilityId,
        _access: NuvaAccessRight,
    ) -> Result<(), NuvaError> {
        Ok(())
    }

    /// Check IPC operation permission.
    /// Migrated from: fn msg_queue_create/sem_create/shm_create with *mut c_void
    fn check_ipc_op(
        &self,
        _capability: NuvaCapabilityId,
        _resource: NuvaResourceHandle,
        _access: NuvaAccessRight,
    ) -> Result<(), NuvaError> {
        Ok(())
    }

    /// Check network operation permission.
    /// Migrated from: fn socket_create/bind/listen/accept with *mut c_void
    fn check_network_op(
        &self,
        _capability: NuvaCapabilityId,
        _access: NuvaAccessRight,
    ) -> Result<(), NuvaError> {
        Ok(())
    }
}

/// Nuva security policy priority (replaces Linux LSM stacking priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum NuvaPolicyPriority {
    Advisory  = 0,
    Standard  = 1,
    High      = 2,
    Mandatory = 3,
}

/// Nuva security policy (replaces simple u32 priority in SecurityModule)
#[derive(Debug, Clone)]
pub struct NuvaSecurityPolicy {
    pub policy_id: u32,
    pub priority: NuvaPolicyPriority,
    pub rule_set: NuvaPolicyRuleSet,
}

/// Nuva policy rule set (placeholder for policy engine rules)
#[derive(Debug, Clone, Default)]
pub struct NuvaPolicyRuleSet {
    pub rules: [u64; 4],
}

/// Nuva security module entry wrapping a NuvaSecurityHook implementation.
/// Migrated from: SecurityModule with u32 priority
pub struct NuvaSecurityModule {
    /// The security hook implementation
    pub hook: &'static dyn NuvaSecurityHook,
    /// Security policy (replaces simple u32 priority)
    pub policy: NuvaSecurityPolicy,
    /// Whether this module is enabled
    pub enabled: bool,
}

impl NuvaSecurityModule {
    pub fn new(
        hook: &'static dyn NuvaSecurityHook,
        priority: NuvaPolicyPriority,
    ) -> Self {
        NuvaSecurityModule {
            hook,
            policy: NuvaSecurityPolicy {
                policy_id: 0,
                priority,
                rule_set: NuvaPolicyRuleSet::default(),
            },
            enabled: true,
        }
    }
}

/// Legacy SecurityHook trait (Linux LSM imitation, deprecated).
/// Retained for backward compatibility; will be removed after full migration.
#[deprecated(since = "0.3.0", note = "Use NuvaSecurityHook instead")]
pub trait SecurityHook: Send + Sync {
    fn name(&self) -> &'static str { "unknown" }
    fn init(&self) -> i32 { 0 }
    fn task_alloc(&self, _task: *mut core::ffi::c_void) -> i32 { 0 }
    fn task_free(&self, _task: *mut core::ffi::c_void) {}
    fn inode_permission(&self, _inode: *mut core::ffi::c_void, _mask: u32) -> i32 { 0 }
    fn file_open(&self, _file: *mut core::ffi::c_void) -> i32 { 0 }
    fn file_permission(&self, _file: *mut core::ffi::c_void, _mask: u32) -> i32 { 0 }
    fn socket_create(&self, _family: u32, _type_: u32, _protocol: u32, _kern: u32) -> i32 { 0 }
    fn socket_bind(&self, _sock: *mut core::ffi::c_void, _addr: *const core::ffi::c_void) -> i32 { 0 }
    fn socket_listen(&self, _sock: *mut core::ffi::c_void, _backlog: i32) -> i32 { 0 }
    fn socket_accept(&self, _sock: *mut core::ffi::c_void, _newsock: *mut core::ffi::c_void) -> i32 { 0 }
    fn msg_queue_create(&self, _msq: *mut core::ffi::c_void) -> i32 { 0 }
    fn sem_create(&self, _sma: *mut core::ffi::c_void) -> i32 { 0 }
    fn shm_create(&self, _shp: *mut core::ffi::c_void) -> i32 { 0 }
}

/// Legacy SecurityModule (Linux LSM style, deprecated).
#[deprecated(since = "0.3.0", note = "Use NuvaSecurityModule instead")]
pub struct SecurityModule {
    pub hook: &'static dyn SecurityHook,
    pub priority: u32,
    pub enabled: bool,
}
