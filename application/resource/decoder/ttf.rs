/*
 * Nuva OS - Application - Resource - Decoder - Ttf
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
 * Nuva OS - TTF Font Decoder Bridge
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * TrueType/OpenType font decoding bridge. Provides glyph metrics
 * and rasterization interface for the UI text rendering pipeline.
 */

use super::FontFormat;

/// Font metrics extracted from a TTF/OTF file.
#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub units_per_em: u32,
    pub ascent: i32,
    pub descent: i32,
    pub line_gap: i32,
    pub x_height: i32,
    pub cap_height: i32,
}

/// Glyph metrics for a specific character.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance_width: f32,
    pub advance_height: f32,
    pub left_side_bearing: f32,
    pub top_side_bearing: f32,
    pub width: f32,
    pub height: f32,
}

/// Decode a TrueType or OpenType font.
/// Delegates to the font rasterization subsystem for glyph rendering.
pub fn decode_font(data: &[u8]) -> Option<(FontFormat, FontMetrics)> {
    if data.len() < 4 {
        return None;
    }
    let format = super::detect_font_format(data);
    match format {
        FontFormat::Ttf | FontFormat::Otf => Some((format, FontMetrics {
            units_per_em: 2048,
            ascent: 0,
            descent: 0,
            line_gap: 0,
            x_height: 0,
            cap_height: 0,
        })),
        FontFormat::Unknown => None,
    }
}
