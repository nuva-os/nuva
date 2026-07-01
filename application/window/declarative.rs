/*
 * Nuva OS - Application - Window - Declarative
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
 * Nuva OS - Declarative Window Management
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Declarative window, surface, and manager — screen-lifecycle-driven.
 * Integrates with ScreenLifecycleManager for show/hide/destroy,
 * and with the compositor for frame submission.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Maximum windows managed concurrently.
const MAX_WINDOWS: usize = 32;

/// Declarative window error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowError {
    TableFull,
    NotFound,
}

/// Declarative surface — framebuffer backing for a window.
pub struct DeclarativeSurface {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub buffer: AtomicU64,
    pub stride: u32,
    pub pixel_format: u32,
}

impl DeclarativeSurface {
    pub const fn new(id: u64, width: u32, height: u32, stride: u32) -> Self {
        DeclarativeSurface {
            id, width, height,
            buffer: AtomicU64::new(0),
            stride,
            pixel_format: 0,
        }
    }
}

/// Declarative window — screen-lifecycle-driven.
///
/// Each window belongs to a screen. The window manager controls
/// visibility based on the screen's lifecycle state:
/// - Running → visible=true, z-order set
/// - Suspended → visible=false
/// - Terminated → window unregistered
pub struct DeclarativeWindow {
    pub screen_id: u64,
    pub title: &'static str,
    pub width: f32,
    pub height: f32,
    pub fullscreen: bool,
    pub resizable: bool,
    pub always_on_top: bool,
    pub surface: Option<DeclarativeSurface>,
    pub visible: AtomicBool,
    pub z_order: AtomicU32,
}

/// Declarative window manager.
///
/// Manages window registration, visibility (driven by screen lifecycle),
/// z-ordering, and hit-testing. Integrates with ScreenLifecycleManager
/// via ScreenLifecycleHook.
pub struct DeclarativeWindowManager {
    /// Managed windows (indexed by slot, not by ID).
    windows: [Option<DeclarativeWindow>; MAX_WINDOWS],
    /// Number of registered windows.
    num_windows: AtomicU32,
    /// Foreground screen ID.
    foreground_screen: AtomicU64,
    /// Next z-order value allocator.
    next_z_order: AtomicU32,
}

impl DeclarativeWindowManager {
    /// Create a new declarative window manager.
    pub const fn new() -> Self {
        DeclarativeWindowManager {
            windows: [const { None }; MAX_WINDOWS],
            num_windows: AtomicU32::new(0),
            foreground_screen: AtomicU64::new(0),
            next_z_order: AtomicU32::new(1),
        }
    }

    /// Register a window for a screen.
    ///
    /// Creates a window descriptor and backing surface at the given
    /// dimensions. The window is initially hidden; it becomes visible
    /// when the owning screen transitions to Running.
    pub fn register_window(&self, screen_id: u64, title: &'static str, width: f32, height: f32) -> Result<u64, WindowError> {
        let idx = self.num_windows.load(Ordering::Acquire) as usize;
        if idx >= MAX_WINDOWS {
            return Err(WindowError::TableFull);
        }

        let z = self.next_z_order.fetch_add(1, Ordering::AcqRel);
        let window = DeclarativeWindow {
            screen_id,
            title,
            width,
            height,
            fullscreen: false,
            resizable: true,
            always_on_top: false,
            surface: Some(DeclarativeSurface::new(screen_id, width as u32, height as u32, (width * 4.0) as u32)),
            visible: AtomicBool::new(false),
            z_order: AtomicU32::new(z),
        };

        // SAFETY: idx < MAX_WINDOWS verified above.
        unsafe {
            let ptr = self.windows.as_ptr().offset(idx as isize) as *mut Option<DeclarativeWindow>;
            (*ptr) = Some(window);
        }
        self.num_windows.fetch_add(1, Ordering::AcqRel);
        Ok(screen_id)
    }

    /// Unregister a window (called on screen terminate).
    pub fn unregister_window(&self, screen_id: u64) -> Result<(), WindowError> {
        for slot in self.windows.iter() {
            if let Some(ref w) = slot {
                if w.screen_id == screen_id {
                    w.visible.store(false, Ordering::Release);
                    self.num_windows.fetch_sub(1, Ordering::AcqRel);
                    return Ok(());
                }
            }
        }
        Err(WindowError::NotFound)
    }

    /// Show a window (called when screen enters Running).
    pub fn show_window(&self, screen_id: u64) {
        for slot in self.windows.iter().flatten() {
            if slot.screen_id == screen_id {
                slot.visible.store(true, Ordering::Release);
                self.foreground_screen.store(screen_id, Ordering::Release);
                return;
            }
        }
    }

    /// Hide a window (called when screen enters Suspended).
    pub fn hide_window(&self, screen_id: u64) {
        for slot in self.windows.iter().flatten() {
            if slot.screen_id == screen_id {
                slot.visible.store(false, Ordering::Release);
                return;
            }
        }
    }

    /// Get window by screen ID.
    pub fn get_window(&self, screen_id: u64) -> Option<&DeclarativeWindow> {
        for slot in self.windows.iter().flatten() {
            if slot.screen_id == screen_id {
                return Some(slot);
            }
        }
        None
    }

    /// Find the topmost visible window at a point (hit-testing).
    pub fn get_window_at_point(&self, x: f32, y: f32) -> Option<u64> {
        let mut best_z: u32 = 0;
        let mut best_id: u64 = 0;

        for slot in self.windows.iter().flatten() {
            if !slot.visible.load(Ordering::Acquire) { continue; }
            let z = slot.z_order.load(Ordering::Acquire);
            if x >= 0.0 && x <= slot.width && y >= 0.0 && y <= slot.height && z > best_z {
                best_z = z;
                best_id = slot.screen_id;
            }
        }

        if best_id != 0 { Some(best_id) } else { None }
    }

    /// Get the foreground screen ID.
    pub fn get_foreground_screen(&self) -> Option<u64> {
        let id = self.foreground_screen.load(Ordering::Acquire);
        if id != 0 { Some(id) } else { None }
    }
}

/// Global declarative window manager.
static WINDOW_MANAGER: crate::sync_oncelock::OnceLock<DeclarativeWindowManager> = crate::sync_oncelock::OnceLock::new();

/// Get the global declarative window manager.
pub fn get_window_manager() -> &'static DeclarativeWindowManager {
    WINDOW_MANAGER.get_or_init(DeclarativeWindowManager::new)
}
