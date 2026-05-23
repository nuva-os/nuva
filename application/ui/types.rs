/*
 * Nuva OS - Declarative UI Types
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Fundamental geometric and color types for the declarative UI framework.
 */

/** 2D point with f32 coordinates. */
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    /** X coordinate. */
    pub x: f32,
    /** Y coordinate. */
    pub y: f32,
}

impl Point {
    /** Create a new point. */
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
}

/** 2D size with f32 dimensions. */
#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    /** Width. */
    pub width: f32,
    /** Height. */
    pub height: f32,
}

impl Size {
    /** Create a new size. */
    pub fn new(width: f32, height: f32) -> Self { Self { width, height } }
}

/** Axis-aligned rectangle. */
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    /** X origin. */
    pub x: f32,
    /** Y origin. */
    pub y: f32,
    /** Width. */
    pub width: f32,
    /** Height. */
    pub height: f32,
}

impl Rect {
    /** Create a new rectangle. */
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /** Test if a point is inside this rectangle. */
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x && point.x <= self.x + self.width
            && point.y >= self.y && point.y <= self.y + self.height
    }
}

/** RGBA color. */
#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    /** Red channel. */
    pub r: u8,
    /** Green channel. */
    pub g: u8,
    /** Blue channel. */
    pub b: u8,
    /** Alpha channel. */
    pub a: u8,
}

impl Color {
    /** Create a new RGBA color. */
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self { Self { r, g, b, a } }

    /** Create an opaque RGB color. */
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self { Self { r, g, b, a: 255 } }

    /** White. */
    pub fn white() -> Self { Self::from_rgb(255, 255, 255) }
    /** Black. */
    pub fn black() -> Self { Self::from_rgb(0, 0, 0) }
    /** Red. */
    pub fn red() -> Self { Self::from_rgb(255, 0, 0) }
    /** Green. */
    pub fn green() -> Self { Self::from_rgb(0, 255, 0) }
    /** Blue. */
    pub fn blue() -> Self { Self::from_rgb(0, 0, 255) }
    /** Gray. */
    pub fn gray() -> Self { Self::from_rgb(128, 128, 128) }
    /** Transparent. */
    pub fn transparent() -> Self { Self { r: 0, g: 0, b: 0, a: 0 } }
}
