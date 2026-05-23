/*
 * Nuva OS - SystemService - App
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

//! Application installer for managing package installation sessions.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Represents the state of an installation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum InstallState {
    None = 0,
    Pending = 1,
    Downloading = 2,
    Verifying = 3,
    Installing = 4,
    Installed = 5,
    Failed = 6,
}

/// Bit flags controlling installation behavior.
pub const INSTALL_FLAG_REPLACE_EXISTING: u32 = 1 << 0;
pub const INSTALL_FLAG_ALLOW_DOWNGRADE: u32 = 1 << 1;
pub const INSTALL_FLAG_FROM_CLI: u32 = 1 << 2;
pub const INSTALL_FLAG_ALL_USERS: u32 = 1 << 3;

/// Describes an application installation request submitted to the installer.
pub struct InstallRequest {
    pub request_id: u64,
    pub package_path: [u8; 256],
    pub flags: u32,
    pub user_id: u32,
    pub installer_package: [u8; 128],
}

/// Tracks the runtime state of an active installation session.
pub struct InstallSession {
    pub session_id: u64,
    pub state: AtomicU32,
    pub progress: AtomicU32,
    pub total_size: AtomicU64,
    pub downloaded_size: AtomicU64,
    pub error_code: AtomicU32,
    pub start_time: AtomicU64,
    pub end_time: AtomicU64,
}

impl Clone for InstallSession {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            state: AtomicU32::new(self.state.load(core::sync::atomic::Ordering::Relaxed)),
            progress: AtomicU32::new(self.progress.load(core::sync::atomic::Ordering::Relaxed)),
            total_size: AtomicU64::new(self.total_size.load(core::sync::atomic::Ordering::Relaxed)),
            downloaded_size: AtomicU64::new(self.downloaded_size.load(core::sync::atomic::Ordering::Relaxed)),
            error_code: AtomicU32::new(self.error_code.load(core::sync::atomic::Ordering::Relaxed)),
            start_time: AtomicU64::new(self.start_time.load(core::sync::atomic::Ordering::Relaxed)),
            end_time: AtomicU64::new(self.end_time.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl InstallSession {
    pub fn new(session_id: u64) -> Self {
        Self {
            session_id,
            state: AtomicU32::new(InstallState::Pending as u32),
            progress: AtomicU32::new(0),
            total_size: AtomicU64::new(0),
            downloaded_size: AtomicU64::new(0),
            error_code: AtomicU32::new(0),
            start_time: AtomicU64::new(0),
            end_time: AtomicU64::new(0),
        }
    }

    pub fn get_state(&self) -> InstallState {
        match self.state.load(Ordering::Relaxed) {
            0 => InstallState::None,
            1 => InstallState::Pending,
            2 => InstallState::Downloading,
            3 => InstallState::Verifying,
            4 => InstallState::Installing,
            5 => InstallState::Installed,
            _ => InstallState::Failed,
        }
    }

    pub fn set_state(&self, state: InstallState) {
        self.state.store(state as u32, Ordering::Relaxed);
    }

    pub fn get_progress(&self) -> u32 {
        self.progress.load(Ordering::Relaxed)
    }

    pub fn set_progress(&self, progress: u32) {
        self.progress.store(progress, Ordering::Relaxed);
    }
}

/// Supported application package formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PackageFormat {
    Unknown = 0,
    NuvaPackage = 1,
    // AndroidApk removed — no longer supported
}

/// On-disk package file header (packed binary layout).
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct PackageHeader {
    pub magic: u32,
    pub version: u32,
    pub format: u32,
    pub flags: u32,
    pub manifest_offset: u64,
    pub manifest_size: u64,
    pub code_offset: u64,
    pub code_size: u64,
    pub resources_offset: u64,
    pub resources_size: u64,
    pub signature_offset: u64,
    pub signature_size: u64,
    pub checksum: u32,
}

pub const NPK_MAGIC: u32 = 0x4E50_4B21; // "NPK!"

/// Application installer that manages package installation sessions and
/// processes install requests.
pub struct AppInstaller {
    sessions: [Option<InstallSession>; 16],
    num_sessions: AtomicU32,
    next_session_id: AtomicU64,
    next_request_id: AtomicU64,
}

impl AppInstaller {
    pub const fn new() -> Self {
        Self {
            sessions: [const { None }; 16],
            num_sessions: AtomicU32::new(0),
            next_session_id: AtomicU64::new(1),
            next_request_id: AtomicU64::new(1),
        }
    }

    pub fn install(&mut self, _package_path: &str) -> u64 { 0 }
    pub fn get_session(&mut self, _session_id: u64) -> Option<&mut InstallSession> { None }
    fn parse_package(&self, _path: &str) -> bool { true }
    fn verify_signature(&self, _path: &str) -> bool { true }
    fn do_install(&self, _path: &str) -> bool { true }
}
