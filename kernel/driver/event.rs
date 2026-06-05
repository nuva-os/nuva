/*
 * Nuva OS - Kernel - Driver - Event
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
 * Nuva OS - Kernel - Device Event System
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Event notification system for device drivers.
 */

use crate::pr_info;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Event Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    // Device events
    /// Device added
    DeviceAdded = 0,
    /// Device removed
    DeviceRemoved = 1,
    /// Device changed
    DeviceChanged = 2,

    // Input events
    /// Key event
    KeyEvent = 10,
    /// Touch event
    TouchEvent = 11,
    /// Mouse event
    MouseEvent = 12,

    // Audio events
    /// Audio buffer ready
    AudioBuffer = 20,
    /// Audio underrun
    AudioUnderrun = 21,

    // Sensor events
    /// Sensor data ready
    SensorData = 30,

    // Power events
    /// Power status changed
    PowerStatus = 40,
    /// Battery low
    BatteryLow = 41,

    // USB events
    /// USB device connected
    UsbConnected = 50,
    /// USB device disconnected
    UsbDisconnected = 51,

    // Custom events
    Custom = 255,
}

/// Event Priority
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
}

/// Event Header
#[repr(C)]
pub struct EventHeader {
    /// Event type
    pub event_type: EventType,
    /// Event ID
    pub event_id: u32,
    /// Source device ID
    pub source_id: u32,
    /// Timestamp (nanoseconds)
    pub timestamp: u64,
    /// Priority
    pub priority: EventPriority,
    /// Data size
    pub data_size: u16,
    /// Flags
    pub flags: u16,
}

/// Event Structure
#[repr(C)]
pub struct DeviceEvent {
    /// Event header
    pub header: EventHeader,
    /// Event data (max 256 bytes)
    pub data: [u8; 256],
}

impl DeviceEvent {
    /// Create a new event
    pub const fn new(event_type: EventType, source_id: u32) -> Self {
        DeviceEvent {
            header: EventHeader {
                event_type,
                event_id: 0,
                source_id,
                timestamp: 0,
                priority: EventPriority::Normal,
                data_size: 0,
                flags: 0,
            },
            data: [0; 256],
        }
    }

    /// Set event data
    pub fn set_data(&mut self, data: &[u8]) {
        let len = data.len().min(256);
        self.data[..len].copy_from_slice(&data[..len]);
        self.header.data_size = len as u16;
    }

    /// Get event data
    pub fn get_data(&self) -> &[u8] {
        &self.data[..self.header.data_size as usize]
    }
}

/// Event Subscriber ID
pub type SubscriberId = u32;

/// Event Subscriber Callback
pub type EventCallback = unsafe extern "C" fn(*const DeviceEvent, *mut core::ffi::c_void);

/// Event Subscriber
pub struct EventSubscriber {
    /// Subscriber ID
    pub id: SubscriberId,
    /// Event types to subscribe
    pub event_types: u64,
    /// Callback function
    pub callback: EventCallback,
    /// User data
    pub user_data: *mut core::ffi::c_void,
    /// Priority (higher = called first)
    pub priority: i32,
    /// Active flag
    pub active: bool,
}

/// Event Queue
pub struct EventQueue {
    /// Queue buffer
    pub buffer: [DeviceEvent; 64],
    /// Head index
    pub head: AtomicU32,
    /// Tail index
    pub tail: AtomicU32,
    /// Count
    pub count: AtomicU32,
    /// Dropped count
    pub dropped: AtomicU64,
}

impl EventQueue {
    pub const fn new() -> Self {
        EventQueue {
            buffer: [const { DeviceEvent::new(EventType::Custom, 0) }; 64],
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            count: AtomicU32::new(0),
            dropped: AtomicU64::new(0),
        }
    }

    /// Push event to queue
    pub fn push(&mut self, event: &DeviceEvent) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let next_head = (head + 1) % 64;

        if next_head == self.tail.load(Ordering::Acquire) {
            // Queue full
            self.dropped.fetch_add(1, Ordering::AcqRel);
            return false;
        }

        self.buffer[head as usize] = event.clone();
        self.head.store(next_head, Ordering::Release);
        self.count.fetch_add(1, Ordering::AcqRel);

        true
    }

    /// Pop event from queue
    pub fn pop(&mut self) -> Option<DeviceEvent> {
        let tail = self.tail.load(Ordering::Acquire);

        if tail == self.head.load(Ordering::Acquire) {
            return None;
        }

        let event = self.buffer[tail as usize].clone();
        self.tail.store((tail + 1) % 64, Ordering::Release);
        self.count.fetch_sub(1, Ordering::AcqRel);

        Some(event)
    }

    /// Get queue length
    pub fn len(&self) -> u32 {
        self.count.load(Ordering::Acquire)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Clone for DeviceEvent {
    fn clone(&self) -> Self {
        DeviceEvent {
            header: EventHeader {
                event_type: self.header.event_type,
                event_id: self.header.event_id,
                source_id: self.header.source_id,
                timestamp: self.header.timestamp,
                priority: self.header.priority,
                data_size: self.header.data_size,
                flags: self.header.flags,
            },
            data: self.data,
        }
    }
}

/// Event Manager
pub struct EventManager {
    /// Next event ID
    next_event_id: AtomicU32,
    /// Next subscriber ID
    next_subscriber_id: AtomicU32,
    /// Event queue
    event_queue: EventQueue,
    /// Statistics
    stats: EventStats,
}

/// Event Statistics
pub struct EventStats {
    /// Total events generated
    pub total_events: AtomicU64,
    /// Events delivered
    pub delivered: AtomicU64,
    /// Events dropped
    pub dropped: AtomicU64,
    /// Subscribers notified
    pub notified: AtomicU64,
}

impl EventStats {
    pub const fn new() -> Self {
        EventStats {
            total_events: AtomicU64::new(0),
            delivered: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            notified: AtomicU64::new(0),
        }
    }
}

impl EventManager {
    pub const fn new() -> Self {
        EventManager {
            next_event_id: AtomicU32::new(1),
            next_subscriber_id: AtomicU32::new(1),
            event_queue: EventQueue::new(),
            stats: EventStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Event manager initialized");
    }

    /// Generate event
    pub fn generate_event(&mut self, event_type: EventType, source_id: u32, data: &[u8]) -> u32 {
        let event_id = self.next_event_id.fetch_add(1, Ordering::AcqRel);

        let mut event = DeviceEvent::new(event_type, source_id);
        event.header.event_id = event_id;
        event.set_data(data);

        self.stats.total_events.fetch_add(1, Ordering::AcqRel);

        if self.event_queue.push(&event) {
            event_id
        } else {
            0
        }
    }

    /// Process events
    pub fn process_events(&mut self) -> u32 {
        let mut processed = 0u32;

        while let Some(_event) = self.event_queue.pop() {
            // TODO: Notify subscribers
            processed += 1;
            self.stats.delivered.fetch_add(1, Ordering::AcqRel);
        }

        processed
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64, u64, u64) {
        (
            self.stats.total_events.load(Ordering::Acquire),
            self.stats.delivered.load(Ordering::Acquire),
            self.stats.dropped.load(Ordering::Acquire),
            self.stats.notified.load(Ordering::Acquire),
        )
    }
}

/// Global event manager
static EVENT_MANAGER: core::sync::OnceLock<EventManager> = core::sync::OnceLock::new();

/// Get event manager
pub fn event_manager() -> &'static EventManager {
    EVENT_MANAGER.get_or_init(EventManager::new)
}

pub fn init_event_manager() -> &'static EventManager {
    EVENT_MANAGER.get_or_init(EventManager::new)
}

/// Initialize event manager
pub fn init_event_manager() {
    let mgr = event_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_values() {
        assert_eq!(EventType::DeviceAdded as i32, 0);
        assert_eq!(EventType::KeyEvent as i32, 10);
        assert_eq!(EventType::SensorData as i32, 30);
    }

    #[test]
    fn test_event_priority_ordering() {
        assert!(EventPriority::Low < EventPriority::Normal);
        assert!(EventPriority::Normal < EventPriority::High);
        assert!(EventPriority::High < EventPriority::Critical);
    }

    #[test]
    fn test_device_event_new() {
        let event = DeviceEvent::new(EventType::KeyEvent, 1);
        assert_eq!(event.header.event_type, EventType::KeyEvent);
        assert_eq!(event.header.source_id, 1);
        assert_eq!(event.header.data_size, 0);
    }

    #[test]
    fn test_device_event_data() {
        let mut event = DeviceEvent::new(EventType::SensorData, 1);
        let data = [1u8, 2, 3, 4, 5];
        event.set_data(&data);

        assert_eq!(event.header.data_size, 5);
        assert_eq!(event.get_data(), &data[..]);
    }

    #[test]
    fn test_event_queue() {
        let mut queue = EventQueue::new();
        let event = DeviceEvent::new(EventType::KeyEvent, 1);

        assert!(queue.is_empty());
        assert!(queue.push(&event));
        assert_eq!(queue.len(), 1);

        let popped = queue.pop();
        assert!(popped.is_some());
        assert!(queue.is_empty());
    }
}
