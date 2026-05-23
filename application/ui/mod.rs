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

/** Nuva OS Declarative UI Framework
 *
 * Provides a fully declarative UI paradigm for Nuva OS:
 * - **Screen System**: Declarative screen lifecycle and composition
 * - **Component Model**: Pure-function render, 9 built-in components
 * - **State Binding**: Reactive State<T> with dirty marking
 * - **Modifier Chain**: Zero-cost chainable style/event/window modifiers
 * - **Render Pipeline**: Reconcile → Layout → Paint → Composite
 * - **Adaptive Layout**: Breakpoint system, DPI scaling, gesture recognition
 */

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

/** Adaptive layout engine for multi-platform support. */
pub mod adaptive;
/** Declarative UI type definitions (Point, Size, Rect, Color). */
pub mod types;
/** Declarative screen implementation. */
pub mod screen_impl;
/** Declarative component implementations. */
pub mod component_impl;
/** Chainable modifier API. */
pub mod modifier;
/** Render pipeline (Reconcile → Layout → Paint → Composite). */
pub mod pipeline;
/** O(n) diff/reconcile algorithm. */
pub mod reconcile;
/** Layout algorithms (Horizontal, Vertical, Grid, Flex). */
pub mod layout;
/** Style and theme system (FontStyle, BorderStyle, ShadowStyle, Theme). */
pub mod style;
