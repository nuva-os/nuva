/*
 * Nuva OS - SystemService - Image - Transform Pipeline
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

//! Image transform pipeline.
//! Supports Scale, Rotate, Crop, ColorSpaceConvert with
//! Nearest, Bilinear, and Lanczos3 resampling filters.

use alloc::vec::Vec;

use super::error::{ColorSpace, ImageError, ImageFrame};

/// Resampling filter for scaling operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResampleFilter {
    /// Nearest-neighbor (fastest, blocky)
    Nearest = 0,
    /// Bilinear interpolation
    Bilinear = 1,
    /// Lanczos3 (highest quality, slowest)
    Lanczos3 = 2,
}

/// Rotation angle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    /// No rotation
    None = 0,
    /// 90 degrees clockwise
    Rotate90 = 1,
    /// 180 degrees
    Rotate180 = 2,
    /// 270 degrees clockwise (90 CCW)
    Rotate270 = 3,
}

/// Image transform operation
#[derive(Debug, Clone, Copy)]
pub enum ImageTransform {
    /// Scale to target dimensions
    Scale {
        /// Target width (0 = auto-aspect)
        width: u32,
        /// Target height (0 = auto-aspect)
        height: u32,
        /// Resampling filter
        filter: ResampleFilter,
    },
    /// Rotate by a standard angle
    Rotate {
        /// Rotation angle
        angle: Rotation,
    },
    /// Crop to a rectangular region
    Crop {
        /// Left edge
        x: u32,
        /// Top edge
        y: u32,
        /// Crop width
        width: u32,
        /// Crop height
        height: u32,
    },
    /// Convert color space
    ColorSpaceConvert {
        /// Target color space
        target: ColorSpace,
    },
}

/// Transform pipeline: applies a sequence of transforms in order
#[derive(Debug, Clone)]
pub struct TransformPipeline {
    /// Ordered list of transforms
    transforms: Vec<ImageTransform>,
}

impl TransformPipeline {
    /// Create a new empty transform pipeline
    pub fn new() -> Self {
        TransformPipeline {
            transforms: Vec::new(),
        }
    }

    /// Create a pipeline from a list of transforms
    pub fn from_transforms(transforms: Vec<ImageTransform>) -> Self {
        TransformPipeline { transforms }
    }

    /// Add a transform to the pipeline
    pub fn push(&mut self, transform: ImageTransform) {
        self.transforms.push(transform);
    }

    /// Get the number of transforms in the pipeline
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if the pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Get the transforms
    pub fn transforms(&self) -> &[ImageTransform] {
        &self.transforms
    }

    /// Apply the pipeline to an image frame, executing transforms in order
    pub fn apply(&self, frame: &ImageFrame) -> Result<ImageFrame, ImageError> {
        if self.transforms.is_empty() {
            return Ok(frame.clone());
        }

        let mut current = frame.clone();

        for transform in &self.transforms {
            current = match transform {
                ImageTransform::Scale { width, height, filter } => {
                    Self::apply_scale(&current, *width, *height, *filter)?
                }
                ImageTransform::Rotate { angle } => {
                    Self::apply_rotate(&current, *angle)?
                }
                ImageTransform::Crop { x, y, width, height } => {
                    Self::apply_crop(&current, *x, *y, *width, *height)?
                }
                ImageTransform::ColorSpaceConvert { target } => {
                    Self::apply_color_space_convert(&current, *target)?
                }
            };
        }

        Ok(current)
    }

    /// Scale an image frame to the target dimensions
    fn apply_scale(
        frame: &ImageFrame,
        target_width: u32,
        target_height: u32,
        filter: ResampleFilter,
    ) -> Result<ImageFrame, ImageError> {
        if target_width == 0 && target_height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let src_width = frame.width;
        let src_height = frame.height;

        if src_width == 0 || src_height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let dst_width = if target_width == 0 {
            (target_height as u64 * src_width as u64 / src_height as u64) as u32
        } else {
            target_width
        };
        let dst_height = if target_height == 0 {
            (target_width as u64 * src_height as u64 / src_width as u64) as u32
        } else {
            target_height
        };

        if dst_width == 0 || dst_height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let bpp = frame.color_space.bytes_per_pixel();
        let src_stride = frame.effective_stride() as usize;
        let dst_stride = (dst_width as usize) * bpp;

        let mut output_data = Vec::with_capacity(dst_stride * dst_height as usize);

        let x_ratio = src_width as f64 / dst_width as f64;
        let y_ratio = src_height as f64 / dst_height as f64;

        match filter {
            ResampleFilter::Nearest => {
                for y in 0..dst_height {
                    let src_y = ((y as f64 * y_ratio) as u32).min(src_height - 1);
                    for x in 0..dst_width {
                        let src_x = ((x as f64 * x_ratio) as u32).min(src_width - 1);
                        let src_offset = (src_y as usize) * src_stride + (src_x as usize) * bpp;
                        for c in 0..bpp {
                            let byte = if src_offset + c < frame.data.len() {
                                frame.data[src_offset + c]
                            } else {
                                0
                            };
                            output_data.push(byte);
                        }
                    }
                }
            }
            ResampleFilter::Bilinear => {
                for y in 0..dst_height {
                    let src_yf = y as f64 * y_ratio;
                    let y0 = (src_yf as u32).min(src_height - 1);
                    let y1 = (y0 + 1).min(src_height - 1);
                    let fy = src_yf - y0 as f64;

                    for x in 0..dst_width {
                        let src_xf = x as f64 * x_ratio;
                        let x0 = (src_xf as u32).min(src_width - 1);
                        let x1 = (x0 + 1).min(src_width - 1);
                        let fx = src_xf - x0 as f64;

                        for c in 0..bpp {
                            let i00 = (y0 as usize) * src_stride + (x0 as usize) * bpp + c;
                            let i10 = (y0 as usize) * src_stride + (x1 as usize) * bpp + c;
                            let i01 = (y1 as usize) * src_stride + (x0 as usize) * bpp + c;
                            let i11 = (y1 as usize) * src_stride + (x1 as usize) * bpp + c;

                            let v00 = if i00 < frame.data.len() { frame.data[i00] as f64 } else { 0.0 };
                            let v10 = if i10 < frame.data.len() { frame.data[i10] as f64 } else { 0.0 };
                            let v01 = if i01 < frame.data.len() { frame.data[i01] as f64 } else { 0.0 };
                            let v11 = if i11 < frame.data.len() { frame.data[i11] as f64 } else { 0.0 };

                            let top = v00 * (1.0 - fx) + v10 * fx;
                            let bottom = v01 * (1.0 - fx) + v11 * fx;
                            let value = top * (1.0 - fy) + bottom * fy;
                            output_data.push(value.clamp(0.0, 255.0) as u8);
                        }
                    }
                }
            }
            ResampleFilter::Lanczos3 => {
                for y in 0..dst_height {
                    let src_yf = y as f64 * y_ratio;
                    let y_center = src_yf as i64;

                    for x in 0..dst_width {
                        let src_xf = x as f64 * x_ratio;
                        let x_center = src_xf as i64;

                        for c in 0..bpp {
                            let mut sum: f64 = 0.0;
                            let mut weight_sum: f64 = 0.0;

                            for dy in -3i64..=3 {
                                let sy = y_center + dy;
                                if sy < 0 || sy >= src_height as i64 {
                                    continue;
                                }
                                let wy = lanczos3(src_yf - sy as f64);

                                for dx in -3i64..=3 {
                                    let sx = x_center + dx;
                                    if sx < 0 || sx >= src_width as i64 {
                                        continue;
                                    }
                                    let wx = lanczos3(src_xf - sx as f64);
                                    let w = wx * wy;
                                    let offset = (sy as usize) * src_stride + (sx as usize) * bpp + c;
                                    let v = if offset < frame.data.len() {
                                        frame.data[offset] as f64
                                    } else {
                                        0.0
                                    };
                                    sum += v * w;
                                    weight_sum += w;
                                }
                            }

                            if weight_sum > 0.0 {
                                output_data.push((sum / weight_sum).clamp(0.0, 255.0) as u8);
                            } else {
                                output_data.push(0);
                            }
                        }
                    }
                }
            }
        }

        Ok(ImageFrame::from_data(output_data, dst_width, dst_height, frame.color_space))
    }

    /// Rotate an image frame
    fn apply_rotate(frame: &ImageFrame, angle: Rotation) -> Result<ImageFrame, ImageError> {
        let bpp = frame.color_space.bytes_per_pixel();
        let src_stride = frame.effective_stride() as usize;

        match angle {
            Rotation::None => Ok(frame.clone()),
            Rotation::Rotate90 => {
                let dst_width = frame.height;
                let dst_height = frame.width;
                let dst_stride = (dst_width as usize) * bpp;
                let mut output = Vec::with_capacity(dst_stride * dst_height as usize);

                for x in 0..dst_height {
                    for y in 0..dst_width {
                        let src_y = frame.height - 1 - y;
                        let src_x = x;
                        let src_offset = (src_y as usize) * src_stride + (src_x as usize) * bpp;
                        for c in 0..bpp {
                            let byte = if src_offset + c < frame.data.len() {
                                frame.data[src_offset + c]
                            } else {
                                0
                            };
                            output.push(byte);
                        }
                    }
                }

                Ok(ImageFrame::from_data(output, dst_width, dst_height, frame.color_space))
            }
            Rotation::Rotate180 => {
                let mut output = Vec::with_capacity(frame.data.len());
                for y in 0..frame.height {
                    let src_y = frame.height - 1 - y;
                    for x in 0..frame.width {
                        let src_x = frame.width - 1 - x;
                        let src_offset = (src_y as usize) * src_stride + (src_x as usize) * bpp;
                        for c in 0..bpp {
                            let byte = if src_offset + c < frame.data.len() {
                                frame.data[src_offset + c]
                            } else {
                                0
                            };
                            output.push(byte);
                        }
                    }
                }

                Ok(ImageFrame::from_data(output, frame.width, frame.height, frame.color_space))
            }
            Rotation::Rotate270 => {
                let dst_width = frame.height;
                let dst_height = frame.width;
                let dst_stride = (dst_width as usize) * bpp;
                let mut output = Vec::with_capacity(dst_stride * dst_height as usize);

                for x in 0..dst_height {
                    for y in 0..dst_width {
                        let src_y = y;
                        let src_x = frame.width - 1 - x;
                        let src_offset = (src_y as usize) * src_stride + (src_x as usize) * bpp;
                        for c in 0..bpp {
                            let byte = if src_offset + c < frame.data.len() {
                                frame.data[src_offset + c]
                            } else {
                                0
                            };
                            output.push(byte);
                        }
                    }
                }

                Ok(ImageFrame::from_data(output, dst_width, dst_height, frame.color_space))
            }
        }
    }

    /// Crop an image frame to the specified region
    fn apply_crop(
        frame: &ImageFrame,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<ImageFrame, ImageError> {
        if x + width > frame.width || y + height > frame.height {
            return Err(ImageError::InvalidParameter);
        }
        if width == 0 || height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let bpp = frame.color_space.bytes_per_pixel();
        let src_stride = frame.effective_stride() as usize;
        let dst_stride = (width as usize) * bpp;

        let mut output = Vec::with_capacity(dst_stride * height as usize);

        for row in 0..height {
            let src_offset = ((y + row) as usize) * src_stride + (x as usize) * bpp;
            for col in 0..dst_stride {
                let byte = if src_offset + col < frame.data.len() {
                    frame.data[src_offset + col]
                } else {
                    0
                };
                output.push(byte);
            }
        }

        Ok(ImageFrame::from_data(output, width, height, frame.color_space))
    }

    /// Convert the color space of an image frame
    fn apply_color_space_convert(
        frame: &ImageFrame,
        target: ColorSpace,
    ) -> Result<ImageFrame, ImageError> {
        if frame.color_space == target {
            return Ok(frame.clone());
        }

        let src_cs = frame.color_space;
        let dst_cs = target;
        let src_bpp = src_cs.bytes_per_pixel();
        let dst_bpp = dst_cs.bytes_per_pixel();
        let src_stride = frame.effective_stride() as usize;
        let pixels = (frame.width as usize) * (frame.height as usize);
        let mut output = Vec::with_capacity(pixels * dst_bpp);

        for pixel_idx in 0..pixels {
            let src_offset = pixel_idx * src_bpp;

            let r: u8;
            let g: u8;
            let b: u8;
            let a: u8;

            match src_cs {
                ColorSpace::Rgba8 => {
                    r = if src_offset < frame.data.len() { frame.data[src_offset] } else { 0 };
                    g = if src_offset + 1 < frame.data.len() { frame.data[src_offset + 1] } else { 0 };
                    b = if src_offset + 2 < frame.data.len() { frame.data[src_offset + 2] } else { 0 };
                    a = if src_offset + 3 < frame.data.len() { frame.data[src_offset + 3] } else { 255 };
                }
                ColorSpace::Rgb8 => {
                    r = if src_offset < frame.data.len() { frame.data[src_offset] } else { 0 };
                    g = if src_offset + 1 < frame.data.len() { frame.data[src_offset + 1] } else { 0 };
                    b = if src_offset + 2 < frame.data.len() { frame.data[src_offset + 2] } else { 0 };
                    a = 255;
                }
                ColorSpace::Bgra8 => {
                    b = if src_offset < frame.data.len() { frame.data[src_offset] } else { 0 };
                    g = if src_offset + 1 < frame.data.len() { frame.data[src_offset + 1] } else { 0 };
                    r = if src_offset + 2 < frame.data.len() { frame.data[src_offset + 2] } else { 0 };
                    a = if src_offset + 3 < frame.data.len() { frame.data[src_offset + 3] } else { 255 };
                }
                ColorSpace::Bgr8 => {
                    b = if src_offset < frame.data.len() { frame.data[src_offset] } else { 0 };
                    g = if src_offset + 1 < frame.data.len() { frame.data[src_offset + 1] } else { 0 };
                    r = if src_offset + 2 < frame.data.len() { frame.data[src_offset + 2] } else { 0 };
                    a = 255;
                }
                ColorSpace::Gray8 => {
                    let gray = if src_offset < frame.data.len() { frame.data[src_offset] } else { 0 };
                    r = gray;
                    g = gray;
                    b = gray;
                    a = 255;
                }
                ColorSpace::GrayAlpha8 => {
                    let gray = if src_offset < frame.data.len() { frame.data[src_offset] } else { 0 };
                    r = gray;
                    g = gray;
                    b = gray;
                    a = if src_offset + 1 < frame.data.len() { frame.data[src_offset + 1] } else { 255 };
                }
                _ => {
                    r = 0;
                    g = 0;
                    b = 0;
                    a = 255;
                }
            }

            match dst_cs {
                ColorSpace::Rgba8 => {
                    output.extend_from_slice(&[r, g, b, a]);
                }
                ColorSpace::Rgb8 => {
                    output.extend_from_slice(&[r, g, b]);
                }
                ColorSpace::Bgra8 => {
                    output.extend_from_slice(&[b, g, r, a]);
                }
                ColorSpace::Bgr8 => {
                    output.extend_from_slice(&[b, g, r]);
                }
                ColorSpace::Gray8 => {
                    let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;
                    output.push(gray);
                }
                ColorSpace::GrayAlpha8 => {
                    let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;
                    output.extend_from_slice(&[gray, a]);
                }
                _ => {
                    return Err(ImageError::ColorSpaceNotSupported);
                }
            }
        }

        Ok(ImageFrame::from_data(output, frame.width, frame.height, dst_cs))
    }
}

/// Lanczos3 kernel function
fn lanczos3(x: f64) -> f64 {
    if x.abs() < 1e-10 {
        1.0
    } else if x.abs() >= 3.0 {
        0.0
    } else {
        let px = core::f64::consts::PI * x;
        (3.0 * px.sin() * (px / 3.0).sin()) / (px * px)
    }
}
