/*
 * Nuva OS - Syslib - Brain - Inference - Tensor
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
 * Nuva OS - System Library - Brain - Tensor Operations
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Tensor data structure and basic operations for neural network inference.
 */

use alloc::alloc::{alloc, dealloc, Layout};

/// Data type for tensor elements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    /// 32-bit floating point
    Fp32 = 0,
    /// 16-bit floating point
    Fp16 = 1,
    /// 32-bit integer
    Int32 = 2,
    /// 8-bit integer
    Int8 = 3,
    /// 4-bit integer (packed)
    Int4 = 4,
}

/// Tensor shape descriptor
pub struct TensorShape {
    /// Dimension sizes (up to 4 dimensions)
    pub dims: [usize; 4],
    /// Number of dimensions
    pub ndim: usize,
}

impl TensorShape {
    pub const fn new(dims: [usize; 4], ndim: usize) -> Self {
        TensorShape { dims, ndim }
    }
    
    /// Calculate total number of elements
    pub fn num_elements(&self) -> usize {
        let mut count = 1;
        for i in 0..self.ndim {
            count *= self.dims[i];
        }
        count
    }
    
    /// Calculate byte size for a given data type
    pub fn size(&self, dtype: DataType) -> usize {
        let elem_size = match dtype {
            DataType::Fp32 => 4,
            DataType::Fp16 => 2,
            DataType::Int32 => 4,
            DataType::Int8 => 1,
            DataType::Int4 => 1,
        };
        self.num_elements() * elem_size
    }
}

/// Tensor data structure
pub struct Tensor {
    /// Data pointer (virtual address)
    pub data: u64,
    /// Shape descriptor
    pub shape: TensorShape,
    /// Element data type
    pub dtype: DataType,
    /// Whether gradient computation is needed
    pub requires_grad: bool,
}

impl Tensor {
    pub const fn new(data: u64, shape: TensorShape, dtype: DataType) -> Self {
        Tensor {
            data,
            shape,
            dtype,
            requires_grad: false,
        }
    }
    
    /// Get byte size of tensor data
    pub fn size(&self) -> usize {
        self.shape.size(self.dtype)
    }
    
    /// Get total number of elements
    pub fn num_elements(&self) -> usize {
        self.shape.num_elements()
    }
    
    /// Check if the tensor data is valid (non-null)
    pub fn is_valid(&self) -> bool {
        self.data != 0
    }
}

/// Tensor operations
pub struct TensorOps;

impl TensorOps {
    /// Create a zero-filled tensor
    pub fn zeros(shape: &TensorShape, dtype: DataType) -> Option<Tensor> {
        let size = shape.size(dtype);
        let layout = Layout::from_size_align(size, 8).ok()?;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return None;
        }
        // Initialize to zero
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::ptr::write_bytes(ptr, 0, size); }
        Some(Tensor::new(ptr as u64, TensorShape::new(shape.dims, shape.ndim), dtype))
    }
    
    /// Create a one-filled tensor
    pub fn ones(shape: &TensorShape, dtype: DataType) -> Option<Tensor> {
        let size = shape.size(dtype);
        let layout = Layout::from_size_align(size, 8).ok()?;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return None;
        }
        // Initialize to ones based on data type
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            match dtype {
                DataType::Fp32 => {
                    let ones = core::slice::from_raw_parts_mut(ptr as *mut f32, shape.num_elements());
                    for v in ones.iter_mut() { *v = 1.0; }
                }
                DataType::Fp16 => {
                    // FP16 1.0 is 0x3C00
                    let ones = core::slice::from_raw_parts_mut(ptr as *mut u16, shape.num_elements());
                    for v in ones.iter_mut() { *v = 0x3C00; }
                }
                DataType::Int32 => {
                    let ones = core::slice::from_raw_parts_mut(ptr as *mut i32, shape.num_elements());
                    for v in ones.iter_mut() { *v = 1; }
                }
                DataType::Int8 => {
                    let ones = core::slice::from_raw_parts_mut(ptr as *mut i8, shape.num_elements());
                    for v in ones.iter_mut() { *v = 1; }
                }
                DataType::Int4 => {
                    // Packed 4-bit: each byte holds two 4-bit values
                    core::ptr::write_bytes(ptr, 0x11, size);
                }
            }
        }
        Some(Tensor::new(ptr as u64, TensorShape::new(shape.dims, shape.ndim), dtype))
    }
    
    /// Matrix multiplication: C = A @ B
    /// A: [M, K], B: [K, N] -> C: [M, N]
    pub fn matmul(a: &Tensor, b: &Tensor) -> Option<Tensor> {
        if a.dtype != DataType::Fp32 || b.dtype != DataType::Fp32 {
            return None; // Only FP32 supported for now
        }
        if a.shape.ndim < 2 || b.shape.ndim < 2 {
            return None;
        }
        
        let m = a.shape.dims[a.shape.ndim - 2];
        let k_a = a.shape.dims[a.shape.ndim - 1];
        let k_b = b.shape.dims[b.shape.ndim - 2];
        let n = b.shape.dims[b.shape.ndim - 1];
        
        if k_a != k_b {
            return None; // Dimension mismatch
        }
        
        let out_shape = TensorShape::new([m, n, 0, 0], 2);
        let mut result = Self::zeros(&out_shape, DataType::Fp32)?;
        
        // Perform matrix multiplication
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let a_data = core::slice::from_raw_parts(a.data as *const f32, m * k_a);
            let b_data = core::slice::from_raw_parts(b.data as *const f32, k_a * n);
            let c_data = core::slice::from_raw_parts_mut(result.data as *mut f32, m * n);
            
            for i in 0..m {
                for j in 0..n {
                    let mut sum = 0.0f32;
                    for k in 0..k_a {
                        sum += a_data[i * k_a + k] * b_data[k * n + j];
                    }
                    c_data[i * n + j] = sum;
                }
            }
        }
        
        Some(result)
    }
    
    /// Element-wise addition: C = A + B
    pub fn add(a: &Tensor, b: &Tensor) -> Option<Tensor> {
        if a.shape.num_elements() != b.shape.num_elements() || a.dtype != b.dtype {
            return None;
        }
        
        let mut result = Self::zeros(&TensorShape::new(a.shape.dims, a.shape.ndim), a.dtype)?;
        
        if a.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = a.shape.num_elements();
                let a_data = core::slice::from_raw_parts(a.data as *const f32, n);
                let b_data = core::slice::from_raw_parts(b.data as *const f32, n);
                let r_data = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n { r_data[i] = a_data[i] + b_data[i]; }
            }
        }
        
        Some(result)
    }
    
    /// Element-wise subtraction: C = A - B
    pub fn sub(a: &Tensor, b: &Tensor) -> Option<Tensor> {
        if a.shape.num_elements() != b.shape.num_elements() || a.dtype != b.dtype {
            return None;
        }
        
        let mut result = Self::zeros(&TensorShape::new(a.shape.dims, a.shape.ndim), a.dtype)?;
        
        if a.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = a.shape.num_elements();
                let a_data = core::slice::from_raw_parts(a.data as *const f32, n);
                let b_data = core::slice::from_raw_parts(b.data as *const f32, n);
                let r_data = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n { r_data[i] = a_data[i] - b_data[i]; }
            }
        }
        
        Some(result)
    }
    
    /// Element-wise multiplication: C = A * B
    pub fn mul(a: &Tensor, b: &Tensor) -> Option<Tensor> {
        if a.shape.num_elements() != b.shape.num_elements() || a.dtype != b.dtype {
            return None;
        }
        
        let mut result = Self::zeros(&TensorShape::new(a.shape.dims, a.shape.ndim), a.dtype)?;
        
        if a.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = a.shape.num_elements();
                let a_data = core::slice::from_raw_parts(a.data as *const f32, n);
                let b_data = core::slice::from_raw_parts(b.data as *const f32, n);
                let r_data = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n { r_data[i] = a_data[i] * b_data[i]; }
            }
        }
        
        Some(result)
    }
    
    /// Element-wise division: C = A / B
    pub fn div(a: &Tensor, b: &Tensor) -> Option<Tensor> {
        if a.shape.num_elements() != b.shape.num_elements() || a.dtype != b.dtype {
            return None;
        }
        
        let mut result = Self::zeros(&TensorShape::new(a.shape.dims, a.shape.ndim), a.dtype)?;
        
        if a.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = a.shape.num_elements();
                let a_data = core::slice::from_raw_parts(a.data as *const f32, n);
                let b_data = core::slice::from_raw_parts(b.data as *const f32, n);
                let r_data = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    r_data[i] = if b_data[i] != 0.0 { a_data[i] / b_data[i] } else { 0.0 };
                }
            }
        }
        
        Some(result)
    }
    
    /// ReLU activation: output = max(0, input)
    pub fn relu(x: &Tensor) -> Option<Tensor> {
        let mut result = Self::zeros(&TensorShape::new(x.shape.dims, x.shape.ndim), x.dtype)?;
        
        if x.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = x.shape.num_elements();
                let x_data = core::slice::from_raw_parts(x.data as *const f32, n);
                let r_data = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n { r_data[i] = if x_data[i] > 0.0 { x_data[i] } else { 0.0 }; }
            }
        }
        
        Some(result)
    }
    
    /// Sigmoid activation: output = 1 / (1 + exp(-input))
    pub fn sigmoid(x: &Tensor) -> Option<Tensor> {
        let mut result = Self::zeros(&TensorShape::new(x.shape.dims, x.shape.ndim), x.dtype)?;
        
        if x.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = x.shape.num_elements();
                let x_data = core::slice::from_raw_parts(x.data as *const f32, n);
                let r_data = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    // Approximate exp using Taylor series for small values
                    let v = x_data[i];
                    let exp_neg = if v < -80.0 { 1e35f32 } else if v > 80.0 { 0.0 } else { approx_exp(-v) };
                    r_data[i] = 1.0 / (1.0 + exp_neg);
                }
            }
        }
        
        Some(result)
    }
    
    /// Softmax: exp(x - max(x)) / sum(exp(x - max(x)))
    pub fn softmax(x: &Tensor, _axis: usize) -> Option<Tensor> {
        let mut result = Self::zeros(&TensorShape::new(x.shape.dims, x.shape.ndim), x.dtype)?;
        
        if x.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = x.shape.num_elements();
                let x_data = core::slice::from_raw_parts(x.data as *const f32, n);
                let r_data = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                
                // Find max for numerical stability
                let mut max_val = f32::NEG_INFINITY;
                for i in 0..n { if x_data[i] > max_val { max_val = x_data[i]; } }
                
                // Compute exp(x - max) and sum
                let mut sum = 0.0f32;
                for i in 0..n {
                    r_data[i] = approx_exp(x_data[i] - max_val);
                    sum += r_data[i];
                }
                
                // Normalize
                if sum > 0.0 {
                    for i in 0..n { r_data[i] /= sum; }
                }
            }
        }
        
        Some(result)
    }
    
    /// Transpose: swap the last two dimensions
    pub fn transpose(x: &Tensor) -> Option<Tensor> {
        if x.shape.ndim < 2 {
            return None;
        }
        
        let mut new_dims = x.shape.dims;
        new_dims[x.shape.ndim - 2] = x.shape.dims[x.shape.ndim - 1];
        new_dims[x.shape.ndim - 1] = x.shape.dims[x.shape.ndim - 2];
        
        let mut result = Self::zeros(&TensorShape::new(new_dims, x.shape.ndim), x.dtype)?;
        
        if x.dtype == DataType::Fp32 && x.shape.ndim == 2 {
            let m = x.shape.dims[0];
            let n = x.shape.dims[1];
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let x_data = core::slice::from_raw_parts(x.data as *const f32, m * n);
                let r_data = core::slice::from_raw_parts_mut(result.data as *mut f32, m * n);
                for i in 0..m {
                    for j in 0..n {
                        r_data[j * m + i] = x_data[i * n + j];
                    }
                }
            }
        }
        
        Some(result)
    }
    
    /// Reshape: change the shape without modifying data
    pub fn reshape(x: &Tensor, new_shape: &TensorShape) -> Option<Tensor> {
        if x.shape.num_elements() != new_shape.num_elements() {
            return None;
        }
        Some(Tensor::new(x.data, TensorShape::new(new_shape.dims, new_shape.ndim), x.dtype))
    }
}

/// Approximate exp(x) using a polynomial approximation
/// Suitable for the range [-80, 80] with FP32 precision
fn approx_exp(x: f32) -> f32 {
    if x < -80.0 { return 0.0; }
    if x > 80.0 { return f32::INFINITY; }
    
    // Use the identity: exp(x) = 2^(x/ln2)
    // Approximate 2^y using a minimax polynomial for the fractional part
    const LN2: f32 = 0.6931471805599453;
    let y = x / LN2;
    let i = y.floor() as i32;
    let f = y - i as f32;
    
    // Minimax polynomial for 2^f on [0, 1)
    let p = 1.0 + f * (0.6931472 + f * (0.2402265 + f * (0.0554953 + f * 0.0096052)));
    
    // Scale by 2^i using bit manipulation
    if i >= -126 && i <= 127 {
        let scale = f32::from_bits(((127 + i) as u32) << 23);
        p * scale
    } else if i < -126 {
        0.0
    } else {
        f32::INFINITY
    }
}