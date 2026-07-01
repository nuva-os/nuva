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

/** Declarative event system module. */

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
/** Declarative event types and dispatcher. */
pub mod declarative;

/// Event type enumeration
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    FormFactorChanged,
    PowerStateChanged,
    ThermalEvent,
    Custom(u32),
}

/// System event
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub timestamp: u64,
}

impl Event {
    pub fn new(event_type: EventType) -> Self {
        Event { event_type, timestamp: 0 }
    }
}

/// Event dispatcher module
pub mod dispatcher {
    use super::{Event, EventType};

    pub fn broadcast_system_event(_event: Event) {}
}
