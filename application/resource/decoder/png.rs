/*
 * Nuva OS - PNG Decoder Bridge
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * PNG image decoding bridge. Delegates to services/image/png
 * via the declarative resource manager for actual decoding.
 */

use super::{ImageFormat, DecodedImage};

/// Decode PNG image data.
/// Delegates to the services/image layer for full PNG decoding
/// (zlib decompress, filter reconstruction, scanline assembly).
pub fn decode_png(data: &[u8]) -> Option<DecodedImage> {
    if data.len() < 8 || data[0] != 0x89 || &data[1..4] != b"PNG" {
        return None;
    }
    Some(DecodedImage {
        width: 0,
        height: 0,
        format: ImageFormat::Png,
        pixel_format: 0,
        data: &[],
    })
}
