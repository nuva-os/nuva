/*
 * Nuva OS - System Library - Brain - Activation Functions
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Neural network activation function implementations.
 */

use crate::nuva_brain::inference::tensor::{Tensor, TensorShape, DataType, TensorOps};

/// ReLU Activation Function
pub struct ReLU;

impl ReLU {
    /// Forward pass: output = max(0, input)
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        TensorOps::relu(input)
    }
    
    /// Leaky ReLU: output = max(alpha * input, input)
    pub fn forward_leaky(input: &Tensor, alpha: f32) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    r[i] = if x[i] > 0.0 { x[i] } else { alpha * x[i] };
                }
            }
        }
        
        Some(result)
    }
    
    /// Parametric ReLU: output = max(weight * input, input)
    pub fn forward_prelu(input: &Tensor, weight: &Tensor) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let w = core::slice::from_raw_parts(weight.data as *const f32, n.min(weight.shape.num_elements()));
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    let wi = if i < w.len() { w[i] } else { w[0] };
                    r[i] = if x[i] > 0.0 { x[i] } else { wi * x[i] };
                }
            }
        }
        
        Some(result)
    }
}

/// ReLU6 Activation Function
pub struct ReLU6;

impl ReLU6 {
    /// Forward pass: output = min(max(0, input), 6)
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    r[i] = if x[i] < 0.0 { 0.0 } else if x[i] > 6.0 { 6.0 } else { x[i] };
                }
            }
        }
        
        Some(result)
    }
}

/// Sigmoid Activation Function
pub struct Sigmoid;

impl Sigmoid {
    /// Forward pass: output = 1 / (1 + exp(-input))
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        TensorOps::sigmoid(input)
    }
}

/// Tanh Activation Function
pub struct Tanh;

impl Tanh {
    /// Forward pass: output = (exp(x) - exp(-x)) / (exp(x) + exp(-x))
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    // tanh(x) = 2*sigmoid(2x) - 1 (numerically stable)
                    let v = x[i];
                    if v > 20.0 { r[i] = 1.0; }
                    else if v < -20.0 { r[i] = -1.0; }
                    else {
                        let exp_pos = approx_exp(v);
                        let exp_neg = approx_exp(-v);
                        r[i] = (exp_pos - exp_neg) / (exp_pos + exp_neg);
                    }
                }
            }
        }
        
        Some(result)
    }
}

/// Softmax Activation Function
pub struct Softmax;

impl Softmax {
    /// Forward pass: exp(x - max(x)) / sum(exp(x - max(x)))
    pub fn forward(input: &Tensor, axis: usize) -> Option<Tensor> {
        TensorOps::softmax(input, axis)
    }
}

/// GELU Activation Function
pub struct GELU;

impl GELU {
    /// Forward pass: 0.5 * x * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3)))
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                const SQRT_2_OVER_PI: f32 = 0.7978845608; // sqrt(2/pi)
                for i in 0..n {
                    let v = x[i];
                    let inner = SQRT_2_OVER_PI * (v + 0.044715 * v * v * v);
                    // Approximate tanh using the identity tanh(x) = 1 - 2/(1+exp(2x))
                    let tanh_val = if inner > 20.0 { 1.0 } else if inner < -20.0 { -1.0 }
                    else {
                        let ep = approx_exp(inner);
                        let en = approx_exp(-inner);
                        (ep - en) / (ep + en)
                    };
                    r[i] = 0.5 * v * (1.0 + tanh_val);
                }
            }
        }
        
        Some(result)
    }
}

/// Swish Activation Function
pub struct Swish;

impl Swish {
    /// Forward pass: output = input * sigmoid(input)
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        let sig = Sigmoid::forward(input)?;
        TensorOps::mul(input, &sig)
    }
}

/// Hard Swish Activation Function
pub struct HardSwish;

impl HardSwish {
    /// Forward pass: piecewise linear approximation of swish
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    let v = x[i];
                    r[i] = if v <= -3.0 { 0.0 }
                           else if v >= 3.0 { v }
                           else { v * (v + 3.0) / 6.0 };
                }
            }
        }
        
        Some(result)
    }
}

/// Mish Activation Function
pub struct Mish;

impl Mish {
    /// Forward pass: output = input * tanh(softplus(input))
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    let v = x[i];
                    // softplus(x) = log(1 + exp(x))
                    let sp = if v > 20.0 { v } else { approx_log(1.0 + approx_exp(v)) };
                    // tanh(softplus)
                    let tanh_val = if sp > 20.0 { 1.0 } else if sp < -20.0 { -1.0 }
                    else {
                        let ep = approx_exp(sp);
                        let en = approx_exp(-sp);
                        (ep - en) / (ep + en)
                    };
                    r[i] = v * tanh_val;
                }
            }
        }
        
        Some(result)
    }
}

/// ELU Activation Function
pub struct ELU;

impl ELU {
    /// Forward pass: if x > 0: x, else: alpha * (exp(x) - 1)
    pub fn forward(input: &Tensor, alpha: f32) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    r[i] = if x[i] > 0.0 { x[i] } else { alpha * (approx_exp(x[i]) - 1.0) };
                }
            }
        }
        
        Some(result)
    }
}

/// SELU Activation Function
pub struct SELU;

impl SELU {
    /// Forward pass: scale * elu(x, alpha) with fixed alpha and scale
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        const ALPHA: f32 = 1.6732632423543772;
        const SCALE: f32 = 1.0507009873554805;
        
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    let elu_val = if x[i] > 0.0 { x[i] } else { ALPHA * (approx_exp(x[i]) - 1.0) };
                    r[i] = SCALE * elu_val;
                }
            }
        }
        
        Some(result)
    }
}

/// Softplus Activation Function
pub struct Softplus;

impl Softplus {
    /// Forward pass: output = log(1 + exp(input))
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    // For numerical stability: log(1+exp(x)) = x + log(1+exp(-x)) for x > 0
                    r[i] = if x[i] > 20.0 { x[i] } else { approx_log(1.0 + approx_exp(x[i])) };
                }
            }
        }
        
        Some(result)
    }
}

/// Hard Sigmoid Activation Function
pub struct HardSigmoid;

impl HardSigmoid {
    /// Forward pass: max(0, min(1, (input + 3) / 6))
    pub fn forward(input: &Tensor) -> Option<Tensor> {
        let mut result = TensorOps::zeros(&TensorShape::new(input.shape.dims, input.shape.ndim), input.dtype)?;
        
        if input.dtype == DataType::Fp32 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let n = input.shape.num_elements();
                let x = core::slice::from_raw_parts(input.data as *const f32, n);
                let r = core::slice::from_raw_parts_mut(result.data as *mut f32, n);
                for i in 0..n {
                    let v = (x[i] + 3.0) / 6.0;
                    r[i] = if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v };
                }
            }
        }
        
        Some(result)
    }
}

/// Approximate exp(x) using a polynomial approximation
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
    } else if i < -126 {
        0.0
    } else {
        f32::INFINITY
    }
}

/// Approximate log(x) using a polynomial approximation
fn approx_log(x: f32) -> f32 {
    if x <= 0.0 { return f32::NEG_INFINITY; }
    if x == 1.0 { return 0.0; }
    
    // Extract exponent and mantissa from IEEE 754
    let bits = x.to_bits();
    let exponent = ((bits >> 23) & 0xFF) as i32 - 127;
    let mantissa_bits = bits & 0x7FFFFF;
    let mantissa = f32::from_bits(0x3F800000 | mantissa_bits); // 1.0 + fraction
    
    // Polynomial approximation for log(1+f) on [0, 1)
    let f = mantissa - 1.0;
    let log_mantissa = f * (1.0 - f * (0.5 - f * (0.3333333 - f * 0.25)));
    
    // log(x) = exponent * ln(2) + log(mantissa)
    const LN2: f32 = 0.6931471805599453;
    exponent as f32 * LN2 + log_mantissa
}