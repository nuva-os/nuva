/*
 * Nuva OS - Kernel - Security Hook Trait
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva Security Module (NSM) — Composable security framework.
 * Security hook trait for composable security modules.
 */

/// Security hook trait for composable security modules.
/// Each method provides a default no-op implementation returning 0 (allow).
pub trait SecurityHook: Send + Sync {
    /// Module name for identification and logging.
    fn name(&self) -> &'static str {
        "unknown"
    }

    /// Initialize the security module. Returns 0 on success.
    fn init(&self) -> i32 {
        0
    }

    /// Allocate security data for a new task. Returns 0 on success.
    fn task_alloc(&self, _task: *mut core::ffi::c_void) -> i32 {
        0
    }

    /// Free security data for a task.
    fn task_free(&self, _task: *mut core::ffi::c_void) {}

    /// Check inode permission. Returns 0 on allow, negative errno on deny.
    fn inode_permission(&self, _inode: *mut core::ffi::c_void, _mask: u32) -> i32 {
        0
    }

    /// Check file open. Returns 0 on allow.
    fn file_open(&self, _file: *mut core::ffi::c_void) -> i32 {
        0
    }

    /// Check file permission. Returns 0 on allow.
    fn file_permission(&self, _file: *mut core::ffi::c_void, _mask: u32) -> i32 {
        0
    }

    /// Check socket create. Returns 0 on allow.
    fn socket_create(&self, _family: u32, _type_: u32, _protocol: u32, _kern: u32) -> i32 {
        0
    }

    /// Check socket bind. Returns 0 on allow.
    fn socket_bind(&self, _sock: *mut core::ffi::c_void, _addr: *const core::ffi::c_void) -> i32 {
        0
    }

    /// Check socket connect. Returns 0 on allow.
    fn socket_connect(&self, _sock: *mut core::ffi::c_void, _addr: *const core::ffi::c_void) -> i32 {
        0
    }

    /// Check socket listen. Returns 0 on allow.
    fn socket_listen(&self, _sock: *mut core::ffi::c_void, _backlog: i32) -> i32 {
        0
    }

    /// Check socket accept. Returns 0 on allow.
    fn socket_accept(&self, _sock: *mut core::ffi::c_void, _newsock: *mut core::ffi::c_void) -> i32 {
        0
    }

    /// Check message queue create. Returns 0 on allow.
    fn msg_queue_create(&self, _msq: *mut core::ffi::c_void) -> i32 {
        0
    }

    /// Check message queue control. Returns 0 on allow.
    fn msg_queue_msgctl(&self, _msq: *mut core::ffi::c_void, _cmd: i32) -> i32 {
        0
    }

    /// Check semaphore create. Returns 0 on allow.
    fn sem_create(&self, _sma: *mut core::ffi::c_void) -> i32 {
        0
    }

    /// Check shared memory create. Returns 0 on allow.
    fn shm_create(&self, _shp: *mut core::ffi::c_void) -> i32 {
        0
    }
}

/// Security module entry wrapping a SecurityHook implementation.
pub struct SecurityModule {
    /// The security hook implementation
    pub hook: &'static dyn SecurityHook,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Whether this module is enabled
    pub enabled: bool,
}
