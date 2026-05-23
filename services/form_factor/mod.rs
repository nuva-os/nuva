/*
 * Nuva OS - Services - Form Factor Manager
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

//! Form Factor Manager service for runtime device adaptation.
/*!*/
//! Detects the current form factor at boot, handles runtime transitions
//! (docking/undocking, external display connect/disconnect), and coordinates
//! policy changes across power, scheduler, UI, and input subsystems.

use crate::{pr_debug, pr_info, pr_warn};
use crate::hal::platform::{FormFactor, PlatformInfo, get_platform_info};

/// Form factor change event.
#[derive(Debug, Clone, Copy)]
pub struct FormFactorChangedEvent {
    /// Previous form factor.
    pub old_form_factor: FormFactor,
    /// New form factor.
    pub new_form_factor: FormFactor,
    /// Reason for the change.
    pub reason: TransitionReason,
}

/// Reason for a form factor transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionReason {
    /// Initial detection at boot.
    BootDetection,
    /// External display connected.
    ExternalDisplayConnected,
    /// External display disconnected.
    ExternalDisplayDisconnected,
    /// Keyboard attached.
    KeyboardAttached,
    /// Keyboard detached.
    KeyboardDetached,
    /// Docking station connected.
    Docked,
    /// Undocking.
    Undocked,
    /// User manually requested transition.
    UserRequest,
    /// Foldable device folded.
    Folded,
    /// Foldable device unfolded.
    Unfolded,
}

/// System policy configuration for a form factor.
pub struct FormFactorPolicy {
    /// Default power mode.
    pub default_power_mode: PowerModeDefault,
    /// Scheduler target latency in microseconds.
    pub sched_target_latency_us: u32,
    /// Memory reclaim aggressiveness (0-100).
    pub memory_reclaim_aggressiveness: u8,
    /// UI chrome density (compact/medium/full).
    pub ui_chrome_density: u8,
    /// Window mode.
    pub window_mode: WindowModePolicy,
    /// Maximum background app count.
    pub max_background_apps: u32,
    /// Input method priority (0 = touch first, 1 = keyboard first).
    pub input_priority: u8,
}

/// Default power mode for a form factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerModeDefault {
    /// Performance mode (PC on AC).
    Performance,
    /// Balanced mode (mobile, tablet, PC on battery).
    Balanced,
    /// Power save mode (low battery).
    PowerSave,
}

/// Window mode policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowModePolicy {
    /// Full-screen single app.
    Fullscreen,
    /// Split-screen.
    SplitScreen,
    /// Free multi-window.
    FreeMultiWindow,
}

/// Get the default policy for a form factor.
pub fn get_default_policy(form_factor: FormFactor) -> FormFactorPolicy {
    match form_factor {
        FormFactor::Mobile => FormFactorPolicy {
            default_power_mode: PowerModeDefault::Balanced,
            sched_target_latency_us: 6000,
            memory_reclaim_aggressiveness: 80,
            ui_chrome_density: 0, // Compact
            window_mode: WindowModePolicy::Fullscreen,
            max_background_apps: 4,
            input_priority: 0, // Touch first
        },
        FormFactor::Tablet => FormFactorPolicy {
            default_power_mode: PowerModeDefault::Balanced,
            sched_target_latency_us: 4000,
            memory_reclaim_aggressiveness: 50,
            ui_chrome_density: 1, // Medium
            window_mode: WindowModePolicy::SplitScreen,
            max_background_apps: 8,
            input_priority: 0, // Touch ≈ Keyboard
        },
        FormFactor::Pc => FormFactorPolicy {
            default_power_mode: PowerModeDefault::Performance,
            sched_target_latency_us: 3000,
            memory_reclaim_aggressiveness: 20,
            ui_chrome_density: 2, // Full
            window_mode: WindowModePolicy::FreeMultiWindow,
            max_background_apps: u32::MAX, // Unlimited
            input_priority: 1, // Keyboard first
        },
    }
}

/// Form Factor Manager state.
pub struct FormFactorManager {
    /// Current form factor.
    pub current: FormFactor,
    /// Current policy.
    pub policy: FormFactorPolicy,
    /// Whether a transition is in progress.
    pub transitioning: bool,
    /// Number of transitions that have occurred.
    pub transition_count: u32,
}

impl FormFactorManager {
    /// Create a new Form Factor Manager.
    pub const fn new() -> Self {
        FormFactorManager {
            current: FormFactor::Mobile,
            policy: FormFactorPolicy {
                default_power_mode: PowerModeDefault::Balanced,
                sched_target_latency_us: 6000,
                memory_reclaim_aggressiveness: 80,
                ui_chrome_density: 0,
                window_mode: WindowModePolicy::Fullscreen,
                max_background_apps: 4,
                input_priority: 0,
            },
            transitioning: false,
            transition_count: 0,
        }
    }

    /// Initialize the Form Factor Manager from platform detection.
    pub fn init(&mut self) {
        let info = get_platform_info();
        self.current = info.form_factor;
        self.policy = get_default_policy(info.form_factor);

        log_info!("FormFactor: Detected {:?} at boot", self.current);
        log_info!("  Power: {:?}, Sched latency: {}us, Reclaim: {}%",
            self.policy.default_power_mode,
            self.policy.sched_target_latency_us,
            self.policy.memory_reclaim_aggressiveness);
        log_info!("  Window: {:?}, Max BG apps: {}, Input priority: {}",
            self.policy.window_mode,
            self.policy.max_background_apps,
            self.policy.input_priority);
    }

    /// Request a form factor transition.
    /// This coordinates the transition across all subsystems:
    /// 1. Emit FormFactorChanged event to all running applications
    /// 2. Update window manager mode
    /// 3. Adjust power policy defaults
    /// 4. Reconfigure scheduler parameters
    /// 5. Update input method priorities
    /// 6. Trigger layout reflow
    /// Returns true if the transition was accepted.
    pub fn request_transition(&mut self, new_form_factor: FormFactor, reason: TransitionReason) -> bool {
        if self.current == new_form_factor {
            return false; // Already in this form factor
        }

        if self.transitioning {
            log_warn!("FormFactor: Transition already in progress");
            return false;
        }

        let old_form_factor = self.current;
        log_info!("FormFactor: Transitioning from {:?} to {:?} (reason: {:?})",
            old_form_factor, new_form_factor, reason);

        self.transitioning = true;

        // Step 1: Update policy
        self.policy = get_default_policy(new_form_factor);

        // Step 2: Emit event to all running applications
        let event = FormFactorChangedEvent {
            old_form_factor,
            new_form_factor,
            reason,
        };
        self.broadcast_form_factor_changed(event);

        // Step 3: Update power policy
        self.update_power_policy();

        // Step 4: Update scheduler parameters
        self.update_scheduler_params();

        // Step 5: Update window manager mode
        self.update_window_mode();

        // Step 6: Update input method priorities
        self.update_input_priorities();

        // Step 7: Trigger layout reflow
        self.trigger_layout_reflow();

        // Complete transition
        self.current = new_form_factor;
        self.transitioning = false;
        self.transition_count += 1;

        log_info!("FormFactor: Transition complete, now {:?}", self.current);
        true
    }

    /// Broadcast FormFactorChanged event to all running applications.
    fn broadcast_form_factor_changed(&self, event: FormFactorChangedEvent) {
        // Deliver the FormFactorChanged event to all running processes
        // via the application event dispatcher.
        crate::application::event::dispatcher::broadcast_system_event(
            crate::application::event::Event::new(
                crate::application::event::EventType::FormFactorChanged
            )
        );
        log_info!("FormFactor: Broadcasting change event: {:?} -> {:?}",
            event.old_form_factor, event.new_form_factor);
    }

    /// Update power policy based on new form factor.
    fn update_power_policy(&self) {
        // Apply the new default power mode through the power service
        match self.policy.default_power_mode {
            PowerModeDefault::Performance => {
                crate::services::power::manager::set_power_mode(
                    crate::services::power::manager::PowerMode::Performance as u32
                );
            }
            PowerModeDefault::Balanced => {
                crate::services::power::manager::set_power_mode(
                    crate::services::power::manager::PowerMode::Balanced as u32
                );
            }
            PowerModeDefault::PowerSave => {
                crate::services::power::manager::set_power_mode(
                    crate::services::power::manager::PowerMode::Powersave as u32
                );
            }
        }
        log_debug!("FormFactor: Power policy updated to {:?}", self.policy.default_power_mode);
    }

    /// Update scheduler parameters based on new form factor.
    fn update_scheduler_params(&self) {
        // Update CFS scheduler target latency and min granularity
        crate::kernel::sched::set_sched_latency(self.policy.sched_target_latency_us as u64);
        log_debug!("FormFactor: Scheduler latency updated to {}us",
            self.policy.sched_target_latency_us);
    }

    /// Update window manager mode based on new form factor.
    fn update_window_mode(&self) {
        // Change window manager mode (fullscreen/split/free)
        crate::syslib::ui::window::set_window_mode(match self.policy.window_mode {
            WindowModePolicy::Fullscreen => crate::syslib::ui::window::Mode::Fullscreen,
            WindowModePolicy::SplitScreen => crate::syslib::ui::window::Mode::SplitScreen,
            WindowModePolicy::FreeMultiWindow => crate::syslib::ui::window::Mode::FreeMultiWindow,
        });
        log_debug!("FormFactor: Window mode updated to {:?}", self.policy.window_mode);
    }

    /// Update input method priorities based on new form factor.
    fn update_input_priorities(&self) {
        // Set input subsystem priority: 0 = touch-first, 1 = keyboard-first
        crate::hal::input::set_input_priority(self.policy.input_priority as u32);
        log_debug!("FormFactor: Input priority updated to {}", self.policy.input_priority);
    }

    /// Trigger layout reflow across all applications.
    fn trigger_layout_reflow(&self) {
        // Notify all applications to recalculate their layout.
        // Target: complete within 100ms for smooth transition.
        crate::syslib::ui::layout::reflow_all();
        log_debug!("FormFactor: Layout reflow triggered");
    }

    /// Check if an external display connection should trigger a transition.
    pub fn on_external_display_connected(&mut self) -> bool {
        if self.current == FormFactor::Mobile {
            // Offer to transition to PC mode
            log_info!("FormFactor: External display connected, offering PC mode");
            true
        } else {
            false
        }
    }

    /// Check if an external display disconnection should trigger a transition.
    pub fn on_external_display_disconnected(&mut self) -> bool {
        if self.current == FormFactor::Pc {
            // Check if we were originally a mobile device
            let info = get_platform_info();
            if info.is_mobile() || info.is_tablet() {
                log_info!("FormFactor: External display disconnected, reverting to original form factor");
                let target = if info.is_mobile() { FormFactor::Mobile } else { FormFactor::Tablet };
                self.request_transition(target, TransitionReason::ExternalDisplayDisconnected);
                return true;
            }
        }
        false
    }

    /// Check if a keyboard connection should trigger a transition.
    pub fn on_keyboard_attached(&mut self) {
        if self.current == FormFactor::Tablet {
            // Enable PC-style keyboard shortcuts and focus navigation
            log_info!("FormFactor: Keyboard attached to tablet, enabling PC-style input");
            self.policy.input_priority = 1; // Keyboard first
        }
    }

    /// Check if a keyboard disconnection should trigger a transition.
    pub fn on_keyboard_detached(&mut self) {
        if self.current == FormFactor::Tablet {
            // Revert to touch-first input
            log_info!("FormFactor: Keyboard detached from tablet, reverting to touch-first");
            self.policy.input_priority = 0; // Touch first
        }
    }
}

/// Global Form Factor Manager.
static FORM_FACTOR_MANAGER: core::sync::OnceLock<FormFactorManager> = core::sync::OnceLock::new();

/// Get the global Form Factor Manager.
pub fn get_form_factor_manager() -> &'static FormFactorManager {
    // SAFETY: Initialized once during boot.
    unsafe { &FORM_FACTOR_MANAGER }
}

/// Get a mutable reference to the global Form Factor Manager.
pub fn get_form_factor_manager_mut() -> &'static mut FormFactorManager {
    // SAFETY: Only called during single-threaded initialization.
    unsafe { &mut FORM_FACTOR_MANAGER }
}

/// Initialize the Form Factor Manager.
pub fn init_form_factor_manager() {
    let manager = get_form_factor_manager_mut();
    manager.init();
}
