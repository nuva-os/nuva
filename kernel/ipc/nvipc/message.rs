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

//! Nuva IPC MessageImplementation
/*!*/
// ! Messageis Nuva IPC Datatransmitform.

use core::mem::size_of;
use alloc::vec::Vec;
use alloc::boxed::Box;

use super::{PortName, ShmId};

/// MessageFlagBit
bitflags::bitflags! {
 /// MessageFlag
 #[repr(transparent)]
 pub struct MessageBits: u32 {
 /// restorehybridMessage (PackagePortDescriptor)
 const COMPLEX = 1 << 0;
 /// outsideData (Zero-copy)
 const OOL_DATA = 1 << 1;
 /// outsidePort
 const OOL_PORTS = 1 << 2;
 /// PackageCredential
 const VOUCHER = 1 << 3;
 /// PriorityMask (high 8 Bit)
 const PRIORITY_MASK = 0xFF << 24;
 }

}

impl Clone for MessageBits {
    fn clone(&self) -> Self { *self }
}
impl Copy for MessageBits {}
impl core::fmt::Debug for MessageBits {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MessageBits({:#x})", self.bits())
    }
}

/// MessagePriority
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
 /// lowPriority
 Low = 0,
 /// DefaultPriority
 Default = 1,
 /// highPriority
 High = 2,
 /// realtimePriority
 RealTime = 3,
}

impl Default for MessagePriority {
 fn default() -> Self {
 Self::Default
 }
}

/// MessageHead
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MessageHeader {
 /// MessageFlagBit
 pub bits: MessageBits,
 /// MessageSize (PackageHeadsumVolume)
 pub size: u32,
 /// targetPort
 pub remote_port: PortName,
 /// roundrestorePort
 pub local_port: PortName,
 /// CredentialPort
 pub voucher_port: PortName,
 /// Message ID (usestandardidentifierMessageType)
 pub id: u32,
}

impl Default for MessageHeader {
 fn default() -> Self {
 Self {
 bits: MessageBits::empty(),
 size: size_of::<MessageHeader>() as u32,
 remote_port: 0,
 local_port: 0,
 voucher_port: 0,
 id: 0,
 }
 }
}

impl MessageHeader {
 /// CreatenewMessageHead
 pub fn new(remote_port: PortName) -> Self {
 Self {
 remote_port,
 ..Default::default()
 }
 }

 /// SetMessage ID
 pub fn with_id(mut self, id: u32) -> Self {
 self.id = id;
 self
 }

 /// SetroundrestorePort
 pub fn with_reply_port(mut self, port: PortName) -> Self {
 self.local_port = port;
 self
 }

 /// SetPriority
 pub fn with_priority(mut self, priority: MessagePriority) -> Self {
 self.bits |= MessageBits::from_bits_truncate((priority as u32) << 24);
 self
 }

 /// GetPriority
 pub fn priority(&self) -> MessagePriority {
 let priority_bits = (self.bits.bits() >> 24) & 0xFF;
 match priority_bits {
 0 => MessagePriority::Low,
 1 => MessagePriority::Default,
 2 => MessagePriority::High,
 3 => MessagePriority::RealTime,
 _ => MessagePriority::Default,
 }
 }

 /// CheckifasrestorehybridMessage
 pub fn is_complex(&self) -> bool {
 self.bits.contains(MessageBits::COMPLEX)
 }

 /// CheckifPackageoutsideData
 pub fn has_ool_data(&self) -> bool {
 self.bits.contains(MessageBits::OOL_DATA)
 }

 /// CheckifPackageoutsidePort
 pub fn has_ool_ports(&self) -> bool {
 self.bits.contains(MessageBits::OOL_PORTS)
 }
}

/// PortDescriptor (usetransmitPortPermission)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PortDescriptor {
 /// PortName
 pub name: PortName,
 /// PermissionHandlemethodstyle
 pub disposition: RightDisposition,
 /// Copymethodstyle
 pub copy: PortCopy,
}

/// PermissionHandlemethodstyle
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

/// PortCopymethodstyle
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortCopy {
 /// PhysicsCopy
 PhysicalCopy = 0,
 /// imaginarysimulatedCopy
 VirtualCopy = 1,
}

/// MessageVolume
#[derive(Debug, Clone)]
pub struct MessageBody {
 /// Descriptorcount
 pub descriptor_count: u32,
 /// insideData
 pub inline_data: Vec<u8>,
 /// PortDescriptorList
 pub port_descriptors: Vec<PortDescriptor>,
}

impl Default for MessageBody {
 fn default() -> Self {
 Self {
 descriptor_count: 0,
 inline_data: Vec::new(),
 port_descriptors: Vec::new(),
 }
 }
}

impl MessageBody {
 /// CreatenullMessageVolume
 pub fn new() -> Self {
 Self::default()
 }

 /// CreatebandData MessageVolume
 pub fn with_data(data: &[u8]) -> Self {
 Self {
 descriptor_count: 0,
 inline_data: data.to_vec(),
 port_descriptors: Vec::new(),
 }
 }

 /// addPortDescriptor
 pub fn add_port(&mut self, descriptor: PortDescriptor) {
 self.port_descriptors.push(descriptor);
 self.descriptor_count = self.port_descriptors.len() as u32;
 }

 /// GetDataSize
 pub fn data_size(&self) -> usize {
 self.inline_data.len()
 }

 /// Check if empty
 pub fn is_empty(&self) -> bool {
 self.inline_data.is_empty() && self.port_descriptors.is_empty()
 }
}

/// outsideDataDescriptor
#[derive(Debug, Clone)]
pub struct OolDataDescriptor {
 /// SharedMemory ID
 pub shm_id: ShmId,
 /// DataSize
 pub size: usize,
 /// ifneedFree
 pub deallocate: bool,
}

/// Message dispatch strategy for zero-copy optimization.
/// Determines how message data is transmitted based on size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDispatch {
    /// Register path: small messages (<=256 bytes) passed via registers
    RegisterPath,
    /// Shared memory path: large messages (>4KB) via shared memory
    ShmPath,
    /// Copy path: medium messages (256B < size <= 4KB) via traditional copy
    CopyPath,
}

/// Threshold for register-path dispatch (bytes)
const DISPATCH_REGISTER_THRESHOLD: usize = 256;

/// Threshold for shared-memory-path dispatch (bytes)
const DISPATCH_SHM_THRESHOLD: usize = 4096;

/// Determine the optimal dispatch strategy for a message of the given size.
/// Returns the dispatch strategy without any allocation or lock acquisition.
#[inline(always)]
pub fn message_dispatch_strategy(size: usize) -> MessageDispatch {
    if size <= DISPATCH_REGISTER_THRESHOLD {
        MessageDispatch::RegisterPath
    } else if size > DISPATCH_SHM_THRESHOLD {
        MessageDispatch::ShmPath
    } else {
        MessageDispatch::CopyPath
    }
}

/// Nuva IPC Message
#[derive(Debug, Clone)]
pub struct MachMessage {
 /// MessageHead
 pub header: MessageHeader,
 /// MessageVolume
 pub body: MessageBody,
 /// outsideData (Zero-copy)
 pub ool_data: Option<OolDataDescriptor>,
}

impl MachMessage {
 /// CreatesmallMessage (insideData)
 pub fn new_small(data: &[u8]) -> Self {
 let body = MessageBody::with_data(data);
 let size = size_of::<MessageHeader>() as u32 + body.data_size() as u32;
 
 Self {
 header: MessageHeader {
 size,
 ..Default::default()
 },
 body,
 ool_data: None,
 }
 }

 /// CreatelargeMessage (outsideData, Zero-copy)
 pub fn new_large(data: &[u8]) -> Self {
 // loglargeMessage, shouldthemakeuseSharedMemory
 // thisSimplifiedHandle, realactualshouldtheCreateSharedMemory
 let body = MessageBody::with_data(data);
 let size = size_of::<MessageHeader>() as u32 + body.data_size() as u32;
 
 Self {
 header: MessageHeader {
 bits: MessageBits::OOL_DATA,
 size,
 ..Default::default()
 },
 body,
 ool_data: None, // realactualshouldtheSetSharedMemoryDescriptor
 }
 }

 /// CreatebandSharedMemory Message
 pub fn with_shared_memory(shm_id: ShmId, size: usize) -> Self {
 Self {
 header: MessageHeader {
 bits: MessageBits::OOL_DATA,
 size: size_of::<MessageHeader>() as u32,
 ..Default::default()
 },
 body: MessageBody::new(),
 ool_data: Some(OolDataDescriptor {
 shm_id,
 size,
 deallocate: false,
 }),
 }
 }

 /// SettargetPort
 pub fn to(mut self, port: PortName) -> Self {
 self.header.remote_port = port;
 self
 }

 /// SetroundrestorePort
 pub fn reply_to(mut self, port: PortName) -> Self {
 self.header.local_port = port;
 self
 }

 /// SetMessage ID
 pub fn with_id(mut self, id: u32) -> Self {
 self.header.id = id;
 self
 }

 /// SetPriority
 pub fn with_priority(mut self, priority: MessagePriority) -> Self {
 self.header = self.header.with_priority(priority);
 self
 }

 /// addPortDescriptor
 pub fn add_port_descriptor(&mut self, descriptor: PortDescriptor) {
 self.body.add_port(descriptor);
 self.header.bits |= MessageBits::COMPLEX;
 }

 /// GetMessageSize
 pub fn size(&self) -> usize {
 self.header.size as usize
 }

 /// CheckifaslargeMessage
 pub fn is_large(&self) -> bool {
 self.header.has_ool_data() || self.body.data_size() > 4096
 }

 /// GetPriority
 pub fn priority(&self) -> super::QueuePriority {
 match self.header.priority() {
 MessagePriority::Low => super::QueuePriority::Low,
 MessagePriority::Default => super::QueuePriority::Default,
 MessagePriority::High => super::QueuePriority::High,
 MessagePriority::RealTime => super::QueuePriority::High,
 }
 }

 /// GetinsideData
 pub fn data(&self) -> &[u8] {
 &self.body.inline_data
 }

 /// CheckifasrestorehybridMessage
 pub fn is_complex(&self) -> bool {
 self.header.is_complex()
 }

 /// Determine dispatch strategy based on total message size
 pub fn dispatch_strategy(&self) -> MessageDispatch {
 message_dispatch_strategy(self.size())
 }
}

/// MessageBuilddevice
pub struct MessageBuilder {
 message: MachMessage,
}

impl MessageBuilder {
 /// Create newBuilddevice
 pub fn new(data: &[u8]) -> Self {
 Self {
 message: MachMessage::new_small(data),
 }
 }

 /// SettargetPort
 pub fn to(mut self, port: PortName) -> Self {
 self.message = self.message.to(port);
 self
 }

 /// SetroundrestorePort
 pub fn reply_to(mut self, port: PortName) -> Self {
 self.message = self.message.reply_to(port);
 self
 }

 /// SetMessage ID
 pub fn id(mut self, id: u32) -> Self {
 self.message = self.message.with_id(id);
 self
 }

 /// SetPriority
 pub fn priority(mut self, priority: MessagePriority) -> Self {
 self.message = self.message.with_priority(priority);
 self
 }

 /// addPortDescriptor
 pub fn add_port(mut self, name: PortName, disposition: RightDisposition) -> Self {
 self.message.add_port_descriptor(PortDescriptor {
 name,
 disposition,
 copy: PortCopy::PhysicalCopy,
 });
 self
 }

 /// BuildMessage
 pub fn build(self) -> MachMessage {
 self.message
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_message_create() {
 let msg = MachMessage::new_small(b"hello");
 assert_eq!(msg.data(), b"hello");
 assert!(!msg.is_large());
 }

 #[test]
 fn test_message_builder() {
 let msg = MessageBuilder::new(b"test")
 .to(1)
 .reply_to(2)
 .id(100)
 .priority(MessagePriority::High)
 .build();
 
 assert_eq!(msg.header.remote_port, 1);
 assert_eq!(msg.header.local_port, 2);
 assert_eq!(msg.header.id, 100);
 }

 #[test]
 fn test_message_priority() {
 let msg = MachMessage::new_small(b"test")
 .with_priority(MessagePriority::High);
 assert_eq!(msg.header.priority(), MessagePriority::High);
 }
}
