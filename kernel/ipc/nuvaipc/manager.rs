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

//! Nuva IPC PortManager
/*!*/
// ! managementadministrationplacefinitePortsumNamespace kernelComponent.

use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex as SpinLock;

use super::{
 IpcError, TaskId, PortId, PortName, SendOptions, ReceiveOptions,
 MachMessage, MachPort, PortNamespace, PortRights, RightType,
 RightsManager, QueuePriority,
};

/// GlobalPort ID Allocatedevice
static PORT_ID_ALLOCATOR: AtomicU64 = AtomicU64::new(1);

/// PortManager
pub struct PortManager {
 /// Namespaceform: TaskId -> PortNamespace
 namespaces: SpinLock<BTreeMap<TaskId, Arc<PortNamespace>>>,
 /// GlobalPortform: PortId -> MachPort
 global_ports: SpinLock<BTreeMap<PortId, Arc<MachPort>>>,
 /// PermissionManager
 rights_manager: RightsManager,
 /// PortNameto ID Map
 name_to_id: SpinLock<BTreeMap<PortName, PortId>>,
}

impl PortManager {
 /// CreatenewPortManager
 pub fn new() -> Self {
 Self {
 namespaces: SpinLock::new(BTreeMap::new()),
 global_ports: SpinLock::new(BTreeMap::new()),
 rights_manager: RightsManager::new(),
 name_to_id: SpinLock::new(BTreeMap::new()),
 }
 }

 /// CreateNamespace
 pub fn create_namespace(&self, task_id: TaskId) -> Arc<PortNamespace> {
 let ns = Arc::new(PortNamespace::new(task_id));
 self.namespaces.lock().insert(task_id, ns.clone());
 ns
 }

 /// GetNamespace
 pub fn get_namespace(&self, task_id: TaskId) -> Result<Arc<PortNamespace>, IpcError> {
 self.namespaces
 .lock()
 .get(&task_id)
 .cloned()
 .ok_or(IpcError::PortNotFound)
 }

 /// DestroyNamespace
 pub fn destroy_namespace(&self, task_id: TaskId) {
 // clearadministrationPermission
 self.rights_manager.cleanup_task(task_id);
 
 // DivideNamespace
 if let Some(ns) = self.namespaces.lock().remove(&task_id) {
 // clearadministrationplacefinitePort
 // Note: PortNamespace doesn't support iteration, ports will be dropped
 let _ = ns;
 }
 }

 /// CreatenewPort
 pub fn port_create(&self, task_id: TaskId) -> Result<PortName, IpcError> {
 let ns = self.get_namespace(task_id)?;
 
 // AllocateGlobalPort ID
 let port_id = PORT_ID_ALLOCATOR.fetch_add(1, Ordering::AcqRel);
 
 // CreatePort
 let port = Arc::new(MachPort::new(port_id));
 port.set_receiver(task_id);
 
 // AllocatePortName
 let port_name = ns.allocate_name();
 
 // SetinitialbeginPermission (CreateerownfiniteReceivePermission)
 port.set_rights(PortRights::RECEIVE);
 self.rights_manager.grant_right(task_id, port_name, RightType::Receive)?;
 
 // RegistertoNamespace
 ns.insert(port_name, port.clone());
 ns.set_rights(port_name, PortRights::RECEIVE);
 
 // RegistertoGlobalform
 self.global_ports.lock().insert(port_id, port);
 self.name_to_id.lock().insert(port_name, port_id);
 
 Ok(port_name)
 }

 /// DestroyPort
 pub fn port_destroy(&self, task_id: TaskId, port_name: PortName) -> Result<(), IpcError> {
 // CheckReceivePermission
 self.rights_manager.check_receive(task_id, port_name)?;
 
 let ns = self.get_namespace(task_id)?;
 
 // GetPort
 let port = ns.lookup(port_name).ok_or(IpcError::PortNotFound)?;
 
 // Markerasdeadperish
 port.mark_dead();
 
 // secondaryNamespaceDivide
 ns.remove(port_name);
 
 // secondaryGlobalformDivide
 if let Some(id) = self.name_to_id.lock().remove(&port_name) {
 self.global_ports.lock().remove(&id);
 }
 
 Ok(())
 }

 /// SendMessage
 pub fn ipc_send(
 &self,
 task_id: TaskId,
 port_name: PortName,
 message: MachMessage,
 options: SendOptions,
 ) -> Result<(), IpcError> {
 // CheckSendPermission
 self.rights_manager.check_send(task_id, port_name)?;
 
 // GetNamespace
 let ns = self.get_namespace(task_id)?;
 
 // GettargetPort
 let port = ns.lookup(port_name).ok_or(IpcError::PortNotFound)?;
 
 // CheckPortState
 if !port.is_active() {
 return Err(IpcError::PortDead);
 }
 
 // HandleatimeitySendPermission
 let right_type = self.rights_manager.get_right_type(task_id, port_name);
 if right_type == Some(RightType::SendOnce) {
 // atimeityPermissionusethenselfdynamic
 self.rights_manager.revoke_right(task_id, port_name)?;
 }
 
 // enterqueueMessage
 if options.priority > QueuePriority::Default {
 port.enqueue(message)?;
 } else {
 port.enqueue(message)?;
 }
 
 // WakeWaiter
 if let Some(waiter) = port.wake_one_waiter() {
 // WakeWaitReceive Task
 // realactualImplementationinfixshouldthetuneusetuneDegreedeviceWakeTask
 }
 
 Ok(())
 }

 /// ReceiveMessage
 pub fn ipc_receive(
 &self,
 task_id: TaskId,
 port_name: PortName,
 options: ReceiveOptions,
 ) -> Result<MachMessage, IpcError> {
 // CheckReceivePermission
 self.rights_manager.check_receive(task_id, port_name)?;
 
 // GetNamespace
 let ns = self.get_namespace(task_id)?;
 
 // GetPort
 let port = ns.lookup(port_name).ok_or(IpcError::PortNotFound)?;
 
 loop {
 // tryexitqueue
 if let Some(msg) = port.dequeue() {
 return Ok(msg);
 }
 
 // CheckPortState
 if !port.is_active() {
 return Err(IpcError::PortDead);
 }
 
 // Non-blockingMode
 if !options.block {
 return Err(IpcError::WouldBlock);
 }
 
 // addPlustoWaitQueue
 port.add_waiter(task_id);
 
 // BlockingWait
 // realactualImplementationinfixshouldtheletexit CPU, WaitbyWake
 // thisSimplifiedHandle, directacceptReturn WouldBlock
 return Err(IpcError::WouldBlock);
 }
 }

 /// transmitPortPermission
 pub fn port_transfer_right(
 &self,
 from_task: TaskId,
 to_task: TaskId,
 port_name: PortName,
 disposition: super::RightDisposition,
 ) -> Result<PortName, IpcError> {
 self.rights_manager.transfer_right(from_task, to_task, port_name, disposition)
 }

 /// GetPort
 pub fn get_port(&self, port_name: PortName) -> Result<Arc<MachPort>, IpcError> {
 let id = self.name_to_id.lock()
 .get(&port_name)
 .copied()
 .ok_or(IpcError::PortNotFound)?;
 
 self.global_ports.lock()
 .get(&id)
 .cloned()
 .ok_or(IpcError::PortNotFound)
 }

 /// GetPortState
 pub fn port_status(&self, port_name: PortName) -> Result<super::PortState, IpcError> {
 let port = self.get_port(port_name)?;
 Ok(port.state())
 }

 /// GetPortQueueLength
 pub fn port_queue_len(&self, port_name: PortName) -> Result<usize, IpcError> {
 let port = self.get_port(port_name)?;
 Ok(port.queue_len())
 }

 /// SetPortContext
 pub fn port_set_context(
 &self,
 task_id: TaskId,
 port_name: PortName,
 context: usize,
 ) -> Result<(), IpcError> {
 self.rights_manager.check_receive(task_id, port_name)?;
 let port = self.get_port(port_name)?;
 port.set_context(context as u32);
 Ok(())
 }

 /// GetPortContext
 pub fn port_get_context(&self, port_name: PortName) -> Result<usize, IpcError> {
 let port = self.get_port(port_name)?;
 Ok(port.context() as usize)
 }

 /// Get statistics
 pub fn stats(&self) -> PortManagerStats {
 PortManagerStats {
 namespace_count: self.namespaces.lock().len(),
 port_count: self.global_ports.lock().len(),
 }
 }
}

impl Default for PortManager {
 fn default() -> Self {
 Self::new()
 }
}

/// PortManagerstatisticsInfo
#[derive(Debug, Clone, Copy, Default)]
pub struct PortManagerStats {
 /// Namespacecount
 pub namespace_count: usize,
 /// Portcount
 pub port_count: usize,
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_port_create() {
 let manager = PortManager::new();
 manager.create_namespace(1);
 
 let port = manager.port_create(1).unwrap();
 assert!(port > 0);
 }

 #[test]
 fn test_ipc_send_receive() {
 let manager = PortManager::new();
 manager.create_namespace(1);
 
 let port = manager.port_create(1).unwrap();
 
 // SendMessage
 let msg = MachMessage::new_small(b"hello");
 manager.ipc_send(1, port, msg, SendOptions::default()).unwrap();
 
 // ReceiveMessage
 let received = manager.ipc_receive(1, port, ReceiveOptions::no_wait()).unwrap();
 assert_eq!(received.data(), b"hello");
 }

 #[test]
 fn test_port_destroy() {
 let manager = PortManager::new();
 manager.create_namespace(1);
 
 let port = manager.port_create(1).unwrap();
 manager.port_destroy(1, port).unwrap();
 
 // DestroythenshouldtheinfinitelawSend
 let msg = MachMessage::new_small(b"test");
 assert!(manager.ipc_send(1, port, msg, SendOptions::default()).is_err());
 }
}