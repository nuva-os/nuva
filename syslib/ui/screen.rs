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
