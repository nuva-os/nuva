/*
 * Nuva OS - Kernel - Ipc - Nvipc - Port
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
/*
 * Nuva OS - Kernel - IPC - Port
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Nuva IPC Port implementation.
 */

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

use super::{PortId, TaskId, IpcError};
use crate::kernel::types::{NvPortId, NvPortName, NuvaProcessId, NuvaCapabilityId};

/// Port name type
pub type PortName = u64;

/// Port state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    /// Active
    Active = 0,
    /// Dead
    Dead = 1,
    /// Inactive
    Inactive = 2,
}

/// Port rights flags
bitflags::bitflags! {
    /// Port rights
    pub struct PortRights: u32 {
        /// No rights
        const NONE = 0;
        /// Send right
        const SEND = 1 << 0;
        /// Receive right
        const RECEIVE = 1 << 1;
        /// Send-once right
        const SEND_ONCE = 1 << 2;
        /// Port set
        const PORT_SET = 1 << 3;
        /// Dead name
        const DEAD_NAME = 1 << 4;
    }
}

impl Clone for PortRights {
    fn clone(&self) -> Self { *self }
}
impl Copy for PortRights {}

/// Port queue (internal)
struct PortQueue {
    /// Messages
    messages: alloc::collections::VecDeque<super::MachMessage>,
    /// Capacity
    capacity: usize,
}

/// Nuva IPC Port
#[repr(C, align(64))]
pub struct MachPort {
    /// Port ID (nuva native NvPortId)
    pub id: PortId,
    /// Port state
    state: AtomicU32,
    /// Message queue
    queue: Mutex<PortQueue>,
    /// Waiter list
    waiters: Mutex<Vec<TaskId>>,
    /// Context
    context: AtomicU32,
    /// Reference count
    refs: AtomicU32,
    /// Rights
    rights: AtomicU32,
    /// Receiver
    receiver: Mutex<Option<TaskId>>,
    /// Associated capability token (nuva native, replaces permission bits)
    pub capability: Option<NuvaCapabilityId>,
    /// Owner process (nuva native NvProcessId)
    pub owner_process: NuvaProcessId,
    /// Maximum message size
    pub max_msg_size: usize,
}

impl MachPort {
    /// Create new port
    pub fn new(id: PortId) -> Self {
        Self {
            id,
            state: AtomicU32::new(PortState::Active as u32),
            queue: Mutex::new(PortQueue {
                messages: alloc::collections::VecDeque::new(),
                capacity: 1024,
            }),
            waiters: Mutex::new(Vec::new()),
            context: AtomicU32::new(0),
            refs: AtomicU32::new(1),
            rights: AtomicU32::new(0),
            receiver: Mutex::new(None),
            capability: None,
            owner_process: NuvaProcessId::new(0),
            max_msg_size: 4096,
        }
    }

    /// Create port with capacity
    pub fn with_capacity(id: PortId, capacity: usize) -> Self {
        let mut port = Self::new(id);
        port.queue.lock().capacity = capacity;
        port
    }

    /// Get port state
    pub fn state(&self) -> PortState {
        match self.state.load(Ordering::Acquire) {
            0 => PortState::Active,
            1 => PortState::Dead,
            _ => PortState::Inactive,
        }
    }

    /// Set port state
    pub fn set_state(&self, state: PortState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.state() == PortState::Active
    }

    /// Mark port as dead
    pub fn mark_dead(&self) {
        self.set_state(PortState::Dead);
    }

    /// Get rights
    pub fn rights(&self) -> PortRights {
        PortRights::from_bits_truncate(self.rights.load(Ordering::Acquire))
    }

    /// Set rights
    pub fn set_rights(&self, rights: PortRights) {
        self.rights.store(rights.bits(), Ordering::Release);
    }

    /// Set receiver
    pub fn set_receiver(&self, task_id: TaskId) {
        *self.receiver.lock() = Some(task_id);
    }

    /// Get receiver
    pub fn receiver(&self) -> Option<TaskId> {
        *self.receiver.lock()
    }

    /// Clear receiver
    pub fn clear_receiver(&self) {
        *self.receiver.lock() = None;
    }

    /// Enqueue message
    pub fn enqueue(&self, message: super::MachMessage) -> Result<(), IpcError> {
        let mut queue = self.queue.lock();
        if queue.messages.len() >= queue.capacity {
            return Err(IpcError::NoMemory);
        }
        queue.messages.push_back(message);
        Ok(())
    }

    /// Dequeue message
    pub fn dequeue(&self) -> Option<super::MachMessage> {
        self.queue.lock().messages.pop_front()
    }

    /// Peek at front message
    pub fn peek(&self) -> Option<super::MachMessage> {
        self.queue.lock().messages.front().cloned()
    }

    /// Get queue length
    pub fn queue_len(&self) -> usize {
        self.queue.lock().messages.len()
    }

    /// Check if queue is empty
    pub fn is_queue_empty(&self) -> bool {
        self.queue.lock().messages.is_empty()
    }

    /// Add waiter
    pub fn add_waiter(&self, task_id: TaskId) {
        let mut waiters = self.waiters.lock();
        if !waiters.contains(&task_id) {
            waiters.push(task_id);
        }
    }

    /// Remove waiter
    pub fn remove_waiter(&self, task_id: TaskId) {
        self.waiters.lock().retain(|&id| id != task_id);
    }

    /// Wake one waiter
    pub fn wake_one_waiter(&self) -> Option<TaskId> {
        self.waiters.lock().pop()
    }

    /// Wake all waiters
    pub fn wake_all_waiters(&self) -> Vec<TaskId> {
        core::mem::take(&mut *self.waiters.lock())
    }

    /// Get waiter count
    pub fn waiter_count(&self) -> usize {
        self.waiters.lock().len()
    }

    /// Set context
    pub fn set_context(&self, context: u32) {
        self.context.store(context, Ordering::Release);
    }

    /// Get context
    pub fn context(&self) -> u32 {
        self.context.load(Ordering::Acquire)
    }

    /// Add reference
    pub fn add_ref(&self) {
        self.refs.fetch_add(1, Ordering::AcqRel);
    }

    /// Release reference
    pub fn release(&self) -> u32 {
        self.refs.fetch_sub(1, Ordering::AcqRel)
    }

    /// Get reference count
    pub fn ref_count(&self) -> u32 {
        self.refs.load(Ordering::Acquire)
    }
}

/// Port namespace
pub struct PortNamespace {
    /// Task ID
    pub task_id: TaskId,
    /// Port name to port map
    ports: Mutex<BTreeMap<PortName, Arc<MachPort>>>,
    /// Name allocator
    name_allocator: AtomicU32,
    /// Rights table
    rights_table: Mutex<BTreeMap<PortName, PortRights>>,
}

impl PortNamespace {
    /// Create new namespace
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            ports: Mutex::new(BTreeMap::new()),
            name_allocator: AtomicU32::new(1),
            rights_table: Mutex::new(BTreeMap::new()),
        }
    }

    /// Allocate port name
    pub fn allocate_name(&self) -> PortName {
        self.name_allocator.fetch_add(1, Ordering::AcqRel) as u64
    }

    /// Insert port
    pub fn insert(&self, name: PortName, port: Arc<MachPort>) {
        self.ports.lock().insert(name, port);
    }

    /// Lookup port
    pub fn lookup(&self, name: PortName) -> Option<Arc<MachPort>> {
        self.ports.lock().get(&name).cloned()
    }

    /// Remove port
    pub fn remove(&self, name: PortName) -> Option<Arc<MachPort>> {
        self.ports.lock().remove(&name)
    }

    /// Set rights
    pub fn set_rights(&self, name: PortName, rights: PortRights) {
        self.rights_table.lock().insert(name, rights);
    }

    /// Get rights
    pub fn get_rights(&self, name: PortName) -> Option<PortRights> {
        self.rights_table.lock().get(&name).copied()
    }
}
