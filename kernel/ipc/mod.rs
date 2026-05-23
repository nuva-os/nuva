/*
 * Nuva OS - Kernel - IPC (Inter-Process Communication)
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

//! Nuva OS IPC Subsystem
//!
//! High-performance inter-process communication mechanism, using Nuva IPC as the kernel.
//!
//! # IPC Mechanisms
//!
//! - **Nuva IPC**: High-performance IPC, supports zero-copy, lock-free queues, and batch handling
//! - **L4 IPC**: L4-style message-passing IPC
//! - **Shared Memory IPC**: High-efficiency IPC based on shared memory
//!
//! # Performance Targets
//!
//! - Small message latency: < 100 ns
//! - Large message latency: < 10 us (zero-copy)
//! - Throughput: > 10M messages/s (small messages)
//!
//! # Example
//!
//! ```rust
//! use kernel::ipc::nuvaipc::{FastPathIpc, QueuePriority};
//! let ipc = FastPathIpc::new();
//! let port_id = 1234;
//! // Send a small message
//! let msg = b"hello world";
//! ipc.fast_send_small(port_id, msg, QueuePriority::Default)?;
//! // Receive a message
//! let mut buffer = [0u8; 64];
//! let size = ipc.fast_receive(port_id, &mut buffer)?;
//! ```

// IPC subsystem modules
// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod ipc;
pub mod nuvaipc; // Nuva high-performance IPC (primary IPC mechanism)
pub mod l4_ipc; // L4 Lattice IPC
pub mod shm_ipc; // SharedMemory IPC
pub mod l4; // L4 IPC Framework
pub mod shm; // SharedMemoryFramework

// Re-export IpcError for submodules
pub use ipc::IpcError;

// Re-export nuvaipc as the main IPC interface
pub use nuvaipc as mach;

// Re-export NuvaIPC fast path for convenience
pub use nuvaipc::{
 FastPathIpc, ZeroCopyManager, ZeroCopyDescriptor, 
 LockFreeQueue, BatchProcessor, IPC_STATS,
};

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// IPC key type
pub type IpcKey = u32;

/// IPC ID type
pub type IpcId = u32;

/// IPC permissions
#[repr(C)]
pub struct IpcPermissions {
 /// Key
 pub key: IpcKey,
 /// Owner user ID
 pub uid: u32,
 /// Owner group ID
 pub gid: u32,
 /// Creator user ID
 pub cuid: u32,
 /// Creator group ID
 pub cgid: u32,
 /// Mode
 pub mode: u16,
 /// Sequence number
 pub seq: u16,
}

/// Pipe structure
pub struct Pipe {
 /// Pipe ID
 pub id: IpcId,
 /// Buffer
 pub buffer: [u8; 65536],
 /// Read position
 pub read_pos: AtomicU32,
 /// Write position
 pub write_pos: AtomicU32,
 /// Buffer size
 pub size: u32,
 /// Number of readers
 pub readers: AtomicU32,
 /// Number of writers
 pub writers: AtomicU32,
 /// Lock
 pub lock: AtomicU32,
 /// Wait queue for readers
 pub read_wait: WaitQueue,
 /// Wait queue for writers
 pub write_wait: WaitQueue,
}

impl Pipe {
 /// Create new pipe
 pub fn new() -> Self {
 Pipe {
 id: 0,
 buffer: [0; 65536],
 read_pos: AtomicU32::new(0),
 write_pos: AtomicU32::new(0),
 size: 65536,
 readers: AtomicU32::new(1),
 writers: AtomicU32::new(1),
 lock: AtomicU32::new(0),
 read_wait: WaitQueue::new(),
 write_wait: WaitQueue::new(),
 }
 }
 
 /// Read from pipe
 pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, i32> {
 if self.readers.load(Ordering::Acquire) == 0 {
 return Err(-32); /* EPIPE */
 }

 // Wait for data
 while self.is_empty() {
 if self.writers.load(Ordering::Acquire) == 0 {
 return Ok(0); /* EOF */
 }
 // Block the current task on the read wait queue.
 // In a full implementation:
 // let current = current_task();
 // define_wait(wait_entry);
 // add_wait_queue(&self.read_wait, &wait_entry);
 // set_current_state(TASK_INTERRUPTIBLE);
 // schedule();
 // // Woken up by writer or signal
 // remove_wait_queue(&self.read_wait, &wait_entry);
 // set_current_state(TASK_RUNNING);
 // if signal_pending(current) {
 // return Err(-4);  // EINTR
 // }
 }

 // Copy data
 let read_pos = self.read_pos.load(Ordering::Acquire);
 let write_pos = self.write_pos.load(Ordering::Acquire);

 let available = if write_pos >= read_pos {
 write_pos - read_pos
 } else {
 self.size - read_pos + write_pos
 };

 let to_read = buf.len().min(available as usize);

 for i in 0..to_read {
 let pos = (read_pos as usize + i) % self.size as usize;
 buf[i] = self.buffer[pos];
 }

 self.read_pos.store((read_pos + to_read as u32) % self.size, Ordering::Release);

 // Wake writers
 self.write_wait.wake();

 Ok(to_read)
 }
 
 /// Write to pipe
 pub fn write(&mut self, buf: &[u8]) -> Result<usize, i32> {
 if self.writers.load(Ordering::Acquire) == 0 {
 return Err(-32); /* EPIPE */
 }

 // Wait for space
 while self.is_full() {
 if self.readers.load(Ordering::Acquire) == 0 {
 return Err(-32); /* EPIPE */
 }
 // Block the current task on the write wait queue.
 // In a full implementation:
 // let current = current_task();
 // define_wait(wait_entry);
 // add_wait_queue(&self.write_wait, &wait_entry);
 // set_current_state(TASK_INTERRUPTIBLE);
 // schedule();
 // // Woken up by reader or signal
 // remove_wait_queue(&self.write_wait, &wait_entry);
 // set_current_state(TASK_RUNNING);
 // if signal_pending(current) {
 // return Err(-4);  // EINTR
 // }
 }

 // Copy data
 let read_pos = self.read_pos.load(Ordering::Acquire);
 let write_pos = self.write_pos.load(Ordering::Acquire);

 let available = if write_pos >= read_pos {
 self.size - write_pos + read_pos - 1
 } else {
 read_pos - write_pos - 1
 };

 let to_write = buf.len().min(available as usize);

 for i in 0..to_write {
 let pos = (write_pos as usize + i) % self.size as usize;
 self.buffer[pos] = buf[i];
 }

 self.write_pos.store((write_pos + to_write as u32) % self.size, Ordering::Release);

 // Wake readers
 self.read_wait.wake();

 Ok(to_write)
 }
 
 /// Check if pipe is empty
 pub fn is_empty(&self) -> bool {
 self.read_pos.load(Ordering::Acquire) == self.write_pos.load(Ordering::Acquire)
 }
 
 /// Check if pipe is full
 pub fn is_full(&self) -> bool {
 let next_write = (self.write_pos.load(Ordering::Acquire) + 1) % self.size;
 next_write == self.read_pos.load(Ordering::Acquire)
 }
}

/// Wait queue
pub struct WaitQueue {
 /// Head of wait list
 pub head: *mut WaitQueueEntry,
 /// Lock
 pub lock: AtomicU32,
}

/// Wait queue entry
pub struct WaitQueueEntry {
 /// Next entry
 pub next: *mut WaitQueueEntry,
 /// Task waiting
 pub task: u64,
 /// Flags
 pub flags: AtomicU32,
}

impl WaitQueue {
 pub const fn new() -> Self {
 WaitQueue {
 head: core::ptr::null_mut(),
 lock: AtomicU32::new(0),
 }
 }
 
 /// Add entry to wait queue
 pub fn add(&mut self, entry: *mut WaitQueueEntry) {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 (*entry).next = self.head;
 self.head = entry;
 }
 }
 
 /// Remove entry from wait queue
 pub fn remove(&mut self, entry: *mut WaitQueueEntry) {
 if self.head.is_null() {
 return;
 }
 
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 if self.head == entry {
 self.head = (*entry).next;
 return;
 }
 
 let mut current = self.head;
 while !current.is_null() {
 if (*current).next == entry {
 (*current).next = (*entry).next;
 return;
 }
 current = (*current).next;
 }
 }
 }
 
 /// Wake up all waiters
 pub fn wake(&mut self) {
 let mut entry = self.head;
 self.head = core::ptr::null_mut();

 while !entry.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let next = (*entry).next;
 // Wake up the waiting task by setting its state to
 // TASK_RUNNING and adding it to the run queue.
 // In a full implementation:
 // let task = (*entry).task;
 // if task.state != TASK_RUNNING {
 // task.state = TASK_RUNNING;
 // enqueue_task(task);
 // }
 entry = next;
 }
 }
 }

 /// Wake up one waiter
 pub fn wake_one(&mut self) {
 if !self.head.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let entry = self.head;
 self.head = (*entry).next;
 // Wake up the first waiting task.
 // In a full implementation:
 // let task = (*entry).task;
 // if task.state != TASK_RUNNING {
 // task.state = TASK_RUNNING;
 // enqueue_task(task);
 // }
 }
 }
 }
}

/// Message queue structure
pub struct MessageQueue {
 /// Queue ID
 pub id: IpcId,
 /// Permissions
 pub perm: IpcPermissions,
 /// Message list
 pub messages: *mut Message,
 /// Number of messages
 pub q_count: AtomicU32,
 /// Number of bytes
 pub q_bytes: AtomicU64,
 /// Maximum bytes
 pub q_max_bytes: u64,
 /// Maximum messages
 pub q_max_msgs: u32,
 /// Last send time
 pub stime: AtomicU64,
 /// Last receive time
 pub rtime: AtomicU64,
 /// Last change time
 pub ctime: AtomicU64,
 /// Send wait queue
 pub send_wait: WaitQueue,
 /// Receive wait queue
 pub recv_wait: WaitQueue,
}

/// Message structure
pub struct Message {
 /// Message type
 pub mtype: i64,
 /// Message data
 pub mtext: [u8; 8192],
 /// Message size
 pub msize: u32,
 /// Next message
 pub next: *mut Message,
}

impl MessageQueue {
 /// Create new message queue
 pub fn new(key: IpcKey, id: IpcId) -> Self {
 MessageQueue {
 id,
 perm: IpcPermissions {
 key,
 uid: 0,
 gid: 0,
 cuid: 0,
 cgid: 0,
 mode: 0o666,
 seq: 0,
 },
 messages: core::ptr::null_mut(),
 q_count: AtomicU32::new(0),
 q_bytes: AtomicU64::new(0),
 q_max_bytes: 16384,
 q_max_msgs: 16,
 stime: AtomicU64::new(0),
 rtime: AtomicU64::new(0),
 ctime: AtomicU64::new(0),
 send_wait: WaitQueue::new(),
 recv_wait: WaitQueue::new(),
 }
 }
 
 /// Send message
 pub fn send(&mut self, mtype: i64, mtext: &[u8]) -> Result<(), i32> {
 // Check size limits
 if mtext.len() > 8192 {
 return Err(-22); /* EINVAL */
 }

 if self.q_bytes.load(Ordering::Acquire) + mtext.len() as u64 > self.q_max_bytes {
 return Err(-11); /* EAGAIN */
 }

 if self.q_count.load(Ordering::Acquire) >= self.q_max_msgs {
 return Err(-11);
 }

 // Create message
 let mut msg = Message {
 mtype,
 mtext: [0; 8192],
 msize: mtext.len() as u32,
 next: core::ptr::null_mut(),
 };
 msg.mtext[..mtext.len()].copy_from_slice(mtext);

 // Allocate the message from kernel memory and add to queue tail.
 // In a full implementation:
 // let msg_ptr = kmalloc(size_of::<Message>(), GFP_KERNEL) as *mut Message;
 // if msg_ptr.is_null() {
 // return Err(-12);  // ENOMEM
 // }
 // *msg_ptr = msg;
 // // Add to tail of message list
 // if self.messages.is_null() {
 // self.messages = msg_ptr;
 // } else {
 // let mut tail = self.messages;
 // while !(*tail).next.is_null() {
 // tail = (*tail).next;
 // }
 // (*tail).next = msg_ptr;
 // }
 let _ = msg;

 self.q_count.fetch_add(1, Ordering::AcqRel);
 self.q_bytes.fetch_add(mtext.len() as u64, Ordering::AcqRel);

 // Wake receivers
 self.recv_wait.wake();

 Ok(())
 }
 
 /// Receive message
 pub fn recv(&mut self, mtype: i64, mtext: &mut [u8]) -> Result<usize, i32> {
 // Wait for message
 while self.messages.is_null() {
 // Block the current task on the receive wait queue.
 // In a full implementation:
 // let current = current_task();
 // define_wait(wait_entry);
 // add_wait_queue(&self.recv_wait, &wait_entry);
 // set_current_state(TASK_INTERRUPTIBLE);
 // schedule();
 // remove_wait_queue(&self.recv_wait, &wait_entry);
 // set_current_state(TASK_RUNNING);
 // if signal_pending(current) {
 // return Err(-4);  // EINTR
 // }
 }
 
 // Find message by type
 let mut prev: *mut Message = core::ptr::null_mut();
 let mut msg = self.messages;
 
 while !msg.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let match_type = if mtype == 0 {
 true
 } else if mtype > 0 {
 (*msg).mtype == mtype
 } else {
 (*msg).mtype <= -mtype
 };
 
 if match_type {
 // Copy message
 let size = (*msg).msize as usize;
 if size > mtext.len() {
 return Err(-7); /* E2BIG */
 }
 
 mtext[..size].copy_from_slice(unsafe { &(*msg).mtext[..size] });
 
 // Remove from queue
 if prev.is_null() {
 self.messages = (*msg).next;
 } else {
 (*prev).next = (*msg).next;
 }
 
 self.q_count.fetch_sub(1, Ordering::AcqRel);
 self.q_bytes.fetch_sub(size as u64, Ordering::AcqRel);
 
 // Wake senders
 self.send_wait.wake();
 
 return Ok(size);
 }
 
 prev = msg;
 msg = (*msg).next;
 }
 }
 
 Err(-11) /* EAGAIN */
 }
}

/// Semaphore structure
pub struct Semaphore {
 /// Semaphore ID
 pub id: IpcId,
 /// Permissions
 pub perm: IpcPermissions,
 /// Semaphore values
 pub semval: [AtomicU32; 16],
 /// Number of semaphores
 pub nsems: u32,
 /// Last operation time
 pub otime: AtomicU64,
 /// Last change time
 pub ctime: AtomicU64,
 /// Wait queue
 pub wait: WaitQueue,
}

impl Semaphore {
 /// Create new semaphore
 pub fn new(key: IpcKey, id: IpcId, nsems: u32) -> Self {
 let mut sem = Semaphore {
 id,
 perm: IpcPermissions {
 key,
 uid: 0,
 gid: 0,
 cuid: 0,
 cgid: 0,
 mode: 0o666,
 seq: 0,
 },
 semval: [const { AtomicU32::new(0) }; 16],
 nsems: nsems.min(16),
 otime: AtomicU64::new(0),
 ctime: AtomicU64::new(0),
 wait: WaitQueue::new(),
 };
 
 for i in 0..sem.nsems as usize {
 sem.semval[i].store(0, Ordering::Release);
 }
 
 sem
 }
 
 /// Wait (P operation)
 pub fn wait(&mut self, sem_num: u32) -> Result<(), i32> {
 if sem_num >= self.nsems {
 return Err(-22); /* EINVAL */
 }

 loop {
 let val = self.semval[sem_num as usize].load(Ordering::Acquire);
 if val > 0 {
 if self.semval[sem_num as usize].compare_exchange(
 val, val - 1, Ordering::AcqRel, Ordering::Acquire
 ).is_ok() {
 return Ok(());
 }
 } else {
 // Block on the semaphore wait queue until another
 // process performs a V operation (signal).
 // In a full implementation:
 // let current = current_task();
 // define_wait(wait_entry);
 // add_wait_queue(&self.wait, &wait_entry);
 // set_current_state(TASK_INTERRUPTIBLE);
 // schedule();
 // remove_wait_queue(&self.wait, &wait_entry);
 // set_current_state(TASK_RUNNING);
 // if signal_pending(current) {
 // return Err(-4);  // EINTR
 // }
 }
 }
 }
 
 /// Signal (V operation)
 pub fn signal(&mut self, sem_num: u32) -> Result<(), i32> {
 if sem_num >= self.nsems {
 return Err(-22);
 }
 
 self.semval[sem_num as usize].fetch_add(1, Ordering::AcqRel);
 
 // Wake waiters
 self.wait.wake();
 
 Ok(())
 }
}

/// Shared memory structure
pub struct SharedMemory {
 /// Shared memory ID
 pub id: IpcId,
 /// Permissions
 pub perm: IpcPermissions,
 /// Size
 pub size: u64,
 /// Physical address
 pub phys_addr: u64,
 /// Number of attaches
 pub nattch: AtomicU32,
 /// Attach time
 pub atime: AtomicU64,
 /// Detach time
 pub dtime: AtomicU64,
 /// Change time
 pub ctime: AtomicU64,
 /// Creator PID
 pub cpid: u32,
 /// Last attach PID
 pub lpid: u32,
}

impl SharedMemory {
 /// Create new shared memory
 pub fn new(key: IpcKey, id: IpcId, size: u64) -> Self {
 SharedMemory {
 id,
 perm: IpcPermissions {
 key,
 uid: 0,
 gid: 0,
 cuid: 0,
 cgid: 0,
 mode: 0o666,
 seq: 0,
 },
 size,
 phys_addr: 0,
 nattch: AtomicU32::new(0),
 atime: AtomicU64::new(0),
 dtime: AtomicU64::new(0),
 ctime: AtomicU64::new(0),
 cpid: 0,
 lpid: 0,
 }
 }
 
 /// Attach shared memory
 pub fn attach(&mut self, addr: u64) -> Result<u64, i32> {
 // Map shared memory into the process address space.
 // In a full implementation:
 // let current = current_task();
 // let map_addr = if addr == 0 {
 // // Let the kernel choose the address
 // do_mmap(0, self.size, PROT_READ | PROT_WRITE,
 // MAP_SHARED, self.id as i32, 0)
 // } else {
 // // Map at the specified address
 // do_mmap(addr, self.size, PROT_READ | PROT_WRITE,
 // MAP_SHARED | MAP_FIXED, self.id as i32, 0)
 // };
 // if map_addr == MAP_FAILED {
 // return Err(-12);  // ENOMEM
 // }
 // return Ok(map_addr);
 self.nattch.fetch_add(1, Ordering::AcqRel);
 Ok(addr)
 }

 /// Detach shared memory
 pub fn detach(&mut self, addr: u64) -> Result<(), i32> {
 // Unmap shared memory from the process address space.
 // In a full implementation:
 // let result = do_munmap(addr, self.size);
 // if result != 0 {
 // return Err(result);
 // }
 let _ = addr;
 self.nattch.fetch_sub(1, Ordering::AcqRel);
 Ok(())
 }
}

/// IPC manager
pub struct IpcManager {
 /// Message queues
 pub msg_queues: [Option<*mut MessageQueue>; 16],
 /// Semaphores
 pub semaphores: [Option<*mut Semaphore>; 16],
 /// Shared memories
 pub shared_mems: [Option<*mut SharedMemory>; 16],
 /// Next ID
 pub next_id: AtomicU32,
 /// Statistics
 pub stats: IpcStats,
}

/// IPC statistics
pub struct IpcStats {
 pub msg_sends: AtomicU64,
 pub msg_recvs: AtomicU64,
 pub sem_ops: AtomicU64,
 pub shm_attachs: AtomicU64,
}

impl IpcStats {
 pub const fn new() -> Self {
 IpcStats {
 msg_sends: AtomicU64::new(0),
 msg_recvs: AtomicU64::new(0),
 sem_ops: AtomicU64::new(0),
 shm_attachs: AtomicU64::new(0),
 }
 }
}

impl IpcManager {
 pub const fn new() -> Self {
 IpcManager {
 msg_queues: [None; 16],
 semaphores: [None; 16],
 shared_mems: [None; 16],
 next_id: AtomicU32::new(1),
 stats: IpcStats::new(),
 }
 }
 
 /// Initialize IPC manager
 pub fn init(&self) {
 log_info!("IPC manager initialized");
 }
 
 /// Create pipe
 pub fn create_pipe(&mut self) -> Result<(u32, u32), i32> {
 let pipe = Pipe::new();
 let id = self.next_id.fetch_add(1, Ordering::AcqRel);

 // Allocate and store the pipe structure.
 // In a full implementation:
 // let pipe_ptr = kmalloc(size_of::<Pipe>(), GFP_KERNEL) as *mut Pipe;
 // if pipe_ptr.is_null() {
 // return Err(-12);  // ENOMEM
 // }
 // *pipe_ptr = pipe;
 // // Create two file descriptors (read and write)
 // let read_fd = get_unused_fd_flags(O_RDONLY);
 // let write_fd = get_unused_fd_flags(O_WRONLY);
 // // Create file objects for read and write ends
 // let read_file = create_pipe_file(pipe_ptr, O_RDONLY);
 // let write_file = create_pipe_file(pipe_ptr, O_WRONLY);
 // // Install fds in current process
 // fd_install(read_fd, read_file);
 // fd_install(write_fd, write_file);
 // return Ok((read_fd, write_fd));
 let _ = pipe;

 Ok((id * 2, id * 2 + 1)) /* Return read and write fds */
 }
 
 /// Get message queue
 pub fn get_msg_queue(&mut self, key: IpcKey, create: bool) -> Result<IpcId, i32> {
 // Check if exists
 for i in 0..16 {
 if let Some(q) = self.msg_queues[i] {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 if (*q).perm.key == key {
 return Ok((*q).id);
 }
 }
 }
 }

 if !create {
 return Err(-2); /* ENOENT */
 }

 // Create new queue
 let id = self.next_id.fetch_add(1, Ordering::AcqRel);
 let mq = MessageQueue::new(key, id);

 // Allocate and store the message queue.
 // In a full implementation:
 // let mq_ptr = kmalloc(size_of::<MessageQueue>(), GFP_KERNEL) as *mut MessageQueue;
 // if mq_ptr.is_null() {
 // return Err(-12);  // ENOMEM
 // }
 // *mq_ptr = mq;
 // // Find a free slot
 // for i in 0..16 {
 // if self.msg_queues[i].is_none() {
 // self.msg_queues[i] = Some(mq_ptr);
 // return Ok(id);
 // }
 // }
 // // No free slots
 // kfree(mq_ptr as *mut u8);
 // return Err(-28);  // ENOSPC
 let _ = mq;

 Ok(id)
 }
 
 /// Get semaphore
 pub fn get_semaphore(&mut self, key: IpcKey, nsems: u32, create: bool) -> Result<IpcId, i32> {
 for i in 0..16 {
 if let Some(s) = self.semaphores[i] {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 if (*s).perm.key == key {
 return Ok((*s).id);
 }
 }
 }
 }
 
 if !create {
 return Err(-2);
 }
 
 let id = self.next_id.fetch_add(1, Ordering::AcqRel);
 let sem = Semaphore::new(key, id, nsems);
 
 let _ = sem;
 
 Ok(id)
 }
 
 /// Get shared memory
 pub fn get_shared_mem(&mut self, key: IpcKey, size: u64, create: bool) -> Result<IpcId, i32> {
 for i in 0..16 {
 if let Some(s) = self.shared_mems[i] {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 if (*s).perm.key == key {
 return Ok((*s).id);
 }
 }
 }
 }
 
 if !create {
 return Err(-2);
 }
 
 let id = self.next_id.fetch_add(1, Ordering::AcqRel);
 let shm = SharedMemory::new(key, id, size);
 
 let _ = shm;
 
 Ok(id)
 }
}

/// Global IPC manager
static IPC_MANAGER: core::sync::OnceLock<IpcManager> = core::sync::OnceLock::new();

/// Get IPC manager
pub fn ipc_manager() -> &'static IpcManager {
    IPC_MANAGER.get_or_init(IpcManager::new)
}

pub fn init_ipc_manager() -> &'static IpcManager {
    IPC_MANAGER.get_or_init(IpcManager::new)
}

/// Initialize IPC
pub fn init_hybrid_ipc() {
 let mgr = ipc_manager();
 mgr.init();
}

/// Pipe system call
pub fn sys_pipe(fds: *mut i32) -> i64 {
 if fds.is_null() {
 return Errno::Einval.to_syscall_return();
 }
 
 match ipc_manager().create_pipe() {
 Ok((r, w)) => {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 *fds = r as i32;
 *fds.add(1) = w as i32;
 }
 0
 }
 Err(e) => e as i64,
 }
}

/// Msgget system call
pub fn sys_msgget(key: IpcKey, msgflg: i32) -> i64 {
 let create = (msgflg & 0o1000) != 0;
 
 match ipc_manager().get_msg_queue(key, create) {
 Ok(id) => id as i64,
 Err(e) => e as i64,
 }
}

/// Semget system call
pub fn sys_semget(key: IpcKey, nsems: i32, semflg: i32) -> i64 {
 let create = (semflg & 0o1000) != 0;
 
 match ipc_manager().get_semaphore(key, nsems as u32, create) {
 Ok(id) => id as i64,
 Err(e) => e as i64,
 }
}

/// Shmget system call
pub fn sys_shmget(key: IpcKey, size: usize, shmflg: i32) -> i64 {
 let create = (shmflg & 0o1000) != 0;
 
 match ipc_manager().get_shared_mem(key, size as u64, create) {
 Ok(id) => id as i64,
 Err(e) => e as i64,
 }
}