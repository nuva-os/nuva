/*
 * Nuva OS - SystemService - OpenGL - Software Renderer
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

//! Software rendering fallback path for when GPU hardware is unavailable.
//! Implements basic Clear and DrawArrays operations via CPU.

use super::command::GlCommand;
use super::error::{ClearMask, GlError, Rgba};

/// Software renderer state
pub struct SoftwareRenderer {
    /// Current clear color
    clear_color: Rgba,
    /// Current clear depth
    clear_depth: f32,
    /// Current clear stencil
    clear_stencil: i32,
    /// Framebuffer width
    fb_width: u32,
    /// Framebuffer height
    fb_height: u32,
    /// Color buffer (RGBA, row-major)
    color_buffer: alloc::vec::Vec<u8>,
    /// Depth buffer
    depth_buffer: alloc::vec::Vec<f32>,
    /// Whether the renderer is initialized
    initialized: bool,
}

impl SoftwareRenderer {
    /// Create a new software renderer with the given framebuffer dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width as usize) * (height as usize);
        SoftwareRenderer {
            clear_color: Rgba::BLACK,
            clear_depth: 1.0,
            clear_stencil: 0,
            fb_width: width,
            fb_height: height,
            color_buffer: alloc::vec![0u8; pixel_count * 4],
            depth_buffer: alloc::vec![1.0f32; pixel_count],
            initialized: true,
        }
    }

    /// Check if the software renderer is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get framebuffer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.fb_width, self.fb_height)
    }

    /// Execute a single GL command via software rendering
    pub fn execute(&mut self, cmd: &GlCommand) -> Result<(), GlError> {
        if !self.initialized {
            return Err(GlError::NotInitialized);
        }

        match cmd {
            GlCommand::Clear { mask, color, depth, stencil } => {
                self.cmd_clear(*mask, *color, *depth, *stencil)
            }
            GlCommand::DrawArrays { mode, first, count } => {
                self.cmd_draw_arrays(*mode, *first, *count)
            }
            GlCommand::DrawElements { mode, count, index_type, offset } => {
                self.cmd_draw_elements(*mode, *count, *index_type, *offset)
            }
            GlCommand::BindVertexBuffer { binding, buffer_id, offset, stride } => {
                log_debug!(
                    "SW: BindVertexBuffer binding={} buf={} off={} stride={}",
                    binding,
                    buffer_id,
                    offset,
                    stride
                );
                Ok(())
            }
            GlCommand::UseProgram { program_id } => {
                log_debug!("SW: UseProgram prog={}", program_id);
                Ok(())
            }
            GlCommand::Uniform { location, value } => {
                log_debug!("SW: Uniform loc={}", location);
                let _ = value;
                Ok(())
            }
            GlCommand::BindTexture { unit, texture_id } => {
                log_debug!("SW: BindTexture unit={} tex={}", unit, texture_id);
                Ok(())
            }
            GlCommand::BlitFramebuffer {
                src_x0, src_y0, src_x1, src_y1,
                dst_x0, dst_y0, dst_x1, dst_y1,
                mask, linear,
            } => {
                self.cmd_blit(
                    *src_x0, *src_y0, *src_x1, *src_y1,
                    *dst_x0, *dst_y0, *dst_x1, *dst_y1,
                    *mask, *linear,
                )
            }
        }
    }

    /// Software Clear implementation
    fn cmd_clear(&mut self, mask: ClearMask, color: Rgba, depth: f32, stencil: i32) -> Result<(), GlError> {
        self.clear_color = color;
        self.clear_depth = depth;
        self.clear_stencil = stencil;

        let pixel_count = (self.fb_width as usize) * (self.fb_height as usize);

        if mask.0 & ClearMask::COLOR.0 != 0 {
            let r = (color.0 * 255.0) as u8;
            let g = (color.1 * 255.0) as u8;
            let b = (color.2 * 255.0) as u8;
            let a = (color.3 * 255.0) as u8;
            for i in 0..pixel_count {
                let base = i * 4;
                // SAFETY: base + 3 is within bounds since color_buffer has pixel_count * 4 elements
                self.color_buffer[base] = r;
                self.color_buffer[base + 1] = g;
                self.color_buffer[base + 2] = b;
                self.color_buffer[base + 3] = a;
            }
        }

        if mask.0 & ClearMask::DEPTH.0 != 0 {
            for d in self.depth_buffer.iter_mut() {
                *d = depth;
            }
        }

        // Stencil clear is a no-op in this minimal software renderer
        let _ = stencil;

        log_debug!("SW: Clear mask={}", mask.0);
        Ok(())
    }

    /// Software DrawArrays implementation (minimal: no actual rasterization)
    fn cmd_draw_arrays(
        &mut self,
        mode: super::error::PrimitiveMode,
        first: u32,
        count: u32,
    ) -> Result<(), GlError> {
        // Minimal software renderer: no vertex fetch or rasterization.
        // In a full implementation, this would read vertex data and
        // rasterize primitives into the color buffer.
        log_debug!("SW: DrawArrays mode={:?} first={} count={}", mode, first, count);
        Ok(())
    }

    /// Software DrawElements implementation (minimal: no actual rasterization)
    fn cmd_draw_elements(
        &mut self,
        mode: super::error::PrimitiveMode,
        count: u32,
        index_type: super::error::IndexType,
        offset: u32,
    ) -> Result<(), GlError> {
        log_debug!(
            "SW: DrawElements mode={:?} count={} idx={:?} off={}",
            mode,
            count,
            index_type,
            offset
        );
        Ok(())
    }

    /// Software BlitFramebuffer implementation (minimal: no-op)
    fn cmd_blit(
        &mut self,
        src_x0: i32, src_y0: i32, src_x1: i32, src_y1: i32,
        dst_x0: i32, dst_y0: i32, dst_x1: i32, dst_y1: i32,
        mask: ClearMask,
        linear: bool,
    ) -> Result<(), GlError> {
        log_debug!(
            "SW: BlitFramebuffer src=({},{})-({},{}) dst=({},{})-({},{}) mask={} lin={}",
            src_x0, src_y0, src_x1, src_y1,
            dst_x0, dst_y0, dst_x1, dst_y1,
            mask.0, linear
        );
        Ok(())
    }

    /// Read a pixel from the color buffer at (x, y)
    pub fn read_pixel(&self, x: u32, y: u32) -> Option<Rgba> {
        if x >= self.fb_width || y >= self.fb_height {
            return None;
        }
        let idx = ((y as usize) * (self.fb_width as usize) + (x as usize)) * 4;
        if idx + 3 >= self.color_buffer.len() {
            return None;
        }
        // SAFETY: bounds checked above
        let r = self.color_buffer[idx] as f32 / 255.0;
        let g = self.color_buffer[idx + 1] as f32 / 255.0;
        let b = self.color_buffer[idx + 2] as f32 / 255.0;
        let a = self.color_buffer[idx + 3] as f32 / 255.0;
        Some(Rgba(r, g, b, a))
    }
}
