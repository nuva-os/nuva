/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - SchedCoop
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
 * Nuva OS - Kernel - NvPowerMgr Scheduler Cooperation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Power optimization considers scheduling needs:
 * NvPowerMgr queries NvScheduler for active task
 * information and never sleeps devices with active
 * high-priority tasks.
 */

use core::sync::atomic::{AtomicU64, Ordering};

/// Scheduling context for power decisions
#[derive(Clone, Debug)]
pub struct SchedContextForPower {
    /// Number of active high-priority tasks
    pub active_high_prio_tasks: u32,
    /// Device indices with active tasks
    pub active_task_devices: [bool; 16],
    /// Current scheduling pressure (0-100)
    pub sched_pressure: u32,
}

impl SchedContextForPower {
    /// Create an empty context
    pub const fn new() -> Self {
        SchedContextForPower {
            active_high_prio_tasks: 0,
            active_task_devices: [false; 16],
            sched_pressure: 0,
        }
    }

    /// Check if a device has active tasks
    pub fn device_has_active_tasks(&self, device_index: usize) -> bool {
        if device_index < 16 {
            self.active_task_devices[device_index]
        } else {
            false
        }
    }
}

/// PowerSchedCoop: power-scheduling cooperation
///
/// NvPowerMgr queries NvScheduler for active task
/// information before making power decisions.
/// Devices with active high-priority tasks are
/// never put to sleep.
pub struct PowerSchedCoop {
    /// Cooperation events
    coop_events: AtomicU64,
    /// Sleep prevented due to active tasks
    sleep_prevented: AtomicU64,
}

impl PowerSchedCoop {
    /// Create a new power-scheduling cooperation
    pub const fn new() -> Self {
        PowerSchedCoop {
            coop_events: AtomicU64::new(0),
            sleep_prevented: AtomicU64::new(0),
        }
    }

    /// Check if a device can be put to sleep
    ///
    /// @param device_index: Target device
    /// @param sched_ctx: Current scheduling context
    /// @return: true if device can sleep
    pub fn can_sleep(&self, device_index: usize, sched_ctx: &SchedContextForPower) -> bool {
        self.coop_events.fetch_add(1, Ordering::Relaxed);

        if sched_ctx.device_has_active_tasks(device_index) {
            self.sleep_prevented.fetch_add(1, Ordering::Relaxed);
            return false;
        }

        if sched_ctx.active_high_prio_tasks > 0 && sched_ctx.sched_pressure > 70 {
            return false;
        }

        true
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64) {
        (
            self.coop_events.load(Ordering::Acquire),
            self.sleep_prevented.load(Ordering::Acquire),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_sleep_no_active_tasks() {
        let coop = PowerSchedCoop::new();
        let ctx = SchedContextForPower::new();
        assert!(coop.can_sleep(0, &ctx));
    }

    #[test]
    fn test_cannot_sleep_active_tasks() {
        let coop = PowerSchedCoop::new();
        let mut ctx = SchedContextForPower::new();
        ctx.active_task_devices[0] = true;
        assert!(!coop.can_sleep(0, &ctx));
    }

    #[test]
    fn test_cannot_sleep_high_pressure() {
        let coop = PowerSchedCoop::new();
        let mut ctx = SchedContextForPower::new();
        ctx.active_high_prio_tasks = 5;
        ctx.sched_pressure = 80;
        assert!(!coop.can_sleep(0, &ctx));
    }
}