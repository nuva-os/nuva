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


use alloc::boxed::Box;
use core::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use core::mem::MaybeUninit;
use core::ptr;
use crate::{pr_debug, pr_info};

/// RingformBufferSize (mustmustis 2 Power)
pub const RING_SIZE: usize = 1024;

/// MessageMaxSize
pub const MAX_MSG_SIZE: usize = 256;

/// MessageType
#[derive(Debug, Clone, Copy)]
pub struct Message {
 /// MessageType
 pub msg_type: u32,
 /// MessageLength
 pub len: u32,
 /// MessageData
 pub data: [u8; MAX_MSG_SIZE],
}

impl Message {
 pub fn new(msg_type: u32, data: &[u8]) -> Self {
 let mut msg = Message {
 msg_type,
 len: data.len() as u32,
 data: [0; MAX_MSG_SIZE],
 };
 let copy_len = data.len().min(MAX_MSG_SIZE);
 msg.data[..copy_len].copy_from_slice(&data[..copy_len]);
 msg
 }
}

/// infiniteLockformcreateproducterformer (SPSC) RingformBuffer
pub struct SpscRingBuffer {
 /// Buffer
 buffer: [MaybeUninit<Message>; RING_SIZE],
 /// Headpointer (createproducter)
 head: AtomicUsize,
 /// Tailpointer (er)
 tail: AtomicUsize,
}

impl SpscRingBuffer {
 /// Create new SPSC RingformBuffer
 pub const fn new() -> Self {
 SpscRingBuffer {
 // SAFETY: unsafe block required for low-level memory or hardware access
 buffer: unsafe { MaybeUninit::uninit().assume_init() },
 head: AtomicUsize::new(0),
 tail: AtomicUsize::new(0),
 }
 }
 
 /// createproducter: WriteMessage
 #[inline(always)]
 pub fn push(&self, msg: Message) -> Result<(), Message> {
 let head = self.head.load(Ordering::Relaxed);
 let tail = self.tail.load(Ordering::Acquire);
 
 // Checkifsatisfy 
 if head.wrapping_sub(tail) >= RING_SIZE {
 return Err(msg);
 }
 
 // WriteMessage
 let slot = head % RING_SIZE;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 ptr::write(self.buffer[slot].as_ptr() as *mut Message, msg);
 }
 
 // UpdateHeadpointer
 self.head.store(head.wrapping_add(1), Ordering::Release);
 
 Ok(())
 }
 
 /// er: ReadMessage
 #[inline(always)]
 pub fn pop(&self) -> Option<Message> {
 let tail = self.tail.load(Ordering::Relaxed);
 let head = self.head.load(Ordering::Acquire);
 
 // Checkifempty 
 if tail >= head {
 return None;
 }
 
 // ReadMessage
 let slot = tail % RING_SIZE;
 // SAFETY: unsafe block required for low-level memory or hardware access
 let msg = unsafe {
 ptr::read(self.buffer[slot].as_ptr() as *const Message)
 };
 
 // UpdateTailpointer
 self.tail.store(tail.wrapping_add(1), Ordering::Release);
 
 Some(msg)
 }
 
 /// Getcanuseemptybetween
 #[inline(always)]
 pub fn available(&self) -> usize {
 let head = self.head.load(Ordering::Relaxed);
 let tail = self.tail.load(Ordering::Relaxed);
 RING_SIZE - head.wrapping_sub(tail)
 }
 
 /// Getalreadyuseemptybetween
 #[inline(always)]
 pub fn len(&self) -> usize {
 let head = self.head.load(Ordering::Relaxed);
 let tail = self.tail.load(Ordering::Relaxed);
 head.wrapping_sub(tail)
 }
 
 /// ifasempty
 #[inline(always)]
 pub fn is_empty(&self) -> bool {
 self.len() == 0
 }
 
 /// ifalreadysatisfy
 #[inline(always)]
 pub fn is_full(&self) -> bool {
 self.len() >= RING_SIZE
 }
}

/// infiniteLockmanycreateproductermanyer (MPMC) RingformBuffer
pub struct MpmcRingBuffer {
 /// Buffer
 buffer: [AtomicPtr<Message>; RING_SIZE],
 /// Headpointer
 head: AtomicUsize,
 /// Tailpointer
 tail: AtomicUsize,
}

impl MpmcRingBuffer {
 /// Create new MPMC RingformBuffer
 pub const fn new() -> Self {
 MpmcRingBuffer {
 buffer: [const { AtomicPtr::new(ptr::null_mut()) }; RING_SIZE],
 head: AtomicUsize::new(0),
 tail: AtomicUsize::new(0),
 }
 }
 
 /// createproducter: WriteMessage
 #[inline(always)]
 pub fn push(&self, msg: Message) -> Result<(), Message> {
 loop {
 let head = self.head.load(Ordering::Acquire);
 let tail = self.tail.load(Ordering::Acquire);
 
 // Checkifsatisfy 
 if head.wrapping_sub(tail) >= RING_SIZE {
 return Err(msg);
 }
 
 // tryreserveslotBit
 let slot = head % RING_SIZE;
 
 // CAS UpdateHeadpointer
 match self.head.compare_exchange_weak(
 head,
 head.wrapping_add(1),
 Ordering::Release,
 Ordering::Relaxed,
 ) {
 Ok(_) => {
 // WriteMessage
 let msg_ptr = Box::into_raw(Box::new(msg));
 self.buffer[slot].store(msg_ptr, Ordering::Release);
 return Ok(());
 }
 Err(_) => {
 // CAS Failure,retry
 core::hint::spin_loop();
 }
 }
 }
 }
 
 /// er: ReadMessage
 #[inline(always)]
 pub fn pop(&self) -> Option<Message> {
 loop {
 let tail = self.tail.load(Ordering::Acquire);
 let head = self.head.load(Ordering::Acquire);
 
 // Checkifempty 
 if tail >= head {
 return None;
 }
 
 // tryreserveslotBit
 let slot = tail % RING_SIZE;
 
 // CAS UpdateTailpointer
 match self.tail.compare_exchange_weak(
 tail,
 tail.wrapping_add(1),
 Ordering::Release,
 Ordering::Relaxed,
 ) {
 Ok(_) => {
 // ReadMessage
 loop {
 let msg_ptr = self.buffer[slot].load(Ordering::Acquire);
 if !msg_ptr.is_null() {
 // ClearslotBit
 self.buffer[slot].store(ptr::null_mut(), Ordering::Release);
 
 // returnMessage
 // SAFETY: unsafe block required for low-level memory or hardware access
 let msg = unsafe { *Box::from_raw(msg_ptr) };
 return Some(msg);
 }
 // WaitcreateproducterWrite
 core::hint::spin_loop();
 }
 }
 Err(_) => {
 // CAS Failure,retry
 core::hint::spin_loop();
 }
 }
 }
 }
}

/// SharedMemory IPC channel
pub struct ShmIpcChannel {
 /// SendBuffer
 send_buffer: SpscRingBuffer,
 /// ReceiveBuffer
 recv_buffer: SpscRingBuffer,
 /// Channel ID
 pub channel_id: u32,
 /// logendProcess ID
 pub peer_pid: u32,
}

impl ShmIpcChannel {
 /// Create newchannel
 pub fn new(channel_id: u32, peer_pid: u32) -> Self {
 ShmIpcChannel {
 send_buffer: SpscRingBuffer::new(),
 recv_buffer: SpscRingBuffer::new(),
 channel_id,
 peer_pid,
 }
 }
 
 /// SendMessage
 #[inline(always)]
 pub fn send(&self, msg: Message) -> Result<(), Message> {
 self.send_buffer.push(msg)
 }
 
 /// ReceiveMessage
 #[inline(always)]
 pub fn recv(&self) -> Option<Message> {
 self.recv_buffer.pop()
 }
 
 /// SendData
 #[inline(always)]
 pub fn send_data(&self, msg_type: u32, data: &[u8]) -> Result<(), Message> {
 let msg = Message::new(msg_type, data);
 self.send(msg)
 }
 
 /// GetSendBuffercanuseemptybetween
 #[inline(always)]
 pub fn send_available(&self) -> usize {
 self.send_buffer.available()
 }
 
 /// GetReceiveBufferMessagenumber
 #[inline(always)]
 pub fn recv_len(&self) -> usize {
 self.recv_buffer.len()
 }
}

/// SharedMemory IPC Manager
pub struct ShmIpcManager {
 /// Channel count
 channel_count: AtomicUsize,
 /// NextChannel ID
 next_channel_id: AtomicUsize,
}

impl ShmIpcManager {
 pub const fn new() -> Self {
 ShmIpcManager {
 channel_count: AtomicUsize::new(0),
 next_channel_id: AtomicUsize::new(1),
 }
 }
 
 /// Initialize
 pub fn init(&self) {
 log_info!("Shared memory IPC initialized");
 log_info!(" Ring size: {}", RING_SIZE);
 log_info!(" Max message size: {} bytes", MAX_MSG_SIZE);
 }
 
 /// Create channel
 pub fn create_channel(&self, peer_pid: u32) -> u32 {
 let channel_id = self.next_channel_id.fetch_add(1, Ordering::AcqRel) as u32;
 self.channel_count.fetch_add(1, Ordering::AcqRel);
 
 log_debug!("Created IPC channel: {} -> {}", channel_id, peer_pid);
 
 channel_id
 }
 
 /// GetChannel count
 pub fn get_channel_count(&self) -> usize {
 self.channel_count.load(Ordering::Acquire)
 }
}

/// GlobalSharedMemory IPC Manager
static SHM_IPC_MANAGER: core::sync::OnceLock<ShmIpcManager> = core::sync::OnceLock::new();

pub fn shm_ipc_manager() -> &'static ShmIpcManager {
    SHM_IPC_MANAGER.get_or_init(ShmIpcManager::new)
}

pub fn init_shm_ipc_manager() -> &'static ShmIpcManager {
    SHM_IPC_MANAGER.get_or_init(ShmIpcManager::new)
}

pub fn init_shm_ipc() {
 let manager = shm_ipc_manager();
 manager.init();
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_constants() {
 assert_eq!(RING_SIZE, 1024);
 assert_eq!(MAX_MSG_SIZE, 256);
 }

 #[test]
 fn test_message_new() {
 let data = [1, 2, 3, 4, 5];
 let msg = Message::new(100, &data);

 assert_eq!(msg.msg_type, 100);
 assert_eq!(msg.len, 5);
 assert_eq!(msg.data[0], 1);
 assert_eq!(msg.data[4], 5);
 }

 #[test]
 fn test_message_truncation() {
 let data = [0u8; 300];
 let msg = Message::new(1, &data);

 assert_eq!(msg.len, MAX_MSG_SIZE as u32);
 }

 #[test]
 fn test_spsc_ring_buffer_new() {
 let rb = SpscRingBuffer::new();

 assert!(rb.is_empty());
 assert!(!rb.is_full());
 assert_eq!(rb.len(), 0);
 assert_eq!(rb.available(), RING_SIZE);
 }

 #[test]
 fn test_spsc_ring_buffer_push_pop() {
 let rb = SpscRingBuffer::new();

 let msg = Message::new(1, b"hello");
 let result = rb.push(msg);
 assert!(result.is_ok());

 assert!(!rb.is_empty());
 assert_eq!(rb.len(), 1);

 let popped = rb.pop();
 assert!(popped.is_some());

 let popped_msg = popped.unwrap();
 assert_eq!(popped_msg.msg_type, 1);
 assert_eq!(popped_msg.len, 5);
 }

 #[test]
 fn test_spsc_ring_buffer_multiple() {
 let rb = SpscRingBuffer::new();

 for i in 0..10 {
 let msg = Message::new(i, &[i as u8]);
 assert!(rb.push(msg).is_ok());
 }

 assert_eq!(rb.len(), 10);

 for i in 0..10 {
 let msg = rb.pop().unwrap();
 assert_eq!(msg.msg_type, i);
 }

 assert!(rb.is_empty());
 }

 #[test]
 fn test_spsc_ring_buffer_full() {
 let rb = SpscRingBuffer::new();

 // fillsatisfyBuffer
 for i in 0..RING_SIZE {
 let msg = Message::new(i as u32, &[]);
 assert!(rb.push(msg).is_ok());
 }

 assert!(rb.is_full());
 assert_eq!(rb.available(), 0);

 // againaitemshouldtheFailure
 let msg = Message::new(999, &[]);
 let result = rb.push(msg);
 assert!(result.is_err());
 }

 #[test]
 fn test_spsc_ring_buffer_empty_pop() {
 let rb = SpscRingBuffer::new();

 let result = rb.pop();
 assert!(result.is_none());
 }

 #[test]
 fn test_spsc_ring_buffer_fifo_order() {
 let rb = SpscRingBuffer::new();

 rb.push(Message::new(1, b"first")).unwrap();
 rb.push(Message::new(2, b"second")).unwrap();
 rb.push(Message::new(3, b"third")).unwrap();

 assert_eq!(rb.pop().unwrap().msg_type, 1);
 assert_eq!(rb.pop().unwrap().msg_type, 2);
 assert_eq!(rb.pop().unwrap().msg_type, 3);
 }

 #[test]
 fn test_mpmc_ring_buffer_new() {
 let rb = MpmcRingBuffer::new();

 assert_eq!(rb.head.load(Ordering::Relaxed), 0);
 assert_eq!(rb.tail.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_mpmc_ring_buffer_push_pop() {
 let rb = MpmcRingBuffer::new();

 let msg = Message::new(42, b"test");
 let result = rb.push(msg);
 assert!(result.is_ok());

 let popped = rb.pop();
 assert!(popped.is_some());

 let popped_msg = popped.unwrap();
 assert_eq!(popped_msg.msg_type, 42);
 }

 #[test]
 fn test_mpmc_ring_buffer_multiple() {
 let rb = MpmcRingBuffer::new();

 for i in 0..10 {
 let msg = Message::new(i, &[i as u8]);
 assert!(rb.push(msg).is_ok());
 }

 for i in 0..10 {
 let msg = rb.pop().unwrap();
 assert_eq!(msg.msg_type, i);
 }
 }

 #[test]
 fn test_mpmc_ring_buffer_full() {
 let rb = MpmcRingBuffer::new();

 // fillsatisfyBuffer
 for i in 0..RING_SIZE {
 let msg = Message::new(i as u32, &[]);
 assert!(rb.push(msg).is_ok());
 }

 // againaitemshouldtheFailure
 let msg = Message::new(999, &[]);
 let result = rb.push(msg);
 assert!(result.is_err());
 }

 #[test]
 fn test_mpmc_ring_buffer_empty_pop() {
 let rb = MpmcRingBuffer::new();

 let result = rb.pop();
 assert!(result.is_none());
 }

 #[test]
 fn test_shm_ipc_channel_new() {
 let channel = ShmIpcChannel::new(1, 100);

 assert_eq!(channel.channel_id, 1);
 assert_eq!(channel.peer_pid, 100);
 }

 #[test]
 fn test_shm_ipc_channel_send() {
 let channel = ShmIpcChannel::new(1, 100);

 let result = channel.send_data(1, b"hello");
 assert!(result.is_ok());
 }

 #[test]
 fn test_shm_ipc_channel_available() {
 let channel = ShmIpcChannel::new(1, 100);

 assert_eq!(channel.send_available(), RING_SIZE);

 channel.send_data(1, b"test").unwrap();
 assert_eq!(channel.send_available(), RING_SIZE - 1);
 }

 #[test]
 fn test_shm_ipc_channel_recv_len() {
 let channel = ShmIpcChannel::new(1, 100);

 assert_eq!(channel.recv_len(), 0);
 }

 #[test]
 fn test_shm_ipc_manager_new() {
 let manager = ShmIpcManager::new();

 assert_eq!(manager.get_channel_count(), 0);
 }

 #[test]
 fn test_shm_ipc_manager_create_channel() {
 let manager = ShmIpcManager::new();

 let id1 = manager.create_channel(100);
 assert_eq!(id1, 1);
 assert_eq!(manager.get_channel_count(), 1);

 let id2 = manager.create_channel(200);
 assert_eq!(id2, 2);
 assert_eq!(manager.get_channel_count(), 2);
 }

 #[test]
 fn test_message_data_copy() {
 let data = b"Hello, World!";
 let msg = Message::new(1, data);

 assert_eq!(msg.len, 13);
 assert_eq!(&msg.data[..13], data);
 }

 #[test]
 fn test_spsc_ring_buffer_wrap_around() {
 let rb = SpscRingBuffer::new();

 // fillsatisfythenthenClearmanytime
 for _ in 0..3 {
 for i in 0..RING_SIZE {
 rb.push(Message::new(i as u32, &[])).unwrap();
 }
 for i in 0..RING_SIZE {
 let msg = rb.pop().unwrap();
 assert_eq!(msg.msg_type, i as u32);
 }
 assert!(rb.is_empty());
 }
 }
}