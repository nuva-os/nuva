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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_debug, pr_info};

/// ApplicationState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
 /// run
 NotRunning = 0,
 /// Startinfix
 Starting = 1,
 /// runinfix
 Running = 2,
 /// Suspend
 Paused = 3,
 /// Stopinfix
 Stopping = 4,
}

/// ApplicationInfo
pub struct AppInfo {
 /// Application ID
 pub app_id: AtomicU64,
 /// Packagename
 pub package_name: &'static str,
 /// Versionsignal
 pub version: u32,
 /// UID
 pub uid: AtomicU32,
 /// PID
 pub pid: AtomicU32,
 /// State
 pub state: AtomicU32,
 /// Priority
 pub priority: u32,
 /// MemoryLimit
 pub memory_limit: usize,
}

impl Clone for AppInfo {
    fn clone(&self) -> Self {
        Self {
            app_id: AtomicU64::new(self.app_id.load(core::sync::atomic::Ordering::Relaxed)),
            package_name: self.package_name.clone(),
            version: self.version.clone(),
            uid: AtomicU32::new(self.uid.load(core::sync::atomic::Ordering::Relaxed)),
            pid: AtomicU32::new(self.pid.load(core::sync::atomic::Ordering::Relaxed)),
            state: AtomicU32::new(self.state.load(core::sync::atomic::Ordering::Relaxed)),
            priority: self.priority.clone(),
            memory_limit: self.memory_limit.clone(),
        }
    }
}

impl AppInfo {
 pub const fn new(app_id: u64, package_name: &'static str) -> Self {
 AppInfo {
 app_id: AtomicU64::new(app_id),
 package_name,
 version: 1,
 uid: AtomicU32::new(0),
 pid: AtomicU32::new(0),
 state: AtomicU32::new(AppState::NotRunning as u32),
 priority: 0,
 memory_limit: 128 * 1024 * 1024, // 128 MB
 }
 }
 
 /// GetState
 pub fn get_state(&self) -> AppState {
 match self.state.load(Ordering::Acquire) {
 0 => AppState::NotRunning,
 1 => AppState::Starting,
 2 => AppState::Running,
 3 => AppState::Paused,
 4 => AppState::Stopping,
 _ => AppState::NotRunning,
 }
 }
 
 /// SetState
 pub fn set_state(&self, state: AppState) {
 self.state.store(state as u32, Ordering::Release);
 }
}

/// ApplicationManager
pub struct AppManager {
 /// ApplicationArray
 apps: [Option<AppInfo>; 64],
 /// Application count
 num_apps: u32,
 /// NextApplication ID
 next_app_id: AtomicU64,
}

impl AppManager {
 pub const fn new() -> Self {
 AppManager {
 apps: [const { None }; 64],
 num_apps: 0,
 next_app_id: AtomicU64::new(1),
 }
 }
 
 /// Initialize
 pub fn init(&mut self) -> i32 {
 log_info!("App manager initialized");
 0
 }
 
 /// installApplication
 pub fn install(&mut self, package_name: &'static str) -> Option<u64> {
 let app_id = self.next_app_id.fetch_add(1, Ordering::AcqRel);
 
 for slot in self.apps.iter_mut() {
 if slot.is_none() {
 *slot = Some(AppInfo::new(app_id, package_name));
 self.num_apps += 1;
 
 log_info!("App installed: {} (id={})", package_name, app_id);
 return Some(app_id);
 }
 }
 
 None
 }
 
 /// uninstallApplication
 pub fn uninstall(&mut self, app_id: u64) -> i32 {
 for slot in self.apps.iter_mut() {
 if let Some(ref app) = slot {
 if app.app_id.load(Ordering::Acquire) == app_id {
 // Checkifpositiveinrun
 if app.get_state() != AppState::NotRunning {
 return -1;
 }
 
 *slot = None;
 self.num_apps -= 1;
 
 log_info!("App uninstalled: {}", app_id);
 return 0;
 }
 }
 }
 -1
 }
 
 /// StartApplication
 pub fn start(&mut self, app_id: u64) -> i32 {
 for slot in self.apps.iter_mut() {
 if let Some(ref app) = slot {
 if app.app_id.load(Ordering::Acquire) == app_id {
 if app.get_state() != AppState::NotRunning {
 return -1;
 }
 
 app.set_state(AppState::Starting);
 
 // TODO: CreateProcess
 // 1. Allocate UID
 // 2. CreateProcess
 // 3. loadApplicationCode
 // 4. Startmain Activity
 
 app.set_state(AppState::Running);
 
 log_info!("App started: {}", app_id);
 return 0;
 }
 }
 }
 -1
 }
 
 /// StopApplication
 pub fn stop(&mut self, app_id: u64) -> i32 {
 for slot in self.apps.iter_mut() {
 if let Some(ref app) = slot {
 if app.app_id.load(Ordering::Acquire) == app_id {
 if app.get_state() == AppState::NotRunning {
 return -1;
 }
 
 app.set_state(AppState::Stopping);
 
 // TODO: StopProcess
 // 1. Stopall Activity
 // 2. Freeresource
 // 3. TerminateProcess
 
 app.set_state(AppState::NotRunning);
 
 log_info!("App stopped: {}", app_id);
 return 0;
 }
 }
 }
 -1
 }
 
 /// SuspendApplication
 pub fn pause(&mut self, app_id: u64) -> i32 {
 for slot in self.apps.iter_mut() {
 if let Some(ref app) = slot {
 if app.app_id.load(Ordering::Acquire) == app_id {
 if app.get_state() != AppState::Running {
 return -1;
 }
 
 app.set_state(AppState::Paused);
 
 // TODO: SuspendProcess
 
 log_debug!("App paused: {}", app_id);
 return 0;
 }
 }
 }
 -1
 }
 
 /// RecoveryApplication
 pub fn resume(&mut self, app_id: u64) -> i32 {
 for slot in self.apps.iter_mut() {
 if let Some(ref app) = slot {
 if app.app_id.load(Ordering::Acquire) == app_id {
 if app.get_state() != AppState::Paused {
 return -1;
 }
 
 app.set_state(AppState::Running);
 
 // TODO: RecoveryProcess
 
 log_debug!("App resumed: {}", app_id);
 return 0;
 }
 }
 }
 -1
 }
 
 /// FindApplication
 pub fn find_app(&self, package_name: &str) -> Option<u64> {
 for slot in self.apps.iter() {
 if let Some(ref app) = slot {
 if app.package_name == package_name {
 return Some(app.app_id.load(Ordering::Acquire));
 }
 }
 }
 None
 }
 
 /// GetApplicationInfo
 pub fn get_app_info(&self, app_id: u64) -> Option<&AppInfo> {
 for slot in self.apps.iter() {
 if let Some(ref app) = slot {
 if app.app_id.load(Ordering::Acquire) == app_id {
 return Some(app);
 }
 }
 }
 None
 }
}

/// GlobalApplicationManager
static APP_MANAGER: core::sync::OnceLock<AppManager> = core::sync::OnceLock::new();

pub fn get_app_manager() -> &'static mut AppManager {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &mut APP_MANAGER }
}

pub fn init_app_manager() {
 let manager = get_app_manager();
 manager.init();
}