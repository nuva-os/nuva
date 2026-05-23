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

//! Nuva OS Graphics Rendering Engine
//!
//! 2D/3D graphics rendering capabilities.

use core::sync::atomic::{AtomicU32, Ordering};
use alloc::vec::Vec;

// ============================================================================
// Graphics Type Definitions
// ============================================================================

/// Pixel format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB565 (16-bit)
    Rgb565 = 0,
    /// RGB888 (24-bit)
    Rgb888 = 1,
    /// RGBA8888 (32-bit)
    Rgba8888 = 2,
    /// BGRA8888 (32-bit)
    Bgra8888 = 3,
}

/// Graphics drawing operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsOp {
    /// Draw a point
    DrawPoint = 0,
    /// Draw a line
    DrawLine = 1,
    /// Draw a rectangle outline
    DrawRect = 2,
    /// Draw a circle outline
    DrawCircle = 3,
    /// Draw text
    DrawText = 4,
    /// Draw an image
    DrawImage = 5,
    /// Fill a region
    Fill = 6,
    /// Blend operation
    Blend = 7,
}

// ============================================================================
// Color Structure
// ============================================================================

/// RGBA color value.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn black() -> Self {
        Self::from_rgb(0, 0, 0)
    }

    pub const fn white() -> Self {
        Self::from_rgb(255, 255, 255)
    }

    pub const fn red() -> Self {
        Self::from_rgb(255, 0, 0)
    }

    pub const fn green() -> Self {
        Self::from_rgb(0, 255, 0)
    }

    pub const fn blue() -> Self {
        Self::from_rgb(0, 0, 255)
    }
}

// ============================================================================
// Point and Rectangle
// ============================================================================

/// A point in 2D space.
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A rectangle in 2D space.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

// ============================================================================
// Graphics Context
// ============================================================================

/// A software framebuffer-based graphics context for 2D rendering.
pub struct GraphicsContext {
    /// Backing framebuffer
    framebuffer: Vec<u8>,
    /// Framebuffer width (pixels)
    width: u32,
    /// Framebuffer height (pixels)
    height: u32,
    /// Pixel format of the framebuffer
    pixel_format: PixelFormat,
    /// Current drawing color
    current_color: Color,
    /// Clipping region
    clip_rect: Rect,
}

impl GraphicsContext {
    /// Create a new graphics context with the given dimensions and pixel format.
    pub fn new(width: u32, height: u32, pixel_format: PixelFormat) -> Self {
        let bytes_per_pixel = match pixel_format {
            PixelFormat::Rgb565 => 2,
            PixelFormat::Rgb888 => 3,
            PixelFormat::Rgba8888 | PixelFormat::Bgra8888 => 4,
        };

        let buffer_size = (width * height * bytes_per_pixel) as usize;

        Self {
            framebuffer: vec![0; buffer_size],
            width,
            height,
            pixel_format,
            current_color: Color::black(),
            clip_rect: Rect::new(0, 0, width, height),
        }
    }

    /// Set the current drawing color.
    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    /// Draw a single point at the given coordinates.
    pub fn draw_point(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let offset = self.get_pixel_offset(x as u32, y as u32);
        self.set_pixel(offset, &self.current_color);
    }

    /// Draw a line using Bresenham's algorithm.
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1;
        let mut y = y1;

        loop {
            self.draw_point(x, y);

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw an outlined rectangle.
    pub fn draw_rect(&mut self, x: i32, y: i32, width: u32, height: u32) {
        // Top edge
        self.draw_line(x, y, x + width as i32 - 1, y);
        // Bottom edge
        self.draw_line(x, y + height as i32 - 1, x + width as i32 - 1, y + height as i32 - 1);
        // Left edge
        self.draw_line(x, y, x, y + height as i32 - 1);
        // Right edge
        self.draw_line(x + width as i32 - 1, y, x + width as i32 - 1, y + height as i32 - 1);
    }

    /// Fill a solid rectangle.
    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32) {
        for py in y..(y + height as i32) {
            self.draw_line(x, py, x + width as i32 - 1, py);
        }
    }

    /// Draw an outlined circle using the midpoint circle algorithm.
    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: u32) {
        let r = radius as i32;
        let mut x = r;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.draw_point(cx + x, cy + y);
            self.draw_point(cx + y, cy + x);
            self.draw_point(cx - y, cy + x);
            self.draw_point(cx - x, cy + y);
            self.draw_point(cx - x, cy - y);
            self.draw_point(cx - y, cy - x);
            self.draw_point(cx + y, cy - x);
            self.draw_point(cx + x, cy - y);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    /// Fill a solid circle.
    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: u32) {
        let r = radius as i32;

        for y in -r..=r {
            let width = (r * r - y * y).sqrt();
            self.draw_line(cx - width, cy + y, cx + width, cy + y);
        }
    }

    /// Clear the entire framebuffer to a solid color.
    pub fn clear(&mut self, color: Color) {
        self.set_color(color);
        self.fill_rect(0, 0, self.width, self.height);
    }

    /// Get a reference to the raw framebuffer data.
    pub fn get_framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Calculate the byte offset in the framebuffer for the given pixel coordinates.
    fn get_pixel_offset(&self, x: u32, y: u32) -> usize {
        let bytes_per_pixel = match self.pixel_format {
            PixelFormat::Rgb565 => 2,
            PixelFormat::Rgb888 => 3,
            PixelFormat::Rgba8888 | PixelFormat::Bgra8888 => 4,
        };

        (y * self.width + x) as usize * bytes_per_pixel
    }

    /// Write a color value into the framebuffer at the given byte offset.
    fn set_pixel(&mut self, offset: usize, color: &Color) {
        match self.pixel_format {
            PixelFormat::Rgb565 => {
                let r = (color.r >> 3) as u16;
                let g = (color.g >> 2) as u16;
                let b = (color.b >> 3) as u16;
                let pixel = (r << 11) | (g << 5) | b;
                self.framebuffer[offset] = (pixel & 0xFF) as u8;
                self.framebuffer[offset + 1] = ((pixel >> 8) & 0xFF) as u8;
            }
            PixelFormat::Rgb888 => {
                self.framebuffer[offset] = color.r;
                self.framebuffer[offset + 1] = color.g;
                self.framebuffer[offset + 2] = color.b;
            }
            PixelFormat::Rgba8888 => {
                self.framebuffer[offset] = color.r;
                self.framebuffer[offset + 1] = color.g;
                self.framebuffer[offset + 2] = color.b;
                self.framebuffer[offset + 3] = color.a;
            }
            PixelFormat::Bgra8888 => {
                self.framebuffer[offset] = color.b;
                self.framebuffer[offset + 1] = color.g;
                self.framebuffer[offset + 2] = color.r;
                self.framebuffer[offset + 3] = color.a;
            }
        }
    }
}

// ============================================================================
// 2D Graphics Engine
// ============================================================================

/// High-level 2D graphics engine wrapping a graphics context.
pub struct Graphics2D {
    /// Main rendering context
    context: GraphicsContext,
    /// Render frame counter
    render_count: AtomicU32,
}

impl Graphics2D {
    /// Create a new 2D graphics engine.
    pub fn new(width: u32, height: u32, pixel_format: PixelFormat) -> Self {
        Self {
            context: GraphicsContext::new(width, height, pixel_format),
            render_count: AtomicU32::new(0),
        }
    }

    /// Get a mutable reference to the underlying graphics context.
    pub fn get_context(&mut self) -> &mut GraphicsContext {
        &mut self.context
    }

    /// Render the current frame.
    pub fn render(&mut self) {
        // TODO: Implement render
        self.render_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the number of frames rendered so far.
    pub fn get_render_count(&self) -> u32 {
        self.render_count.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_context() {
        let mut ctx = GraphicsContext::new(800, 600, PixelFormat::Rgba8888);

        // Clear screen to white
        ctx.clear(Color::white());

        // Draw a red rectangle
        ctx.set_color(Color::red());
        ctx.fill_rect(100, 100, 200, 150);

        // Draw a blue circle
        ctx.set_color(Color::blue());
        ctx.fill_circle(400, 300, 50);

        // Draw a green diagonal line
        ctx.set_color(Color::green());
        ctx.draw_line(0, 0, 800, 600);
    }

    #[test]
    fn test_graphics_2d() {
        let mut gfx = Graphics2D::new(800, 600, PixelFormat::Rgba8888);

        gfx.render();
        assert_eq!(gfx.get_render_count(), 1);
    }
}
