/*
 * Nuva OS - Kernel - Declarative Power Management
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

use crate::kernel::error::{KernelError, KernelResult};

/**
 * Device power state in the declarative power state machine.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PmState {
    /** Device is fully powered on and active */
    On,
    /** Device is in low-power idle state */
    Idle,
    /** Device is suspended (context saved, can resume) */
    Suspend,
    /** Device is powered off (context lost) */
    Off,
}

/**
 * A transition in the power state machine.
 *
 * Describes a valid transition from one power state to another,
 * with an optional guard condition and latency.
 */
#[derive(Debug, Clone, Copy)]
pub struct PmTransition {
    /** Source state */
    pub from: PmState,
    /** Destination state */
    pub to: PmState,
    /** Maximum transition latency in microseconds */
    pub latency_us: u32,
}

/**
 * Declarative power state machine.
 *
 * Describes the valid power state transitions for a device.
 * The framework uses this to automatically generate runtime
 * power management logic, reject invalid transitions, and
 * track transition latencies.
 *
 * # Example
 * ```rust
 * static MY_PM: PmStateMachine = PmStateMachine::new(
 *     "my_device",
 *     &[
 *         PmTransition { from: PmState::On, to: PmState::Idle, latency_us: 10 },
 *         PmTransition { from: PmState::Idle, to: PmState::On, latency_us: 10 },
 *         PmTransition { from: PmState::Idle, to: PmState::Suspend, latency_us: 100 },
 *         PmTransition { from: PmState::Suspend, to: PmState::On, latency_us: 500 },
 *         PmTransition { from: PmState::On, to: PmState::Off, latency_us: 1000 },
 *         PmTransition { from: PmState::Off, to: PmState::On, latency_us: 5000 },
 *     ],
 * );
 * ```
 */
pub struct PmStateMachine {
    /** State machine name for debugging */
    pub name: &'static str,
    /** Valid transitions table */
    pub transitions: &'static [PmTransition],
    /** Current state (atomic for lock-free reads) */
    current_state: core::sync::atomic::AtomicU8,
}

/** PmState to u8 mapping for atomic storage */
fn pm_state_to_u8(s: PmState) -> u8 {
    match s {
        PmState::On => 0,
        PmState::Idle => 1,
        PmState::Suspend => 2,
        PmState::Off => 3,
    }
}

/** u8 to PmState mapping */
fn u8_to_pm_state(v: u8) -> Option<PmState> {
    match v {
        0 => Some(PmState::On),
        1 => Some(PmState::Idle),
        2 => Some(PmState::Suspend),
        3 => Some(PmState::Off),
        _ => None,
    }
}

impl PmStateMachine {
    /** Create a new power state machine with initial state On */
    pub const fn new(name: &'static str, transitions: &'static [PmTransition]) -> Self {
        PmStateMachine {
            name,
            transitions,
            current_state: core::sync::atomic::AtomicU8::new(0), // On
        }
    }

    /** Get the current power state */
    #[inline(always)]
    pub fn current_state(&self) -> PmState {
        let raw = self
            .current_state
            .load(core::sync::atomic::Ordering::Acquire);
        u8_to_pm_state(raw).unwrap_or(PmState::On)
    }

    /**
     * Request a state transition.
     *
     * Validates that the transition exists in the state machine
     * before applying it. Returns `Err(KernelError::InvalidState)`
     * for invalid transitions.
     */
    pub fn transition(&self, target: PmState) -> KernelResult<()> {
        let current = self.current_state();

        if current == target {
            return Ok(());
        }

        // Validate transition exists
        let valid = self
            .transitions
            .iter()
            .any(|t| t.from == current && t.to == target);

        if !valid {
            return Err(KernelError::InvalidState);
        }

        // Apply transition
        self.current_state.store(
            pm_state_to_u8(target),
            core::sync::atomic::Ordering::Release,
        );

        Ok(())
    }

    /**
     * Get the latency for a transition.
     *
     * Returns 0 if the transition is not defined.
     */
    pub fn transition_latency(&self, from: PmState, to: PmState) -> u32 {
        self.transitions
            .iter()
            .find(|t| t.from == from && t.to == to)
            .map(|t| t.latency_us)
            .unwrap_or(0)
    }

    /**
     * Check if a transition is valid.
     */
    pub fn can_transition(&self, target: PmState) -> bool {
        let current = self.current_state();
        current == target
            || self
                .transitions
                .iter()
                .any(|t| t.from == current && t.to == target)
    }
}

/**
 * Macro to define a declarative power state machine.
 *
 * # Example
 * ```rust
 * declare_pm! {
 *     MY_DEVICE_PM {
 *         On => Idle: 10us,
 *         Idle => On: 10us,
 *         Idle => Suspend: 100us,
 *         Suspend => On: 500us,
 *         On => Off: 1000us,
 *         Off => On: 5000us,
 *     }
 * }
 * ```
 */
#[macro_export]
macro_rules! declare_pm {
    (
        $name:ident {
            $($from:ident => $to:ident: $latency:tt us),* $(,)?
        }
    ) => {
        static $name: $crate::kernel::driver::declarative_pm::PmStateMachine =
            $crate::kernel::driver::declarative_pm::PmStateMachine::new(
                stringify!($name),
                &[
                    $(
                        $crate::kernel::driver::declarative_pm::PmTransition {
                            from: $crate::kernel::driver::declarative_pm::PmState::$from,
                            to: $crate::kernel::driver::declarative_pm::PmState::$to,
                            latency_us: $latency,
                        }
                    ),*
                ],
            );
    };
}

/**
 * Macro to define a declarative driver.
 *
 * # Example
 * ```rust
 * declare_driver! {
 *     MY_DRIVER {
 *         name: "my_driver",
 *         compatible: &["vendor,my-device"],
 *         resources: &[ResourceDescriptor::Irq { number: 42 }],
 *         capabilities: READ | WRITE,
 *         priority: 0,
 *         hotplug: false,
 *     }
 * }
 * ```
 */
#[macro_export]
macro_rules! declare_driver {
    (
        $name:ident {
            name: $driver_name:expr,
            compatible: $compatible:expr,
            resources: $resources:expr,
            capabilities: $caps:expr,
            priority: $prio:expr,
            hotplug: $hp:expr,
        }
    ) => {
        static $name: $crate::kernel::driver::declarative::DriverDescriptor =
            $crate::kernel::driver::declarative::DriverDescriptor {
                name: $driver_name,
                compatible: $compatible,
                resources: $resources,
                capabilities:
                    $crate::kernel::driver::declarative::CapabilityFlags::from_bits_truncate($caps),
                priority: $prio,
                hotplug: $hp,
            };
    };
}

/**
 * Declarative resource descriptor for automatic resource acquisition and release.
 *
 * Describes a named resource with acquire and release lifecycle hooks.
 * The framework uses this to automatically manage resource lifecycles
 * during driver probe/remove.
 */
#[derive(Debug, Clone)]
pub struct DeclarativeResource {
    /** Resource name for debugging and identification */
    pub name: &'static str,
    /** Resource type identifier */
    pub resource_type: ResourceType,
    /** Whether this resource is optional (probe continues on failure) */
    pub optional: bool,
}

/** Resource type classification for declarative resources. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /** Interrupt request line */
    Irq,
    /** Memory-mapped I/O region */
    Mmio,
    /** Clock source */
    Clock,
    /** Power domain */
    PowerDomain,
    /** GPIO pin */
    Gpio,
    /** I2C bus */
    I2c,
    /** DMA channel */
    Dma,
    /** Regulator (voltage/current supply) */
    Regulator,
    /** Reset control */
    Reset,
}

/**
 * Macro to define a declarative resource binding.
 *
 * # Example
 * ```rust
 * declare_resource! {
 *     MY_DEVICE_RES {
 *         name: "my_device_res",
 *         resource_type: Irq,
 *         optional: false,
 *     }
 * }
 * ```
 */
#[macro_export]
macro_rules! declare_resource {
    (
        $name:ident {
            name: $res_name:expr,
            resource_type: $res_type:ident,
            optional: $opt:expr,
        }
    ) => {
        static $name: $crate::kernel::driver::declarative_pm::DeclarativeResource =
            $crate::kernel::driver::declarative_pm::DeclarativeResource {
                name: $res_name,
                resource_type: $crate::kernel::driver::declarative_pm::ResourceType::$res_type,
                optional: $opt,
            };
    };
}
