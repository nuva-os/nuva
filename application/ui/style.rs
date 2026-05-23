/*
 * Nuva OS - Declarative Style System
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Declarative style definitions: FontStyle, BorderStyle, ShadowStyle,
 * Style, and Theme. Integrates with the modifier chain for component styling.
 */

use super::types::Color;

/// Font weight scale (CSS-compatible).
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

/// Font style variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyles {
    Normal,
    Italic,
    Oblique,
}

/// Font style descriptor.
#[derive(Debug, Clone, Copy)]
pub struct FontStyle {
    pub size: u32,
    pub weight: FontWeight,
    pub style: FontStyles,
    pub family: &'static str,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle {
            size: 14,
            weight: FontWeight::Normal,
            style: FontStyles::Normal,
            family: "sans-serif",
        }
    }
}

/// Border line style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderType {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
}

/// Border style descriptor.
#[derive(Debug, Clone, Copy)]
pub struct BorderStyle {
    pub width: u32,
    pub color: Color,
    pub radius: u32,
    pub style: BorderType,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle {
            width: 0,
            color: Color::transparent(),
            radius: 0,
            style: BorderType::None,
        }
    }
}

/// Shadow style descriptor.
#[derive(Debug, Clone, Copy)]
pub struct ShadowStyle {
    pub offset_x: i32,
    pub offset_y: i32,
    pub blur_radius: u32,
    pub spread_radius: u32,
    pub color: Color,
}

/// Composable style — aggregates visual properties for a component.
#[derive(Debug, Clone)]
pub struct Style {
    pub background_color: Color,
    pub foreground_color: Color,
    pub font: FontStyle,
    pub border: BorderStyle,
    pub shadow: Option<ShadowStyle>,
    pub opacity: f32,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            background_color: Color::white(),
            foreground_color: Color::black(),
            font: FontStyle::default(),
            border: BorderStyle::default(),
            shadow: None,
            opacity: 1.0,
        }
    }
}

impl Style {
    pub fn new() -> Self { Self::default() }

    pub fn background(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground_color = color;
        self
    }

    pub fn font_size(mut self, size: u32) -> Self {
        self.font.size = size;
        self
    }

    pub fn border(mut self, width: u32, color: Color, radius: u32) -> Self {
        self.border = BorderStyle { width, color, radius, style: BorderType::Solid };
        self
    }

    pub fn shadow(mut self, offset_x: i32, offset_y: i32, blur: u32, color: Color) -> Self {
        self.shadow = Some(ShadowStyle {
            offset_x, offset_y, blur_radius: blur, spread_radius: 0, color,
        });
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

/// Theme — named color scheme.
pub struct Theme {
    pub name: &'static str,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub background_color: Color,
    pub surface_color: Color,
    pub error_color: Color,
    pub text_color: Color,
    pub text_secondary_color: Color,
}

impl Theme {
    pub const LIGHT: Theme = Theme {
        name: "light",
        primary_color: Color::new(33, 150, 243, 255),
        secondary_color: Color::new(156, 39, 176, 255),
        background_color: Color::white(),
        surface_color: Color::white(),
        error_color: Color::red(),
        text_color: Color::black(),
        text_secondary_color: Color::gray(),
    };

    pub const DARK: Theme = Theme {
        name: "dark",
        primary_color: Color::new(33, 150, 243, 255),
        secondary_color: Color::new(156, 39, 176, 255),
        background_color: Color::new(18, 18, 18, 255),
        surface_color: Color::new(30, 30, 30, 255),
        error_color: Color::red(),
        text_color: Color::white(),
        text_secondary_color: Color::gray(),
    };
}
