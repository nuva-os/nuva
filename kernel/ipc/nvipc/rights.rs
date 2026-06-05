/*
 * Nuva OS
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

// ! Nuva IPC PortPermissionmanagementadministration
/*!*/
// ! ImplementationPortPermission Check、transmitsummanagementadministration.

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex as SpinLock;

use super::{PortName, PortRights, IpcError, TaskId};
use super::port::MachPort;

/// PermissionType
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightType {
 /// SendPermission
 Send = 0,
 /// ReceivePermission
 Receive = 1,
 /// atimeitySendPermission
 SendOnce = 2,
 /// PortcollectionPermission
 PortSet = 3,
 /// deadperishNotificationPermission
 DeadName = 4,
}

impl From<RightType> for PortRights {
 fn from(rt: RightType) -> Self {
 match rt {
 RightType::Send => PortRights::SEND,
 RightType::Receive => PortRights::RECEIVE,
 RightType::SendOnce => PortRights::SEND_ONCE,
 RightType::PortSet => PortRights::PORT_SET,
 RightType::DeadName => PortRights::DEAD_NAME,
 }
 }
}

/// Permissionstripentry
#[derive(Debug)]
pub struct RightEntry {
 /// PortName
 pub port_name: PortName,
 /// PermissionType
 pub right_type: RightType,
 /// referenceCount
 refs: AtomicU32,
}

impl Clone for RightEntry {
    fn clone(&self) -> Self {
        Self {
            port_name: self.port_name,
            right_type: self.right_type,
            refs: AtomicU32::new(self.refs.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl RightEntry {
 /// Create newstripentry
 pub fn new(port_name: PortName, right_type: RightType) -> Self {
 Self {
 port_name,
 right_type,
 refs: AtomicU32::new(1),
 }
 }

 /// increasePlusreference
 pub fn add_ref(&self) {
 self.refs.fetch_add(1, Ordering::AcqRel);
 }

 /// Minusfewreference
 pub fn release(&self) -> u32 {
 self.refs.fetch_sub(1, Ordering::AcqRel)
 }

 /// GetreferenceCount
 pub fn ref_count(&self) -> u32 {
 self.refs.load(Ordering::Acquire)
 }
}

/// PermissionManager
pub struct RightsManager {
 /// TaskPermissionform: TaskId -> (PortName -> RightEntry)
 rights_table: SpinLock<BTreeMap<TaskId, BTreeMap<PortName, RightEntry>>>,
 /// Portreferenceform: PortName -> referenceCount
 port_refs: SpinLock<BTreeMap<PortName, AtomicU32>>,
}

impl RightsManager {
 /// CreatenewPermissionManager
 pub fn new() -> Self {
 Self {
 rights_table: SpinLock::new(BTreeMap::new()),
 port_refs: SpinLock::new(BTreeMap::new()),
 }
 }

 /// asTaskaddPlusPortPermission
 pub fn grant_right(
 &self,
 task_id: TaskId,
 port_name: PortName,
 right_type: RightType,
 ) -> Result<(), IpcError> {
 let mut table = self.rights_table.lock();
 
 let task_rights = table.entry(task_id).or_insert_with(BTreeMap::new);
 
 if let Some(entry) = task_rights.get(&port_name) {
 // alreadyExists, increasePlusreference
 entry.add_ref();
 } else {
 // newbuildstripentry
 task_rights.insert(port_name, RightEntry::new(port_name, right_type));
 }
 
 // increasePlusPortreference
 self.inc_port_ref(port_name);
 
 Ok(())
 }

 /// Task PortPermission
 pub fn revoke_right(
 &self,
 task_id: TaskId,
 port_name: PortName,
 ) -> Result<(), IpcError> {
 let mut table = self.rights_table.lock();
 
 if let Some(task_rights) = table.get_mut(&task_id) {
 if let Some(entry) = task_rights.remove(&port_name) {
 // MinusfewPortreference
 self.dec_port_ref(port_name);
 
 // ifisatimeitySendPermission, ReturnSpecialMarker
 if entry.right_type == RightType::SendOnce {
 // atimeityPermissionalreadyuse
 }
 
 return Ok(());
 }
 }
 
 Err(IpcError::PortNotFound)
 }

 /// CheckTaskifhaveSendPermission
 pub fn check_send(&self, task_id: TaskId, port_name: PortName) -> Result<(), IpcError> {
 let table = self.rights_table.lock();
 
 if let Some(task_rights) = table.get(&task_id) {
 if let Some(entry) = task_rights.get(&port_name) {
 match entry.right_type {
 RightType::Send | RightType::SendOnce => return Ok(()),
 _ => return Err(IpcError::NoSendPermission),
 }
 }
 }
 
 Err(IpcError::NoSendPermission)
 }

 /// CheckTaskifhaveReceivePermission
 pub fn check_receive(&self, task_id: TaskId, port_name: PortName) -> Result<(), IpcError> {
 let table = self.rights_table.lock();
 
 if let Some(task_rights) = table.get(&task_id) {
 if let Some(entry) = task_rights.get(&port_name) {
 if entry.right_type == RightType::Receive {
 return Ok(());
 }
 }
 }
 
 Err(IpcError::NoReceivePermission)
 }

 /// CheckTaskiffiniteexpfixedPermission
 pub fn check_right(
 &self,
 task_id: TaskId,
 port_name: PortName,
 right_type: RightType,
 ) -> bool {
 let table = self.rights_table.lock();
 
 if let Some(task_rights) = table.get(&task_id) {
 if let Some(entry) = task_rights.get(&port_name) {
 return entry.right_type == right_type;
 }
 }
 
 false
 }

 /// GetTask PortPermissionType
 pub fn get_right_type(
 &self,
 task_id: TaskId,
 port_name: PortName,
 ) -> Option<RightType> {
 let table = self.rights_table.lock();
 
 table
 .get(&task_id)
 .and_then(|task_rights| task_rights.get(&port_name))
 .map(|entry| entry.right_type)
 }

 /// transmitPortPermissiontootheraitemTask
 pub fn transfer_right(
 &self,
 from_task: TaskId,
 to_task: TaskId,
 port_name: PortName,
 disposition: RightDisposition,
 ) -> Result<PortName, IpcError> {
 // GetsourcePermissionType
 let source_type = self.get_right_type(from_task, port_name)
 .ok_or(IpcError::NoSendPermission)?;
 
 // RootevidencetransmitmethodstylecertainfixedtargetPermissionType
 let target_type = match disposition {
 RightDisposition::CopySend => {
 // CopySendPermission, sourceprotected
 if source_type != RightType::Send && source_type != RightType::Receive {
 return Err(IpcError::NoSendPermission);
 }
 RightType::Send
 }
 RightDisposition::MoveSend => {
 // MoveSendPermission, sourcelosego
 if source_type != RightType::Send {
 return Err(IpcError::NoSendPermission);
 }
 self.revoke_right(from_task, port_name)?;
 RightType::Send
 }
 RightDisposition::MakeSend => {
 // fromReceivePermissionCreateSendPermission
 if source_type != RightType::Receive {
 return Err(IpcError::NoReceivePermission);
 }
 RightType::Send
 }
 RightDisposition::CopyReceive => {
 // CopyReceivePermission (constantnotEnable)
 return Err(IpcError::PermissionDenied);
 }
 RightDisposition::MoveReceive => {
 // MoveReceivePermission
 if source_type != RightType::Receive {
 return Err(IpcError::NoReceivePermission);
 }
 self.revoke_right(from_task, port_name)?;
 RightType::Receive
 }
 RightDisposition::MakeSendOnce => {
 // CreateatimeitySendPermission
 if source_type != RightType::Receive {
 return Err(IpcError::NoReceivePermission);
 }
 RightType::SendOnce
 }
 RightDisposition::MoveSendOnce => {
 // MoveatimeitySendPermission
 if source_type != RightType::SendOnce {
 return Err(IpcError::NoSendPermission);
 }
 self.revoke_right(from_task, port_name)?;
 RightType::SendOnce
 }
 };
 
 // targetTaskPermission
 self.grant_right(to_task, port_name, target_type)?;
 
 Ok(port_name)
 }

 /// clearadministrationTask placefinitePermission
 pub fn cleanup_task(&self, task_id: TaskId) {
 let mut table = self.rights_table.lock();
 
 if let Some(task_rights) = table.remove(&task_id) {
 // MinusfewplacefinitePort referenceCount
 for (port_name, _) in task_rights {
 self.dec_port_ref(port_name);
 }
 }
 }

 /// GetTask Permissioncount
 pub fn right_count(&self, task_id: TaskId) -> usize {
 let table = self.rights_table.lock();
 table.get(&task_id).map(|r| r.len()).unwrap_or(0)
 }

 /// increasePlusPortreferenceCount
 fn inc_port_ref(&self, port_name: PortName) {
 let mut refs = self.port_refs.lock();
 if let Some(count) = refs.get(&port_name) {
 count.fetch_add(1, Ordering::AcqRel);
 } else {
 refs.insert(port_name, AtomicU32::new(1));
 }
 }

 /// MinusfewPortreferenceCount
 fn dec_port_ref(&self, port_name: PortName) {
 let refs = self.port_refs.lock();
 if let Some(count) = refs.get(&port_name) {
 count.fetch_sub(1, Ordering::AcqRel);
 }
 }

 /// GetPortreferenceCount
 pub fn port_ref_count(&self, port_name: PortName) -> u32 {
 let refs = self.port_refs.lock();
 refs.get(&port_name)
 .map(|c| c.load(Ordering::Acquire))
 .unwrap_or(0)
 }
}

impl Default for RightsManager {
 fn default() -> Self {
 Self::new()
 }
}

/// Permissiontransmitmethodstyle
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightDisposition {
 /// CopySendPermission
 CopySend = 0,
 /// CreateSendPermission
 MakeSend = 1,
 /// MoveSendPermission
 MoveSend = 2,
 /// CopyReceivePermission
 CopyReceive = 3,
 /// MoveReceivePermission
 MoveReceive = 4,
 /// CreateatimeitySendPermission
 MakeSendOnce = 5,
 /// MoveatimeitySendPermission
 MoveSendOnce = 6,
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_rights_grant() {
 let manager = RightsManager::new();
 
 manager.grant_right(1, 100, RightType::Send).unwrap();
 assert!(manager.check_send(1, 100).is_ok());
 assert!(manager.check_receive(1, 100).is_err());
 }

 #[test]
 fn test_rights_revoke() {
 let manager = RightsManager::new();
 
 manager.grant_right(1, 100, RightType::Send).unwrap();
 manager.revoke_right(1, 100).unwrap();
 assert!(manager.check_send(1, 100).is_err());
 }

 #[test]
 fn test_rights_transfer() {
 let manager = RightsManager::new();
 
 // Task 1 ownfiniteSendPermission
 manager.grant_right(1, 100, RightType::Send).unwrap();
 
 // transmitgiveTask 2
 manager.transfer_right(1, 2, 100, RightDisposition::CopySend).unwrap();
 
 // itemTaskshouldthefinitePermission
 assert!(manager.check_send(1, 100).is_ok());
 assert!(manager.check_send(2, 100).is_ok());
 }
}