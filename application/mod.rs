/*
 * Nuva OS - Application Framework
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

/** Nuva OS Declarative Application Framework.
 *
 * Fully declarative UI paradigm: Screen lifecycle, component model,
 * state binding, modifier chain, render pipeline, window management,
 * event system, and resource management — all Nuva-native.
 */

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod event;
pub mod render;
pub mod resource;
pub mod ui;
pub mod window;

/** Initialize the declarative application framework.
 *
 * Initialization order matters: adaptive layout first (other subsystems
 * depend on breakpoint/density info), then resource manager (components
 * may reference resources), then event dispatcher (window manager needs
 * it for input routing), then window manager (compositor needs window
 * surfaces), then compositor (render pipeline needs it for frame output),
 * and finally the render pipeline.
 */
pub fn init_application_framework() {
    // Phase 1: Layout engine (breakpoints, DPI, form factor)
    ui::adaptive::init_adaptive_layout();

    // Phase 2: Core declarative subsystems
    let _event_disp = event::declarative::get_event_dispatcher();
    let _compositor = render::declarative::get_compositor();
    let _res_mgr = resource::declarative::get_resource_manager();
    let _win_mgr = window::declarative::get_window_manager();

    log_info!("Declarative application framework initialized");
}
