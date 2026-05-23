/*
 * Nuva OS - Kernel - NSM Manager
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva Security Module (NSM) manager.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::vec::Vec;
use crate::pr_info;

use super::security_hook::{SecurityModule, SecurityHook};
use super::credential::Credentials;

/// Security ID type
pub type SecId = u32;

/// Security statistics
pub struct SecStats {
    /// Permission checks performed
    pub perm_checks: AtomicU64,
    /// Permissions denied
    pub denied: AtomicU64,
    /// Audit events recorded
    pub audits: AtomicU64,
}

impl SecStats {
    /// Create zeroed statistics
    pub const fn new() -> Self {
        SecStats {
            perm_checks: AtomicU64::new(0),
            denied: AtomicU64::new(0),
            audits: AtomicU64::new(0),
        }
    }
}

/// Nuva Security Module (NSM) manager.
/// Manages registered security modules and delegates security checks.
pub struct SecurityManager {
    /// Registered security modules (Vec replaces manual linked list)
    modules: Vec<&'static SecurityModule>,
    /// Statistics
    stats: SecStats,
}

impl SecurityManager {
    /// Create an empty security manager
    pub const fn new() -> Self {
        SecurityManager {
            modules: Vec::new(),
            stats: SecStats::new(),
        }
    }

    /// Initialize the security manager
    pub fn init(&self) {
        log_info!("Security manager initialized");
    }

    /// Register a security module
    pub fn register_module(&mut self, module: &'static SecurityModule) -> i32 {
        self.modules.push(module);
        0
    }

    /// Check file permission against all registered modules
    pub fn file_permission(&self, _cred: &Credentials, _path: &[u8], _mode: u32) -> i32 {
        self.stats.perm_checks.fetch_add(1, Ordering::AcqRel);

        for module in &self.modules {
            if module.enabled {
                let result = module.hook.file_permission(core::ptr::null_mut(), _mode);
                if result != 0 {
                    self.stats.denied.fetch_add(1, Ordering::AcqRel);
                    return result;
                }
            }
        }
        0
    }

    /// Check inode permission against all registered modules
    pub fn inode_permission(&self, _cred: &Credentials, _inode: u64, _mode: u32) -> i32 {
        self.stats.perm_checks.fetch_add(1, Ordering::AcqRel);

        for module in &self.modules {
            if module.enabled {
                let result = module.hook.inode_permission(core::ptr::null_mut(), _mode);
                if result != 0 {
                    self.stats.denied.fetch_add(1, Ordering::AcqRel);
                    return result;
                }
            }
        }
        0
    }

    /// Check socket permission against all registered modules
    pub fn socket_permission(&self, _cred: &Credentials, _family: u32, _sock_type: u32, _protocol: u32) -> i32 {
        self.stats.perm_checks.fetch_add(1, Ordering::AcqRel);

        for module in &self.modules {
            if module.enabled {
                let result = module.hook.socket_create(_family, _sock_type, _protocol, 0);
                if result != 0 {
                    self.stats.denied.fetch_add(1, Ordering::AcqRel);
                    return result;
                }
            }
        }
        0
    }

    /// Check capability
    pub fn cap_check(&self, cred: &Credentials, cap: u32) -> i32 {
        self.stats.perm_checks.fetch_add(1, Ordering::AcqRel);

        if cred.has_cap(cap) {
            0
        } else {
            self.stats.denied.fetch_add(1, Ordering::AcqRel);
            -1 // EPERM
        }
    }

    /// Record an audit event
    pub fn audit(&mut self, _event: u32, _data: *const core::ffi::c_void) {
        self.stats.audits.fetch_add(1, Ordering::AcqRel);
    }
}

/// Global security manager
static SECURITY_MANAGER: core::sync::OnceLock<SecurityManager> = core::sync::OnceLock::new();

/// Get a reference to the global security manager
pub fn security_manager() -> &'static SecurityManager {
    SECURITY_MANAGER.get_or_init(SecurityManager::new)
}

pub fn init_security_manager() -> &'static SecurityManager {
    SECURITY_MANAGER.get_or_init(SecurityManager::new)
}

/// Initialize the security subsystem
pub fn init_security() {
    let mgr = security_manager();
    mgr.init();
}

/// Check if the current task has the given capability
pub fn capable(cap: u32) -> bool {
    let cred = Credentials::new(0, 0); // Root
    security_manager().cap_check(&cred, cap) == 0
}

/// Check if the given credentials have the capability
pub fn has_capability(cred: &Credentials, cap: u32) -> bool {
    cred.has_cap(cap)
}
