/*
 * Nuva OS - Kernel - Driver - Class - Camera
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
 * Nuva OS - Kernel - Camera Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for camera/video capture devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Camera Pixel Format (FourCC)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPixelFormat {
    /// YUYV 4:2:2
    Yuyv = 0x56595559,
    /// UYVY 4:2:2
    Uyvy = 0x56595556,
    /// NV12
    Nv12 = 0x3231564E,
    /// NV21
    Nv21 = 0x3132564E,
    /// YUV420
    Yu12 = 0x32315559,
    /// YVU420
    Yv12 = 0x32315659,
    /// MJPEG
    Mjpeg = 0x47504A4D,
    /// RGB24
    Rgb24 = 0x33424752,
    /// BGR24
    Bgr24 = 0x33524742,
    /// RGB32
    Rgb32 = 0x34424752,
    /// BGR32
    Bgr32 = 0x34524742,
    /// GREY (8-bit greyscale)
    Grey = 0x59455247,
    /// Y16 (16-bit greyscale)
    Y16 = 0x20363159,
    /// SRGGB8 (8-bit Bayer)
    Srggb8 = 0x38424752,
    /// SGRBG8
    Sgrbg8 = 0x38424747,
    /// SBGGR8
    Sbggr8 = 0x38424742,
}

/// Camera Format Description
#[repr(C)]
pub struct CameraFormatDesc {
    /// Pixel format
    pub pixel_format: CameraPixelFormat,
    /// Format name
    pub name: [u8; 32],
    /// Bits per pixel
    pub bits_per_pixel: u8,
    /// Color space
    pub color_space: ColorSpace,
    /// Supported resolutions count
    pub num_resolutions: u8,
}

/// Color Space
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// Unknown
    Unknown = 0,
    /// SMPTE 170M (BT.601)
    Smpte170m = 1,
    /// SMPTE 240M
    Smpte240m = 2,
    /// Rec. 709
    Rec709 = 3,
    /// BT.878
    Bt878 = 4,
    /// ITU-R 601
    ItuR601 = 5,
    /// ITU-R 709
    ItuR709 = 6,
    /// JPEG (JFIF)
    Jpeg = 7,
    /// sRGB
    Srgb = 8,
    /// opRGB
    Oprgb = 9,
    /// BT.2020
    Bt2020 = 10,
    /// Raw
    Raw = 11,
}

/// Camera Resolution
#[repr(C)]
pub struct CameraResolution {
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Frame intervals count
    pub num_intervals: u8,
    /// Flags
    pub flags: ResolutionFlags,
}

/// Resolution Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ResolutionFlags: u32 {
        /// Continuous frame interval
        const CONTINUOUS = 1 << 0;
        /// Stepwise frame interval
        const STEPWISE = 1 << 1;
    }
}

/// Frame Interval
#[repr(C)]
pub struct FrameInterval {
    /// Numerator
    pub numerator: u32,
    /// Denominator
    pub denominator: u32,
}

impl FrameInterval {
    /// Create from FPS
    pub fn from_fps(fps: u32) -> Self {
        FrameInterval {
            numerator: 1,
            denominator: fps,
        }
    }

    /// Get FPS
    pub fn fps(&self) -> u32 {
        if self.numerator == 0 {
            return 0;
        }
        self.denominator / self.numerator
    }
}

/// Camera Buffer
#[repr(C)]
pub struct CameraBuffer {
    /// Buffer index
    pub index: u32,
    /// Buffer type
    pub buf_type: BufferType,
    /// Bytes used
    pub bytes_used: u32,
    /// Buffer flags
    pub flags: BufferFlags,
    /// Memory type
    pub memory: MemoryType,
    /// Plane offset
    pub offset: u32,
    /// User pointer
    pub userptr: u64,
    /// Buffer address (for DMA)
    pub paddr: u64,
    /// Length
    pub length: u32,
    /// Timestamp
    pub timestamp: u64,
    /// Sequence number
    pub sequence: u32,
}

/// Buffer Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Video capture
    VideoCapture = 1,
    /// Video output
    VideoOutput = 2,
    /// Video capture MPLANE
    VideoCaptureMplane = 9,
    /// Video output MPLANE
    VideoOutputMplane = 10,
}

/// Buffer Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct BufferFlags: u32 {
        /// Mapped
        const MAPPED = 1 << 0;
        /// Queued
        const QUEUED = 1 << 1;
        /// Done
        const DONE = 1 << 2;
        /// Key frame
        const KEYFRAME = 1 << 3;
        /// P frame
        const PFRAME = 1 << 4;
        /// B frame
        const BFRAME = 1 << 5;
        /// Error
        const ERROR = 1 << 6;
        /// Timecode
        const TIMECODE = 1 << 7;
        /// Prepared
        const PREPARED = 1 << 8;
        /// No cache invalidate
        const NO_CACHE_INVALIDATE = 1 << 9;
        /// No cache clean
        const NO_CACHE_CLEAN = 1 << 10;
    }
}

/// Memory Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// MMAP
    Mmap = 1,
    /// User pointer
    UserPtr = 2,
    /// Overlay
    Overlay = 3,
    /// DMA buffer
    Dmabuf = 4,
}

/// Camera Control ID
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraControlId {
    /// Brightness
    Brightness = 0x00980900,
    /// Contrast
    Contrast = 0x00980901,
    /// Saturation
    Saturation = 0x00980902,
    /// Hue
    Hue = 0x00980903,
    /// Auto white balance
    AutoWhiteBalance = 0x0098090c,
    /// Exposure
    Exposure = 0x00980910,
    /// Auto exposure
    AutoExposure = 0x009a0901,
    /// Gain
    Gain = 0x00980913,
    /// Power line frequency
    PowerLineFrequency = 0x00980917,
    /// H flip
    HFlip = 0x00980914,
    /// V flip
    VFlip = 0x00980915,
    /// Rotation
    Rotation = 0x00980922,
    /// Color effect
    ColorEffect = 0x0098091f,
    /// Zoom absolute
    ZoomAbsolute = 0x009a090d,
    /// Focus absolute
    FocusAbsolute = 0x009a090c,
    /// Auto focus
    AutoFocus = 0x009a0902,
}

/// Camera Control
#[repr(C)]
pub struct CameraControl {
    /// Control ID
    pub id: CameraControlId,
    /// Control type
    pub ctrl_type: ControlType,
    /// Name
    pub name: [u8; 32],
    /// Minimum value
    pub min: i32,
    /// Maximum value
    pub max: i32,
    /// Step
    pub step: i32,
    /// Default value
    pub default: i32,
    /// Current value
    pub value: i32,
    /// Flags
    pub flags: ControlFlags,
}

/// Control Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlType {
    /// Integer
    Integer = 1,
    /// Boolean
    Boolean = 2,
    /// Menu
    Menu = 3,
    /// Integer menu
    IntegerMenu = 4,
    /// Button
    Button = 5,
    /// Integer64
    Integer64 = 6,
    /// String
    String = 7,
    /// Compound
    Compound = 8,
}

/// Control Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ControlFlags: u32 {
        /// Disabled
        const DISABLED = 1 << 0;
        /// Read only
        const READ_ONLY = 1 << 1;
        /// Update
        const UPDATE = 1 << 2;
        /// Inactive
        const INACTIVE = 1 << 3;
        /// Volatile
        const VOLATILE = 1 << 4;
        /// Write only
        const WRITE_ONLY = 1 << 5;
    }
}

/// Camera Stream Config
#[repr(C)]
pub struct CameraStreamConfig {
    /// Pixel format
    pub pixel_format: CameraPixelFormat,
    /// Width
    pub width: u32,
    /// Height: u32,
    pub height: u32,
    /// Frame interval
    pub interval: FrameInterval,
    /// Buffer count
    pub num_buffers: u8,
    /// Memory type
    pub memory: MemoryType,
}

/// Camera Device Operations
pub struct CameraDeviceOps {
    // Device control
    /// Open
    pub open: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Close
    pub close: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    // Format and resolution
    /// Get format
    pub get_format:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut CameraFormatDesc) -> i32>,
    /// Set format
    pub set_format: Option<unsafe extern "C" fn(*mut core::ffi::c_void, CameraPixelFormat) -> i32>,
    /// Get resolutions
    pub get_resolutions:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut CameraResolution, usize) -> i32>,
    /// Set resolution
    pub set_resolution: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, u32) -> i32>,
    /// Get frame interval
    pub get_frame_interval:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut FrameInterval) -> i32>,
    /// Set frame interval
    pub set_frame_interval:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const FrameInterval) -> i32>,

    // Buffer management
    /// Request buffers
    pub reqbufs: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, MemoryType) -> i32>,
    /// Query buffer
    pub querybuf:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, u32, *mut CameraBuffer) -> i32>,
    /// Queue buffer
    pub qbuf: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut CameraBuffer) -> i32>,
    /// Dequeue buffer
    pub dqbuf: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut CameraBuffer) -> i32>,

    // Stream control
    /// Stream on
    pub streamon: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Stream off
    pub streamoff: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    // Controls
    /// Query control
    pub queryctrl: Option<
        unsafe extern "C" fn(*const core::ffi::c_void, CameraControlId, *mut CameraControl) -> i32,
    >,
    /// Get control
    pub g_ctrl: Option<unsafe extern "C" fn(*const core::ffi::c_void, CameraControlId) -> i32>,
    /// Set control
    pub s_ctrl: Option<unsafe extern "C" fn(*mut core::ffi::c_void, CameraControlId, i32) -> i32>,
}

/// Camera ioctl commands
pub mod camera_ioctl {
    /// Get format
    pub const GET_FORMAT: u32 = 0x2001;
    /// Set format
    pub const SET_FORMAT: u32 = 0x2002;
    /// Get resolutions
    pub const GET_RESOLUTIONS: u32 = 0x2003;
    /// Set resolution
    pub const SET_RESOLUTION: u32 = 0x2004;
    /// Get frame interval
    pub const GET_FRAME_INTERVAL: u32 = 0x2005;
    /// Set frame interval
    pub const SET_FRAME_INTERVAL: u32 = 0x2006;
    /// Request buffers
    pub const REQBUFS: u32 = 0x2007;
    /// Query buffer
    pub const QUERYBUF: u32 = 0x2008;
    /// Queue buffer
    pub const QBUF: u32 = 0x2009;
    /// Dequeue buffer
    pub const DQBUF: u32 = 0x200A;
    /// Stream on
    pub const STREAMON: u32 = 0x200B;
    /// Stream off
    pub const STREAMOFF: u32 = 0x200C;
    /// Query control
    pub const QUERYCTRL: u32 = 0x200D;
    /// Get control
    pub const G_CTRL: u32 = 0x200E;
    /// Set control
    pub const S_CTRL: u32 = 0x200F;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_interval() {
        let interval = FrameInterval::from_fps(30);
        assert_eq!(interval.numerator, 1);
        assert_eq!(interval.denominator, 30);
        assert_eq!(interval.fps(), 30);
    }

    #[test]
    fn test_buffer_type() {
        assert_eq!(BufferType::VideoCapture as i32, 1);
        assert_eq!(BufferType::VideoOutput as i32, 2);
    }

    #[test]
    fn test_memory_type() {
        assert_eq!(MemoryType::Mmap as i32, 1);
        assert_eq!(MemoryType::Dmabuf as i32, 4);
    }
}
