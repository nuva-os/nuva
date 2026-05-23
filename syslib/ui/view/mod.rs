/*
 * Nuva OS - SystemLibrary - Ui
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

//! NuvaUI Declarative UI Framework
//!
//! Inspired by SwiftUI design, declarative UI construction.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Initialize view system
pub fn init_view() {
    crate::log_info!("View system initialized");
}

/// View ID
pub type ViewId = u64;

/// Size
#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// Point
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Rectangle
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point { x, y },
            size: Size { width, height },
        }
    }

    pub fn width(&self) -> f32 {
        self.size.width
    }

    pub fn height(&self) -> f32 {
        self.size.height
    }

    pub fn min_x(&self) -> f32 {
        self.origin.x
    }

    pub fn min_y(&self) -> f32 {
        self.origin.y
    }

    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }

    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }
}

/// Color
#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as f32 / 255.0,
            g: ((hex >> 8) & 0xFF) as f32 / 255.0,
            b: (hex & 0xFF) as f32 / 255.0,
            a: 1.0,
        }
    }
}

/// Font
#[derive(Debug, Clone)]
pub struct Font {
    pub family: [u8; 32],
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
}

/// Font Weight
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

/// Font Style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            family: *b"System\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            size: 16.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        }
    }
}

/// View Protocol
pub trait View {
    /// Body type of the view
    type Body: View;

    /// Get the view's body
    fn body(&self) -> Self::Body;

    /// Layout
    fn layout(&self, constraints: &Constraints) -> Size {
        // Default implementation
        constraints.max
    }

    /// Render
    fn render(&self, context: &mut RenderContext);
}

/// Implement View for () as a leaf/empty view
impl View for () {
    type Body = ();
    fn body(&self) -> Self::Body {}
    fn render(&self, _context: &mut RenderContext) {}
}

/// Layout Constraints
#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min: Size,
    pub max: Size,
}

impl Constraints {
    pub fn new(min_width: f32, min_height: f32, max_width: f32, max_height: f32) -> Self {
        Self {
            min: Size { width: min_width, height: min_height },
            max: Size { width: max_width, height: max_height },
        }
    }

    pub fn tight(width: f32, height: f32) -> Self {
        Self {
            min: Size { width, height },
            max: Size { width, height },
        }
    }

    pub fn loose(max_width: f32, max_height: f32) -> Self {
        Self {
            min: Size { width: 0.0, height: 0.0 },
            max: Size { width: max_width, height: max_height },
        }
    }
}

/// Render Context
pub struct RenderContext {
    /// Current clipping region
    pub clip_rect: Rect,

    /// Current transform matrix
    pub transform: Transform,

    /// Current opacity
    pub alpha: f32,

    /// Draw command queue
    pub commands: [DrawCommand; 256],

    /// Command count
    pub num_commands: u32,
}

/// Transform Matrix (2D)
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub a: f32, pub b: f32,
    pub c: f32, pub d: f32,
    pub tx: f32, pub ty: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            a: 1.0, b: 0.0,
            c: 0.0, d: 1.0,
            tx: 0.0, ty: 0.0,
        }
    }
}

/// Draw Command
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Fill rectangle
    FillRect { rect: Rect, color: Color },
    /// Stroke rectangle border
    StrokeRect { rect: Rect, color: Color, width: f32 },
    /// Fill rounded rectangle
    FillRoundedRect { rect: Rect, radius: f32, color: Color },
    /// Draw text
    DrawText { text: [u8; 256], len: u32, pos: Point, font: Font, color: Color },
    /// Draw image
    DrawImage { image_id: u64, rect: Rect },
    /// Set clipping region
    SetClipRect { rect: Rect },
    /// Set transform
    SetTransform { transform: Transform },
}

/// Text View
pub struct Text {
    pub content: [u8; 256],
    pub len: u32,
    pub font: Font,
    pub color: Color,
}

impl Text {
    pub fn new(content: &str) -> Self {
        let mut buf = [0u8; 256];
        let bytes = content.as_bytes();
        let len = bytes.len().min(255);
        buf[..len].copy_from_slice(&bytes[..len]);

        Self {
            content: buf,
            len: len as u32,
            font: Font::default(),
            color: Color::BLACK,
        }
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font.size = size;
        self
    }
}

impl View for Text {
    type Body = ();

    fn body(&self) -> Self::Body {}

    fn layout(&self, constraints: &Constraints) -> Size {
        // Simplified: assume each character width equals font size
        let width = self.len as f32 * self.font.size * 0.6;
        let height = self.font.size * 1.2;

        Size {
            width: width.clamp(constraints.min.width, constraints.max.width),
            height: height.clamp(constraints.min.height, constraints.max.height),
        }
    }

    fn render(&self, context: &mut RenderContext) {
        if context.num_commands < 256 {
            context.commands[context.num_commands as usize] = DrawCommand::DrawText {
                text: self.content,
                len: self.len,
                pos: Point { x: 0.0, y: 0.0 },
                font: self.font.clone(),
                color: self.color,
            };
            context.num_commands += 1;
        }
    }
}

/// Rectangle View
pub struct Rectangle {
    pub rect: Rect,
    pub fill_color: Option<Color>,
    pub stroke_color: Option<Color>,
    pub stroke_width: f32,
    pub corner_radius: f32,
}

impl Rectangle {
    pub fn new() -> Self {
        Self {
            rect: Rect::default(),
            fill_color: None,
            stroke_color: None,
            stroke_width: 1.0,
            corner_radius: 0.0,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.stroke_color = Some(color);
        self.stroke_width = width;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn frame(mut self, width: f32, height: f32) -> Self {
        self.rect.size = Size { width, height };
        self
    }
}

impl View for Rectangle {
    type Body = ();

    fn body(&self) -> Self::Body {}

    fn layout(&self, constraints: &Constraints) -> Size {
        Size {
            width: self.rect.width().clamp(constraints.min.width, constraints.max.width),
            height: self.rect.height().clamp(constraints.min.height, constraints.max.height),
        }
    }

    fn render(&self, context: &mut RenderContext) {
        if context.num_commands >= 256 {
            return;
        }

        if self.corner_radius > 0.0 {
            if let Some(color) = self.fill_color {
                context.commands[context.num_commands as usize] = DrawCommand::FillRoundedRect {
                    rect: self.rect,
                    radius: self.corner_radius,
                    color,
                };
                context.num_commands += 1;
            }
        } else {
            if let Some(color) = self.fill_color {
                context.commands[context.num_commands as usize] = DrawCommand::FillRect {
                    rect: self.rect,
                    color,
                };
                context.num_commands += 1;
            }

            if let Some(color) = self.stroke_color {
                if context.num_commands < 256 {
                    context.commands[context.num_commands as usize] = DrawCommand::StrokeRect {
                        rect: self.rect,
                        color,
                        width: self.stroke_width,
                    };
                    context.num_commands += 1;
                }
            }
        }
    }
}

/// Vertical Stack Layout
pub struct VStack<Content: View> {
    pub content: Content,
    pub spacing: f32,
    pub alignment: HorizontalAlignment,
}

/// Horizontal Alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    Leading,
    Center,
    Trailing,
}

impl<Content: View> VStack<Content> {
    pub fn new(content: Content) -> Self {
        Self {
            content,
            spacing: 0.0,
            alignment: HorizontalAlignment::Center,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl<Content: View> View for VStack<Content> {
    type Body = Content;

    fn body(&self) -> Self::Body {
        // This requires Clone; simplified handling
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::ptr::read(&self.content) }
    }

    fn layout(&self, constraints: &Constraints) -> Size {
        // Implementation: Lay out child views vertically with spacing and alignment
        constraints.max
    }

    fn render(&self, context: &mut RenderContext) {
        self.content.render(context);
    }
}

/// Horizontal Stack Layout
pub struct HStack<Content: View> {
    pub content: Content,
    pub spacing: f32,
    pub alignment: VerticalAlignment,
}

/// Vertical Alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

impl<Content: View> HStack<Content> {
    pub fn new(content: Content) -> Self {
        Self {
            content,
            spacing: 0.0,
            alignment: VerticalAlignment::Center,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<Content: View> View for HStack<Content> {
    type Body = Content;

    fn body(&self) -> Self::Body {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::ptr::read(&self.content) }
    }

    fn layout(&self, constraints: &Constraints) -> Size {
        constraints.max
    }

    fn render(&self, context: &mut RenderContext) {
        self.content.render(context);
    }
}

/// Padding Modifier
pub struct Padding<Content: View> {
    pub content: Content,
    pub insets: EdgeInsets,
}

/// Edge Insets
#[derive(Debug, Clone, Copy)]
pub struct EdgeInsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl EdgeInsets {
    pub fn all(value: f32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }
}

impl<Content: View> Padding<Content> {
    pub fn new(content: Content, insets: EdgeInsets) -> Self {
        Self { content, insets }
    }
}

impl<Content: View> View for Padding<Content> {
    type Body = Content;

    fn body(&self) -> Self::Body {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::ptr::read(&self.content) }
    }

    fn layout(&self, constraints: &Constraints) -> Size {
        let inner_constraints = Constraints::new(
            0.0,
            0.0,
            (constraints.max.width - self.insets.left - self.insets.right).max(0.0),
            (constraints.max.height - self.insets.top - self.insets.bottom).max(0.0),
        );

        let inner_size = self.content.layout(&inner_constraints);

        Size {
            width: inner_size.width + self.insets.left + self.insets.right,
            height: inner_size.height + self.insets.top + self.insets.bottom,
        }
    }

    fn render(&self, context: &mut RenderContext) {
        // Apply offset
        let old_transform = context.transform;
        context.transform.tx += self.insets.left;
        context.transform.ty += self.insets.top;

        self.content.render(context);

        context.transform = old_transform;
    }
}

/// View Extension trait
pub trait ViewExt: View + Sized {
    /// Add padding
    fn padding(self, insets: EdgeInsets) -> Padding<Self> {
        Padding::new(self, insets)
    }

    /// Add uniform padding
    fn padding_all(self, value: f32) -> Padding<Self> {
        Padding::new(self, EdgeInsets::all(value))
    }

    /// Set background
    fn background(self, color: Color) -> Background<Self> {
        Background { content: self, color }
    }
}

impl<V: View> ViewExt for V {}

/// Background Modifier
pub struct Background<Content: View> {
    pub content: Content,
    pub color: Color,
}

impl<Content: View> View for Background<Content> {
    type Body = Content;

    fn body(&self) -> Self::Body {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::ptr::read(&self.content) }
    }

    fn layout(&self, constraints: &Constraints) -> Size {
        self.content.layout(constraints)
    }

    fn render(&self, context: &mut RenderContext) {
        // Draw background first
        let size = self.content.layout(&Constraints::loose(10000.0, 10000.0));
        if context.num_commands < 256 {
            context.commands[context.num_commands as usize] = DrawCommand::FillRect {
                rect: Rect::new(0.0, 0.0, size.width, size.height),
                color: self.color,
            };
            context.num_commands += 1;
        }

        // Then draw content
        self.content.render(context);
    }
}

/// State Property Wrapper
pub struct State<T> {
    value: T,
    view_id: AtomicU64,
}

impl<T> State<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            view_id: AtomicU64::new(0),
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        // Trigger re-render
        // Implementation: Notify view system to re-render the view associated with this state
    }
}

impl<T: Clone> State<T> {
    pub fn get_cloned(&self) -> T {
        self.value.clone()
    }
}
