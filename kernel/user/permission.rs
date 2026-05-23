/*
 * Nuva OS - PermissionControl
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
use super::user::{Uid, Gid};

/// PermissionType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// canread
    Read = 4,
    /// canwrite
    Write = 2,
    /// canexecute
    Execute = 1,
    /// readwrite
    ReadWrite = 6,
    /// readexecute
    ReadExecute = 5,
    /// writeexecute
    WriteExecute = 3,
    /// readwriteexecute
    ReadWriteExecute = 7,
    /// nonePermission
    None = 0,
}

/// FilePermissionMode
#[derive(Debug, Clone, Copy)]
pub struct FileMode {
    /// OwnerPermission
    pub owner: u8,
    /// GroupPermission
    pub group: u8,
    /// OtherPermission
    pub other: u8,
    /// SpecialBit (SUID, SGID, Sticky)
    pub special: u8,
}

impl FileMode {
    /// Create newFileMode
    pub fn new(owner: u8, group: u8, other: u8) -> Self {
        FileMode {
            owner,
            group,
            other,
            special: 0,
        }
    }
    
    /// from u32 Create
    pub fn from_u32(mode: u32) -> Self {
        FileMode {
            owner: ((mode >> 6) & 0x7) as u8,
            group: ((mode >> 3) & 0x7) as u8,
            other: (mode & 0x7) as u8,
            special: ((mode >> 9) & 0x7) as u8,
        }
    }
    
    /// convertas u32
    pub fn to_u32(&self) -> u32 {
        ((self.special as u32) << 9) |
        ((self.owner as u32) << 6) |
        ((self.group as u32) << 3) |
        (self.other as u32)
    }
    
    /// DefaultFilePermission (644)
    pub fn default_file() -> Self {
        Self::new(6, 4, 4)
    }
    
    /// DefaultDirectoryPermission (755)
    pub fn default_dir() -> Self {
        Self::new(7, 5, 5)
    }
    
    /// DefaultcanexecutePermission (755)
    pub fn default_exec() -> Self {
        Self::new(7, 5, 5)
    }
    
    /// Set SUID
    pub fn set_suid(&mut self) {
        self.special |= 0x4;
    }
    
    /// Set SGID
    pub fn set_sgid(&mut self) {
        self.special |= 0x2;
    }
    
    /// Set Sticky
    pub fn set_sticky(&mut self) {
        self.special |= 0x1;
    }
    
    /// ifhave SUID
    pub fn has_suid(&self) -> bool {
        (self.special & 0x4) != 0
    }
    
    /// ifhave SGID
    pub fn has_sgid(&self) -> bool {
        (self.special & 0x2) != 0
    }
    
    /// ifhave Sticky
    pub fn has_sticky(&self) -> bool {
        (self.special & 0x1) != 0
    }
}

/// canForce (Capability)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// exceedlevelUsercanForce
    CapSysAdmin = 0,
    /// Networkmanagementadministration
    CapNetAdmin = 1,
    /// SystemTime
    CapSysTime = 2,
    /// assetsourceLimit
    CapSysResource = 3,
    /// Processmanagementadministration
    CapSysPtrace = 4,
    /// ModulePlusload
    CapSysModule = 5,
    /// File System
    CapSysFs = 6,
    /// Devicemanagementadministration
    CapSysDevice = 7,
    /// MemoryLockfixed
    CapSysMemLock = 8,
    /// tuneDegree
    CapSysNice = 9,
    /// IPC
    CapSysIpc = 10,
    /// Raw I/O
    CapSysRawio = 11,
    /// User ID Set
    CapSetuid = 12,
    /// Group ID Set
    CapSetgid = 13,
    /// FileOwner
    CapFowner = 14,
    /// FileMode
    CapFsetid = 15,
    /// endend
    CapSysTtyConfig = 16,
    /// Audit
    CapAuditControl = 17,
    /// AuditWrite
    CapAuditWrite = 18,
}

/// canForcecollection
pub struct CapabilitySet {
    /// validcanForce
    pub effective: AtomicU32,
    /// allowcancanForce
    pub permitted: AtomicU32,
    /// InheritancecanForce
    pub inheritable: AtomicU32,
}

impl CapabilitySet {
    pub const fn new() -> Self {
        CapabilitySet {
            effective: AtomicU32::new(0),
            permitted: AtomicU32::new(0),
            inheritable: AtomicU32::new(0),
        }
    }
    
    /// SetcanForce
    pub fn set_cap(&self, cap: Capability) {
        let bit = 1u32 << (cap as u32);
        self.effective.fetch_or(bit, Ordering::AcqRel);
        self.permitted.fetch_or(bit, Ordering::AcqRel);
    }
    
    /// clearDividecanForce
    pub fn clear_cap(&self, cap: Capability) {
        let bit = 1u32 << (cap as u32);
        self.effective.fetch_and(!bit, Ordering::AcqRel);
    }
    
    /// CheckcanForce
    pub fn has_cap(&self, cap: Capability) -> bool {
        let bit = 1u32 << (cap as u32);
        (self.effective.load(Ordering::Acquire) & bit) != 0
    }
    
    /// allcanForce
    pub fn full() -> Self {
        CapabilitySet {
            effective: AtomicU32::new(0xFFFFFFFF),
            permitted: AtomicU32::new(0xFFFFFFFF),
            inheritable: AtomicU32::new(0xFFFFFFFF),
        }
    }
    
    /// emptycanForce
    pub fn empty() -> Self {
        Self::new()
    }
}

/// PermissionCheckdevice
pub struct PermissionChecker;

impl PermissionChecker {
    /// CheckFilePermission
    pub fn check_file(
        uid: Uid,
        gid: Gid,
        file_uid: Uid,
        file_gid: Gid,
        mode: FileMode,
        perm: Permission,
    ) -> bool {
        // exceedlevelUserfiniteplacefinitePermission
        if uid == 0 {
            return true;
        }
        
        let perm_bit = perm as u8;
        
        // CheckOwnerPermission
        if uid == file_uid {
            return (mode.owner & perm_bit) == perm_bit;
        }
        
        // CheckGroupPermission
        if gid == file_gid {
            return (mode.group & perm_bit) == perm_bit;
        }
        
        // CheckOtherPermission
        (mode.other & perm_bit) == perm_bit
    }
    
    /// CheckreadPermission
    pub fn can_read(uid: Uid, gid: Gid, file_uid: Uid, file_gid: Gid, mode: FileMode) -> bool {
        Self::check_file(uid, gid, file_uid, file_gid, mode, Permission::Read)
    }
    
    /// CheckwritePermission
    pub fn can_write(uid: Uid, gid: Gid, file_uid: Uid, file_gid: Gid, mode: FileMode) -> bool {
        Self::check_file(uid, gid, file_uid, file_gid, mode, Permission::Write)
    }
    
    /// CheckexecutePermission
    pub fn can_execute(uid: Uid, gid: Gid, file_uid: Uid, file_gid: Gid, mode: FileMode) -> bool {
        Self::check_file(uid, gid, file_uid, file_gid, mode, Permission::Execute)
    }
    
    /// CheckcanForce
    pub fn check_capability(caps: &CapabilitySet, cap: Capability) -> bool {
        caps.has_cap(cap)
    }
    
    /// CheckexceedlevelUser
    pub fn is_superuser(uid: Uid) -> bool {
        uid == 0
    }
}

/// InitializePermissionSystem
pub fn init_permission() {
    log_info!("Permission system initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_values() {
        assert_eq!(Permission::Read as u8, 4);
        assert_eq!(Permission::Write as u8, 2);
        assert_eq!(Permission::Execute as u8, 1);
        assert_eq!(Permission::ReadWrite as u8, 6);
        assert_eq!(Permission::ReadExecute as u8, 5);
        assert_eq!(Permission::WriteExecute as u8, 3);
        assert_eq!(Permission::ReadWriteExecute as u8, 7);
        assert_eq!(Permission::None as u8, 0);
    }

    #[test]
    fn test_file_mode_new() {
        let mode = FileMode::new(7, 5, 5);

        assert_eq!(mode.owner, 7);
        assert_eq!(mode.group, 5);
        assert_eq!(mode.other, 5);
        assert_eq!(mode.special, 0);
    }

    #[test]
    fn test_file_mode_from_u32() {
        // 0o755 = 0o111101101
        let mode = FileMode::from_u32(0o755);

        assert_eq!(mode.owner, 7);
        assert_eq!(mode.group, 5);
        assert_eq!(mode.other, 5);

        // 0o644 = 0o110100100
        let mode = FileMode::from_u32(0o644);

        assert_eq!(mode.owner, 6);
        assert_eq!(mode.group, 4);
        assert_eq!(mode.other, 4);
    }

    #[test]
    fn test_file_mode_to_u32() {
        let mode = FileMode::new(7, 5, 5);
        assert_eq!(mode.to_u32(), 0o755);

        let mode = FileMode::new(6, 4, 4);
        assert_eq!(mode.to_u32(), 0o644);
    }

    #[test]
    fn test_file_mode_roundtrip() {
        let original = 0o755;
        let mode = FileMode::from_u32(original);
        assert_eq!(mode.to_u32(), original);

        let original = 0o644;
        let mode = FileMode::from_u32(original);
        assert_eq!(mode.to_u32(), original);
    }

    #[test]
    fn test_file_mode_defaults() {
        let file_mode = FileMode::default_file();
        assert_eq!(file_mode.owner, 6);
        assert_eq!(file_mode.group, 4);
        assert_eq!(file_mode.other, 4);

        let dir_mode = FileMode::default_dir();
        assert_eq!(dir_mode.owner, 7);
        assert_eq!(dir_mode.group, 5);
        assert_eq!(dir_mode.other, 5);

        let exec_mode = FileMode::default_exec();
        assert_eq!(exec_mode.owner, 7);
        assert_eq!(exec_mode.group, 5);
        assert_eq!(exec_mode.other, 5);
    }

    #[test]
    fn test_file_mode_suid() {
        let mut mode = FileMode::new(7, 5, 5);

        assert!(!mode.has_suid());

        mode.set_suid();
        assert!(mode.has_suid());
        assert_eq!(mode.special, 0x4);
    }

    #[test]
    fn test_file_mode_sgid() {
        let mut mode = FileMode::new(7, 5, 5);

        assert!(!mode.has_sgid());

        mode.set_sgid();
        assert!(mode.has_sgid());
        assert_eq!(mode.special, 0x2);
    }

    #[test]
    fn test_file_mode_sticky() {
        let mut mode = FileMode::new(7, 5, 5);

        assert!(!mode.has_sticky());

        mode.set_sticky();
        assert!(mode.has_sticky());
        assert_eq!(mode.special, 0x1);
    }

    #[test]
    fn test_file_mode_all_special() {
        let mut mode = FileMode::new(7, 5, 5);

        mode.set_suid();
        mode.set_sgid();
        mode.set_sticky();

        assert!(mode.has_suid());
        assert!(mode.has_sgid());
        assert!(mode.has_sticky());
        assert_eq!(mode.special, 0x7);
    }

    #[test]
    fn test_capability_values() {
        assert_eq!(Capability::CapSysAdmin as u32, 0);
        assert_eq!(Capability::CapNetAdmin as u32, 1);
        assert_eq!(Capability::CapSysTime as u32, 2);
        assert_eq!(Capability::CapSetuid as u32, 12);
        assert_eq!(Capability::CapSetgid as u32, 13);
    }

    #[test]
    fn test_capability_set_new() {
        let caps = CapabilitySet::new();

        assert_eq!(caps.effective.load(Ordering::Relaxed), 0);
        assert_eq!(caps.permitted.load(Ordering::Relaxed), 0);
        assert_eq!(caps.inheritable.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_capability_set_set_cap() {
        let caps = CapabilitySet::new();

        caps.set_cap(Capability::CapSysAdmin);

        assert!(caps.has_cap(Capability::CapSysAdmin));
        assert!(!caps.has_cap(Capability::CapNetAdmin));
    }

    #[test]
    fn test_capability_set_clear_cap() {
        let caps = CapabilitySet::new();

        caps.set_cap(Capability::CapSysAdmin);
        assert!(caps.has_cap(Capability::CapSysAdmin));

        caps.clear_cap(Capability::CapSysAdmin);
        assert!(!caps.has_cap(Capability::CapSysAdmin));
    }

    #[test]
    fn test_capability_set_multiple() {
        let caps = CapabilitySet::new();

        caps.set_cap(Capability::CapSysAdmin);
        caps.set_cap(Capability::CapNetAdmin);
        caps.set_cap(Capability::CapSetuid);

        assert!(caps.has_cap(Capability::CapSysAdmin));
        assert!(caps.has_cap(Capability::CapNetAdmin));
        assert!(caps.has_cap(Capability::CapSetuid));
        assert!(!caps.has_cap(Capability::CapSysTime));
    }

    #[test]
    fn test_capability_set_full() {
        let caps = CapabilitySet::full();

        assert_eq!(caps.effective.load(Ordering::Relaxed), 0xFFFFFFFF);
        assert_eq!(caps.permitted.load(Ordering::Relaxed), 0xFFFFFFFF);
        assert_eq!(caps.inheritable.load(Ordering::Relaxed), 0xFFFFFFFF);
    }

    #[test]
    fn test_capability_set_empty() {
        let caps = CapabilitySet::empty();

        assert_eq!(caps.effective.load(Ordering::Relaxed), 0);
        assert_eq!(caps.permitted.load(Ordering::Relaxed), 0);
        assert_eq!(caps.inheritable.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_permission_checker_superuser() {
        assert!(PermissionChecker::is_superuser(0));
        assert!(!PermissionChecker::is_superuser(1));
        assert!(!PermissionChecker::is_superuser(1000));
    }

    #[test]
    fn test_permission_checker_owner_read() {
        let mode = FileMode::new(6, 4, 4);

        // Ownercanwithread
        assert!(PermissionChecker::can_read(100, 100, 100, 100, mode));
        // OtherUsernotcanwrite
        assert!(!PermissionChecker::can_write(200, 200, 100, 100, mode));
    }

    #[test]
    fn test_permission_checker_group() {
        let mode = FileMode::new(6, 4, 0);

        // GroupMembercanwithread
        assert!(PermissionChecker::can_read(200, 100, 100, 100, mode));
        // GroupMembernotcanwrite
        assert!(!PermissionChecker::can_write(200, 100, 100, 100, mode));
    }

    #[test]
    fn test_permission_checker_other() {
        let mode = FileMode::new(6, 4, 4);

        // OtherUsercanwithread
        assert!(PermissionChecker::can_read(300, 200, 100, 100, mode));
        // OtherUsernotcanwrite
        assert!(!PermissionChecker::can_write(300, 200, 100, 100, mode));
    }

    #[test]
    fn test_permission_checker_superuser_bypass() {
        let mode = FileMode::new(0, 0, 0);

        // exceedlevelUserfiniteplacefinitePermission
        assert!(PermissionChecker::can_read(0, 0, 100, 100, mode));
        assert!(PermissionChecker::can_write(0, 0, 100, 100, mode));
        assert!(PermissionChecker::can_execute(0, 0, 100, 100, mode));
    }

    #[test]
    fn test_permission_checker_execute() {
        let mode = FileMode::new(7, 5, 5);

        // Ownercanwithexecute
        assert!(PermissionChecker::can_execute(100, 100, 100, 100, mode));
        // GroupMembercanwithexecute
        assert!(PermissionChecker::can_execute(200, 100, 100, 100, mode));
        // OtherUsercanwithexecute
        assert!(PermissionChecker::can_execute(300, 200, 100, 100, mode));
    }

    #[test]
    fn test_permission_checker_capability() {
        let caps = CapabilitySet::new();

        assert!(!PermissionChecker::check_capability(&caps, Capability::CapSysAdmin));

        caps.set_cap(Capability::CapSysAdmin);
        assert!(PermissionChecker::check_capability(&caps, Capability::CapSysAdmin));
    }

    #[test]
    fn test_permission_checker_no_permission() {
        let mode = FileMode::new(0, 0, 0);

        // noPermission
        assert!(!PermissionChecker::can_read(100, 100, 200, 200, mode));
        assert!(!PermissionChecker::can_write(100, 100, 200, 200, mode));
        assert!(!PermissionChecker::can_execute(100, 100, 200, 200, mode));
    }
}