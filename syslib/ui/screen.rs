/*
 * Nuva OS - Syslib - Ui - Screen
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
 * Nuva OS - Syslib - UI - Screen
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Declarative Screen trait and ScreenContent structure.
 */

use super::component::Element;

/** Screen trait — declarative screen lifecycle.
 *
 * Each screen must implement body() to declare its element tree.
 * Lifecycle callbacks have empty default implementations.
 */
pub trait Screen {
    /** Declare the element tree for this screen.
     *
     * Called each frame by the render pipeline. Must be side-effect-free.
     */
    fn body(&self) -> Element;

    /** Called when the screen is first created. */
    fn on_create(&self) {}

    /** Called when the screen enters Running state (foreground). */
    fn on_resume(&self) {}

    /** Called when the screen enters Suspended state (background). */
    fn on_suspend(&self) {}

    /** Called when the screen is being destroyed. */
    fn on_destroy(&self) {}
}

/** Screen content — the root element tree and screen identity. */
pub struct ScreenContent {
    /** Root element of the screen's declarative tree. */
    pub root: Element,
    /** Screen instance ID. */
    pub screen_id: u64,
}
