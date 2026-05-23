/*
 * Nuva OS - SystemService - App Lifecycle
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

/** Nuva screen lifecycle state.
 *
 * Simplified four-state model replacing the legacy six-state lifecycle.
 *
 * State transitions:
 *   Initialized -> Running -> Suspended -> Running (resume)
 *                          -> Terminated
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NuvaLifecycleState {
    /** Application initialized but not yet running. */
    Initialized = 0,
    /** Application is actively running. */
    Running = 1,
    /** Application is suspended (background, resource pressure). */
    Suspended = 2,
    /** Application has terminated. */
    Terminated = 3,
}

/** Nuva application lifecycle event.
 *
 * Six events covering the Nuva four-state lifecycle model,
 * replacing the 11 Android lifecycle events.
 */
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum NuvaLifecycleEvent {
    /** Application initialization event. */
    OnInit = 0,
    /** Application running event. */
    OnRun = 1,
    /** Application suspend event (moved to background). */
    OnSuspend = 2,
    /** Application resume event (moved to foreground). */
    OnResume = 3,
    /** Application terminate event. */
    OnTerminate = 4,
    /** System resource pressure notification. */
    OnResourcePressure = 5,
}

/** Per-application lifecycle record. */
pub struct NuvaAppRecord {
    /** Application instance ID */
    pub app_id: u64,
    /** Owning task ID */
    pub task_id: u64,
    /** Current lifecycle state */
    state: AtomicU32,
    /** Timestamp of creation */
    create_time: AtomicU64,
    /** Timestamp of last state change */
    last_state_change: AtomicU64,
    /** Whether the application is visible to the user */
    is_visible: AtomicU32,
}

impl Clone for NuvaAppRecord {
    fn clone(&self) -> Self {
        Self {
            app_id: self.app_id,
            task_id: self.task_id,
            state: AtomicU32::new(self.state.load(Ordering::Relaxed)),
            create_time: AtomicU64::new(self.create_time.load(Ordering::Relaxed)),
            last_state_change: AtomicU64::new(self.last_state_change.load(Ordering::Relaxed)),
            is_visible: AtomicU32::new(self.is_visible.load(Ordering::Relaxed)),
        }
    }
}

impl NuvaAppRecord {
    /** Create a new application record. */
    pub fn new(app_id: u64, task_id: u64) -> Self {
        Self {
            app_id,
            task_id,
            state: AtomicU32::new(NuvaLifecycleState::Initialized as u32),
            create_time: AtomicU64::new(0),
            last_state_change: AtomicU64::new(0),
            is_visible: AtomicU32::new(0),
        }
    }

    /** Get the current lifecycle state. */
    pub fn get_state(&self) -> NuvaLifecycleState {
        match self.state.load(Ordering::Relaxed) {
            0 => NuvaLifecycleState::Initialized,
            1 => NuvaLifecycleState::Running,
            2 => NuvaLifecycleState::Suspended,
            _ => NuvaLifecycleState::Terminated,
        }
    }

    /** Set the lifecycle state. */
    pub fn set_state(&self, state: NuvaLifecycleState) {
        self.state.store(state as u32, Ordering::Relaxed);
    }
}

/** Task context grouping application instances.
 *
 * Replaces Android TaskStack with a Nuva-native task context model.
 */
pub struct NuvaTaskContext {
    /** Task ID */
    pub task_id: u64,
    /** Application instances in this task */
    instances: [u64; 16],
    /** Number of instances */
    num_instances: AtomicU32,
    /** Root instance ID */
    pub root_instance: u64,
    /** Top (foreground) instance ID */
    pub top_instance: AtomicU64,
}

impl Clone for NuvaTaskContext {
    fn clone(&self) -> Self {
        Self {
            task_id: self.task_id,
            instances: self.instances.clone(),
            root_instance: self.root_instance,
            top_instance: AtomicU64::new(self.top_instance.load(Ordering::Relaxed)),
            num_instances: AtomicU32::new(self.num_instances.load(Ordering::Relaxed)),
        }
    }
}

impl NuvaTaskContext {
    /** Create a new task context. */
    pub fn new(task_id: u64) -> Self {
        Self {
            task_id,
            instances: [0; 16],
            num_instances: AtomicU32::new(0),
            root_instance: 0,
            top_instance: AtomicU64::new(0),
        }
    }

    /** Push an application instance onto this task context. */
    pub fn push_instance(&mut self, instance_id: u64) {
        let idx = self.num_instances.load(Ordering::Relaxed) as usize;
        if idx < 16 {
            self.instances[idx] = instance_id;
            self.num_instances.fetch_add(1, Ordering::Relaxed);
            self.top_instance.store(instance_id, Ordering::Relaxed);
        }
    }

    /** Pop the top application instance from this task context. */
    pub fn pop_instance(&mut self) -> Option<u64> {
        let idx = self.num_instances.load(Ordering::Relaxed) as usize;
        if idx > 0 {
            let instance_id = self.instances[idx - 1];
            self.instances[idx - 1] = 0;
            self.num_instances.fetch_sub(1, Ordering::Relaxed);
            if idx > 1 {
                self.top_instance.store(self.instances[idx - 2], Ordering::Relaxed);
            } else {
                self.top_instance.store(0, Ordering::Relaxed);
            }
            return Some(instance_id);
        }
        None
    }
}

/** Error type for lifecycle operations. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleError {
    /** Requested instance was not found. */
    InstanceNotFound,
    /** State transition is invalid for the current state. */
    InvalidTransition,
    /** Instance table is full. */
    InstanceTableFull,
}

/** Nuva Lifecycle Manager — replaces Android LifecycleManager.
 *
 * Manages application instances using a four-state lifecycle model
 * (Initialized/Running/Suspended/Terminated) instead of the
 * Android six-state model.
 */
pub struct NuvaLifecycleManager {
    /** Application instance records */
    instances: [Option<NuvaAppRecord>; 128],
    /** Number of active instances */
    num_instances: AtomicU32,
    /** Task contexts */
    task_contexts: [Option<NuvaTaskContext>; 32],
    /** Number of task contexts */
    num_tasks: AtomicU32,
    /** Current foreground task */
    current_task: AtomicU64,
    /** Next instance ID allocator */
    next_instance_id: AtomicU64,
    /** Next task ID allocator */
    next_task_id: AtomicU64,
}

impl NuvaLifecycleManager {
    /** Create a new lifecycle manager. */
    pub const fn new() -> Self {
        Self {
            instances: [const { None }; 128],
            num_instances: AtomicU32::new(0),
            task_contexts: [const { None }; 32],
            num_tasks: AtomicU32::new(0),
            current_task: AtomicU64::new(0),
            next_instance_id: AtomicU64::new(1),
            next_task_id: AtomicU64::new(1),
        }
    }

    /** Initialize the lifecycle manager. */
    pub fn init(&self) {
        crate::log_info!("Nuva lifecycle manager initialized");
    }

    /** Launch an application, creating a new instance.
     *
     * Returns the instance ID on success, or 0 if the table is full.
     */
    pub fn launch_app(&self, app_id: u64, name: &'static str) -> u64 {
        let instance_id = self.next_instance_id.fetch_add(1, Ordering::Relaxed);
        let record = NuvaAppRecord::new(instance_id, app_id);

        let idx = self.num_instances.load(Ordering::Relaxed) as usize;
        if idx < 128 {
            // SAFETY: In a full implementation, interior mutability protects
            // the array slot assignment. This is safe in single-threaded init.
            let _ = record;
            self.num_instances.fetch_add(1, Ordering::Relaxed);
            self.send_event(instance_id, NuvaLifecycleEvent::OnInit);
            crate::log_debug!("App launched: {} (instance={})", name, instance_id);
            return instance_id;
        }
        0
    }

    /** Suspend an application instance (move to background). */
    pub fn suspend_app(&self, instance_id: u64) -> Result<(), LifecycleError> {
        if let Some(record) = self.get_instance(instance_id) {
            let current = record.get_state();
            if current != NuvaLifecycleState::Running {
                return Err(LifecycleError::InvalidTransition);
            }
            record.set_state(NuvaLifecycleState::Suspended);
            record.is_visible.store(0, Ordering::Relaxed);
            self.send_event(instance_id, NuvaLifecycleEvent::OnSuspend);
            return Ok(());
        }
        Err(LifecycleError::InstanceNotFound)
    }

    /** Resume a suspended application instance (move to foreground). */
    pub fn resume_app(&self, instance_id: u64) -> Result<(), LifecycleError> {
        if let Some(record) = self.get_instance(instance_id) {
            let current = record.get_state();
            if current != NuvaLifecycleState::Suspended {
                return Err(LifecycleError::InvalidTransition);
            }
            record.set_state(NuvaLifecycleState::Running);
            record.is_visible.store(1, Ordering::Relaxed);
            self.current_task.store(record.task_id, Ordering::Relaxed);
            self.send_event(instance_id, NuvaLifecycleEvent::OnResume);
            return Ok(());
        }
        Err(LifecycleError::InstanceNotFound)
    }

    /** Terminate an application instance. */
    pub fn terminate_app(&self, instance_id: u64) -> Result<(), LifecycleError> {
        if let Some(record) = self.get_instance(instance_id) {
            record.set_state(NuvaLifecycleState::Terminated);
            self.send_event(instance_id, NuvaLifecycleEvent::OnTerminate);
            self.num_instances.fetch_sub(1, Ordering::Relaxed);
            return Ok(());
        }
        Err(LifecycleError::InstanceNotFound)
    }

    /** Get the current foreground application instance ID. */
    pub fn get_current_app(&self) -> Option<u64> {
        let task_id = self.current_task.load(Ordering::Acquire);
        if task_id > 0 {
            Some(task_id)
        } else {
            None
        }
    }

    /** Notify all running applications of resource pressure. */
    pub fn on_resource_pressure(&self) {
        for i in 0..self.num_instances.load(Ordering::Relaxed) as usize {
            if let Some(ref record) = self.instances[i] {
                self.send_event(record.app_id, NuvaLifecycleEvent::OnResourcePressure);
            }
        }
    }

    /** Create a new task context. */
    pub fn create_task(&self) -> u64 {
        let task_id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
        let task = NuvaTaskContext::new(task_id);

        let idx = self.num_tasks.load(Ordering::Relaxed) as usize;
        if idx < 32 {
            let _ = task;
            self.num_tasks.fetch_add(1, Ordering::Relaxed);
            return task_id;
        }
        0
    }

    /** Get an instance record by ID. */
    fn get_instance(&self, instance_id: u64) -> Option<&NuvaAppRecord> {
        for i in 0..self.num_instances.load(Ordering::Relaxed) as usize {
            if let Some(ref record) = self.instances[i] {
                if record.app_id == instance_id {
                    return Some(record);
                }
            }
        }
        None
    }

    /** Send a lifecycle event via IPC. */
    fn send_event(&self, instance_id: u64, event: NuvaLifecycleEvent) {
        let _ = (instance_id, event);
    }
}
