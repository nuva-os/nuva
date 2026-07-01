/*
 * Nuva OS - Application - Adaptive Layout Engine
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

//! Adaptive layout engine for multi-platform UI rendering.
/*!*/
//! Provides:
//! - Breakpoint system (Compact/Medium/Expanded) for responsive layout
//! - DPI scaler for density-independent pixel (dp) conversion
//! - Input abstraction layer unifying touch, mouse, keyboard, stylus
//! - Gesture recognizer for touch input
//! - Window mode selection based on form factor

// ============================================================================
// Breakpoint System
// ============================================================================

/// Layout breakpoint classification based on screen width in dp.
use crate::{pr_info};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    /// Compact: < 600dp (mobile phone, portrait).
    Compact,
    /// Medium: 600-840dp (tablet portrait, phone landscape).
    Medium,
    /// Expanded: > 840dp (tablet landscape, PC).
    Expanded,
}

impl Breakpoint {
    /// Determine breakpoint from screen width in dp.
    pub fn from_width_dp(width_dp: u32) -> Self {
        if width_dp < 600 {
            Breakpoint::Compact
        } else if width_dp <= 840 {
            Breakpoint::Medium
        } else {
            Breakpoint::Expanded
        }
    }

    /// Determine breakpoint from pixel width and DPI.
    pub fn from_pixels(width_px: u32, dpi: u32) -> Self {
        if dpi == 0 { return Breakpoint::Expanded; }
        let width_dp = (width_px as u32 * 160) / dpi;
        Self::from_width_dp(width_dp)
    }

    /// Get the number of columns for this breakpoint (Nuva responsive grid).
    pub fn columns(&self) -> u32 {
        match self {
            Breakpoint::Compact => 4,
            Breakpoint::Medium => 8,
            Breakpoint::Expanded => 12,
        }
    }

    /// Get the default margin in dp for this breakpoint.
    pub fn margin_dp(&self) -> u32 {
        match self {
            Breakpoint::Compact => 16,
            Breakpoint::Medium => 24,
            Breakpoint::Expanded => 24,
        }
    }

    /// Get the default gutter in dp for this breakpoint.
    pub fn gutter_dp(&self) -> u32 {
        match self {
            Breakpoint::Compact => 16,
            Breakpoint::Medium => 16,
            Breakpoint::Expanded => 24,
        }
    }
}

// ============================================================================
// DPI Density Tiers
// ============================================================================

/// DPI density tier for resource selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DensityTier {
    /// Low density tier (~120 dpi).
    Low,
    /// Baseline density tier (~160 dpi).
    Medium,
    /// High density tier (~240 dpi).
    High,
    /// Extra high density tier (~320 dpi).
    ExtraHigh,
    /// Extra extra high density tier (~480 dpi).
    ExtraExtraHigh,
    /// Maximum density tier (~640 dpi).
    ExtraExtraExtraHigh,
}

impl DensityTier {
    /// Determine density tier from DPI value.
    pub fn from_dpi(dpi: u32) -> Self {
        if dpi <= 120 {
            DensityTier::Low
        } else if dpi <= 160 {
            DensityTier::Medium
        } else if dpi <= 240 {
            DensityTier::High
        } else if dpi <= 320 {
            DensityTier::ExtraHigh
        } else if dpi <= 480 {
            DensityTier::ExtraExtraHigh
        } else {
            DensityTier::ExtraExtraExtraHigh
        }
    }

    /// Get the DPI value for this tier.
    pub fn dpi(&self) -> u32 {
        match self {
            DensityTier::Low => 120,
            DensityTier::Medium => 160,
            DensityTier::High => 240,
            DensityTier::ExtraHigh => 320,
            DensityTier::ExtraExtraHigh => 480,
            DensityTier::ExtraExtraExtraHigh => 640,
        }
    }

    /// Get the scale factor relative to mdpi (160 dpi).
    pub fn scale_factor(&self) -> f32 {
        self.dpi() as f32 / 160.0
    }

    /// Convert dp (density-independent pixels) to physical pixels.
    pub fn dp_to_px(&self, dp: f32) -> f32 {
        dp * self.scale_factor()
    }

    /// Convert physical pixels to dp.
    pub fn px_to_dp(&self, px: f32) -> f32 {
        px / self.scale_factor()
    }
}

// ============================================================================
// Input Abstraction Layer
// ============================================================================

/// Platform-independent unified input event.
#[derive(Debug, Clone)]
pub enum UnifiedInputEvent {
    /// Pointer down (touch start or mouse button press).
    PointerDown {
        /// X coordinate in dp.
        x: f32,
        /// Y coordinate in dp.
        y: f32,
        /// Pointer ID (for multi-touch).
        pointer_id: u32,
        /// Button/pointer type.
        button: PointerButton,
    },
    /// Pointer move (touch drag or mouse move).
    PointerMove {
        x: f32,
        y: f32,
        pointer_id: u32,
    },
    /// Pointer up (touch end or mouse button release).
    PointerUp {
        x: f32,
        y: f32,
        pointer_id: u32,
        button: PointerButton,
    },
    /// Scroll (mouse wheel or touch scroll gesture).
    Scroll {
        /// Horizontal scroll delta in dp.
        dx: f32,
        /// Vertical scroll delta in dp.
        dy: f32,
    },
    /// Key press.
    KeyPress {
        /// Platform-independent key code.
        key: UnifiedKey,
        /// Modifier keys.
        modifiers: Modifiers,
    },
    /// Key release.
    KeyRelease {
        key: UnifiedKey,
        modifiers: Modifiers,
    },
    /// Gesture recognition result.
    Gesture(GestureEvent),
}

/// Pointer button type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    /// Primary button (left mouse, single touch).
    Primary,
    /// Secondary button (right mouse).
    Secondary,
    /// Tertiary button (middle mouse).
    Tertiary,
    /// Touch contact (no specific button).
    Touch,
    /// Stylus tip.
    Stylus,
    /// Stylus eraser.
    StylusEraser,
}

/// Platform-independent key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifiedKey {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Digits
    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,
    // Navigation
    Enter, Escape, Tab, Backspace, Delete,
    // Arrow keys
    Up, Down, Left, Right,
    // Home/End
    Home, End, PageUp, PageDown,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Space
    Space,
    // Media
    VolumeUp, VolumeDown, Mute,
    // Power
    Power,
    /// Navigate back (platform-native)
    NavigateBack,
    // Unknown
    Unknown,
}

/// Keyboard modifier flags.
bitflags::bitflags! {
    /// Modifier key state.
    pub struct Modifiers: u32 {
        /// Shift key held.
        const SHIFT    = 1 << 0;
        /// Control key held.
        const CTRL     = 1 << 1;
        /// Alt/Option key held.
        const ALT      = 1 << 2;
        /// Meta/Command/Windows key held.
        const META     = 1 << 3;
        /// Caps Lock active.
        const CAPS     = 1 << 4;
        /// Num Lock active.
        const NUM      = 1 << 5;
    }
}

impl Clone for Modifiers {
    fn clone(&self) -> Self { *self }
}
impl Copy for Modifiers {}

impl core::fmt::Debug for Modifiers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Modifiers({})", self.bits())
    }
}

// ============================================================================
// Gesture Recognizer
// ============================================================================

/// Recognized gesture events.
#[derive(Debug, Clone, Copy)]
pub enum GestureEvent {
    /// Single tap.
    Tap {
        x: f32,
        y: f32,
    },
    /// Double tap.
    DoubleTap {
        x: f32,
        y: f32,
    },
    /// Long press (hold > 500ms).
    LongPress {
        x: f32,
        y: f32,
    },
    /// Pinch/scale gesture.
    Pinch {
        /// Scale factor (1.0 = no change).
        scale: f32,
        /// Center X of the pinch.
        center_x: f32,
        /// Center Y of the pinch.
        center_y: f32,
    },
    /// Swipe/pan gesture.
    Swipe {
        /// Direction of the swipe.
        direction: SwipeDirection,
        /// Velocity in dp/s.
        velocity: f32,
    },
    /// Drag/pan gesture.
    Pan {
        /// Delta X in dp.
        dx: f32,
        /// Delta Y in dp.
        dy: f32,
    },
}

/// Swipe direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Gesture recognizer state.
pub struct GestureRecognizer {
    /// Touch start X.
    start_x: f32,
    /// Touch start Y.
    start_y: f32,
    /// Touch start time in ms.
    start_time: u64,
    /// Previous tap time for double-tap detection.
    last_tap_time: u64,
    /// Previous tap X.
    last_tap_x: f32,
    /// Previous tap Y.
    last_tap_y: f32,
    /// Whether a long press has been detected.
    long_press_detected: bool,
    /// Current pointer count (for pinch detection).
    pointer_count: u32,
    /// Previous pointer distance (for pinch).
    prev_pointer_distance: f32,
}

/// Tap timeout in ms.
const TAP_TIMEOUT_MS: u64 = 180;

/// Long press timeout in ms.
const LONG_PRESS_TIMEOUT_MS: u64 = 500;

/// Double-tap timeout in ms.
const DOUBLE_TAP_TIMEOUT_MS: u64 = 300;

/// Minimum movement for scroll/swipe in dp.
const TOUCH_SLOP_DP: f32 = 8.0;

/// Minimum velocity for swipe in dp/s.
const MIN_SWIPE_VELOCITY: f32 = 100.0;

impl GestureRecognizer {
    /// Create a new gesture recognizer.
    pub const fn new() -> Self {
        GestureRecognizer {
            start_x: 0.0,
            start_y: 0.0,
            start_time: 0,
            last_tap_time: 0,
            last_tap_x: 0.0,
            last_tap_y: 0.0,
            long_press_detected: false,
            pointer_count: 0,
            prev_pointer_distance: 0.0,
        }
    }

    /// Handle pointer down event.
    pub fn on_pointer_down(&mut self, x: f32, y: f32, time_ms: u64) -> Option<GestureEvent> {
        self.start_x = x;
        self.start_y = y;
        self.start_time = time_ms;
        self.long_press_detected = false;
        self.pointer_count += 1;
        None
    }

    /// Handle pointer move event.
    pub fn on_pointer_move(&mut self, x: f32, y: f32) -> Option<GestureEvent> {
        let dx = x - self.start_x;
        let dy = y - self.start_y;
        let distance = if dx * dx + dy * dy > 0.0 { true } else { false };

        if distance && !self.long_press_detected {
            // Movement detected: emit pan gesture
            Some(GestureEvent::Pan { dx, dy })
        } else {
            None
        }
    }

    /// Handle pointer up event.
    pub fn on_pointer_up(&mut self, x: f32, y: f32, time_ms: u64) -> Option<GestureEvent> {
        self.pointer_count = self.pointer_count.saturating_sub(1);

        let dx = x - self.start_x;
        let dy = y - self.start_y;
        let distance_sq = dx * dx + dy * dy;
        let elapsed = time_ms.saturating_sub(self.start_time);

        // Check for tap (short press with minimal movement)
        if distance_sq < TOUCH_SLOP_DP * TOUCH_SLOP_DP && elapsed < TAP_TIMEOUT_MS {
            // Check for double-tap
            let time_since_last_tap = time_ms.saturating_sub(self.last_tap_time);
            let tap_dx = x - self.last_tap_x;
            let tap_dy = y - self.last_tap_y;
            let tap_distance_sq = tap_dx * tap_dx + tap_dy * tap_dy;

            if time_since_last_tap < DOUBLE_TAP_TIMEOUT_MS
                && tap_distance_sq < TOUCH_SLOP_DP * TOUCH_SLOP_DP
            {
                self.last_tap_time = 0; // Reset to prevent triple-tap
                return Some(GestureEvent::DoubleTap { x, y });
            }

            self.last_tap_time = time_ms;
            self.last_tap_x = x;
            self.last_tap_y = y;
            return Some(GestureEvent::Tap { x, y });
        }

        // Check for swipe (significant movement with velocity)
        if distance_sq > TOUCH_SLOP_DP * TOUCH_SLOP_DP && elapsed > 0 {
            let velocity = distance_sq * 1000.0 * 1000.0 / (elapsed as f32); // TODO: no_std sqrt
            if velocity > MIN_SWIPE_VELOCITY {
                let direction = if dx.abs() > dy.abs() {
                    if dx > 0.0 { SwipeDirection::Right } else { SwipeDirection::Left }
                } else {
                    if dy > 0.0 { SwipeDirection::Down } else { SwipeDirection::Up }
                };
                return Some(GestureEvent::Swipe { direction, velocity });
            }
        }

        None
    }

    /// Check for long press (call periodically or from timer).
    pub fn check_long_press(&mut self, time_ms: u64) -> Option<GestureEvent> {
        if self.long_press_detected { return None; }
        if self.pointer_count == 0 { return None; }

        let elapsed = time_ms.saturating_sub(self.start_time);
        if elapsed >= LONG_PRESS_TIMEOUT_MS {
            self.long_press_detected = true;
            return Some(GestureEvent::LongPress {
                x: self.start_x,
                y: self.start_y,
            });
        }
        None
    }
}

// ============================================================================
// Window Mode
// ============================================================================

/// Window management mode, selected by form factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    /// Full-screen single app (mobile).
    Fullscreen,
    /// Split-screen two apps (tablet).
    SplitScreen,
    /// Free-floating multi-window (PC).
    FreeMultiWindow,
}

impl WindowMode {
    /// Get the default window mode for a form factor.
    pub fn for_form_factor(form_factor: crate::hal::platform::FormFactor) -> Self {
        match form_factor {
            crate::hal::platform::FormFactor::Mobile => WindowMode::Fullscreen,
            crate::hal::platform::FormFactor::Tablet => WindowMode::SplitScreen,
            crate::hal::platform::FormFactor::Pc => WindowMode::FreeMultiWindow,
        }
    }
}

// ============================================================================
// Adaptive Layout Engine
// ============================================================================

/// The adaptive layout engine coordinates breakpoint resolution, DPI scaling,
/// and form factor adaptation for the UI framework.
pub struct AdaptiveLayoutEngine {
    /// Current breakpoint.
    pub breakpoint: Breakpoint,
    /// Current density tier.
    pub density: DensityTier,
    /// Current DPI.
    pub dpi: u32,
    /// Screen width in pixels.
    pub screen_width: u32,
    /// Screen height in pixels.
    pub screen_height: u32,
    /// Current window mode.
    pub window_mode: WindowMode,
    /// Gesture recognizer.
    pub gesture_recognizer: GestureRecognizer,
}

impl AdaptiveLayoutEngine {
    /// Create a new adaptive layout engine.
    pub fn new() -> Self {
        AdaptiveLayoutEngine {
            breakpoint: Breakpoint::Compact,
            density: DensityTier::Medium,
            dpi: 160,
            screen_width: 1080,
            screen_height: 1920,
            window_mode: WindowMode::Fullscreen,
            gesture_recognizer: GestureRecognizer::new(),
        }
    }

    /// Initialize the layout engine from platform display information.
    pub fn init_from_platform(&mut self, width: u32, height: u32, dpi: u32) {
        self.screen_width = width;
        self.screen_height = height;
        self.dpi = dpi;

        // Determine density tier
        self.density = DensityTier::from_dpi(dpi);

        // Determine breakpoint
        self.breakpoint = Breakpoint::from_pixels(width, dpi);

        // Determine window mode from form factor
        let form_factor = crate::hal::platform::get_platform_info().form_factor;
        self.window_mode = WindowMode::for_form_factor(form_factor);

        log_info!("Layout: {}x{} @ {}dpi, density={:?}, breakpoint={:?}, mode={:?}",
            width, height, dpi, self.density, self.breakpoint, self.window_mode);
    }

    /// Convert dp to pixels.
    pub fn dp_to_px(&self, dp: f32) -> f32 {
        self.density.dp_to_px(dp)
    }

    /// Convert pixels to dp.
    pub fn px_to_dp(&self, px: f32) -> f32 {
        self.density.px_to_dp(px)
    }

    /// Get the screen width in dp.
    pub fn screen_width_dp(&self) -> f32 {
        self.px_to_dp(self.screen_width as f32)
    }

    /// Get the screen height in dp.
    pub fn screen_height_dp(&self) -> f32 {
        self.px_to_dp(self.screen_height as f32)
    }

    /// Process a raw input event and return unified events.
    /// This method maps platform-specific input to platform-independent
    /// UnifiedInputEvent and may also produce gesture events.
    pub fn process_input(&mut self, event: UnifiedInputEvent, time_ms: u64) -> [Option<UnifiedInputEvent>; 2] {
        let mut result: [Option<UnifiedInputEvent>; 2] = [None, None];

        // Run gesture recognition on pointer events
        match &event {
            UnifiedInputEvent::PointerDown { x, y, .. } => {
                if let Some(gesture) = self.gesture_recognizer.on_pointer_down(*x, *y, time_ms) {
                    result[1] = Some(UnifiedInputEvent::Gesture(gesture));
                }
            }
            UnifiedInputEvent::PointerMove { x, y, .. } => {
                if let Some(gesture) = self.gesture_recognizer.on_pointer_move(*x, *y) {
                    result[1] = Some(UnifiedInputEvent::Gesture(gesture));
                }
            }
            UnifiedInputEvent::PointerUp { x, y, .. } => {
                if let Some(gesture) = self.gesture_recognizer.on_pointer_up(*x, *y, time_ms) {
                    result[1] = Some(UnifiedInputEvent::Gesture(gesture));
                }
            }
            _ => {}
        }

        result[0] = Some(event);
        result
    }

    /// Handle form factor change at runtime.
    pub fn on_form_factor_changed(&mut self, new_form_factor: crate::hal::platform::FormFactor) {
        self.window_mode = WindowMode::for_form_factor(new_form_factor);

        // Recalculate breakpoint if screen info changed
        self.breakpoint = Breakpoint::from_pixels(self.screen_width, self.dpi);

        log_info!("Layout: Form factor changed to {:?}, window mode={:?}", new_form_factor, self.window_mode);
    }
}

/** Global adaptive layout engine instance. */
static LAYOUT_ENGINE: crate::sync_oncelock::OnceLock<AdaptiveLayoutEngine> = crate::sync_oncelock::OnceLock::new();

/** Get a reference to the global layout engine. */
pub fn get_layout_engine() -> &'static AdaptiveLayoutEngine {
    LAYOUT_ENGINE.get_or_init(AdaptiveLayoutEngine::new)
}

/** Initialize the adaptive layout engine.
 *
 * Called during application framework initialization. Uses OnceLock
 * to ensure single initialization. The init_from_platform call is
 * deferred until the first screen is created, since platform info
 * must be fully resolved by HAL before layout engine configuration.
 */
pub fn init_adaptive_layout() {
    let _engine = get_layout_engine();
    log_info!("Adaptive layout engine initialized");
}
