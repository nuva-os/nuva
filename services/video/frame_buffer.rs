/*
 * Nuva OS - SystemService - Video - Frame Buffer
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

//! Frame buffer management for video decode/encode.
//! Frame data is stored in shared memory for zero-copy transfer
//! between the video service and callers via Nuva IPC.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::error::{PixelFormat, VideoError};

/// Frame buffer identifier
pub type FrameBufferId = u64;

/// Frame buffer containing decoded video frame data
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
    /// Stride (bytes per row) for the Y plane
    pub stride: u32,
    /// Pixel format
    pub pixel_format: PixelFormat,
    /// Frame pixel data
    pub data: Vec<u8>,
}

impl FrameBuffer {
    /// Create a new frame buffer
    pub fn new(width: u32, height: u32, pixel_format: PixelFormat) -> Self {
        let stride = width;
        let bpp = match pixel_format {
            PixelFormat::Yuv420P | PixelFormat::Nv12 | PixelFormat::Nv21 => 3,
            PixelFormat::Yuv422P => 2,
            PixelFormat::Yuv444P => 3,
            PixelFormat::Rgba8888 | PixelFormat::Bgra8888 => 4,
        };
        let y_size = (stride * height) as usize;
        let total_size = match pixel_format {
            PixelFormat::Yuv420P | PixelFormat::Nv12 | PixelFormat::Nv21 => {
                y_size + y_size / 2
            }
            PixelFormat::Yuv422P => y_size + y_size,
            PixelFormat::Yuv444P => y_size * 3,
            PixelFormat::Rgba8888 | PixelFormat::Bgra8888 => {
                (width * height * bpp) as usize
            }
        };

        let mut data = Vec::with_capacity(total_size);
        for _ in 0..total_size {
            data.push(0);
        }

        FrameBuffer {
            width,
            height,
            stride,
            pixel_format,
            data,
        }
    }

    /// Get the size in bytes of this frame buffer
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }

    /// Get the Y plane slice
    pub fn y_plane(&self) -> &[u8] {
        let y_size = (self.stride * self.height) as usize;
        if y_size > self.data.len() {
            return &[];
        }
        &self.data[..y_size]
    }

    /// Get the UV plane slice (for NV12/NV21)
    pub fn uv_plane(&self) -> &[u8] {
        let y_size = (self.stride * self.height) as usize;
        if y_size >= self.data.len() {
            return &[];
        }
        &self.data[y_size..]
    }
}

/// Reference to a frame in the frame buffer pool
#[derive(Debug, Clone, Copy)]
pub struct FrameRef {
    /// Buffer ID in the frame pool
    pub buffer_id: FrameBufferId,
    /// Presentation timestamp in microseconds
    pub pts_us: i64,
}

/// Decode result containing decoded frames and metadata
#[derive(Debug, Clone)]
pub struct DecodeResult {
    /// Decoded frame buffers
    pub frames: Vec<FrameBuffer>,
    /// Frame references (for zero-copy transfer)
    pub frame_refs: Vec<FrameRef>,
    /// Number of input bytes consumed
    pub bytes_consumed: usize,
}

/// Frame buffer state in the pool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameState {
    /// Frame buffer is free
    Free = 0,
    /// Frame buffer is in use by decoder
    InUse = 1,
    /// Frame buffer is held by client
    HeldByClient = 2,
}

/// Frame buffer entry in the pool
struct FramePoolEntry {
    /// Buffer ID
    id: FrameBufferId,
    /// State
    state: AtomicU32,
    /// Width
    width: u32,
    /// Height
    height: u32,
    /// Shared memory region ID for zero-copy
    shm_region_id: u64,
}

/// Maximum frame buffers in the pool
pub const MAX_FRAME_BUFFERS: usize = 64;

/// Frame buffer pool manager
pub struct FrameBufferPool {
    /// Pool entries
    entries: [FramePoolEntry; MAX_FRAME_BUFFERS],
    /// Next buffer ID
    next_id: AtomicU64,
    /// Allocated frame buffers
    buffers: BTreeMap<FrameBufferId, FrameBuffer>,
    /// Total allocated bytes
    allocated_bytes: AtomicU64,
}

impl FrameBufferPool {
    /// Create a new frame buffer pool
    pub fn new() -> Self {
        let default_entry = FramePoolEntry {
            id: 0,
            state: AtomicU32::new(FrameState::Free as u32),
            width: 0,
            height: 0,
            shm_region_id: 0,
        };
        // SAFETY: FramePoolEntry contains AtomicU32 which is zero-initializable.
        // All other fields (u32, u64) have valid zero representations.
        let entries: [FramePoolEntry; MAX_FRAME_BUFFERS] = unsafe {
            core::mem::zeroed()
        };

        FrameBufferPool {
            entries,
            next_id: AtomicU64::new(1),
            buffers: BTreeMap::new(),
            allocated_bytes: AtomicU64::new(0),
        }
    }

    /// Allocate a frame buffer from the pool
    pub fn allocate(
        &mut self,
        width: u32,
        height: u32,
        pixel_format: PixelFormat,
    ) -> Result<(FrameBufferId, &FrameBuffer), VideoError> {
        let buffer_id = self.next_id.fetch_add(1, Ordering::AcqRel);

        for i in 0..MAX_FRAME_BUFFERS {
            let state = match self.entries[i].state.load(Ordering::Acquire) {
                0 => FrameState::Free,
                _ => continue,
            };

            if state == FrameState::Free {
                self.entries[i].id = buffer_id;
                self.entries[i].width = width;
                self.entries[i].height = height;
                self.entries[i].state.store(
                    FrameState::InUse as u32,
                    Ordering::Release,
                );

                let frame = FrameBuffer::new(width, height, pixel_format);
                let size = frame.size_bytes();
                self.allocated_bytes.fetch_add(size as u64, Ordering::Relaxed);
                self.buffers.insert(buffer_id, frame);

                return Ok((
                    buffer_id,
                    self.buffers.get(&buffer_id)
                        .ok_or(VideoError::OutOfMemory)?,
                ));
            }
        }

        Err(VideoError::FrameBufferExhausted)
    }

    /// Release a frame buffer back to the pool
    pub fn release(&mut self, buffer_id: FrameBufferId) -> Result<(), VideoError> {
        for i in 0..MAX_FRAME_BUFFERS {
            if self.entries[i].id == buffer_id {
                let size = self.buffers.get(&buffer_id).map(|f| f.size_bytes()).unwrap_or(0);
                self.entries[i].state.store(
                    FrameState::Free as u32,
                    Ordering::Release,
                );
                self.entries[i].id = 0;
                self.buffers.remove(&buffer_id);
                self.allocated_bytes.fetch_sub(size as u64, Ordering::Relaxed);
                return Ok(());
            }
        }
        Err(VideoError::DecoderNotFound)
    }

    /// Get a frame buffer by ID
    pub fn get(&self, buffer_id: FrameBufferId) -> Option<&FrameBuffer> {
        self.buffers.get(&buffer_id)
    }

    /// Mark a frame as held by client
    pub fn hold(&self, buffer_id: FrameBufferId) -> Result<(), VideoError> {
        for i in 0..MAX_FRAME_BUFFERS {
            if self.entries[i].id == buffer_id {
                self.entries[i].state.store(
                    FrameState::HeldByClient as u32,
                    Ordering::Release,
                );
                return Ok(());
            }
        }
        Err(VideoError::DecoderNotFound)
    }

    /// Get total allocated bytes
    pub fn allocated_bytes(&self) -> u64 {
        self.allocated_bytes.load(Ordering::Acquire)
    }
}
