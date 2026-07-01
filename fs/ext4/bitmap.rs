/*
 * Nuva OS - Fs - Ext4 - Bitmap Operations
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

pub struct Ext4BitmapOps;

impl Ext4BitmapOps {
    pub fn find_free_bit(bitmap: &[u8]) -> Option<usize> {
        for (byte_idx, &byte) in bitmap.iter().enumerate() {
            if byte != 0xFF {
                for bit in 0..8 {
                    if (byte & (1 << bit)) == 0 {
                        return Some(byte_idx * 8 + bit);
                    }
                }
            }
        }
        None
    }

    pub fn set_bit(bitmap: &mut [u8], bit: usize) {
        let byte_idx = bit / 8;
        let bit_offset = bit % 8;
        if byte_idx < bitmap.len() {
            bitmap[byte_idx] |= 1 << bit_offset;
        }
    }

    pub fn clear_bit(bitmap: &mut [u8], bit: usize) {
        let byte_idx = bit / 8;
        let bit_offset = bit % 8;
        if byte_idx < bitmap.len() {
            bitmap[byte_idx] &= !(1 << bit_offset);
        }
    }

    pub fn test_bit(bitmap: &[u8], bit: usize) -> bool {
        let byte_idx = bit / 8;
        let bit_offset = bit % 8;
        if byte_idx < bitmap.len() {
            (bitmap[byte_idx] & (1 << bit_offset)) != 0
        } else {
            false
        }
    }

    pub fn count_free(bitmap: &[u8]) -> usize {
        let mut count = 0usize;
        for &byte in bitmap.iter() {
            count += (!byte).count_ones() as usize;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_free_bit() {
        let mut bitmap = [0xFFu8; 512];
        bitmap[0] = 0xFE;
        assert_eq!(Ext4BitmapOps::find_free_bit(&bitmap), Some(0));

        bitmap[0] = 0xFF;
        bitmap[1] = 0xFB;
        assert_eq!(Ext4BitmapOps::find_free_bit(&bitmap), Some(10));
    }

    #[test]
    fn test_set_clear_bit() {
        let mut bitmap = [0u8; 512];
        Ext4BitmapOps::set_bit(&mut bitmap, 5);
        assert!(Ext4BitmapOps::test_bit(&bitmap, 5));
        Ext4BitmapOps::clear_bit(&mut bitmap, 5);
        assert!(!Ext4BitmapOps::test_bit(&bitmap, 5));
    }

    #[test]
    fn test_count_free() {
        let bitmap = [0xFFu8; 512];
        assert_eq!(Ext4BitmapOps::count_free(&bitmap), 0);

        let bitmap = [0u8; 512];
        assert_eq!(Ext4BitmapOps::count_free(&bitmap), 512 * 8);
    }
}