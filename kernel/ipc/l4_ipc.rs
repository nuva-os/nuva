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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_debug, pr_info};

/// endDot ID Type
pub type EndpointId = u32;

/// canForceType
pub type Capability = u64;

/// MessageLabel
#[derive(Debug, Clone, Copy)]
pub struct MessageTag {
 /// MessageType
 pub msg_type: u16,
 /// MessageFlag
 pub flags: u16,
}

/// L4 Message
#[derive(Debug, Clone, Copy)]
pub struct L4Message {
 /// MessageLabel
 pub tag: MessageTag,
 /// MessageRegister (directacceptoverRegistertransmit)
 pub regs: [u64; 4],
 /// canForce
 pub caps: [Capability; 2],
}

impl L4Message {
 /// CreatenewMessage
 pub fn new(msg_type: u16) -> Self {
 L4Message {
 tag: MessageTag {
 msg_type,
 flags: 0,
 },
 regs: [0; 4],
 caps: [0; 2],
 }
 }
 
 /// SetRegistervalue
 pub fn set_reg(&mut self, idx: usize, value: u64) {
 if idx < 4 {
 self.regs[idx] = value;
 }
 }
 
 /// GetRegistervalue
 pub fn get_reg(&self, idx: usize) -> u64 {
 if idx < 4 {
 self.regs[idx]
 } else {
 0
 }
 }
 
 /// SetcanForce
 pub fn set_cap(&mut self, idx: usize, cap: Capability) {
 if idx < 2 {
 self.caps[idx] = cap;
 }
 }
 
 /// GetcanForce
 pub fn get_cap(&self, idx: usize) -> Capability {
 if idx < 2 {
 self.caps[idx]
 } else {
 0
 }
 }
}

/// endDotState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointState {
 /// emptyidle
 Idle = 0,
 /// WaitSend
 SendWaiting = 1,
 /// WaitReceive
 RecvWaiting = 2,
 /// active
 Active = 3,
}

/// endDot
pub struct Endpoint {
 /// endDot ID
 pub id: EndpointId,
 /// placebelongProcess ID
 pub owner_pid: u32,
 /// State
 pub state: AtomicU32,
 /// Message Queue
 pub msg_queue: [L4Message; 16],
 /// QueueHead
 pub queue_head: AtomicU32,
 /// QueueTail
 pub queue_tail: AtomicU32,
 /// Wait Sender
 pub waiting_sender: AtomicU32,
 /// Wait Receiveer
 pub waiting_receiver: AtomicU32,
}

impl Endpoint {
 /// Create newendDot
 pub fn new(id: EndpointId, owner_pid: u32) -> Self {
 Endpoint {
 id,
 owner_pid,
 state: AtomicU32::new(EndpointState::Idle as u32),
 msg_queue: [L4Message::new(0); 16],
 queue_head: AtomicU32::new(0),
 queue_tail: AtomicU32::new(0),
 waiting_sender: AtomicU32::new(0),
 waiting_receiver: AtomicU32::new(0),
 }
 }
 
 /// GetState
 pub fn get_state(&self) -> EndpointState {
 match self.state.load(Ordering::Acquire) {
 0 => EndpointState::Idle,
 1 => EndpointState::SendWaiting,
 2 => EndpointState::RecvWaiting,
 3 => EndpointState::Active,
 _ => EndpointState::Idle,
 }
 }
 
 /// SetState
 pub fn set_state(&self, state: EndpointState) {
 self.state.store(state as u32, Ordering::Release);
 }
}

/// IPC Error code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
 /// Success
 Success = 0,
 /// invalidendDot
 InvalidEndpoint = 1,
 /// endDotalreadyClose
 EndpointClosed = 2,
 /// SendTimeout
 SendTimeout = 3,
 /// ReceiveTimeout
 RecvTimeout = 4,
 /// Permissionnotmeet
 PermissionDenied = 5,
 /// Messageoverlarge
 MessageTooLarge = 6,
 /// Queuealreadysatisfy
 QueueFull = 7,
}

/// L4 IPC System
pub struct L4IpcSystem {
 /// endDotcount
 endpoint_count: AtomicU32,
 /// NextendDot ID
 next_endpoint_id: AtomicU32,
 /// SendCount
 send_count: AtomicU64,
 /// ReceiveCount
 recv_count: AtomicU64,
}

impl L4IpcSystem {
 pub const fn new() -> Self {
 L4IpcSystem {
 endpoint_count: AtomicU32::new(0),
 next_endpoint_id: AtomicU32::new(1),
 send_count: AtomicU64::new(0),
 recv_count: AtomicU64::new(0),
 }
 }
 
 /// Initialize
 pub fn init(&mut self) {
 log_info!("L4 IPC system initialized");
 log_info!(" Message registers: 4");
 log_info!(" Capability slots: 2");
 }
 
 /// CreateendDot
 pub fn create_endpoint(&self, owner_pid: u32) -> EndpointId {
 let id = self.next_endpoint_id.fetch_add(1, Ordering::AcqRel);
 self.endpoint_count.fetch_add(1, Ordering::AcqRel);
 
 log_debug!("Created endpoint: {} for process {}", id, owner_pid);
 
 id
 }
 
 /// SendMessage (Synchronous)
 #[inline(always)]
 pub fn send(&self, _endpoint: EndpointId, _msg: &L4Message) -> Result<(), IpcError> {
 // increasePlusSendCount
 self.send_count.fetch_add(1, Ordering::AcqRel);
 
 // TODO: Implementationrealactual IPC Send
 // 1. CheckendDotvalidity
 // 2. CheckPermission
 // 3. ifReceiveerpositiveinWait,directaccepttransmit
 // 4. whetherprinciplereleaseenterQueue
 
 Ok(())
 }
 
 /// ReceiveMessage (Synchronous)
 #[inline(always)]
 pub fn recv(&self, _endpoint: EndpointId, _msg: &mut L4Message) -> Result<(), IpcError> {
 // increasePlusReceiveCount
 self.recv_count.fetch_add(1, Ordering::AcqRel);
 
 // TODO: Implementationrealactual IPC Receive
 // 1. CheckendDotvalidity
 // 2. CheckQueueifhaveMessage
 // 3. iffinite,directacceptReturn
 // 4. elseBlockingWait
 
 Ok(())
 }
 
 /// SendparallelReceive (Call)
 #[inline(always)]
 pub fn call(&self, endpoint: EndpointId, msg: &mut L4Message) -> Result<(), IpcError> {
 self.send(endpoint, msg)?;
 self.recv(endpoint, msg)?;
 Ok(())
 }
 
 /// roundrestoreMessage
 #[inline(always)]
 pub fn reply(&self, _endpoint: EndpointId, _msg: &L4Message) -> Result<(), IpcError> {
 // TODO: Implementationroundrestore
 Ok(())
 }
 
 /// GetendDotcount
 pub fn get_endpoint_count(&self) -> u32 {
 self.endpoint_count.load(Ordering::Acquire)
 }
 
 /// GetSendCount
 pub fn get_send_count(&self) -> u64 {
 self.send_count.load(Ordering::Acquire)
 }
 
 /// GetReceiveCount
 pub fn get_recv_count(&self) -> u64 {
 self.recv_count.load(Ordering::Acquire)
 }
 
 /// Get IPC throughputquantification
 pub fn get_throughput(&self) -> u64 {
 self.send_count.load(Ordering::Acquire) + self.recv_count.load(Ordering::Acquire)
 }
}

/// Global L4 IPC System
static L4_IPC_SYSTEM: crate::sync_oncelock::OnceLock<L4IpcSystem> = crate::sync_oncelock::OnceLock::new();

pub fn l4_ipc() -> &'static L4IpcSystem {
    L4_IPC_SYSTEM.get_or_init(L4IpcSystem::new)
}

pub fn init_l4_ipc() {
 let ipc = l4_ipc();
 ipc.init();
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_message_tag() {
 let tag = MessageTag {
 msg_type: 100,
 flags: 1,
 };

 assert_eq!(tag.msg_type, 100);
 assert_eq!(tag.flags, 1);
 }

 #[test]
 fn test_l4_message_new() {
 let msg = L4Message::new(42);

 assert_eq!(msg.tag.msg_type, 42);
 assert_eq!(msg.tag.flags, 0);
 assert_eq!(msg.regs, [0; 4]);
 assert_eq!(msg.caps, [0; 2]);
 }

 #[test]
 fn test_l4_message_regs() {
 let mut msg = L4Message::new(0);

 msg.set_reg(0, 100);
 msg.set_reg(1, 200);
 msg.set_reg(2, 300);
 msg.set_reg(3, 400);

 assert_eq!(msg.get_reg(0), 100);
 assert_eq!(msg.get_reg(1), 200);
 assert_eq!(msg.get_reg(2), 300);
 assert_eq!(msg.get_reg(3), 400);
 }

 #[test]
 fn test_l4_message_reg_out_of_bounds() {
 let mut msg = L4Message::new(0);

 msg.set_reg(0, 100);
 assert_eq!(msg.get_reg(0), 100);

 // exceedboundaryaccessReturn 0
 assert_eq!(msg.get_reg(4), 0);
 assert_eq!(msg.get_reg(5), 0);
 }

 #[test]
 fn test_l4_message_caps() {
 let mut msg = L4Message::new(0);

 msg.set_cap(0, 0x12345678);
 msg.set_cap(1, 0xABCDEF00);

 assert_eq!(msg.get_cap(0), 0x12345678);
 assert_eq!(msg.get_cap(1), 0xABCDEF00);
 }

 #[test]
 fn test_l4_message_cap_out_of_bounds() {
 let mut msg = L4Message::new(0);

 msg.set_cap(0, 100);
 assert_eq!(msg.get_cap(0), 100);

 // exceedboundaryaccessReturn 0
 assert_eq!(msg.get_cap(2), 0);
 assert_eq!(msg.get_cap(3), 0);
 }

 #[test]
 fn test_endpoint_state_values() {
 assert_eq!(EndpointState::Idle as u32, 0);
 assert_eq!(EndpointState::SendWaiting as u32, 1);
 assert_eq!(EndpointState::RecvWaiting as u32, 2);
 assert_eq!(EndpointState::Active as u32, 3);
 }

 #[test]
 fn test_endpoint_new() {
 let ep = Endpoint::new(1, 100);

 assert_eq!(ep.id, 1);
 assert_eq!(ep.owner_pid, 100);
 assert_eq!(ep.get_state(), EndpointState::Idle);
 assert_eq!(ep.queue_head.load(Ordering::Relaxed), 0);
 assert_eq!(ep.queue_tail.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_endpoint_state_transitions() {
 let ep = Endpoint::new(1, 100);

 assert_eq!(ep.get_state(), EndpointState::Idle);

 ep.set_state(EndpointState::SendWaiting);
 assert_eq!(ep.get_state(), EndpointState::SendWaiting);

 ep.set_state(EndpointState::RecvWaiting);
 assert_eq!(ep.get_state(), EndpointState::RecvWaiting);

 ep.set_state(EndpointState::Active);
 assert_eq!(ep.get_state(), EndpointState::Active);
 }

 #[test]
 fn test_ipc_error_values() {
 assert_eq!(IpcError::Success as u32, 0);
 assert_eq!(IpcError::InvalidEndpoint as u32, 1);
 assert_eq!(IpcError::EndpointClosed as u32, 2);
 assert_eq!(IpcError::SendTimeout as u32, 3);
 assert_eq!(IpcError::RecvTimeout as u32, 4);
 assert_eq!(IpcError::PermissionDenied as u32, 5);
 assert_eq!(IpcError::MessageTooLarge as u32, 6);
 assert_eq!(IpcError::QueueFull as u32, 7);
 }

 #[test]
 fn test_ipc_error_equality() {
 assert_eq!(IpcError::Success, IpcError::Success);
 assert_ne!(IpcError::Success, IpcError::InvalidEndpoint);
 }

 #[test]
 fn test_l4_ipc_system_new() {
 let ipc = L4IpcSystem::new();

 assert_eq!(ipc.get_endpoint_count(), 0);
 assert_eq!(ipc.get_send_count(), 0);
 assert_eq!(ipc.get_recv_count(), 0);
 assert_eq!(ipc.get_throughput(), 0);
 }

 #[test]
 fn test_l4_ipc_system_create_endpoint() {
 let ipc = L4IpcSystem::new();

 let id1 = ipc.create_endpoint(100);
 assert_eq!(id1, 1);
 assert_eq!(ipc.get_endpoint_count(), 1);

 let id2 = ipc.create_endpoint(200);
 assert_eq!(id2, 2);
 assert_eq!(ipc.get_endpoint_count(), 2);
 }

 #[test]
 fn test_l4_ipc_system_send() {
 let ipc = L4IpcSystem::new();
 let msg = L4Message::new(1);

 let result = ipc.send(1, &msg);
 assert!(result.is_ok());
 assert_eq!(ipc.get_send_count(), 1);
 }

 #[test]
 fn test_l4_ipc_system_recv() {
 let ipc = L4IpcSystem::new();
 let mut msg = L4Message::new(0);

 let result = ipc.recv(1, &mut msg);
 assert!(result.is_ok());
 assert_eq!(ipc.get_recv_count(), 1);
 }

 #[test]
 fn test_l4_ipc_system_call() {
 let ipc = L4IpcSystem::new();
 let mut msg = L4Message::new(1);

 let result = ipc.call(1, &mut msg);
 assert!(result.is_ok());
 assert_eq!(ipc.get_send_count(), 1);
 assert_eq!(ipc.get_recv_count(), 1);
 }

 #[test]
 fn test_l4_ipc_system_reply() {
 let ipc = L4IpcSystem::new();
 let msg = L4Message::new(1);

 let result = ipc.reply(1, &msg);
 assert!(result.is_ok());
 }

 #[test]
 fn test_l4_ipc_system_throughput() {
 let ipc = L4IpcSystem::new();
 let msg = L4Message::new(1);
 let mut recv_msg = L4Message::new(0);

 ipc.send(1, &msg).unwrap();
 ipc.send(1, &msg).unwrap();
 ipc.recv(1, &mut recv_msg).unwrap();

 assert_eq!(ipc.get_throughput(), 3);
 }

 #[test]
 fn test_endpoint_message_queue() {
 let ep = Endpoint::new(1, 100);

 // Message QueueInitializeasempty
 assert_eq!(ep.queue_head.load(Ordering::Relaxed), 0);
 assert_eq!(ep.queue_tail.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_endpoint_waiting_threads() {
 let ep = Endpoint::new(1, 100);

 assert_eq!(ep.waiting_sender.load(Ordering::Relaxed), 0);
 assert_eq!(ep.waiting_receiver.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_l4_message_copy() {
 let mut msg = L4Message::new(42);
 msg.set_reg(0, 12345);
 msg.set_cap(0, 67890);

 let copied = msg;
 assert_eq!(copied.tag.msg_type, 42);
 assert_eq!(copied.get_reg(0), 12345);
 assert_eq!(copied.get_cap(0), 67890);
 }
}