/* * Nuva OS - Kernel - machinecontrol
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

// ! machinecontrol
/*!*/
// ! ProcessleavesumassetsourceLimitWorkcan.
/*!*/
// ! # Workcan
/*!*/
// ! - File Systemleave (chroot)
// ! - Networkleave
// ! - assetsourceLimit (CPU, Memory, FileDescriptor)
// ! - SystemcallFiltering
// ! - nameemptyIntervalleave
/*!*/
// ! # useExample
/*!*/
//! ```ignore
// ! // Create
//! let mut sandbox = Sandbox::new();
//! sandbox.set_root("/sandbox/app1");
//! sandbox.limit_memory(100 * 1024 * 1024); // 100MB
//! sandbox.limit_cpu(50); // 50%
/*!*/
// ! // ininfixexecuteProcess
//! sandbox.exec("/bin/app", &args);
//! ```

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    /// Activate
    Inactive = 0,
    /// active
    Active = 1,
    /// alreadySuspend
    Paused = 2,
    /// alreadyTerminate
    Terminated = 3,
}

/// assetsourceLimitType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ResourceLimit {
    /// CPU Time (second)
    CpuTime = 0,
    /// FileSize (Byte)
    FileSize = 1,
    /// DataparagraphSize (Byte)
    DataSize = 2,
    /// StackSize (Byte)
    StackSize = 3,
    /// kernelbranchSize (Byte)
    CoreSize = 4,
    /// collectionSize (Byte)
    ResidentSet = 5,
    /// Processnumber
    Processes = 6,
    /// OpenFilenumber
    OpenFiles = 7,
    /// LockfixedMemory (Byte)
    LockedMemory = 8,
    /// Address Space (Byte)
    AddressSpace = 9,
    /// Message QueueByte
    MessageQueue = 10,
    /// Priority
    Priority = 11,
    /// realtimePriority
    RtPriority = 12,
    /// realtimeTime (us)
    RtTime = 13,
}

/// assetsourceLimitvalue
pub struct RLimit {
    /// softLimit
    pub rlim_cur: u64,
    /// hardLimit
    pub rlim_max: u64,
}

impl RLimit {
    pub const fn new(cur: u64, max: u64) -> Self {
        RLimit {
            rlim_cur: cur,
            rlim_max: max,
        }
    }

    pub const fn unlimited() -> Self {
        RLimit::new(u64::MAX, u64::MAX)
    }
}

/// NamespaceType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NamespaceType {
    /// Mount namespace
    Mount = 0,
    /// UTS Namespace (mainmachinename)
    Uts = 1,
    /// IPC Namespace
    Ipc = 2,
    /// NetworkNamespace
    Network = 3,
    /// PID Namespace
    Pid = 4,
    /// UserNamespace
    User = 5,
    /// Cgroup Namespace
    Cgroup = 6,
}

/// Namespace
pub struct Namespace {
    /// Type
    pub ns_type: NamespaceType,
    /// ID
    pub id: u64,
    /// referenceCount
    pub ref_count: AtomicU32,
}

impl Namespace {
    pub fn new(ns_type: NamespaceType, id: u64) -> Self {
        Namespace {
            ns_type,
            id,
            ref_count: AtomicU32::new(1),
        }
    }

    pub fn inc_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_ref(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::Relaxed)
    }
}

/// SystemtuneuseFilteringRule
#[derive(Debug, Clone, Copy)]
pub struct SeccompFilter {
    /// Systemtuneusesignal
    pub syscall_nr: u32,
    /// Action
    pub action: SeccompAction,
    /// ParameterMask
    pub arg_mask: [u64; 6],
    /// Parametervalue
    pub arg_val: [u64; 6],
}

/// Seccomp Action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeccompAction {
    /// Enable
    Allow = 0,
    /// Reject (ReturnError)
    Errno = 1,
    /// Reject (TerminateProcess)
    Kill = 2,
    /// Tracking
    Trace = 3,
    /// EnableparallelRecord
    Log = 4,
}

/// Config
pub struct SandboxConfig {
    /// RootDirectoryPath
    pub root_path: [u8; 256],
    /// RootDirectoryLength
    pub root_len: u32,
    /// assetsourceLimit
    pub limits: [RLimit; 14],
    /// Enable Namespace
    pub namespaces: u32,
    /// Networkleave
    pub network_isolated: bool,
    /// readFile System
    pub readonly_fs: bool,
    /// Disableexecutenewprocessorder
    pub no_new_privs: bool,
}

impl SandboxConfig {
    pub const fn new() -> Self {
        SandboxConfig {
            root_path: [0; 256],
            root_len: 0,
            limits: [
                RLimit::unlimited(),                    // CpuTime
                RLimit::unlimited(),                    // FileSize
                RLimit::unlimited(),                    // DataSize
                RLimit::new(8 * 1024 * 1024, u64::MAX), // StackSize: 8MB
                RLimit::new(0, 0),                      // CoreSize: 0
                RLimit::unlimited(),                    // ResidentSet
                RLimit::new(1024, 4096),                // Processes
                RLimit::new(1024, 4096),                // OpenFiles
                RLimit::unlimited(),                    // LockedMemory
                RLimit::unlimited(),                    // AddressSpace
                RLimit::unlimited(),                    // MessageQueue
                RLimit::new(0, 0),                      // Priority
                RLimit::new(0, 0),                      // RtPriority
                RLimit::unlimited(),                    // RtTime
            ],
            namespaces: 0,
            network_isolated: false,
            readonly_fs: false,
            no_new_privs: false,
        }
    }

    /// SetRootDirectory
    pub fn set_root(&mut self, path: &[u8]) {
        let len = path.len().min(self.root_path.len());
        self.root_path[..len].copy_from_slice(&path[..len]);
        self.root_len = len as u32;
    }

    /// SetassetsourceLimit
    pub fn set_limit(&mut self, resource: ResourceLimit, cur: u64, max: u64) {
        let idx = resource as usize;
        if idx < self.limits.len() {
            self.limits[idx] = RLimit::new(cur, max);
        }
    }

    /// EnableNamespace
    pub fn enable_namespace(&mut self, ns_type: NamespaceType) {
        self.namespaces |= 1 << (ns_type as u32);
    }

    /// CheckNamespaceifEnable
    pub fn has_namespace(&self, ns_type: NamespaceType) -> bool {
        (self.namespaces & (1 << (ns_type as u32))) != 0
    }
}

/// Instance
pub struct Sandbox {
    /// Config
    pub config: SandboxConfig,
    /// State
    pub state: AtomicU32,
    /// ID
    pub id: u64,
    /// Processnumber
    pub process_count: AtomicU32,
    /// Memoryuse
    pub memory_usage: AtomicU64,
    /// CPU useTime
    pub cpu_time: AtomicU64,
    /// SystemcallFilter
    pub seccomp_filters: [Option<SeccompFilter>; 64],
    /// Filtercount
    pub filter_count: u32,
}

impl Sandbox {
    /// Create new
    pub fn new(id: u64) -> Self {
        Sandbox {
            config: SandboxConfig::new(),
            state: AtomicU32::new(SandboxState::Inactive as u32),
            id,
            process_count: AtomicU32::new(0),
            memory_usage: AtomicU64::new(0),
            cpu_time: AtomicU64::new(0),
            seccomp_filters: [None; 64],
            filter_count: 0,
        }
    }

    /// Activate
    pub fn activate(&self) {
        self.state
            .store(SandboxState::Active as u32, Ordering::Release);
    }

    /// Suspend
    pub fn pause(&self) {
        self.state
            .store(SandboxState::Paused as u32, Ordering::Release);
    }

    /// Terminate
    pub fn terminate(&self) {
        self.state
            .store(SandboxState::Terminated as u32, Ordering::Release);
    }

    /// GetState
    pub fn get_state(&self) -> SandboxState {
        match self.state.load(Ordering::Acquire) {
            0 => SandboxState::Inactive,
            1 => SandboxState::Active,
            2 => SandboxState::Paused,
            3 => SandboxState::Terminated,
            _ => SandboxState::Inactive,
        }
    }

    /// addSystemcallFilter
    pub fn add_seccomp_filter(&mut self, filter: SeccompFilter) -> bool {
        if self.filter_count as usize >= self.seccomp_filters.len() {
            return false;
        }
        self.seccomp_filters[self.filter_count as usize] = Some(filter);
        self.filter_count += 1;
        true
    }

    /// CheckSystemcall
    pub fn check_syscall(&self, syscall_nr: u32) -> SeccompAction {
        for i in 0..self.filter_count as usize {
            if let Some(ref filter) = self.seccomp_filters[i] {
                if filter.syscall_nr == syscall_nr {
                    return filter.action;
                }
            }
        }
        SeccompAction::Allow
    }

    /// CheckassetsourceLimit
    pub fn check_resource_limit(&self, resource: ResourceLimit, value: u64) -> bool {
        let idx = resource as usize;
        if idx >= self.config.limits.len() {
            return true;
        }
        let limit = &self.config.limits[idx];
        value <= limit.rlim_cur
    }

    /// RecordMemorymakeuse
    pub fn record_memory(&self, size: u64) {
        self.memory_usage.fetch_add(size, Ordering::Relaxed);
    }

    /// Record CPU Time
    pub fn record_cpu_time(&self, time: u64) {
        self.cpu_time.fetch_add(time, Ordering::Relaxed);
    }

    /// printstampInfo
    pub fn print_info(&self) {
        log_info!("Sandbox {}:", self.id);
        log_info!(" State: {:?}", self.get_state());
        log_info!(" Processes: {}", self.process_count.load(Ordering::Relaxed));
        log_info!(
            " Memory: {} bytes",
            self.memory_usage.load(Ordering::Relaxed)
        );
        log_info!(" CPU time: {} ns", self.cpu_time.load(Ordering::Relaxed));
        log_info!(" Seccomp filters: {}", self.filter_count);
    }
}

/// Manager
pub struct SandboxManager {
    /// Array
    sandboxes: [Option<Sandbox>; 16],
    /// count
    sandbox_count: AtomicU32,
    /// Next ID
    next_id: AtomicU64,
}

impl SandboxManager {
    pub const fn new() -> Self {
        SandboxManager {
            sandboxes: [None; 16],
            sandbox_count: AtomicU32::new(0),
            next_id: AtomicU64::new(1),
        }
    }

    /// Create
    pub fn create_sandbox(&mut self) -> Option<u64> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        for i in 0..self.sandboxes.len() {
            if self.sandboxes[i].is_none() {
                self.sandboxes[i] = Some(Sandbox::new(id));
                self.sandbox_count.fetch_add(1, Ordering::Relaxed);
                return Some(id);
            }
        }

        None
    }

    /// Get
    pub fn get_sandbox(&self, id: u64) -> Option<&Sandbox> {
        for sandbox in &self.sandboxes {
            if let Some(ref s) = sandbox {
                if s.id == id {
                    return Some(s);
                }
            }
        }
        None
    }

    /// Getcanchange
    pub fn get_sandbox_mut(&mut self, id: u64) -> Option<&mut Sandbox> {
        for sandbox in &mut self.sandboxes {
            if let Some(ref s) = sandbox {
                if s.id == id {
                    // SAFETY: We matched Some(ref s) above, so as_mut() is Some
                    return sandbox.as_mut();
                }
            }
        }
        None
    }

    /// Destroy
    pub fn destroy_sandbox(&mut self, id: u64) -> bool {
        for i in 0..self.sandboxes.len() {
            if let Some(ref s) = self.sandboxes[i] {
                if s.id == id {
                    self.sandboxes[i] = None;
                    self.sandbox_count.fetch_sub(1, Ordering::Relaxed);
                    return true;
                }
            }
        }
        false
    }

    /// printstampplacefinite
    pub fn print_all(&self) {
        log_info!("=== Sandboxes ===");
        log_info!("Total: {}", self.sandbox_count.load(Ordering::Relaxed));

        for sandbox in &self.sandboxes {
            if let Some(ref s) = sandbox {
                s.print_info();
            }
        }
    }
}

/// GlobalManager
static SANDBOX_MANAGER: crate::sync_oncelock::OnceLock<SandboxManager> = crate::sync_oncelock::OnceLock::new();

/// GetManager
pub fn sandbox_manager() -> &'static SandboxManager {
    SANDBOX_MANAGER.get_or_init(SandboxManager::new)
}

pub fn init_sandbox_manager() -> &'static SandboxManager {
    SANDBOX_MANAGER.get_or_init(SandboxManager::new)
}

/// InitializeSystem
pub fn init_sandbox() {
    log_info!("Sandbox system initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_new() {
        let sandbox = Sandbox::new(1);
        assert_eq!(sandbox.id, 1);
        assert_eq!(sandbox.get_state(), SandboxState::Inactive);
        assert_eq!(sandbox.process_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_sandbox_state() {
        let sandbox = Sandbox::new(1);

        sandbox.activate();
        assert_eq!(sandbox.get_state(), SandboxState::Active);

        sandbox.pause();
        assert_eq!(sandbox.get_state(), SandboxState::Paused);

        sandbox.terminate();
        assert_eq!(sandbox.get_state(), SandboxState::Terminated);
    }

    #[test]
    fn test_sandbox_config() {
        let mut config = SandboxConfig::new();

        config.set_root(b"/sandbox/app");
        assert_eq!(config.root_len, 12);

        config.set_limit(ResourceLimit::OpenFiles, 100, 200);
        assert_eq!(
            config.limits[ResourceLimit::OpenFiles as usize].rlim_cur,
            100
        );

        config.enable_namespace(NamespaceType::Mount);
        assert!(config.has_namespace(NamespaceType::Mount));
        assert!(!config.has_namespace(NamespaceType::Network));
    }

    #[test]
    fn test_seccomp_filter() {
        let mut sandbox = Sandbox::new(1);

        let filter = SeccompFilter {
            syscall_nr: 57, // fork
            action: SeccompAction::Kill,
            arg_mask: [0; 6],
            arg_val: [0; 6],
        };

        assert!(sandbox.add_seccomp_filter(filter));
        assert_eq!(sandbox.check_syscall(57), SeccompAction::Kill);
        assert_eq!(sandbox.check_syscall(60), SeccompAction::Allow);
    }

    #[test]
    fn test_resource_limit() {
        let mut sandbox = Sandbox::new(1);
        sandbox.config.set_limit(ResourceLimit::OpenFiles, 100, 200);

        assert!(sandbox.check_resource_limit(ResourceLimit::OpenFiles, 50));
        assert!(sandbox.check_resource_limit(ResourceLimit::OpenFiles, 100));
        assert!(!sandbox.check_resource_limit(ResourceLimit::OpenFiles, 150));
    }

    #[test]
    fn test_sandbox_manager() {
        let mut manager = SandboxManager::new();

        let id = manager.create_sandbox();
        assert!(id.is_some());

        let sandbox = manager.get_sandbox(id.unwrap());
        assert!(sandbox.is_some());

        assert!(manager.destroy_sandbox(id.unwrap()));
        assert!(manager.get_sandbox(id.unwrap()).is_none());
    }

    #[test]
    fn test_namespace() {
        let ns = Namespace::new(NamespaceType::Mount, 1);
        assert_eq!(ns.ns_type, NamespaceType::Mount);
        assert_eq!(ns.id, 1);
        assert_eq!(ns.ref_count.load(Ordering::Relaxed), 1);

        ns.inc_ref();
        assert_eq!(ns.ref_count.load(Ordering::Relaxed), 2);

        ns.dec_ref();
        assert_eq!(ns.ref_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_rlimit() {
        let limit = RLimit::new(100, 200);
        assert_eq!(limit.rlim_cur, 100);
        assert_eq!(limit.rlim_max, 200);

        let unlimited = RLimit::unlimited();
        assert_eq!(unlimited.rlim_cur, u64::MAX);
    }
}
