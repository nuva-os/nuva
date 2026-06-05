/*
 * Nuva OS - Syslib - Brain - Operators - Pooling
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
 * Nuva OS - System Library - Brain - Pooling Operations
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Neural network pooling layer implementations.
 */

use crate::nuva_brain::inference::tensor::{Tensor, TensorShape, DataType, TensorOps};

/// Pool type
#[derive(Debug, Clone, Copy)]
pub enum PoolType {
    /// Max pooling
    Max = 0,
    /// Average pooling
    Avg = 1,
    /// Global max pooling
    GlobalMax = 2,
    /// Global average pooling
    GlobalAvg = 3,
}

/// Pool parameters
pub struct PoolParams {
    /// Pool type
    pub pool_type: PoolType,
    /// Kernel size
    pub kernel_size: (usize, usize),
    /// Stride
    pub stride: (usize, usize),
    /// Padding
    pub padding: (usize, usize),
    /// Whether to include padding in average computation
    pub count_include_pad: bool,
}

/// 2D Pooling
pub struct Pool2d;

impl Pool2d {
    /// Forward pass dispatching to the appropriate pool type
    pub fn forward(input: &Tensor, params: &PoolParams) -> Option<Tensor> {
        match params.pool_type {
            PoolType::Max => Self::max_pool2d(input, params.kernel_size, params.stride, params.padding),
            PoolType::Avg => Self::avg_pool2d(input, params.kernel_size, params.stride, params.padding, params.count_include_pad),
            _ => None,
        }
    }
    
    /// Max pooling 2D
    pub fn max_pool2d(input: &Tensor, kernel_size: (usize, usize), stride: (usize, usize), padding: (usize, usize)) -> Option<Tensor> {
        if input.dtype != DataType::Fp32 || input.shape.ndim < 3 {
            return None;
        }
        
        let batch = input.shape.dims[0];
        let in_h = input.shape.dims[1];
        let in_w = input.shape.dims[2];
        let channels = if input.shape.ndim >= 4 { input.shape.dims[3] } else { 1 };
        
        let out_h = (in_h + 2 * padding.0 - kernel_size.0) / stride.0 + 1;
        let out_w = (in_w + 2 * padding.1 - kernel_size.1) / stride.1 + 1;
        
        let out_shape = TensorShape::new([batch, out_h, out_w, channels], input.shape.ndim);
        let mut result = TensorOps::zeros(&out_shape, input.dtype)?;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let x = core::slice::from_raw_parts(input.data as *const f32, input.shape.num_elements());
            let r = core::slice::from_raw_parts_mut(result.data as *mut f32, out_shape.num_elements());
            
            for b in 0..batch {
                for c in 0..channels {
                    for oh in 0..out_h {
                        for ow in 0..out_w {
                            let mut max_val = f32::NEG_INFINITY;
                            for kh in 0..kernel_size.0 {
                                for kw in 0..kernel_size.1 {
                                    let ih = oh * stride.0 + kh;
                                    let iw = ow * stride.1 + kw;
                                    if ih < in_h && iw < in_w {
                                        let idx = b * (in_h * in_w * channels) + ih * (in_w * channels) + iw * channels + c;
                                        if idx < x.len() && x[idx] > max_val {
                                            max_val = x[idx];
                                        }
                                    }
                                }
                            }
                            let oidx = b * (out_h * out_w * channels) + oh * (out_w * channels) + ow * channels + c;
                            if oidx < r.len() { r[oidx] = max_val; }
                        }
                    }
                }
            }
        }
        
        Some(result)
    }
    
    /// Average pooling 2D
    pub fn avg_pool2d(input: &Tensor, kernel_size: (usize, usize), stride: (usize, usize), padding: (usize, usize), _count_include_pad: bool) -> Option<Tensor> {
        if input.dtype != DataType::Fp32 || input.shape.ndim < 3 {
            return None;
        }
        
        let batch = input.shape.dims[0];
        let in_h = input.shape.dims[1];
        let in_w = input.shape.dims[2];
        let channels = if input.shape.ndim >= 4 { input.shape.dims[3] } else { 1 };
        
        let out_h = (in_h + 2 * padding.0 - kernel_size.0) / stride.0 + 1;
        let out_w = (in_w + 2 * padding.1 - kernel_size.1) / stride.1 + 1;
        
        let out_shape = TensorShape::new([batch, out_h, out_w, channels], input.shape.ndim);
        let mut result = TensorOps::zeros(&out_shape, input.dtype)?;
        
        let window_size = (kernel_size.0 * kernel_size.1) as f32;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let x = core::slice::from_raw_parts(input.data as *const f32, input.shape.num_elements());
            let r = core::slice::from_raw_parts_mut(result.data as *mut f32, out_shape.num_elements());
            
            for b in 0..batch {
                for c in 0..channels {
                    for oh in 0..out_h {
                        for ow in 0..out_w {
                            let mut sum = 0.0f32;
                            for kh in 0..kernel_size.0 {
                                for kw in 0..kernel_size.1 {
                                    let ih = oh * stride.0 + kh;
                                    let iw = ow * stride.1 + kw;
                                    if ih < in_h && iw < in_w {
                                        let idx = b * (in_h * in_w * channels) + ih * (in_w * channels) + iw * channels + c;
                                        if idx < x.len() { sum += x[idx]; }
                                    }
                                }
                            }
                            let oidx = b * (out_h * out_w * channels) + oh * (out_w * channels) + ow * channels + c;
                            if oidx < r.len() { r[oidx] = sum / window_size; }
                        }
                    }
                }
            }
        }
        
        Some(result)
    }
    
    /// Calculate output shape
    pub fn calc_output_shape(input_shape: &TensorShape, params: &PoolParams) -> TensorShape {
        let batch = input_shape.dims[0];
        let in_h = input_shape.dims[1];
        let in_w = input_shape.dims[2];
        let channels = input_shape.dims[3];
        
        let out_h = (in_h + 2 * params.padding.0 - params.kernel_size.0) / params.stride.0 + 1;
        let out_w = (in_w + 2 * params.padding.1 - params.kernel_size.1) / params.stride.1 + 1;
        
        TensorShape::new([batch, out_h, out_w, channels], 4)
    }
}

/// Global average pooling
pub struct GlobalAvgPool2d;

impl GlobalAvgPool2d {
    /// Forward pass: average over spatial dimensions
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        if input.dtype != DataType::Fp32 || input.shape.ndim < 3 {
            return None;
        }
        
        let batch = input.shape.dims[0];
        let in_h = input.shape.dims[1];
        let in_w = input.shape.dims[2];
        let channels = if input.shape.ndim >= 4 { input.shape.dims[3] } else { 1 };
        
        let out_shape = TensorShape::new([batch, 1, 1, channels], input.shape.ndim);
        let mut result = TensorOps::zeros(&out_shape, input.dtype)?;
        
        let spatial_size = (in_h * in_w) as f32;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let x = core::slice::from_raw_parts(input.data as *const f32, input.shape.num_elements());
            let r = core::slice::from_raw_parts_mut(result.data as *mut f32, out_shape.num_elements());
            
            for b in 0..batch {
                for c in 0..channels {
                    let mut sum = 0.0f32;
                    for h in 0..in_h {
                        for w in 0..in_w {
                            let idx = b * (in_h * in_w * channels) + h * (in_w * channels) + w * channels + c;
                            if idx < x.len() { sum += x[idx]; }
                        }
                    }
                    let oidx = b * channels + c;
                    if oidx < r.len() { r[oidx] = sum / spatial_size; }
                }
            }
        }
        
        Some(result)
    }
}

/// Global max pooling
pub struct GlobalMaxPool2d;

impl GlobalMaxPool2d {
    /// Forward pass: max over spatial dimensions
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        if input.dtype != DataType::Fp32 || input.shape.ndim < 3 {
            return None;
        }
        
        let batch = input.shape.dims[0];
        let in_h = input.shape.dims[1];
        let in_w = input.shape.dims[2];
        let channels = if input.shape.ndim >= 4 { input.shape.dims[3] } else { 1 };
        
        let out_shape = TensorShape::new([batch, 1, 1, channels], input.shape.ndim);
        let mut result = TensorOps::zeros(&out_shape, input.dtype)?;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let x = core::slice::from_raw_parts(input.data as *const f32, input.shape.num_elements());
            let r = core::slice::from_raw_parts_mut(result.data as *mut f32, out_shape.num_elements());
            
            for b in 0..batch {
                for c in 0..channels {
                    let mut max_val = f32::NEG_INFINITY;
                    for h in 0..in_h {
                        for w in 0..in_w {
                            let idx = b * (in_h * in_w * channels) + h * (in_w * channels) + w * channels + c;
                            if idx < x.len() && x[idx] > max_val { max_val = x[idx]; }
                        }
                    }
                    let oidx = b * channels + c;
                    if oidx < r.len() { r[oidx] = max_val; }
                }
            }
        }
        
        Some(result)
    }
}

/// Adaptive pooling
pub struct AdaptivePool2d;

impl AdaptivePool2d {
    /// Adaptive average pooling: automatically compute stride and kernel size
    pub fn adaptive_avg_pool2d(input: &Tensor, output_size: (usize, usize)) -> Option<Tensor> {
        if input.shape.ndim < 3 { return None; }
        let in_h = input.shape.dims[1];
        let in_w = input.shape.dims[2];
        let stride = (in_h / output_size.0, in_w / output_size.1);
        let kernel_size = (in_h - (output_size.0 - 1) * stride.0, in_w - (output_size.1 - 1) * stride.1);
        Pool2d::avg_pool2d(input, kernel_size, stride, (0, 0), false)
    }
    
    /// Adaptive max pooling
    pub fn adaptive_max_pool2d(input: &Tensor, output_size: (usize, usize)) -> Option<Tensor> {
        if input.shape.ndim < 3 { return None; }
        let in_h = input.shape.dims[1];
        let in_w = input.shape.dims[2];
        let stride = (in_h / output_size.0, in_w / output_size.1);
        let kernel_size = (in_h - (output_size.0 - 1) * stride.0, in_w - (output_size.1 - 1) * stride.1);
        Pool2d::max_pool2d(input, kernel_size, stride, (0, 0))
    }
}

/// 3D Pooling
pub struct Pool3d;

impl Pool3d {
    /// Forward pass for 3D pooling (placeholder for future implementation)
    pub fn forward(_input: &Tensor, _params: &PoolParams) -> Option<Tensor> {
        // 3D pooling requires 5D tensors [batch, depth, height, width, channels]
        // This is a less common operation and can be implemented when needed
        None
    }
}

/// Fractional max pooling
pub struct FractionalMaxPool2d;

impl FractionalMaxPool2d {
    /// Forward pass for fractional max pooling
    pub fn forward(_input: &Tensor, _output_ratio: (f32, f32)) -> Option<Tensor> {
        // Fractional max pooling uses random pooling regions
        // This is a specialized operation for certain network architectures
        None
    }
}

/// Lp Pooling
pub struct LpPool2d;

impl LpPool2d {
    /// Forward pass: output = (sum(|x|^p))^(1/p)
    pub fn forward(input: &Tensor, p: f32, kernel_size: (usize, usize), stride: (usize, usize)) -> Option<Tensor> {
        if input.dtype != DataType::Fp32 || input.shape.ndim < 3 || p <= 0.0 {
            return None;
        }
        
        let batch = input.shape.dims[0];
        let in_h = input.shape.dims[1];
        let in_w = input.shape.dims[2];
        let channels = if input.shape.ndim >= 4 { input.shape.dims[3] } else { 1 };
        
        let out_h = (in_h - kernel_size.0) / stride.0 + 1;
        let out_w = (in_w - kernel_size.1) / stride.1 + 1;
        
        let out_shape = TensorShape::new([batch, out_h, out_w, channels], input.shape.ndim);
        let mut result = TensorOps::zeros(&out_shape, input.dtype)?;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let x = core::slice::from_raw_parts(input.data as *const f32, input.shape.num_elements());
            let r = core::slice::from_raw_parts_mut(result.data as *mut f32, out_shape.num_elements());
            
            for b in 0..batch {
                for c in 0..channels {
                    for oh in 0..out_h {
                        for ow in 0..out_w {
                            let mut sum_p = 0.0f32;
                            for kh in 0..kernel_size.0 {
                                for kw in 0..kernel_size.1 {
                                    let ih = oh * stride.0 + kh;
                                    let iw = ow * stride.1 + kw;
                                    let idx = b * (in_h * in_w * channels) + ih * (in_w * channels) + iw * channels + c;
                                    if idx < x.len() {
                                        let abs_val = if x[idx] >= 0.0 { x[idx] } else { -x[idx] };
                                        sum_p += approx_pow(abs_val, p);
                                    }
                                }
                            }
                            let oidx = b * (out_h * out_w * channels) + oh * (out_w * channels) + ow * channels + c;
                            if oidx < r.len() { r[oidx] = approx_pow(sum_p, 1.0 / p); }
                        }
                    }
                }
            }
        }
        
        Some(result)
    }
}

/// Approximate pow(x, y) for positive x using exp(y * ln(x))
fn approx_pow(x: f32, y: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    if y == 0.0 { return 1.0; }
    if y == 1.0 { return x; }
    if y == 2.0 { return x * x; }
    
    // Use exp(y * ln(x))
    let ln_x = approx_log(x);
    approx_exp(y * ln_x)
}

fn approx_exp(x: f32) -> f32 {
    if x < -80.0 { return 0.0; }
    if x > 80.0 { return f32::INFINITY; }
    const LN2: f32 = 0.6931471805599453;
    let y = x / LN2;
    let i = y.floor() as i32;
    let f = y - i as f32;
    let p = 1.0 + f * (0.6931472 + f * (0.2402265 + f * (0.0554953 + f * 0.0096052)));
    if i >= -126 && i <= 127 {
        let scale = f32::from_bits(((127 + i) as u32) << 23);
        p * scale
    } else if i < -126 { 0.0 } else { f32::INFINITY }
}

fn approx_log(x: f32) -> f32 {
    if x <= 0.0 { return f32::NEG_INFINITY; }
    if x == 1.0 { return 0.0; }
    let bits = x.to_bits();
    let exponent = ((bits >> 23) & 0xFF) as i32 - 127;
    let mantissa_bits = bits & 0x7FFFFF;
    let mantissa = f32::from_bits(0x3F800000 | mantissa_bits);
    let f = mantissa - 1.0;
    let log_mantissa = f * (1.0 - f * (0.5 - f * (0.3333333 - f * 0.25)));
    const LN2: f32 = 0.6931471805599453;
    exponent as f32 * LN2 + log_mantissa
}