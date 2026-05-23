/*
 * Nuva OS - SystemLibrary - Ml
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

//! Tensor Operation

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::alloc::{alloc, dealloc, Layout};
use crate::{pr_err};

/// DataType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DataType {
 Float32 = 0,
 Float16 = 1,
 Int32 = 2,
 Int16 = 3,
 Int8 = 4,
 UInt8 = 5,
 Bool = 6,
}

impl DataType {
 pub fn size(&self) -> usize {
 match self {
 Self::Float32 => 4,
 Self::Float16 => 2,
 Self::Int32 => 4,
 Self::Int16 => 2,
 Self::Int8 => 1,
 Self::UInt8 => 1,
 Self::Bool => 1,
 }
 }
}

/// TensorShape
#[derive(Debug, Clone, Copy)]
pub struct Shape {
 pub dims: [usize; 8],
 pub ndim: u8,
}

impl Shape {
 pub fn new(dims: &[usize]) -> Self {
 let mut shape = Self {
 dims: [1; 8],
 ndim: dims.len().min(8) as u8,
 };
 for i in 0..shape.ndim as usize {
 shape.dims[i] = dims[i];
 }
 shape
 }

 pub fn scalar() -> Self {
 Self { dims: [1; 8], ndim: 0 }
 }

 pub fn vector(len: usize) -> Self {
 Self::new(&[len])
 }

 pub fn matrix(rows: usize, cols: usize) -> Self {
 Self::new(&[rows, cols])
 }

 pub fn tensor3d(d0: usize, d1: usize, d2: usize) -> Self {
 Self::new(&[d0, d1, d2])
 }

 pub fn tensor4d(n: usize, c: usize, h: usize, w: usize) -> Self {
 Self::new(&[n, c, h, w])
 }

 pub fn numel(&self) -> usize {
 let mut size = 1;
 for i in 0..self.ndim as usize {
 size *= self.dims[i];
 }
 size
 }

 pub fn is_scalar(&self) -> bool {
 self.ndim == 0
 }

 pub fn is_vector(&self) -> bool {
 self.ndim == 1
 }

 pub fn is_matrix(&self) -> bool {
 self.ndim == 2
 }
}

/// Tensorexist
#[derive(Debug, Clone)]
pub struct TensorStorage {
 pub data: *mut u8,
 pub size: usize,
 pub dtype: DataType,
 pub device: DeviceType,
}

/// DeviceType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceType {
 CPU = 0,
 GPU = 1,
 NPU = 2,
}

impl TensorStorage {
 pub fn new(size: usize, dtype: DataType, device: DeviceType) -> Self {
 let layout = Layout::from_size_align(size, 16).unwrap();
 // SAFETY: Layout is constructed with valid size and 16-byte alignment;
 // alloc returns a pointer to size bytes of uninitialized memory, or null on failure.
 let data = unsafe { alloc(layout) };
 
 Self { data, size, dtype, device }
 }

 pub fn as_slice<T>(&self) -> &[T] {
 // SAFETY: self.data is a valid allocation of self.size bytes; the slice length
 // (size / size_of::<T>()) ensures we do not exceed the allocated region; caller
 // must ensure T's alignment is compatible with the allocation's 16-byte alignment.
 unsafe {
 core::slice::from_raw_parts(
 self.data as *const T,
 self.size / core::mem::size_of::<T>(),
 )
 }
 }

 pub fn as_slice_mut<T>(&mut self) -> &mut [T] {
 // SAFETY: self.data is a valid allocation of self.size bytes; the mutable slice
 // length (size / size_of::<T>()) ensures we do not exceed the allocated region;
 // exclusive &mut self prevents aliasing; caller must ensure T's alignment is
 // compatible with the allocation's 16-byte alignment.
 unsafe {
 core::slice::from_raw_parts_mut(
 self.data as *mut T,
 self.size / core::mem::size_of::<T>(),
 )
 }
 }
}

impl Drop for TensorStorage {
 fn drop(&mut self) {
 if !self.data.is_null() {
 let layout = Layout::from_size_align(self.size, 16).unwrap();
 // SAFETY: self.data was allocated by core::alloc::alloc with the same layout
 // (size=self.size, align=16); the pointer is non-null (checked above); the
 // allocated memory has not been freed or modified beyond the allocation bounds.
 unsafe {
 dealloc(self.data, layout);
 }
 }
 }
}

/// Tensor
#[derive(Debug, Clone)]
pub struct Tensor {
 pub storage: TensorStorage,
 pub shape: Shape,
 pub offset: usize,
 pub strides: [usize; 8],
}

impl Tensor {
 pub fn new(shape: Shape, dtype: DataType, device: DeviceType) -> Self {
 let size = shape.numel() * dtype.size();
 let storage = TensorStorage::new(size, dtype, device);
 
 let mut strides = [0usize; 8];
 if shape.ndim > 0 {
 strides[shape.ndim as usize - 1] = 1;
 for i in (0..shape.ndim as usize - 1).rev() {
 strides[i] = strides[i + 1] * shape.dims[i + 1];
 }
 }
 
 Self {
 storage,
 shape,
 offset: 0,
 strides,
 }
 }

 pub fn zeros(shape: Shape, dtype: DataType) -> Self {
 let mut tensor = Self::new(shape, dtype, DeviceType::CPU);
 let data = tensor.storage.as_slice_mut::<u8>();
 for b in data.iter_mut() {
 *b = 0;
 }
 tensor
 }

 pub fn ones(shape: Shape, dtype: DataType) -> Self {
 let mut tensor = Self::new(shape, dtype, DeviceType::CPU);
 match dtype {
 DataType::Float32 => {
 let data = tensor.storage.as_slice_mut::<f32>();
 for v in data.iter_mut() {
 *v = 1.0;
 }
 }
 DataType::Int32 => {
 let data = tensor.storage.as_slice_mut::<i32>();
 for v in data.iter_mut() {
 *v = 1;
 }
 }
 _ => {}
 }
 tensor
 }

 pub fn from_data<T: Copy>(data: &[T], shape: Shape) -> Self {
 let dtype = match core::mem::size_of::<T>() {
 4 => DataType::Float32,
 2 => DataType::Float16,
 1 => DataType::Int8,
 _ => DataType::Float32,
 };
 
 let mut tensor = Self::new(shape, dtype, DeviceType::CPU);
 let tensor_data = tensor.storage.as_slice_mut::<T>();
 tensor_data[..data.len()].copy_from_slice(data);
 tensor
 }

 pub fn shape(&self) -> &Shape {
 &self.shape
 }

 pub fn dtype(&self) -> DataType {
 self.storage.dtype
 }

 pub fn device(&self) -> DeviceType {
 self.storage.device
 }

 pub fn numel(&self) -> usize {
 self.shape.numel()
 }

 pub fn data_ptr(&self) -> *const u8 {
 // SAFETY: self.storage.data is a valid allocation and self.offset is within
 // the allocation bounds (offset < size by construction); pointer arithmetic
 // yields a valid pointer to the tensor's data region.
 unsafe { self.storage.data.add(self.offset) }
 }

 pub fn data_ptr_mut(&mut self) -> *mut u8 {
 // SAFETY: self.storage.data is a valid allocation and self.offset is within
 // the allocation bounds; exclusive &mut self prevents aliasing.
 unsafe { self.storage.data.add(self.offset) }
 }

 /// Matrix Multiplication
 pub fn matmul(&self, other: &Tensor) -> Tensor {
 if self.shape.ndim != 2 || other.shape.ndim != 2 {
 log_error!("matmul requires 2D tensors, got {}D and {}D", self.shape.ndim, other.shape.ndim);
 return Tensor::zeros(Shape::scalar(), self.dtype());
 }
 
 let m = self.shape.dims[0];
 let k = self.shape.dims[1];
 let k2 = other.shape.dims[0];
 let n = other.shape.dims[1];
 
 if k != k2 {
 log_error!("matmul dimension mismatch: {} vs {}", k, k2);
 return Tensor::zeros(Shape::scalar(), self.dtype());
 }
 
 let mut result = Tensor::zeros(Shape::matrix(m, n), self.dtype());
 
 // SAFETY: self.data_ptr() points to m*k f32 elements in a valid allocation;
 // the slice length matches the 2D tensor dimensions for matrix A.
 let a = unsafe { core::slice::from_raw_parts(self.data_ptr() as *const f32, m * k) };
 // SAFETY: other.data_ptr() points to k*n f32 elements in a valid allocation;
 // the slice length matches the 2D tensor dimensions for matrix B.
 let b = unsafe { core::slice::from_raw_parts(other.data_ptr() as *const f32, k * n) };
 // SAFETY: result.data_ptr_mut() points to m*n f32 elements in a freshly allocated
 // zero-initialized tensor; exclusive mutable access is safe.
 let c = unsafe { core::slice::from_raw_parts_mut(result.data_ptr_mut() as *mut f32, m * n) };
 
 for i in 0..m {
 for j in 0..n {
 let mut sum = 0.0f32;
 for l in 0..k {
 sum += a[i * k + l] * b[l * n + j];
 }
 c[i * n + j] = sum;
 }
 }
 
 result
 }

 /// primePluslaw
 pub fn add(&self, other: &Tensor) -> Tensor {
 let mut result = Tensor::zeros(self.shape, self.dtype());
 
 // SAFETY: self.data_ptr() points to self.numel() f32 elements in a valid allocation.
 let a = unsafe { core::slice::from_raw_parts(self.data_ptr() as *const f32, self.numel()) };
 // SAFETY: other.data_ptr() points to other.numel() f32 elements in a valid allocation.
 let b = unsafe { core::slice::from_raw_parts(other.data_ptr() as *const f32, other.numel()) };
 // SAFETY: result.data_ptr_mut() points to result.numel() f32 elements in a freshly
 // allocated zero-initialized tensor; exclusive mutable access is safe.
 let c = unsafe { core::slice::from_raw_parts_mut(result.data_ptr_mut() as *mut f32, result.numel()) };
 
 for i in 0..self.numel() {
 c[i] = a[i] + b[i % other.numel()];
 }
 
 result
 }

 /// primeMultiplylaw
 pub fn mul(&self, other: &Tensor) -> Tensor {
 let mut result = Tensor::zeros(self.shape, self.dtype());
 
 // SAFETY: self.data_ptr() points to self.numel() f32 elements in a valid allocation.
 let a = unsafe { core::slice::from_raw_parts(self.data_ptr() as *const f32, self.numel()) };
 // SAFETY: other.data_ptr() points to other.numel() f32 elements in a valid allocation.
 let b = unsafe { core::slice::from_raw_parts(other.data_ptr() as *const f32, other.numel()) };
 // SAFETY: result.data_ptr_mut() points to result.numel() f32 elements in a freshly
 // allocated zero-initialized tensor; exclusive mutable access is safe.
 let c = unsafe { core::slice::from_raw_parts_mut(result.data_ptr_mut() as *mut f32, result.numel()) };
 
 for i in 0..self.numel() {
 c[i] = a[i] * b[i % other.numel()];
 }
 
 result
 }

 /// ReLU Activate
 pub fn relu(&self) -> Tensor {
 let mut result = Tensor::zeros(self.shape, self.dtype());
 
 // SAFETY: self.data_ptr() points to self.numel() f32 elements in a valid allocation.
 let a = unsafe { core::slice::from_raw_parts(self.data_ptr() as *const f32, self.numel()) };
 // SAFETY: result.data_ptr_mut() points to result.numel() f32 elements in a freshly
 // allocated zero-initialized tensor; exclusive mutable access is safe.
 let c = unsafe { core::slice::from_raw_parts_mut(result.data_ptr_mut() as *mut f32, result.numel()) };
 
 for i in 0..self.numel() {
 c[i] = if a[i] > 0.0 { a[i] } else { 0.0 };
 }
 
 result
 }

 /// Softmax
 pub fn softmax(&self, dim: usize) -> Tensor {
 let mut result = Tensor::zeros(self.shape, self.dtype());
 
 // SAFETY: self.data_ptr() points to self.numel() f32 elements in a valid allocation.
 let a = unsafe { core::slice::from_raw_parts(self.data_ptr() as *const f32, self.numel()) };
 // SAFETY: result.data_ptr_mut() points to result.numel() f32 elements in a freshly
 // allocated zero-initialized tensor; exclusive mutable access is safe.
 let c = unsafe { core::slice::from_raw_parts_mut(result.data_ptr_mut() as *mut f32, result.numel()) };
 
 // SimplifiedImplementation: falseset dim = 0
 let _ = dim;
 
 // findMaxvalue
 let mut max_val = f32::MIN;
 for &v in a.iter() {
 if v > max_val { max_val = v; }
 }
 
 // Compute exp sum sum
 let mut sum = 0.0f32;
 for i in 0..a.len() {
     // Taylor series approximation of exp(x) for no_std
     let x = a[i] - max_val;
     let exp_val = {
         let mut result = 1.0f32;
         let mut term = 1.0f32;
         for n in 1..10 {
             term *= x / (n as f32);
             result += term;
         }
         result
     };
     c[i] = exp_val;
     sum += exp_val;
 }
 
 // Normalization
 for v in c.iter_mut() {
 *v /= sum;
 }
 
 result
 }

 /// Transpose
 pub fn transpose(&self) -> Tensor {
 if self.shape.ndim != 2 {
 log_error!("transpose requires 2D tensor, got {}D", self.shape.ndim);
 return Tensor::zeros(Shape::scalar(), self.dtype());
 }
 
 let rows = self.shape.dims[0];
 let cols = self.shape.dims[1];
 
 let mut result = Tensor::zeros(Shape::matrix(cols, rows), self.dtype());
 
 // SAFETY: self.data_ptr() points to rows*cols f32 elements in a valid 2D tensor allocation.
 let a = unsafe { core::slice::from_raw_parts(self.data_ptr() as *const f32, rows * cols) };
 // SAFETY: result.data_ptr_mut() points to rows*cols f32 elements in a freshly allocated
 // zero-initialized tensor of shape [cols, rows]; exclusive mutable access is safe.
 let c = unsafe { core::slice::from_raw_parts_mut(result.data_ptr_mut() as *mut f32, rows * cols) };
 
 for i in 0..rows {
 for j in 0..cols {
 c[j * rows + i] = a[i * cols + j];
 }
 }
 
 result
 }

 /// reshape
 pub fn reshape(&self, shape: Shape) -> Tensor {
 if shape.numel() != self.numel() {
 log_error!("reshape size mismatch: {} vs {}", shape.numel(), self.numel());
 return Tensor::zeros(self.shape, self.dtype());
 }
 
 let mut result = Tensor::zeros(shape, self.dtype());
 // SAFETY: self.data_ptr() points to numel()*dtype().size() bytes in a valid allocation;
 // the byte slice covers the entire tensor data region for the copy source.
 let src = unsafe { core::slice::from_raw_parts(self.data_ptr(), self.numel() * self.dtype().size()) };
 // SAFETY: result.data_ptr_mut() points to result.numel()*result.dtype().size() bytes
 // in a freshly allocated tensor; numel matches (checked above), so sizes are equal;
 // exclusive mutable access is safe.
 let dst = unsafe { core::slice::from_raw_parts_mut(result.data_ptr_mut(), result.numel() * result.dtype().size()) };
 dst.copy_from_slice(src);
 result
 }
}