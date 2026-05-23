/*
 * Nuva OS - SystemService - OpenGL - Command Buffer
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

//! OpenGL command definitions and batch submission.

use alloc::vec::Vec;

use super::error::{ClearMask, GlError, IndexType, PrimitiveMode, Rgba, UniformValue};

/// OpenGL rendering command
#[derive(Debug, Clone, PartialEq)]
pub enum GlCommand {
    /// Clear framebuffer buffers
    Clear {
        /// Which buffers to clear
        mask: ClearMask,
        /// Clear color value
        color: Rgba,
        /// Clear depth value
        depth: f32,
        /// Clear stencil value
        stencil: i32,
    },
    /// Draw primitives from array data
    DrawArrays {
        /// Primitive topology
        mode: PrimitiveMode,
        /// First vertex index
        first: u32,
        /// Number of vertices
        count: u32,
    },
    /// Draw primitives from indexed array data
    DrawElements {
        /// Primitive topology
        mode: PrimitiveMode,
        /// Number of elements
        count: u32,
        /// Index type
        index_type: IndexType,
        /// Byte offset into index buffer
        offset: u32,
    },
    /// Bind a vertex buffer to a binding point
    BindVertexBuffer {
        /// Binding point index
        binding: u32,
        /// Buffer resource ID
        buffer_id: u64,
        /// Byte offset
        offset: u32,
        /// Stride in bytes
        stride: u32,
    },
    /// Activate a shader program
    UseProgram {
        /// Program resource ID
        program_id: u64,
    },
    /// Set a uniform variable value
    Uniform {
        /// Uniform location index
        location: u32,
        /// Uniform value
        value: UniformValue,
    },
    /// Bind a texture to a texture unit
    BindTexture {
        /// Texture unit index
        unit: u32,
        /// Texture resource ID
        texture_id: u64,
    },
    /// Copy a region between framebuffers
    BlitFramebuffer {
        /// Source region (x0, y0, x1, y1)
        src_x0: i32,
        src_y0: i32,
        src_x1: i32,
        src_y1: i32,
        /// Destination region (x0, y0, x1, y1)
        dst_x0: i32,
        dst_y0: i32,
        dst_x1: i32,
        dst_y1: i32,
        /// Mask bits (color/depth/stencil)
        mask: ClearMask,
        /// Linear filtering if true, nearest if false
        linear: bool,
    },
}

/// A batch of OpenGL commands for atomic submission
#[derive(Debug, Clone)]
pub struct GlCommandBatch {
    /// Commands in this batch
    commands: Vec<GlCommand>,
}

impl GlCommandBatch {
    /// Create an empty command batch
    pub fn new() -> Self {
        GlCommandBatch {
            commands: Vec::new(),
        }
    }

    /// Create a command batch with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        GlCommandBatch {
            commands: Vec::with_capacity(capacity),
        }
    }

    /// Push a command into the batch
    pub fn push(&mut self, cmd: GlCommand) {
        self.commands.push(cmd);
    }

    /// Get the number of commands in the batch
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get an iterator over the commands
    pub fn iter(&self) -> core::slice::Iter<'_, GlCommand> {
        self.commands.iter()
    }

    /// Submit the command batch for execution.
    /// Returns the number of commands submitted.
    pub fn submit(&self) -> Result<usize, GlError> {
        if self.commands.is_empty() {
            return Ok(0);
        }
        // In a full implementation, this sends the command batch
        // to the GPU command queue via the HAL GPU interface.
        log_debug!("Submitting GL command batch with {} commands", self.commands.len());
        Ok(self.commands.len())
    }

    /// Clear all commands from the batch
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

impl Default for GlCommandBatch {
    fn default() -> Self {
        Self::new()
    }
}
