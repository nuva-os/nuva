/*
 * Nuva OS - SystemService - Screen
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
use super::lifecycle::{NuvaLifecycleState, NuvaLifecycleEvent, NuvaTaskContext, LifecycleError};

/** Nuva screen state.
 *
 * Four-state model for the Nuva declarative screen lifecycle.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuvaScreenState {
    /** Screen initialized but not yet running. */
    Initialized = 0,
    /** Screen is actively running. */
    Running = 1,
    /** Screen is suspended (background). */
    Suspended = 2,
    /** Screen has terminated. */
    Terminated = 3,
}

/** Nuva screen instance descriptor. */
pub struct NuvaScreenInstance {
    /** Instance ID */
    pub instance_id: AtomicU64,
    /** Application ID */
    pub app_id: u64,
    /** Screen name */
    pub name: &'static str,
    /** Current state */
    state: AtomicU32,
    /** Owning task ID */
    pub task_id: u32,
}

impl NuvaScreenInstance {
    /** Create a new screen instance. */
    pub const fn new(instance_id: u64, app_id: u64, name: &'static str) -> Self {
        NuvaScreenInstance {
            instance_id: AtomicU64::new(instance_id),
            app_id,
            name,
            state: AtomicU32::new(NuvaScreenState::Initialized as u32),
            task_id: 0,
        }
    }

    /** Get the current screen state. */
    pub fn get_state(&self) -> NuvaScreenState {
        match self.state.load(Ordering::Acquire) {
            0 => NuvaScreenState::Initialized,
            1 => NuvaScreenState::Running,
            2 => NuvaScreenState::Suspended,
            _ => NuvaScreenState::Terminated,
        }
    }

    /** Set the screen state. */
    pub fn set_state(&self, state: NuvaScreenState) {
        self.state.store(state as u32, Ordering::Release);
    }
}

/** Hook for screen lifecycle events.
 *
 * Implement this trait to receive callbacks when screen state changes.
 * The render pipeline, window manager, and event dispatcher integrate
 * through this hook.
 */
pub trait ScreenLifecycleHook {
    /** Called when a screen enters Running state. */
    fn on_screen_running(&self, _screen_id: u64) {}
    /** Called when a screen enters Suspended state. */
    fn on_screen_suspended(&self, _screen_id: u64) {}
    /** Called when a screen enters Terminated state. */
    fn on_screen_terminated(&self, _screen_id: u64) {}
}

/** Nuva Screen Lifecycle Manager.
 *
 * Manages screen instances with a four-state lifecycle model,
 * replacing the legacy activity-based lifecycle management.
 */
pub struct ScreenLifecycleManager {
    /** Screen instances */
    instances: [Option<NuvaScreenInstance>; 128],
    /** Number of instances */
    num_instances: AtomicU32,
    /** Task contexts */
    task_contexts: [Option<NuvaTaskContext>; 32],
    /** Number of tasks */
    num_tasks: AtomicU32,
    /** Current foreground task */
    current_task: AtomicU64,
    /** Next instance ID allocator */
    next_instance_id: AtomicU64,
    /** Next task ID allocator */
    next_task_id: AtomicU64,
}

impl ScreenLifecycleManager {
    /** Create a new screen lifecycle manager. */
    pub const fn new() -> Self {
        ScreenLifecycleManager {
            instances: [None; 128],
            num_instances: AtomicU32::new(0),
            task_contexts: [const { None }; 32],
            num_tasks: AtomicU32::new(0),
            current_task: AtomicU64::new(0),
            next_instance_id: AtomicU64::new(1),
            next_task_id: AtomicU64::new(1),
        }
    }

    /** Initialize the screen lifecycle manager. */
    pub fn init(&self) {
        log_info!("Screen lifecycle manager initialized");
    }

    /** Launch a screen, creating a new instance. */
    pub fn launch_screen(&self, app_id: u64, name: &'static str) -> Option<u64> {
        let instance_id = self.next_instance_id.fetch_add(1, Ordering::AcqRel);

        for slot in self.instances.iter() {
            if slot.is_none() {
                let _ = NuvaScreenInstance::new(instance_id, app_id, name);
                self.num_instances.fetch_add(1, Ordering::AcqRel);
                self.current_task.store(instance_id, Ordering::Release);
                log_debug!("Screen launched: {} (instance={})", name, instance_id);
                return Some(instance_id);
            }
        }

        None
    }

    /** Suspend a screen instance (move to background). */
    pub fn suspend_screen(&self, instance_id: u64) -> Result<(), LifecycleError> {
        for slot in self.instances.iter() {
            if let Some(ref instance) = slot {
                if instance.instance_id.load(Ordering::Acquire) == instance_id {
                    instance.set_state(NuvaScreenState::Suspended);
                    return Ok(());
                }
            }
        }
        Err(LifecycleError::InstanceNotFound)
    }

    /** Resume a suspended screen instance. */
    pub fn resume_screen(&self, instance_id: u64) -> Result<(), LifecycleError> {
        for slot in self.instances.iter() {
            if let Some(ref instance) = slot {
                if instance.instance_id.load(Ordering::Acquire) == instance_id {
                    instance.set_state(NuvaScreenState::Running);
                    self.current_task.store(instance_id, Ordering::Release);
                    return Ok(());
                }
            }
        }
        Err(LifecycleError::InstanceNotFound)
    }

    /** Terminate a screen instance. */
    pub fn terminate_screen(&self, instance_id: u64) -> Result<(), LifecycleError> {
        for slot in self.instances.iter() {
            if let Some(ref instance) = slot {
                if instance.instance_id.load(Ordering::Acquire) == instance_id {
                    instance.set_state(NuvaScreenState::Terminated);
                    self.num_instances.fetch_sub(1, Ordering::AcqRel);
                    return Ok(());
                }
            }
        }
        Err(LifecycleError::InstanceNotFound)
    }

    /** Get the current foreground screen instance ID. */
    pub fn get_current_screen(&self) -> Option<u64> {
        let instance_id = self.current_task.load(Ordering::Acquire);
        if instance_id > 0 { Some(instance_id) } else { None }
    }

    /** Notify all running screens of resource pressure. */
    pub fn on_resource_pressure(&self) {
        for slot in self.instances.iter() {
            if let Some(ref instance) = slot {
                if instance.get_state() == NuvaScreenState::Running {
                    instance.set_state(NuvaScreenState::Suspended);
                }
            }
        }
    }
}

/** Global screen lifecycle manager instance. */
static SCREEN_LIFECYCLE_MANAGER: core::sync::OnceLock<ScreenLifecycleManager> = core::sync::OnceLock::new();

/** Get a reference to the global screen lifecycle manager. */
pub fn get_screen_lifecycle_manager() -> &'static ScreenLifecycleManager {
    SCREEN_LIFECYCLE_MANAGER.get_or_init(ScreenLifecycleManager::new)
}

/** Initialize the global screen lifecycle manager. */
pub fn init_screen_lifecycle_manager() {
    let manager = get_screen_lifecycle_manager();
    manager.init();
}
