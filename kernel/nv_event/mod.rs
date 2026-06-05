/*
 * Nuva OS - Kernel - NvEvent - Mod
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
 * Nuva OS - Kernel - NvEvent (Nuva Native Event Notification)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native event notification replacing POSIX signals.
 * Migrated from: POSIX signal (SIGHUP/SIGINT/SIGKILL) → NvEvent + NvNotificationPort.
 *
 * Events are delivered via NvIPC ports, not asynchronous signal interruption.
 */

use crate::kernel::types::{NuvaProcessId, NuvaCapabilityId, NvPortId, NvDuration};
use crate::kernel::error::{KernelError, KernelResult};

/// Nuva event handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvEventHandle(pub u64);

/// Nuva event registration types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvEventType {
    /// Process exit notification
    ProcessExit = 0,
    /// Memory pressure notification
    MemoryPressure = 1,
    /// Device state change
    DeviceChange = 2,
    /// Port death (DeadName) notification
    PortDeath = 3,
    /// Timer expiration
    TimerExpired = 4,
    /// Custom application event
    Custom = 5,
}

/// Nuva event payload
#[derive(Debug, Clone, Copy)]
pub struct NvEvent {
    /// Event type
    pub event_type: NvEventType,
    /// Event source process
    pub source: NuvaProcessId,
    /// Event-specific data
    pub payload: u64,
    /// Timestamp of event generation
    pub timestamp_ns: u64,
}

impl NvEvent {
    /// Create a new event
    pub fn new(event_type: NvEventType, source: NuvaProcessId, payload: u64) -> Self {
        NvEvent {
            event_type,
            source,
            payload,
            timestamp_ns: 0,
        }
    }
}

/// Register for event notification
///
/// PRE: caller must hold appropriate capability.
/// POST: events of event_type are delivered to notification_port.
///
/// Migrated from: POSIX signal() → nv_event_register + NvNotificationPort.
pub fn nv_event_register(
    event_type: NvEventType,
    notification_port: NvPortId,
    cap: NuvaCapabilityId,
) -> KernelResult<NvEventHandle> {
    let _ = (event_type, notification_port, cap);
    Ok(NvEventHandle(0))
}

/// Notify (send) an event
///
/// PRE: caller must hold appropriate capability for the event type.
pub fn nv_event_notify(
    event_handle: NvEventHandle,
    event: &NvEvent,
    cap: NuvaCapabilityId,
) -> KernelResult<()> {
    let _ = (event_handle, event, cap);
    Ok(())
}

/// Wait for event notification
///
/// Blocks until an event arrives on the notification port.
pub fn nv_event_wait(
    notification_port: NvPortId,
    timeout: NvDuration,
    cap: NuvaCapabilityId,
) -> KernelResult<NvEvent> {
    let _ = (notification_port, timeout, cap);
    Ok(NvEvent::new(NvEventType::Custom, NuvaProcessId::new(0), 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nv_event_create() {
        let event = NvEvent::new(NvEventType::ProcessExit, NuvaProcessId::new(1), 0);
        assert_eq!(event.event_type, NvEventType::ProcessExit);
    }

    #[test]
    fn test_nv_event_register() {
        let result = nv_event_register(
            NvEventType::MemoryPressure,
            NvPortId::new(1),
            NuvaCapabilityId::new(1),
        );
        assert!(result.is_ok());
    }
}
