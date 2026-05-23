/*
 * Nuva OS - Kernel - Kernel
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


// Security submodules
// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod aslr;
pub mod capability;
pub mod credential;
pub mod security_hook;
pub mod nsm_manager;
pub mod lsm;
pub mod stack_canary;
pub mod signature;
pub mod secureboot;
pub mod memcrypt;
pub mod dilithium_sign;
pub mod ai_cap;

// Re-export key types
pub use aslr::{AslrState, MmStruct, init_aslr, randomize_stack, randomize_mmap};
pub use stack_canary::{StackCanary, TaskStackCanary, init_stack_canary, get_global_canary};

// Re-export NSM/LSM types
pub use security_hook::{SecurityHook, SecurityModule};
pub use capability::{CapSet, cap};
pub use credential::Credentials;
pub use nsm_manager::{SecurityManager as NsmSecurityManager, SecStats, SecId, capable, has_capability};

// Re-export LSM compatibility types (deprecated)
pub use lsm::{SecurityOps as LsmSecurityOps, LegacySecurityModule};

pub use signature::{
    CodeSignature, SignatureAlgorithm, SignatureResult, SignatureChain,
    SignatureChainEntry, SignatureContext,
    init_signature, get_signature_context,
    MAX_SIGNATURE_SIZE, MAX_PUBKEY_HASH_SIZE, MAX_SIGNER_NAME,
    SIG_FLAG_TRUSTED, SIG_FLAG_REVOKED, SIG_FLAG_EXPERIMENTAL, SIG_FLAG_SYSTEM,
};

pub use secureboot::{
    SecureBootState, BootComponent, BootVerifyResult, BootConfig,
    MeasurementEntry, verify_boot_chain, lock_boot_config, measured_boot,
    init_secure_boot, get_boot_config,
    MAX_BOOT_COMPONENTS, BOOT_HASH_SIZE,
};

pub use memcrypt::{
    MemoryEncryptionConfig, MemoryEncryptionManager, EncryptionAlgorithm,
    EncryptionKey, KeyState, PageEncryptStatus,
    init_mem_encrypt, get_mem_encrypt_manager,
    MAX_KEY_SIZE, MAX_ENCRYPTION_KEYS,
};

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// canForceType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
 /// Fileread
 FileRead = 0,
 /// Filewrite
 FileWrite = 1,
 /// Fileexecute
 FileExec = 2,
 /// NetworkJoin
 NetConnect = 3,
 /// Networklisten
 NetListen = 4,
 /// ProcessCreate
 ProcCreate = 5,
 /// ProcessTerminate
 ProcKill = 6,
 /// MemoryMap
 MemMap = 7,
 /// Deviceaccess
 DeviceAccess = 8,
 /// Systemmanagementadministration
 SysAdmin = 9,
 /// Debugging
 Debug = 10,
 /// realtimetuneDegree
 RealTime = 11,
 /// Set UID
 SetUid = 12,
 /// Set GID
 SetGid = 13,
 /// improvechangeRootDirectory
 Chroot = 14,
 /// MountFile System
 Mount = 15,
 /// closemachine
 Shutdown = 16,
}

/// canForcecollection
pub struct CapabilitySet {
 /// canForceBitGraph
 bits: AtomicU64,
}

impl CapabilitySet {
 pub const fn new() -> Self {
 CapabilitySet {
 bits: AtomicU64::new(0),
 }
 }
 
 /// fromBitGraphCreate
 pub const fn from_bits(bits: u64) -> Self {
 CapabilitySet {
 bits: AtomicU64::new(bits),
 }
 }
 
 /// addPluscanForce
 pub fn add(&self, cap: Capability) {
 self.bits.fetch_or(1 << (cap as u64), Ordering::AcqRel);
 }
 
 /// DividecanForce
 pub fn remove(&self, cap: Capability) {
 self.bits.fetch_and(!(1 << (cap as u64)), Ordering::AcqRel);
 }
 
 /// CheckiffinitecanForce
 pub fn has(&self, cap: Capability) -> bool {
 (self.bits.load(Ordering::Acquire) & (1 << (cap as u64))) != 0
 }
 
 /// GetBitGraph
 pub fn get_bits(&self) -> u64 {
 self.bits.load(Ordering::Acquire)
 }
 
 /// SetBitGraph
 pub fn set_bits(&self, bits: u64) {
 self.bits.store(bits, Ordering::Release);
 }
 
 /// ClearplacefinitecanForce
 pub fn clear(&self) {
 self.bits.store(0, Ordering::Release);
 }
 
 /// addPlusplacefinitecanForce
 pub fn add_all(&self) {
 self.bits.store(u64::MAX, Ordering::Release);
 }
}

/// Access ControlCheckresult
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessResult {
 /// Enable
 Allow = 0,
 /// Reject
 Deny = 1,
 /// needwantenterastepCheck
 CheckNext = 2,
}

/// SecurityContext
pub struct SecurityContext {
 /// User ID
 pub uid: AtomicU32,
 /// Group ID
 pub gid: AtomicU32,
 /// validUser ID
 pub euid: AtomicU32,
 /// validGroup ID
 pub egid: AtomicU32,
 /// Save User ID
 pub suid: AtomicU32,
 /// Save Group ID
 pub sgid: AtomicU32,
 /// File SystemUser ID
 pub fsuid: AtomicU32,
 /// File SystemGroup ID
 pub fsgid: AtomicU32,
 /// canForcecollection
 pub caps: CapabilitySet,
 /// SecurityLabel
 pub label: AtomicU32,
}

impl SecurityContext {
 pub const fn new() -> Self {
 SecurityContext {
 uid: AtomicU32::new(0),
 gid: AtomicU32::new(0),
 euid: AtomicU32::new(0),
 egid: AtomicU32::new(0),
 suid: AtomicU32::new(0),
 sgid: AtomicU32::new(0),
 fsuid: AtomicU32::new(0),
 fsgid: AtomicU32::new(0),
 caps: CapabilitySet::new(),
 label: AtomicU32::new(0),
 }
 }
 
 /// Create root Context
 pub fn root() -> Self {
 let ctx = SecurityContext::new();
 ctx.caps.add_all();
 ctx
 }
 
 /// GetUser ID
 pub fn get_uid(&self) -> u32 {
 self.uid.load(Ordering::Acquire)
 }
 
 /// SetUser ID
 pub fn set_uid(&self, uid: u32) {
 self.uid.store(uid, Ordering::Release);
 self.euid.store(uid, Ordering::Release);
 self.suid.store(uid, Ordering::Release);
 self.fsuid.store(uid, Ordering::Release);
 }
 
 /// GetGroup ID
 pub fn get_gid(&self) -> u32 {
 self.gid.load(Ordering::Acquire)
 }
 
 /// SetGroup ID
 pub fn set_gid(&self, gid: u32) {
 self.gid.store(gid, Ordering::Release);
 self.egid.store(gid, Ordering::Release);
 self.sgid.store(gid, Ordering::Release);
 self.fsgid.store(gid, Ordering::Release);
 }
 
 /// ifis root
 pub fn is_root(&self) -> bool {
 self.euid.load(Ordering::Acquire) == 0
 }
 
 /// CheckcanForce
 pub fn has_capability(&self, cap: Capability) -> bool {
 self.is_root() || self.caps.has(cap)
 }
}

/// SecurityOperation
pub struct SecurityOps {
 /// CheckFileaccess
 pub file_permission: Option<fn(&SecurityContext, u32, u32) -> AccessResult>,
 /// CheckProcessaccess
 pub task_permission: Option<fn(&SecurityContext, u32) -> AccessResult>,
 /// CheckNetworkaccess
 pub socket_permission: Option<fn(&SecurityContext, u32) -> AccessResult>,
 /// CheckDeviceaccess
 pub device_permission: Option<fn(&SecurityContext, u32, u32) -> AccessResult>,
}

/// SecurityManager
pub struct SecurityManager {
 /// Operation
 pub ops: SecurityOps,
 /// Checktimenumber
 pub check_count: AtomicU64,
 /// Rejecttimenumber
 pub deny_count: AtomicU64,
}

impl SecurityManager {
 pub const fn new() -> Self {
 SecurityManager {
 ops: SecurityOps {
 file_permission: None,
 task_permission: None,
 socket_permission: None,
 device_permission: None,
 },
 check_count: AtomicU64::new(0),
 deny_count: AtomicU64::new(0),
 }
 }
 
 /// Initialize
 pub fn init(&self) {
 log_info!("Security manager initialized");
 }
 
 /// CheckFileaccessPermission
 pub fn check_file_access(&self, ctx: &SecurityContext, 
 _mode: u32, _mask: u32) -> AccessResult {
 self.check_count.fetch_add(1, Ordering::AcqRel);
 
 // root UsertotalisEnable
 if ctx.is_root() {
 return AccessResult::Allow;
 }
 
 if let Some(check) = self.ops.file_permission {
 let result = check(ctx, _mode, _mask);
 if result == AccessResult::Deny {
 self.deny_count.fetch_add(1, Ordering::AcqRel);
 }
 return result;
 }
 
 // Default: CheckcanForce
 if ctx.has_capability(Capability::FileRead) {
 AccessResult::Allow
 } else {
 self.deny_count.fetch_add(1, Ordering::AcqRel);
 AccessResult::Deny
 }
 }
 
 /// CheckProcessaccessPermission
 pub fn check_task_access(&self, ctx: &SecurityContext, 
 _target_pid: u32) -> AccessResult {
 self.check_count.fetch_add(1, Ordering::AcqRel);
 
 if ctx.is_root() {
 return AccessResult::Allow;
 }
 
 if let Some(check) = self.ops.task_permission {
 let result = check(ctx, _target_pid);
 if result == AccessResult::Deny {
 self.deny_count.fetch_add(1, Ordering::AcqRel);
 }
 return result;
 }
 
 AccessResult::Allow
 }
 
 /// CheckNetworkaccessPermission
 pub fn check_socket_access(&self, ctx: &SecurityContext, 
 _operation: u32) -> AccessResult {
 self.check_count.fetch_add(1, Ordering::AcqRel);
 
 if ctx.is_root() {
 return AccessResult::Allow;
 }
 
 if let Some(check) = self.ops.socket_permission {
 let result = check(ctx, _operation);
 if result == AccessResult::Deny {
 self.deny_count.fetch_add(1, Ordering::AcqRel);
 }
 return result;
 }
 
 // Default: CheckcanForce
 let cap = if _operation == 0 {
 Capability::NetConnect
 } else {
 Capability::NetListen
 };
 
 if ctx.has_capability(cap) {
 AccessResult::Allow
 } else {
 self.deny_count.fetch_add(1, Ordering::AcqRel);
 AccessResult::Deny
 }
 }
 
 /// CheckDeviceaccessPermission
 pub fn check_device_access(&self, ctx: &SecurityContext, 
 _dev: u32, _mode: u32) -> AccessResult {
 self.check_count.fetch_add(1, Ordering::AcqRel);
 
 if ctx.is_root() {
 return AccessResult::Allow;
 }
 
 if let Some(check) = self.ops.device_permission {
 let result = check(ctx, _dev, _mode);
 if result == AccessResult::Deny {
 self.deny_count.fetch_add(1, Ordering::AcqRel);
 }
 return result;
 }
 
 // Default: CheckcanForce
 if ctx.has_capability(Capability::DeviceAccess) {
 AccessResult::Allow
 } else {
 self.deny_count.fetch_add(1, Ordering::AcqRel);
 AccessResult::Deny
 }
 }
 
 /// CheckcanForce
 pub fn check_capability(&self, ctx: &SecurityContext, 
 cap: Capability) -> AccessResult {
 self.check_count.fetch_add(1, Ordering::AcqRel);
 
 if ctx.has_capability(cap) {
 AccessResult::Allow
 } else {
 self.deny_count.fetch_add(1, Ordering::AcqRel);
 AccessResult::Deny
 }
 }
 
 /// GetChecktimenumber
 pub fn get_check_count(&self) -> u64 {
 self.check_count.load(Ordering::Acquire)
 }
 
 /// GetRejecttimenumber
 pub fn get_deny_count(&self) -> u64 {
 self.deny_count.load(Ordering::Acquire)
 }
 
 /// printstampStatisticsInfo
 pub fn print_stats(&self) {
 log_info!("Security Manager Statistics:");
 log_info!(" Total checks: {}", self.check_count.load(Ordering::Acquire));
 log_info!(" Denied: {}", self.deny_count.load(Ordering::Acquire));
 }
}

/// GlobalSecurityManager
static SECURITY_MANAGER: core::sync::OnceLock<SecurityManager> = core::sync::OnceLock::new();

pub fn security_manager() -> &'static SecurityManager {
    SECURITY_MANAGER.get_or_init(SecurityManager::new)
}

pub fn init_security_manager() -> &'static SecurityManager {
    SECURITY_MANAGER.get_or_init(SecurityManager::new)
}

pub fn init_security() {
 let sec = security_manager();
 sec.init();
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_capability_values() {
 assert_eq!(Capability::FileRead as u32, 0);
 assert_eq!(Capability::FileWrite as u32, 1);
 assert_eq!(Capability::FileExec as u32, 2);
 assert_eq!(Capability::NetConnect as u32, 3);
 assert_eq!(Capability::NetListen as u32, 4);
 assert_eq!(Capability::ProcCreate as u32, 5);
 assert_eq!(Capability::ProcKill as u32, 6);
 assert_eq!(Capability::MemMap as u32, 7);
 assert_eq!(Capability::DeviceAccess as u32, 8);
 assert_eq!(Capability::SysAdmin as u32, 9);
 assert_eq!(Capability::SetUid as u32, 12);
 assert_eq!(Capability::SetGid as u32, 13);
 assert_eq!(Capability::Shutdown as u32, 16);
 }

 #[test]
 fn test_capability_set_new() {
 let caps = CapabilitySet::new();
 assert_eq!(caps.get_bits(), 0);
 }

 #[test]
 fn test_capability_set_from_bits() {
 let caps = CapabilitySet::from_bits(0xFF);
 assert_eq!(caps.get_bits(), 0xFF);
 }

 #[test]
 fn test_capability_set_add() {
 let caps = CapabilitySet::new();

 caps.add(Capability::FileRead);
 assert!(caps.has(Capability::FileRead));
 assert!(!caps.has(Capability::FileWrite));

 caps.add(Capability::FileWrite);
 assert!(caps.has(Capability::FileWrite));
 }

 #[test]
 fn test_capability_set_remove() {
 let caps = CapabilitySet::new();

 caps.add(Capability::FileRead);
 assert!(caps.has(Capability::FileRead));

 caps.remove(Capability::FileRead);
 assert!(!caps.has(Capability::FileRead));
 }

 #[test]
 fn test_capability_set_clear() {
 let caps = CapabilitySet::new();

 caps.add(Capability::FileRead);
 caps.add(Capability::FileWrite);
 caps.add(Capability::NetConnect);

 caps.clear();
 assert_eq!(caps.get_bits(), 0);
 }

 #[test]
 fn test_capability_set_add_all() {
 let caps = CapabilitySet::new();

 caps.add_all();
 assert_eq!(caps.get_bits(), u64::MAX);
 }

 #[test]
 fn test_capability_set_set_bits() {
 let caps = CapabilitySet::new();

 caps.set_bits(0x1234);
 assert_eq!(caps.get_bits(), 0x1234);
 }

 #[test]
 fn test_access_result_values() {
 assert_eq!(AccessResult::Allow as u32, 0);
 assert_eq!(AccessResult::Deny as u32, 1);
 assert_eq!(AccessResult::CheckNext as u32, 2);
 }

 #[test]
 fn test_security_context_new() {
 let ctx = SecurityContext::new();

 assert_eq!(ctx.get_uid(), 0);
 assert_eq!(ctx.get_gid(), 0);
 assert_eq!(ctx.euid.load(Ordering::Relaxed), 0);
 assert_eq!(ctx.egid.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_security_context_root() {
 let ctx = SecurityContext::root();

 assert!(ctx.is_root());
 assert!(ctx.has_capability(Capability::FileRead));
 assert!(ctx.has_capability(Capability::SysAdmin));
 }

 #[test]
 fn test_security_context_set_uid() {
 let ctx = SecurityContext::new();

 ctx.set_uid(100);

 assert_eq!(ctx.get_uid(), 100);
 assert_eq!(ctx.euid.load(Ordering::Relaxed), 100);
 assert_eq!(ctx.suid.load(Ordering::Relaxed), 100);
 assert_eq!(ctx.fsuid.load(Ordering::Relaxed), 100);
 }

 #[test]
 fn test_security_context_set_gid() {
 let ctx = SecurityContext::new();

 ctx.set_gid(100);

 assert_eq!(ctx.get_gid(), 100);
 assert_eq!(ctx.egid.load(Ordering::Relaxed), 100);
 assert_eq!(ctx.sgid.load(Ordering::Relaxed), 100);
 assert_eq!(ctx.fsgid.load(Ordering::Relaxed), 100);
 }

 #[test]
 fn test_security_context_is_root() {
 let ctx = SecurityContext::new();

 assert!(ctx.is_root());

 ctx.set_uid(100);
 assert!(!ctx.is_root());
 }

 #[test]
 fn test_security_context_has_capability() {
 let ctx = SecurityContext::new();

 // root finiteplacefinitecanForce
 assert!(ctx.has_capability(Capability::FileRead));

 ctx.set_uid(100);
 // root needwantexplicitstyleaddPluscanForce
 assert!(!ctx.has_capability(Capability::FileRead));

 ctx.caps.add(Capability::FileRead);
 assert!(ctx.has_capability(Capability::FileRead));
 }

 #[test]
 fn test_security_manager_new() {
 let mgr = SecurityManager::new();

 assert_eq!(mgr.get_check_count(), 0);
 assert_eq!(mgr.get_deny_count(), 0);
 }

 #[test]
 fn test_security_manager_check_file_access_root() {
 let mgr = SecurityManager::new();
 let ctx = SecurityContext::root();

 let result = mgr.check_file_access(&ctx, 0, 0);
 assert_eq!(result, AccessResult::Allow);
 assert_eq!(mgr.get_check_count(), 1);
 }

 #[test]
 fn test_security_manager_check_file_access_non_root() {
 let mgr = SecurityManager::new();
 let ctx = SecurityContext::new();
 ctx.set_uid(100);

 // finitecanForce root UserwillbyReject
 let result = mgr.check_file_access(&ctx, 0, 0);
 assert_eq!(result, AccessResult::Deny);
 assert_eq!(mgr.get_deny_count(), 1);

 // addPluscanForcethenEnable
 ctx.caps.add(Capability::FileRead);
 let result = mgr.check_file_access(&ctx, 0, 0);
 assert_eq!(result, AccessResult::Allow);
 }

 #[test]
 fn test_security_manager_check_task_access_root() {
 let mgr = SecurityManager::new();
 let ctx = SecurityContext::root();

 let result = mgr.check_task_access(&ctx, 1234);
 assert_eq!(result, AccessResult::Allow);
 }

 #[test]
 fn test_security_manager_check_task_access_non_root() {
 let mgr = SecurityManager::new();
 let ctx = SecurityContext::new();
 ctx.set_uid(100);

 // DefaultEnableProcessaccess
 let result = mgr.check_task_access(&ctx, 1234);
 assert_eq!(result, AccessResult::Allow);
 }

 #[test]
 fn test_security_manager_check_socket_access() {
 let mgr = SecurityManager::new();
 let ctx = SecurityContext::new();
 ctx.set_uid(100);

 // finiteNetworkcanForce
 let result = mgr.check_socket_access(&ctx, 0);
 assert_eq!(result, AccessResult::Deny);

 // addPlusJoincanForce
 ctx.caps.add(Capability::NetConnect);
 let result = mgr.check_socket_access(&ctx, 0);
 assert_eq!(result, AccessResult::Allow);

 // ListenneedwantnotsamecanForce
 let result = mgr.check_socket_access(&ctx, 1);
 assert_eq!(result, AccessResult::Deny);

 ctx.caps.add(Capability::NetListen);
 let result = mgr.check_socket_access(&ctx, 1);
 assert_eq!(result, AccessResult::Allow);
 }

 #[test]
 fn test_security_manager_check_device_access() {
 let mgr = SecurityManager::new();
 let ctx = SecurityContext::new();
 ctx.set_uid(100);

 // finiteDeviceaccesscanForce
 let result = mgr.check_device_access(&ctx, 0, 0);
 assert_eq!(result, AccessResult::Deny);

 // addPlusDeviceaccesscanForce
 ctx.caps.add(Capability::DeviceAccess);
 let result = mgr.check_device_access(&ctx, 0, 0);
 assert_eq!(result, AccessResult::Allow);
 }

 #[test]
 fn test_security_manager_check_capability() {
 let mgr = SecurityManager::new();
 let ctx = SecurityContext::new();
 ctx.set_uid(100);

 let result = mgr.check_capability(&ctx, Capability::SysAdmin);
 assert_eq!(result, AccessResult::Deny);

 ctx.caps.add(Capability::SysAdmin);
 let result = mgr.check_capability(&ctx, Capability::SysAdmin);
 assert_eq!(result, AccessResult::Allow);
 }

 #[test]
 fn test_security_manager_stats() {
 let mgr = SecurityManager::new();
 let ctx = SecurityContext::new();
 ctx.set_uid(100);

 // executeasomeCheck
 mgr.check_file_access(&ctx, 0, 0);
 mgr.check_task_access(&ctx, 0);
 mgr.check_socket_access(&ctx, 0);

 assert_eq!(mgr.get_check_count(), 3);
 assert_eq!(mgr.get_deny_count(), 2); // file sum socket byReject
 }

 #[test]
 fn test_security_ops_default() {
 let mgr = SecurityManager::new();

 assert!(mgr.ops.file_permission.is_none());
 assert!(mgr.ops.task_permission.is_none());
 assert!(mgr.ops.socket_permission.is_none());
 assert!(mgr.ops.device_permission.is_none());
 }

 #[test]
 fn test_security_context_label() {
 let ctx = SecurityContext::new();

 assert_eq!(ctx.label.load(Ordering::Relaxed), 0);

 ctx.label.store(42, Ordering::Relaxed);
 assert_eq!(ctx.label.load(Ordering::Relaxed), 42);
 }
}