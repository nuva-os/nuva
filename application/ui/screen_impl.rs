/*
 * Nuva OS - Application - Ui - ScreenImpl
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
