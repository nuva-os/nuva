/*
 * Nuva OS - Kernel - Driver - Class - Display
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
 * Nuva OS - Kernel - Display Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for display/graphics devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Pixel Format
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// Unknown
    Unknown = 0,
    /// RGB565 (16-bit)
    Rgb565 = 1,
    /// RGB888 (24-bit)
    Rgb888 = 2,
    /// XRGB8888 (32-bit)
    Xrgb8888 = 3,
    /// ARGB8888 (32-bit with alpha)
    Argb8888 = 4,
    /// BGR565 (16-bit)
    Bgr565 = 5,
    /// BGR888 (24-bit)
    Bgr888 = 6,
    /// XBGR8888 (32-bit)
    Xbgr8888 = 7,
    /// ABGR8888 (32-bit with alpha)
    Abgr8888 = 8,
    /// YUV422
    Yuv422 = 9,
    /// YUV420
    Yuv420 = 10,
    /// NV12
    Nv12 = 11,
    /// NV21
    Nv21 = 12,
}

impl PixelFormat {
    /// Get bytes per pixel
    pub fn bytes_per_pixel(&self) -> u8 {
        match self {
            PixelFormat::Rgb565 | PixelFormat::Bgr565 => 2,
            PixelFormat::Rgb888 | PixelFormat::Bgr888 => 3,
            PixelFormat::Xrgb8888
            | PixelFormat::Argb8888
            | PixelFormat::Xbgr8888
            | PixelFormat::Abgr8888 => 4,
            _ => 0,
        }
    }

    /// Get bits per pixel
    pub fn bits_per_pixel(&self) -> u8 {
        self.bytes_per_pixel() * 8
    }
}

/// Display Mode
#[repr(C)]
pub struct DisplayMode {
    /// Name
    pub name: [u8; 32],
    /// Horizontal resolution
    pub hdisplay: u16,
    /// Horizontal sync start
    pub hsync_start: u16,
    /// Horizontal sync end
    pub hsync_end: u16,
    /// Horizontal total
    pub htotal: u16,
    /// Vertical resolution
    pub vdisplay: u16,
    /// Vertical sync start
    pub vsync_start: u16,
    /// Vertical sync end
    pub vsync_end: u16,
    /// Vertical total
    pub vtotal: u16,
    /// Pixel clock (kHz)
    pub clock: u32,
    /// Refresh rate (Hz)
    pub vrefresh: u8,
    /// Flags
    pub flags: ModeFlags,
}

/// Mode Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ModeFlags: u32 {
        /// PHSYNC - positive horizontal sync
        const PHSYNC = 1 << 0;
        /// NHSYNC - negative horizontal sync
        const NHSYNC = 1 << 1;
        /// PVSYNC - positive vertical sync
        const PVSYNC = 1 << 2;
        /// NVSYNC - negative vertical sync
        const NVSYNC = 1 << 3;
        /// Interlace
        const INTERLACE = 1 << 4;
        /// Double scan
        const DBLSCAN = 1 << 5;
        /// Clock divided by 2
        const CLKDIV2 = 1 << 6;
        /// Preferred mode
        const PREFERRED = 1 << 7;
    }
}

/// Display Info
#[repr(C)]
pub struct DisplayInfo {
    /// Width (mm)
    pub width_mm: u16,
    /// Height (mm)
    pub height_mm: u16,
    /// Minimum width
    pub min_width: u16,
    /// Maximum width
    pub max_width: u16,
    /// Minimum height
    pub min_height: u16,
    /// Maximum height
    pub max_height: u16,
    /// Number of modes
    pub num_modes: u8,
    /// Current mode index
    pub current_mode: u8,
    /// Pixel format
    pub pixel_format: PixelFormat,
    /// Connector type
    pub connector_type: ConnectorType,
    /// Connected
    pub connected: bool,
}

/// Connector Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorType {
    /// Unknown
    Unknown = 0,
    /// VGA
    Vga = 1,
    /// DVI-I
    DviI = 2,
    /// DVI-D
    DviD = 3,
    /// DVI-A
    DviA = 4,
    /// Composite
    Composite = 5,
    /// S-Video
    SVideo = 6,
    /// LVDS
    Lvds = 7,
    /// Component
    Component = 8,
    /// DisplayPort
    DisplayPort = 9,
    /// HDMI-A
    HdmiA = 10,
    /// HDMI-B
    HdmiB = 11,
    /// eDP
    Edp = 12,
    /// Virtual
    Virtual = 13,
    /// DSI
    Dsi = 14,
    /// DPI
    Dpi = 15,
    /// MIPI DBI
    Dbi = 16,
    /// SPI
    Spi = 17,
    /// USB
    Usb = 18,
}

/// Display Buffer
#[repr(C)]
pub struct DisplayBuffer {
    /// Buffer ID
    pub id: u32,
    /// Physical address
    pub paddr: u64,
    /// Virtual address
    pub vaddr: *mut u8,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Pitch (bytes per line)
    pub pitch: u32,
    /// Pixel format
    pub format: PixelFormat,
    /// Size in bytes
    pub size: usize,
    /// Flags
    pub flags: BufferFlags,
}

/// Buffer Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct BufferFlags: u32 {
        /// Primary buffer
        const PRIMARY = 1 << 0;
        /// Scanout buffer
        const SCANOUT = 1 << 1;
        /// Render target
        const RENDER = 1 << 2;
        /// CPU accessible
        const CPU_ACCESS = 1 << 3;
        /// DMA buffer
        const DMA = 1 << 4;
        /// Flipped vertically
        const FLIP_V = 1 << 5;
        /// Flipped horizontally
        const FLIP_H = 1 << 6;
    }
}

/// Display State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayState {
    /// Off
    Off = 0,
    /// On
    On = 1,
    /// Suspended
    Suspended = 2,
    /// Standby
    Standby = 3,
}

/// Display Operations
pub struct DisplayDeviceOps {
    // Initialization
    /// Initialize display
    pub init: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Deinitialize display
    pub deinit: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    // Mode management
    /// Get display info
    pub get_info: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut DisplayInfo) -> i32>,
    /// Get modes
    pub get_modes:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut DisplayMode, usize) -> i32>,
    /// Set mode
    pub set_mode: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8) -> i32>,

    // Buffer management
    /// Create buffer
    pub create_buffer: Option<
        unsafe extern "C" fn(
            *mut core::ffi::c_void,
            u32,
            u32,
            PixelFormat,
            *mut DisplayBuffer,
        ) -> i32,
    >,
    /// Destroy buffer
    pub destroy_buffer: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Get primary buffer
    pub get_primary_buffer:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut DisplayBuffer) -> i32>,

    // Display control
    /// Set power state
    pub set_power: Option<unsafe extern "C" fn(*mut core::ffi::c_void, DisplayState) -> i32>,
    /// Get power state
    pub get_power: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> DisplayState>,
    /// Set brightness
    pub set_brightness: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8) -> i32>,
    /// Get brightness
    pub get_brightness: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u8>,

    // Rendering
    /// Present buffer
    pub present: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const DisplayBuffer) -> i32>,
    /// Flip buffers
    pub flip: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Wait for VBlank
    pub wait_vblank: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    // Update
    /// Update region
    pub update_region:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, u32, u32, u32) -> i32>,
    /// Flush
    pub flush: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// Display ioctl commands
pub mod display_ioctl {
    /// Get display info
    pub const GET_INFO: u32 = 0xE001;
    /// Get modes
    pub const GET_MODES: u32 = 0xE002;
    /// Set mode
    pub const SET_MODE: u32 = 0xE003;
    /// Create buffer
    pub const CREATE_BUFFER: u32 = 0xE004;
    /// Destroy buffer
    pub const DESTROY_BUFFER: u32 = 0xE005;
    /// Get primary buffer
    pub const GET_PRIMARY: u32 = 0xE006;
    /// Set power
    pub const SET_POWER: u32 = 0xE007;
    /// Get power
    pub const GET_POWER: u32 = 0xE008;
    /// Set brightness
    pub const SET_BRIGHTNESS: u32 = 0xE009;
    /// Get brightness
    pub const GET_BRIGHTNESS: u32 = 0xE00A;
    /// Present
    pub const PRESENT: u32 = 0xE00B;
    /// Flip
    pub const FLIP: u32 = 0xE00C;
    /// Wait VBlank
    pub const WAIT_VBLANK: u32 = 0xE00D;
    /// Update region
    pub const UPDATE_REGION: u32 = 0xE00E;
    /// Flush
    pub const FLUSH: u32 = 0xE00F;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_format_bytes() {
        assert_eq!(PixelFormat::Rgb565.bytes_per_pixel(), 2);
        assert_eq!(PixelFormat::Rgb888.bytes_per_pixel(), 3);
        assert_eq!(PixelFormat::Argb8888.bytes_per_pixel(), 4);
    }

    #[test]
    fn test_pixel_format_bits() {
        assert_eq!(PixelFormat::Rgb565.bits_per_pixel(), 16);
        assert_eq!(PixelFormat::Argb8888.bits_per_pixel(), 32);
    }

    #[test]
    fn test_connector_type() {
        assert_eq!(ConnectorType::HdmiA as i32, 10);
        assert_eq!(ConnectorType::Dsi as i32, 14);
    }

    #[test]
    fn test_mode_flags() {
        let flags = ModeFlags::PHSYNC | ModeFlags::PVSYNC | ModeFlags::PREFERRED;
        assert!(flags.contains(ModeFlags::PHSYNC));
        assert!(flags.contains(ModeFlags::PREFERRED));
    }
}
