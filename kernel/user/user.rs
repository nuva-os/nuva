/* * Nuva OS - Usermanagementadministration
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

/// User ID Type
pub type Uid = u32;

/// Group ID Type
pub type Gid = u32;

/// MaxUsernumber
pub const MAX_USERS: usize = 256;

/// MaxGroupnumber
pub const MAX_GROUPS: usize = 256;

/// MaxUsernameLength
pub const MAX_USERNAME_LEN: usize = 32;

/// UserFlag
pub mod user_flags {
 /// exceedlevelUser
 pub const SUPERUSER: u32 = 1 << 0;
 /// SystemUser
 pub const SYSTEM: u32 = 1 << 1;
 /// Already disabled
 pub const DISABLED: u32 = 1 << 2;
 /// needPassword
 pub const PASSWORD: u32 = 1 << 3;
}

/// Userstruct
pub struct User {
 /// User ID
 pub uid: Uid,
 /// mainGroup ID
 pub gid: Gid,
 /// Username
 pub name: [u8; MAX_USERNAME_LEN],
 /// UsernameLength
 pub name_len: usize,
 /// UserFlag
 pub flags: AtomicU32,
 /// loginrecordTime
 pub login_time: AtomicU64,
 /// loginrecordtimenumber
 pub login_count: AtomicU32,
 /// Processnumber
 pub process_count: AtomicU32,
}

impl User {
 /// CreatenewUser
 pub fn new(uid: Uid, gid: Gid, name: &[u8]) -> Self {
 let mut name_buf = [0u8; MAX_USERNAME_LEN];
 let len = name.len().min(MAX_USERNAME_LEN);
 name_buf[..len].copy_from_slice(&name[..len]);
 
 User {
 uid,
 gid,
 name: name_buf,
 name_len: len,
 flags: AtomicU32::new(user_flags::PASSWORD),
 login_time: AtomicU64::new(0),
 login_count: AtomicU32::new(0),
 process_count: AtomicU32::new(0),
 }
 }
 
 /// GetUsername
 pub fn get_name(&self) -> &[u8] {
 &self.name[..self.name_len]
 }
 
 /// ifisexceedlevelUser
 pub fn is_superuser(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & user_flags::SUPERUSER) != 0
 }
 
 /// ifisSystemUser
 pub fn is_system(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & user_flags::SYSTEM) != 0
 }
 
 /// ifAlready disabled
 pub fn is_disabled(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & user_flags::DISABLED) != 0
 }
 
 /// SetexceedlevelUser
 pub fn set_superuser(&self, enable: bool) {
 if enable {
 self.flags.fetch_or(user_flags::SUPERUSER, Ordering::AcqRel);
 } else {
 self.flags.fetch_and(!user_flags::SUPERUSER, Ordering::AcqRel);
 }
 }
 
 /// DisableUser
 pub fn set_disabled(&self, disabled: bool) {
 if disabled {
 self.flags.fetch_or(user_flags::DISABLED, Ordering::AcqRel);
 } else {
 self.flags.fetch_and(!user_flags::DISABLED, Ordering::AcqRel);
 }
 }
 
 /// loginrecord
 pub fn login(&self, time: u64) {
 self.login_time.store(time, Ordering::Release);
 self.login_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// loginexit
 pub fn logout(&self) {
 self.login_time.store(0, Ordering::Release);
 }
 
 /// increasePlusProcess
 pub fn add_process(&self) {
 self.process_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// MinusfewProcess
 pub fn remove_process(&self) {
 self.process_count.fetch_sub(1, Ordering::AcqRel);
 }
}

/// Groupstruct
pub struct Group {
 /// Group ID
 pub gid: Gid,
 /// Groupname
 pub name: [u8; MAX_USERNAME_LEN],
 /// GroupnameLength
 pub name_len: usize,
 /// Membernumber
 pub member_count: AtomicU32,
}

impl Group {
 /// CreatenewGroup
 pub fn new(gid: Gid, name: &[u8]) -> Self {
 let mut name_buf = [0u8; MAX_USERNAME_LEN];
 let len = name.len().min(MAX_USERNAME_LEN);
 name_buf[..len].copy_from_slice(&name[..len]);
 
 Group {
 gid,
 name: name_buf,
 name_len: len,
 member_count: AtomicU32::new(0),
 }
 }
 
 /// GetGroupname
 pub fn get_name(&self) -> &[u8] {
 &self.name[..self.name_len]
 }
}

/// UserManager
pub struct UserManager {
 /// UserArray
 users: [Option<User>; MAX_USERS],
 /// GroupArray
 groups: [Option<Group>; MAX_GROUPS],
 /// User count
 user_count: AtomicU32,
 /// Groupcount
 group_count: AtomicU32,
 /// CurrentUser ID
 current_uid: AtomicU32,
 /// NextUser ID
 next_uid: AtomicU32,
 /// NextGroup ID
 next_gid: AtomicU32,
}

impl UserManager {
 pub const fn new() -> Self {
 UserManager {
 users: [None; MAX_USERS],
 groups: [None; MAX_GROUPS],
 user_count: AtomicU32::new(0),
 group_count: AtomicU32::new(0),
 current_uid: AtomicU32::new(0),
 next_uid: AtomicU32::new(1),
 next_gid: AtomicU32::new(1),
 }
 }
 
 /// Initialize
 pub fn init(&self) {
 // Create root User
 let root = User::new(0, 0, b"root");
 self.users[0] = Some(root);
 self.user_count.store(1, Ordering::Release);
 
 // Create root Group
 let root_group = Group::new(0, b"root");
 self.groups[0] = Some(root_group);
 self.group_count.store(1, Ordering::Release);
 
 log_info!("User manager initialized");
 log_info!(" Default user: root (uid=0)");
 }
 
 /// CreateUser
 pub fn create_user(&mut self, name: &[u8], gid: Gid) -> Option<Uid> {
 let uid = self.next_uid.fetch_add(1, Ordering::AcqRel);
 
 if uid as usize >= MAX_USERS {
 return None;
 }
 
 let user = User::new(uid, gid, name);
 self.users[uid as usize] = Some(user);
 self.user_count.fetch_add(1, Ordering::AcqRel);
 
 log_info!("Created user: {} (uid={})", 
 core::str::from_utf8(name).unwrap_or("?"), uid);
 
 Some(uid)
 }
 
 /// CreateGroup
 pub fn create_group(&mut self, name: &[u8]) -> Option<Gid> {
 let gid = self.next_gid.fetch_add(1, Ordering::AcqRel);
 
 if gid as usize >= MAX_GROUPS {
 return None;
 }
 
 let group = Group::new(gid, name);
 self.groups[gid as usize] = Some(group);
 self.group_count.fetch_add(1, Ordering::AcqRel);
 
 log_info!("Created group: {} (gid={})", 
 core::str::from_utf8(name).unwrap_or("?"), gid);
 
 Some(gid)
 }
 
 /// FindUser
 pub fn find_user(&self, uid: Uid) -> Option<&User> {
 if uid as usize >= MAX_USERS {
 return None;
 }
 self.users[uid as usize].as_ref()
 }
 
 /// FindUser (canchange)
 pub fn find_user_mut(&mut self, uid: Uid) -> Option<&mut User> {
 if uid as usize >= MAX_USERS {
 return None;
 }
 self.users[uid as usize].as_mut()
 }
 
 /// byNameFindUser
 pub fn find_user_by_name(&self, name: &[u8]) -> Option<&User> {
 for slot in self.users.iter() {
 if let Some(ref user) = slot {
 if user.get_name() == name {
 return Some(user);
 }
 }
 }
 None
 }
 
 /// FindGroup
 pub fn find_group(&self, gid: Gid) -> Option<&Group> {
 if gid as usize >= MAX_GROUPS {
 return None;
 }
 self.groups[gid as usize].as_ref()
 }
 
 /// GetCurrentUser
 pub fn get_current_uid(&self) -> Uid {
 self.current_uid.load(Ordering::Acquire)
 }
 
 /// SetCurrentUser
 pub fn set_current_uid(&self, uid: Uid) {
 self.current_uid.store(uid, Ordering::Release);
 }
 
 /// DeleteUser
 pub fn delete_user(&mut self, uid: Uid) -> bool {
 if uid == 0 {
 return false; // notcanDelete root
 }
 
 if uid as usize >= MAX_USERS {
 return false;
 }
 
 if self.users[uid as usize].take().is_some() {
 self.user_count.fetch_sub(1, Ordering::AcqRel);
 log_info!("Deleted user: uid={}", uid);
 return true;
 }
 
 false
 }
 
 /// GetUser count
 pub fn get_user_count(&self) -> u32 {
 self.user_count.load(Ordering::Acquire)
 }
 
 /// GetGroupcount
 pub fn get_group_count(&self) -> u32 {
 self.group_count.load(Ordering::Acquire)
 }
}

/// GlobalUserManager
static USER_MANAGER: core::sync::OnceLock<UserManager> = core::sync::OnceLock::new();

pub fn user_manager() -> &'static UserManager {
    USER_MANAGER.get_or_init(UserManager::new)
}

pub fn init_user_manager() -> &'static UserManager {
    USER_MANAGER.get_or_init(UserManager::new)
}

pub fn init_user_manager() {
 let manager = user_manager();
 manager.init();
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_constants() {
 assert_eq!(MAX_USERS, 256);
 assert_eq!(MAX_GROUPS, 256);
 assert_eq!(MAX_USERNAME_LEN, 32);
 }

 #[test]
 fn test_user_flags() {
 assert_eq!(user_flags::SUPERUSER, 1 << 0);
 assert_eq!(user_flags::SYSTEM, 1 << 1);
 assert_eq!(user_flags::DISABLED, 1 << 2);
 assert_eq!(user_flags::PASSWORD, 1 << 3);
 }

 #[test]
 fn test_user_new() {
 let user = User::new(100, 100, b"testuser");

 assert_eq!(user.uid, 100);
 assert_eq!(user.gid, 100);
 assert_eq!(user.get_name(), b"testuser");
 assert!(!user.is_superuser());
 assert!(!user.is_disabled());
 }

 #[test]
 fn test_user_name_truncation() {
 let long_name = [b'a'; 50];
 let user = User::new(1, 1, &long_name);

 assert_eq!(user.name_len, MAX_USERNAME_LEN);
 }

 #[test]
 fn test_user_superuser_flag() {
 let user = User::new(0, 0, b"root");

 assert!(!user.is_superuser());

 user.set_superuser(true);
 assert!(user.is_superuser());

 user.set_superuser(false);
 assert!(!user.is_superuser());
 }

 #[test]
 fn test_user_disabled_flag() {
 let user = User::new(100, 100, b"test");

 assert!(!user.is_disabled());

 user.set_disabled(true);
 assert!(user.is_disabled());

 user.set_disabled(false);
 assert!(!user.is_disabled());
 }

 #[test]
 fn test_user_login_logout() {
 let user = User::new(100, 100, b"test");

 assert_eq!(user.login_count.load(Ordering::Relaxed), 0);
 assert_eq!(user.login_time.load(Ordering::Relaxed), 0);

 user.login(12345);
 assert_eq!(user.login_count.load(Ordering::Relaxed), 1);
 assert_eq!(user.login_time.load(Ordering::Relaxed), 12345);

 user.login(67890);
 assert_eq!(user.login_count.load(Ordering::Relaxed), 2);

 user.logout();
 assert_eq!(user.login_time.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_user_process_count() {
 let user = User::new(100, 100, b"test");

 assert_eq!(user.process_count.load(Ordering::Relaxed), 0);

 user.add_process();
 assert_eq!(user.process_count.load(Ordering::Relaxed), 1);

 user.add_process();
 user.add_process();
 assert_eq!(user.process_count.load(Ordering::Relaxed), 3);

 user.remove_process();
 assert_eq!(user.process_count.load(Ordering::Relaxed), 2);
 }

 #[test]
 fn test_group_new() {
 let group = Group::new(100, b"testgroup");

 assert_eq!(group.gid, 100);
 assert_eq!(group.get_name(), b"testgroup");
 assert_eq!(group.member_count.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_user_manager_new() {
 let mgr = UserManager::new();

 assert_eq!(mgr.get_user_count(), 0);
 assert_eq!(mgr.get_group_count(), 0);
 assert_eq!(mgr.get_current_uid(), 0);
 }

 #[test]
 fn test_user_manager_init() {
 let mut mgr = UserManager::new();
 mgr.init();

 // Initializethenshouldthefinite root UsersumGroup
 assert_eq!(mgr.get_user_count(), 1);
 assert_eq!(mgr.get_group_count(), 1);

 let root = mgr.find_user(0);
 assert!(root.is_some());
 assert_eq!(root.unwrap().get_name(), b"root");
 }

 #[test]
 fn test_user_manager_create_user() {
 let mut mgr = UserManager::new();
 mgr.init();

 let uid = mgr.create_user(b"testuser", 0);
 assert!(uid.is_some());
 assert_eq!(uid.unwrap(), 1);
 assert_eq!(mgr.get_user_count(), 2);

 let user = mgr.find_user(1);
 assert!(user.is_some());
 assert_eq!(user.unwrap().get_name(), b"testuser");
 }

 #[test]
 fn test_user_manager_create_group() {
 let mut mgr = UserManager::new();
 mgr.init();

 let gid = mgr.create_group(b"testgroup");
 assert!(gid.is_some());
 assert_eq!(gid.unwrap(), 1);
 assert_eq!(mgr.get_group_count(), 2);

 let group = mgr.find_group(1);
 assert!(group.is_some());
 assert_eq!(group.unwrap().get_name(), b"testgroup");
 }

 #[test]
 fn test_user_manager_find_user_by_name() {
 let mut mgr = UserManager::new();
 mgr.init();
 mgr.create_user(b"alice", 0);
 mgr.create_user(b"bob", 0);

 let alice = mgr.find_user_by_name(b"alice");
 assert!(alice.is_some());
 assert_eq!(alice.unwrap().uid, 1);

 let bob = mgr.find_user_by_name(b"bob");
 assert!(bob.is_some());
 assert_eq!(bob.unwrap().uid, 2);

 let not_found = mgr.find_user_by_name(b"charlie");
 assert!(not_found.is_none());
 }

 #[test]
 fn test_user_manager_delete_user() {
 let mut mgr = UserManager::new();
 mgr.init();
 mgr.create_user(b"testuser", 0);

 assert_eq!(mgr.get_user_count(), 2);

 // DeleteUser
 let result = mgr.delete_user(1);
 assert!(result);
 assert_eq!(mgr.get_user_count(), 1);

 // notcanDelete root
 let result = mgr.delete_user(0);
 assert!(!result);
 assert_eq!(mgr.get_user_count(), 1);
 }

 #[test]
 fn test_user_manager_current_user() {
 let mgr = UserManager::new();

 assert_eq!(mgr.get_current_uid(), 0);

 mgr.set_current_uid(100);
 assert_eq!(mgr.get_current_uid(), 100);
 }

 #[test]
 fn test_user_manager_find_user_mut() {
 let mut mgr = UserManager::new();
 mgr.init();
 mgr.create_user(b"testuser", 0);

 let user = mgr.find_user_mut(1);
 assert!(user.is_some());

 let user = user.unwrap();
 user.set_superuser(true);
 assert!(user.is_superuser());
 }

 #[test]
 fn test_user_manager_max_users() {
 let mut mgr = UserManager::new();
 mgr.init();

 // CreateUserdirecttoreachtoMaxvalue
 for i in 1..MAX_USERS {
 let result = mgr.create_user(b"user", 0);
 if result.is_none() {
 break;
 }
 }

 // shouldthereachtoMaxUsernumber
 assert_eq!(mgr.get_user_count() as usize, MAX_USERS);
 }

 #[test]
 fn test_user_system_flag() {
 let user = User::new(1, 1, b"daemon");

 assert!(!user.is_system());

 user.flags.fetch_or(user_flags::SYSTEM, Ordering::Relaxed);
 assert!(user.is_system());
 }
}