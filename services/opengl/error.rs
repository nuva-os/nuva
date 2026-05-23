/*
 * Nuva OS - SystemService - OpenGL - Error Model
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

//! OpenGL service specific error types and rendering data types.

use core::fmt;

/// OpenGL service error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlError {
    /// OpenGL service not initialized
    NotInitialized = 0,
    /// Out of GPU memory
    OutOfMemory = 1,
    /// Invalid rendering context
    InvalidContext = 2,
    /// Invalid GPU resource handle
    InvalidResource = 3,
    /// Invalid GL command
    InvalidCommand = 4,
    /// GPU hardware error
    GpuError = 5,
    /// Software fallback is active
    FallbackActive = 6,
}

impl fmt::Display for GlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlError::NotInitialized => write!(f, "OpenGL not initialized"),
            GlError::OutOfMemory => write!(f, "Out of GPU memory"),
            GlError::InvalidContext => write!(f, "Invalid GL context"),
            GlError::InvalidResource => write!(f, "Invalid GL resource"),
            GlError::InvalidCommand => write!(f, "Invalid GL command"),
            GlError::GpuError => write!(f, "GPU hardware error"),
            GlError::FallbackActive => write!(f, "Software fallback active"),
        }
    }
}

/// RGBA color value
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

impl Rgba {
    /// Transparent black
    pub const TRANSPARENT: Rgba = Rgba(0.0, 0.0, 0.0, 0.0);
    /// Opaque black
    pub const BLACK: Rgba = Rgba(0.0, 0.0, 0.0, 1.0);
    /// Opaque white
    pub const WHITE: Rgba = Rgba(1.0, 1.0, 1.0, 1.0);
}

/// Framebuffer clear mask bits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearMask(pub u32);

impl ClearMask {
    /// Clear color buffer
    pub const COLOR: ClearMask = ClearMask(1);
    /// Clear depth buffer
    pub const DEPTH: ClearMask = ClearMask(2);
    /// Clear stencil buffer
    pub const STENCIL: ClearMask = ClearMask(4);
    /// Clear all buffers
    pub const ALL: ClearMask = ClearMask(7);
}

/// Primitive topology for draw calls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveMode {
    /// Points
    Points = 0,
    /// Line segments
    Lines = 1,
    /// Connected line strip
    LineStrip = 2,
    /// Line loop
    LineLoop = 3,
    /// Triangles
    Triangles = 4,
    /// Connected triangle strip
    TriangleStrip = 5,
    /// Triangle fan
    TriangleFan = 6,
}

/// Index buffer element type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// Unsigned 8-bit indices
    U8 = 0,
    /// Unsigned 16-bit indices
    U16 = 1,
    /// Unsigned 32-bit indices
    U32 = 2,
}

/// Uniform variable value
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UniformValue {
    /// Single float
    Float(f32),
    /// 2-component float vector
    Vec2(f32, f32),
    /// 3-component float vector
    Vec3(f32, f32, f32),
    /// 4-component float vector
    Vec4(f32, f32, f32, f32),
    /// 4x4 float matrix (row-major, 16 elements)
    Mat4([[f32; 4]; 4]),
    /// Signed integer
    Int(i32),
    /// Unsigned integer
    Uint(u32),
}
