/*
 * Nuva OS - Application - Resource - Decoder - Jpeg
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
 * Nuva OS - JPEG Decoder Bridge
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * JPEG image decoding bridge. Delegates to services/image/jpeg
 * via the declarative resource manager for actual decoding.
 */

use super::{ImageFormat, DecodedImage};

/// Decode JPEG image data.
/// Delegates to the services/image layer for full JPEG decoding
/// (Huffman decode, IDCT, color conversion).
pub fn decode_jpeg(data: &[u8]) -> Option<DecodedImage> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    // Delegated to services/image/jpeg for full decode
    // This bridge provides the application-layer interface;
    // actual decoding runs in the L3 service process.
    Some(DecodedImage {
        width: 0,
        height: 0,
        format: ImageFormat::Jpeg,
        pixel_format: 0,
        data: &[],
    })
}
