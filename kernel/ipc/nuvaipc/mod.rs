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

//! Nuva IPC ChildSystem
/*!*/
// ! Nuva OS sourcecreate IPC machinecontrol, higheffect Processbetweenmessagemachinecontrol.
/*!*/
// ! # DesignDot
/*!*/
// ! - **PortNamespace**: PeritemTaskownfiniteexclusivecube PortNamespace
// ! - **PortPermission**: Support Send、Receive、SendOnce、PortSet、DeadName Permission
// ! - **Zero-copy**: largeMessageSupportoutsideData (Out-of-line Data) Zero-copytransmit
// ! - **MessagePriority**: SupportMessagePriorityQueue
// ! - **highPerformance**: smallMessage < 100ns, largeMessage < 10μs(Zero-copy)
/*!*/
// ! # Performancelogratio
/*!*/
// ! | System | smallMessageDelay | largeMessageDelay |
//! |------|-----------|-----------|
//! | Android Binder | ~1μs | ~100μs |
//! | iOS XPC | ~2μs | ~200μs |
//! | NuvaIPC | <100ns | <10μs |
/*!*/
//! # Example
/*!*/
//! ```rust
//! use kernel::ipc::nuvaipc::{PortManager, MachMessage, SendOptions};
/*!*/
//! // CreatePort
//! let port = port_manager.port_create(&task)?;
/*!*/
//! // SendMessage
//! let msg = MachMessage::new_small(b"hello");
//! port_manager.ipc_send(&task, port, msg, SendOptions::default())?;
/*!*/
//! // ReceiveMessage
//! let received = port_manager.ipc_receive(&task, port, ReceiveOptions::default())?;
//! ```

mod port;
mod message;
mod rights;
mod queue;
mod manager;
mod fastpath;
mod quantum_secure;

pub use port::{MachPort, PortName, PortRights, PortState, PortNamespace};
pub use message::{MachMessage, MessageHeader, MessageBody, MessageBits, PortDescriptor};
pub use rights::{RightsManager, RightType, RightDisposition};
pub use queue::{MessageQueue, QueuePriority};
pub use manager::PortManager;
pub use fastpath::{
 FastPathIpc, ZeroCopyManager, ZeroCopyDescriptor, LockFreeQueue,
 BatchProcessor, IpcStats, IPC_STATS,
 SMALL_MESSAGE_SIZE, MEDIUM_MESSAGE_SIZE, BATCH_SIZE,
};
pub use quantum_secure::{
 QuantumEncryption, AIOptimizer, SmartRouter, EnhancedIpc,
 EnhancedIpcStats, PerformanceData, ENHANCED_IPC,
};

/// IPC ErrorType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
 /// Portnotexist
 PortNotFound,
 /// noneSendPermission
 NoSendPermission,
 /// noneReceivePermission
 NoReceivePermission,
 /// PortDead
 PortDead,
 /// MessageTimeout
 Timeout,
 /// Non-blockingOperationinfinitelawComplete
 WouldBlock,
 /// Insufficient memory
 NoMemory,
 /// invalidParameter
 InvalidArgument,
 /// Permissionnotmeet
 PermissionDenied,
 /// Messageoverlarge
 MessageTooLarge,
 /// Namespacealreadysatisfy
 NamespaceFull,
}

/// SendOption
#[derive(Debug, Clone, Copy, Default)]
pub struct SendOptions {
 /// ifBlocking
 pub block: bool,
 /// TimeoutTime (ms), 0 forminfinitelimitWait
 pub timeout_ms: u32,
 /// MessagePriority
 pub priority: QueuePriority,
 /// iftransmitCredential
 pub voucher: bool,
}

/// ReceiveOption
#[derive(Debug, Clone, Copy, Default)]
pub struct ReceiveOptions {
 /// ifBlocking
 pub block: bool,
 /// TimeoutTime (ms), 0 forminfinitelimitWait
 pub timeout_ms: u32,
 /// ifReceiveCredential
 pub voucher: bool,
}

impl ReceiveOptions {
 /// CreateNon-blockingReceiveOption
 pub fn no_wait() -> Self {
 Self {
 block: false,
 timeout_ms: 0,
 voucher: false,
 }
 }
}

/// Port ID Type
pub type PortId = u64;

/// Task ID Type
pub type TaskId = u32;

/// SharedMemory ID
pub type ShmId = u32;