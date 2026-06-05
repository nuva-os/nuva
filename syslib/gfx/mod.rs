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

//! Graphics library module.
//!
//! Supports multiple backends:
//! - **Vulkan**: Zero-copy GPU direct passthrough (when vulkan feature enabled)
//! - **Software**: CPU software rendering fallback
//!
//! Vulkan backend is preferred when available. If Vulkan initialization
//! fails, automatically degrades to software rendering.

pub mod fps;

/// Graphics rendering backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsBackend {
    /// Vulkan zero-copy GPU direct passthrough (preferred)
    Vulkan,
    /// CPU software rendering (fallback)
    Software,
}

/// Current active graphics backend
static ACTIVE_BACKEND: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

impl GraphicsBackend {
    pub fn as_u8(&self) -> u8 {
        match self {
            GraphicsBackend::Vulkan   => 0,
            GraphicsBackend::Software => 1,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => GraphicsBackend::Vulkan,
            _ => GraphicsBackend::Software,
        }
    }
}

/// Initialize graphics with preferred backend.
/// Vulkan backend is tried first; falls back to software on failure.
pub fn init_gfx_with_backend(preferred: GraphicsBackend) -> GraphicsBackend {
    match preferred {
        GraphicsBackend::Vulkan => {
            #[cfg(feature = "vulkan")]
            {
                if init_vulkan_backend().is_ok() {
                    ACTIVE_BACKEND.store(GraphicsBackend::Vulkan.as_u8(), core::sync::atomic::Ordering::Release);
                    return GraphicsBackend::Vulkan;
                }
            }
            ACTIVE_BACKEND.store(GraphicsBackend::Software.as_u8(), core::sync::atomic::Ordering::Release);
            GraphicsBackend::Software
        }
        GraphicsBackend::Software => {
            ACTIVE_BACKEND.store(GraphicsBackend::Software.as_u8(), core::sync::atomic::Ordering::Release);
            GraphicsBackend::Software
        }
    }
}

/// Initialize Vulkan graphics backend
#[cfg(feature = "vulkan")]
fn init_vulkan_backend() -> Result<(), ()> {
    // TODO: Create NvVulkanInstance, enumerate devices, create device
    Ok(())
}

/// Get current active backend
pub fn get_active_backend() -> GraphicsBackend {
    GraphicsBackend::from_u8(ACTIVE_BACKEND.load(core::sync::atomic::Ordering::Acquire))
}

// Initialize graphics library (default: try Vulkan, fallback to Software)
pub fn init_gfx() {
    init_gfx_with_backend(GraphicsBackend::Vulkan);
}
