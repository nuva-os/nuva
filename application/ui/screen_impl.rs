/*
 * Nuva OS - Declarative Screen Implementation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Default Screen trait implementation and lifecycle hook integration.
 */

use crate::services::app::screen::ScreenLifecycleHook;

/** NuvaScreen default implementation.
 *
 * Provides empty default implementations for all Screen lifecycle
 * callbacks. Concrete screens override body() and optionally
 * any lifecycle callback.
 */
pub struct NuvaScreen;

impl ScreenLifecycleHook for NuvaScreen {
    /** Render pipeline resumes, window shows, events resume. */
    fn on_screen_running(&self, screen_id: u64) {
        let _ = screen_id;
    }

    /** Render pipeline pauses, window hides, events pause. */
    fn on_screen_suspended(&self, screen_id: u64) {
        let _ = screen_id;
    }

    /** Render pipeline releases, window destroyed, resources freed. */
    fn on_screen_terminated(&self, screen_id: u64) {
        let _ = screen_id;
    }
}
