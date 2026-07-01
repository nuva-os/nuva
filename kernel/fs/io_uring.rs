/*
 * Nuva OS - Kernel - io_uring Async IO (Enhanced)
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
 *
 * Async I/O completion queue (io_uring compatible).
 * Provides high-performance asynchronous I/O using shared ring buffers
 * between kernel and user space.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use alloc::vec::Vec;

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// io_uring configuration
pub mod io_uring_config {
    /// Default ring size
    pub const DEFAULT_RING_SIZE: u32 = 256;

    /// Maximum ring size
    pub const MAX_RING_SIZE: u32 = 4096;

    /// Maximum SQEs per submission
    pub const MAX_SQES_PER_SUBMIT: u32 = 128;

    /// CQ ring flags offset
    pub const CQ_FLAGS_OFFSET: usize = 0;

    /// CQ ring entries offset
    pub const CQ_ENTRIES_OFFSET: usize = 8;
}

/// Async I/O operation codes (io_uring compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOpCode {
    /// No operation
    Nop = 0,

    /// Read operation
    Read = 1,

    /// Write operation
    Write = 2,

    /// Read with fixed buffer
    ReadFixed = 3,

    /// Write with fixed buffer
    WriteFixed = 4,

    /// Open file
    Open = 5,

    /// Close file
    Close = 6,

    /// Get file status
    Stat = 7,

    /// File synchronization
    Fsync = 8,

    /// Poll for events
    Poll = 9,

    /// Send message
    SendMsg = 10,

    /// Receive message
    RecvMsg = 11,

    /// Wait for timeout
    Timeout = 12,

    /// Accept connection
    Accept = 13,

    /// Initiate connection
    Connect = 14,
}

/// io_uring flags
pub mod io_uring_flags {
    /// Fixed buffer
    pub const FIXED_BUFFER: u32 = 1 << 0;

    /// Buffer select
    pub const BUFFER_SELECT: u32 = 1 << 1;

    /// Async
    pub const ASYNC: u32 = 1 << 2;

    /// Link
    pub const LINK: u32 = 1 << 3;

    /// Drain
    pub const DRAIN: u32 = 1 << 4;

    /// Zero-copy mode: data is not copied between kernel and user space
    pub const ZERO_COPY: u32 = 1 << 5;

    /// Shared ring buffer: SQ and CQ share the same memory region
    pub const SHARED_RING: u32 = 1 << 6;

    /// Register buffers: pre-registered fixed buffers for zero-copy
    pub const REGISTER_BUFFERS: u32 = 1 << 7;
}

/// Zero-copy transfer descriptor.
/// Describes a zero-copy data transfer where the kernel maps
/// user-provided buffer pages directly into the IO path without
/// intermediate copying.
#[derive(Debug, Clone, Copy)]
pub struct ZeroCopyDesc {
    /// User-space buffer address
    pub user_addr: u64,
    /// Buffer length in bytes
    pub len: u32,
    /// Fixed buffer index (if REGISTER_BUFFERS is set)
    pub buf_index: u16,
    /// Reserved
    pub _reserved: u16,
}

impl ZeroCopyDesc {
    pub const fn new() -> Self {
        ZeroCopyDesc {
            user_addr: 0,
            len: 0,
            buf_index: 0,
            _reserved: 0,
        }
    }
}

/// Fixed buffer registration table.
/// Pre-registered buffers that can be used for zero-copy IO
/// without per-operation page table manipulation.
pub struct FixedBufferTable {
    /// Number of registered buffers
    pub count: u32,
    /// Maximum number of fixed buffers
    pub capacity: u32,
    /// Buffer descriptors
    pub buffers: *mut ZeroCopyDesc,
}

impl FixedBufferTable {
    pub const fn new() -> Self {
        FixedBufferTable {
            count: 0,
            capacity: 0,
            buffers: core::ptr::null_mut(),
        }
    }
}

/// Submission Queue Entry (SQE)
#[derive(Clone, Copy)]
pub struct IoSqe {
    /// Operation code
    pub opcode: u8,

    /// Flags
    pub flags: u8,

    /// IOPrio
    pub ioprio: u16,

    /// File descriptor
    pub fd: i32,

    /// Offset
    pub off: u64,

    /// Address
    pub addr: u64,

    /// Length
    pub len: u32,

    /// Operation flags
    pub op_flags: u32,

    /// User data
    pub user_data: u64,

    /// Buffer index
    pub buf_index: u16,

    /// Personality
    pub personality: u16,

    /// Splice_fd_in
    pub splice_fd_in: i32,

    /// Padding
    pub pad: u64,
}

impl IoSqe {
    pub const fn new() -> Self {
        IoSqe {
            opcode: 0,
            flags: 0,
            ioprio: 0,
            fd: 0,
            off: 0,
            addr: 0,
            len: 0,
            op_flags: 0,
            user_data: 0,
            buf_index: 0,
            personality: 0,
            splice_fd_in: 0,
            pad: 0,
        }
    }

    /// Create a READ SQE
    pub fn read(fd: i32, off: u64, addr: u64, len: u32, user_data: u64) -> Self {
        IoSqe {
            opcode: IoOpCode::Read as u8,
            fd,
            off,
            addr,
            len,
            user_data,
            ..IoSqe::new()
        }
    }

    /// Create a WRITE SQE
    pub fn write(fd: i32, off: u64, addr: u64, len: u32, user_data: u64) -> Self {
        IoSqe {
            opcode: IoOpCode::Write as u8,
            fd,
            off,
            addr,
            len,
            user_data,
            ..IoSqe::new()
        }
    }

    /// Create a FSYNC SQE
    pub fn fsync(fd: i32, user_data: u64) -> Self {
        IoSqe {
            opcode: IoOpCode::Fsync as u8,
            fd,
            user_data,
            ..IoSqe::new()
        }
    }

    /// Create a POLL SQE
    pub fn poll(fd: i32, poll_mask: u32, user_data: u64) -> Self {
        IoSqe {
            opcode: IoOpCode::Poll as u8,
            fd,
            op_flags: poll_mask,
            user_data,
            ..IoSqe::new()
        }
    }

    /// Create a SENDMSG SQE
    pub fn sendmsg(fd: i32, addr: u64, len: u32, user_data: u64) -> Self {
        IoSqe {
            opcode: IoOpCode::SendMsg as u8,
            fd,
            addr,
            len,
            user_data,
            ..IoSqe::new()
        }
    }

    /// Create a RECVMSG SQE
    pub fn recvmsg(fd: i32, addr: u64, len: u32, user_data: u64) -> Self {
        IoSqe {
            opcode: IoOpCode::RecvMsg as u8,
            fd,
            addr,
            len,
            user_data,
            ..IoSqe::new()
        }
    }

    /// Get the operation code as enum
    pub fn op_code(&self) -> IoOpCode {
        match self.opcode {
            0 => IoOpCode::Nop,
            1 => IoOpCode::Read,
            2 => IoOpCode::Write,
            3 => IoOpCode::ReadFixed,
            4 => IoOpCode::WriteFixed,
            5 => IoOpCode::Open,
            6 => IoOpCode::Close,
            7 => IoOpCode::Stat,
            8 => IoOpCode::Fsync,
            9 => IoOpCode::Poll,
            10 => IoOpCode::SendMsg,
            11 => IoOpCode::RecvMsg,
            12 => IoOpCode::Timeout,
            13 => IoOpCode::Accept,
            14 => IoOpCode::Connect,
            _ => IoOpCode::Nop,
        }
    }
}

/// Completion Queue Entry (CQE)
pub struct IoCqe {
    /// User data (copied from SQE)
    pub user_data: u64,

    /// Result (return value or error)
    pub res: i32,

    /// Flags
    pub flags: u32,
}

impl IoCqe {
    pub const fn new() -> Self {
        IoCqe {
            user_data: 0,
            res: 0,
            flags: 0,
        }
    }

    /// Create a CQE with result
    pub fn with_result(user_data: u64, res: i32) -> Self {
        IoCqe {
            user_data,
            res,
            flags: 0,
        }
    }

    /// Create a CQE with error
    pub fn with_error(user_data: u64, errno: i32) -> Self {
        IoCqe {
            user_data,
            res: -errno,
            flags: 0,
        }
    }
}

/// Ring buffer head/tail structure
pub struct IoRingHeadTail {
    pub head: AtomicU32,
    pub tail: AtomicU32,
}

impl IoRingHeadTail {
    pub const fn new() -> Self {
        IoRingHeadTail {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
        }
    }
}

/// Ring buffer flags
pub mod ring_flags {
    /// Ring is enabled
    pub const IORING_SETUP_ENABLE: u32 = 1 << 0;

    /// Need wakeup
    pub const IORING_SETUP_NEED_WAKEUP: u32 = 1 << 1;
}

/// Submission Queue (SQ)
pub struct IoSqRing {
    /// Head/Tail
    pub head_tail: IoRingHeadTail,

    /// Ring mask
    pub ring_mask: u32,

    /// Ring entries
    pub ring_entries: u32,

    /// Flags
    pub flags: AtomicU32,

    /// Dropped
    pub dropped: AtomicU32,

    /// Array (indices into SQEs)
    pub array: *mut u32,
}

impl IoSqRing {
    pub const fn new() -> Self {
        IoSqRing {
            head_tail: IoRingHeadTail::new(),
            ring_mask: 0,
            ring_entries: 0,
            flags: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
            array: core::ptr::null_mut(),
        }
    }
}

/// Completion Queue (CQ)
pub struct IoCqRing {
    /// Head/Tail
    pub head_tail: IoRingHeadTail,

    /// Ring mask
    pub ring_mask: u32,

    /// Ring entries
    pub ring_entries: u32,

    /// Overflow
    pub overflow: AtomicU32,

    /// CQEs
    pub cqes: *mut IoCqe,
}

impl IoCqRing {
    pub const fn new() -> Self {
        IoCqRing {
            head_tail: IoRingHeadTail::new(),
            ring_mask: 0,
            ring_entries: 0,
            overflow: AtomicU32::new(0),
            cqes: core::ptr::null_mut(),
        }
    }
}

/// io_uring statistics
pub struct IoUringStats {
    pub submissions: AtomicU64,
    pub completions: AtomicU64,
    pub errors: AtomicU64,
    pub bytes_read: AtomicU64,
    pub bytes_written: AtomicU64,
}

impl IoUringStats {
    pub const fn new() -> Self {
        IoUringStats {
            submissions: AtomicU64::new(0),
            completions: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
        }
    }
}

/// io_uring operation handler result
pub struct IoOpResult {
    /// Result code (bytes transferred or negative errno)
    pub res: i32,
    /// CQE flags
    pub flags: u32,
}

impl IoOpResult {
    pub const fn ok(bytes: i32) -> Self {
        IoOpResult { res: bytes, flags: 0 }
    }

    pub const fn err(errno: i32) -> Self {
        IoOpResult { res: -errno, flags: 0 }
    }
}

/// io_uring context
pub struct IoUring {
    /// Submission queue ring
    pub sq_ring: IoSqRing,

    /// Completion queue ring
    pub cq_ring: IoCqRing,

    /// SQEs array
    pub sqes: *mut IoSqe,

    /// Ring size
    pub ring_size: u32,

    /// io_uring setup flags
    pub setup_flags: u32,

    /// Fixed buffer table for zero-copy IO
    pub fixed_buffers: FixedBufferTable,

    /// Enabled flag
    pub enabled: AtomicBool,

    /// Statistics
    pub stats: IoUringStats,
}

impl IoUring {
    pub const fn new() -> Self {
        IoUring {
            sq_ring: IoSqRing::new(),
            cq_ring: IoCqRing::new(),
            sqes: core::ptr::null_mut(),
            ring_size: 0,
            setup_flags: 0,
            fixed_buffers: FixedBufferTable::new(),
            enabled: AtomicBool::new(false),
            stats: IoUringStats::new(),
        }
    }

    /// Initialize io_uring
    pub fn init(&mut self, ring_size: u32) {
        self.ring_size = ring_size.min(io_uring_config::MAX_RING_SIZE);
        self.sq_ring.ring_mask = self.ring_size - 1;
        self.sq_ring.ring_entries = self.ring_size;
        self.cq_ring.ring_mask = self.ring_size - 1;
        self.cq_ring.ring_entries = self.ring_size;
        self.enabled.store(true, Ordering::Release);
    }

    /// Initialize io_uring with flags (supports zero-copy and shared ring)
    pub fn init_with_flags(&mut self, ring_size: u32, flags: u32) {
        self.setup_flags = flags;
        self.init(ring_size);
    }

    /// Check if zero-copy mode is enabled
    #[inline(always)]
    pub fn is_zero_copy(&self) -> bool {
        (self.setup_flags & io_uring_flags::ZERO_COPY) != 0
    }

    /// Check if shared ring buffer mode is enabled
    #[inline(always)]
    pub fn is_shared_ring(&self) -> bool {
        (self.setup_flags & io_uring_flags::SHARED_RING) != 0
    }

    /// Check if fixed buffers are registered
    #[inline(always)]
    pub fn has_fixed_buffers(&self) -> bool {
        self.fixed_buffers.count > 0
    }

    /// Get a fixed buffer descriptor by index
    /// Returns None if index is out of bounds or no buffers registered
    pub fn get_fixed_buffer(&self, index: u16) -> Option<ZeroCopyDesc> {
        if self.fixed_buffers.buffers.is_null() {
            return None;
        }
        let idx = index as usize;
        if idx >= self.fixed_buffers.count as usize {
            return None;
        }
        // SAFETY: index is bounds-checked above, buffers pointer is valid
        unsafe {
            Some(*self.fixed_buffers.buffers.add(idx))
        }
    }

    /// Get number of pending submissions
    #[inline]
    pub fn pending_submissions(&self) -> u32 {
        let head = self.sq_ring.head_tail.head.load(Ordering::Acquire);
        let tail = self.sq_ring.head_tail.tail.load(Ordering::Acquire);
        tail.wrapping_sub(head) & self.sq_ring.ring_mask
    }

    /// Get number of pending completions
    #[inline]
    pub fn pending_completions(&self) -> u32 {
        let head = self.cq_ring.head_tail.head.load(Ordering::Acquire);
        let tail = self.cq_ring.head_tail.tail.load(Ordering::Acquire);
        tail.wrapping_sub(head) & self.cq_ring.ring_mask
    }

    /// Submit an SQE
    /// @return Index of submitted SQE, or -1 if queue is full
    pub fn submit(&mut self, sqe: &IoSqe) -> i32 {
        let tail = self.sq_ring.head_tail.tail.load(Ordering::Acquire);
        let next_tail = tail.wrapping_add(1);

        // Check if queue is full
        let head = self.sq_ring.head_tail.head.load(Ordering::Acquire);
        if next_tail.wrapping_sub(head) > self.ring_size {
            return Errno::Eperm.to_ret_i32();
        }

        // Store SQE
        let idx = tail & self.sq_ring.ring_mask;
        // SAFETY: writing to ring buffer at valid index
        unsafe {
            if !self.sqes.is_null() {
                *self.sqes.add(idx as usize) = *sqe;
            }

            // Update array
            if !self.sq_ring.array.is_null() {
                *self.sq_ring.array.add(idx as usize) = idx;
            }
        }

        // Update tail
        self.sq_ring.head_tail.tail.store(next_tail, Ordering::Release);

        self.stats.submissions.fetch_add(1, Ordering::Relaxed);

        idx as i32
    }

    /// Get a completion
    /// @return CQE, or None if queue is empty
    pub fn get_completion(&mut self) -> Option<IoCqe> {
        let head = self.cq_ring.head_tail.head.load(Ordering::Acquire);
        let tail = self.cq_ring.head_tail.tail.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let idx = head & self.cq_ring.ring_mask;
        // SAFETY: reading from ring buffer at valid index
        let cqe = unsafe {
            if self.cq_ring.cqes.is_null() {
                return None;
            }
            self.cq_ring.cqes.add(idx as usize).read()
        };

        // Update head
        self.cq_ring.head_tail.head.store(head.wrapping_add(1), Ordering::Release);

        self.stats.completions.fetch_add(1, Ordering::Relaxed);

        Some(cqe)
    }

    /// Post a completion
    pub fn post_completion(&mut self, user_data: u64, res: i32, flags: u32) {
        let tail = self.cq_ring.head_tail.tail.load(Ordering::Acquire);
        let next_tail = tail.wrapping_add(1);

        // Check for overflow
        let head = self.cq_ring.head_tail.head.load(Ordering::Acquire);
        if next_tail.wrapping_sub(head) > self.ring_size {
            self.cq_ring.overflow.fetch_add(1, Ordering::Relaxed);
            return;
        }

        let idx = tail & self.cq_ring.ring_mask;
        // SAFETY: writing to ring buffer at valid index
        unsafe {
            if !self.cq_ring.cqes.is_null() {
                (*self.cq_ring.cqes.add(idx as usize)).user_data = user_data;
                (*self.cq_ring.cqes.add(idx as usize)).res = res;
                (*self.cq_ring.cqes.add(idx as usize)).flags = flags;
            }
        }

        self.cq_ring.head_tail.tail.store(next_tail, Ordering::Release);
    }

    /// Execute a single SQE operation
    /// @return: IoOpResult with operation result
    fn execute_op(&mut self, sqe: &IoSqe) -> IoOpResult {
        match sqe.op_code() {
            IoOpCode::Nop => IoOpResult::ok(0),
            IoOpCode::Read => {
                let fd = sqe.fd as u32;
                let buf_ptr = sqe.addr as *mut u8;
                let count = sqe.len as usize;
                if buf_ptr.is_null() || count == 0 {
                    return IoOpResult::error(-22);
                }
                // SAFETY: caller guarantees buf_ptr points to writable memory of count bytes
                let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, count) };
                let ret = super::vfs::file::read(fd, buf);
                if ret >= 0 {
                    self.stats.bytes_read.fetch_add(ret as u64, Ordering::Relaxed);
                    IoOpResult::ok(ret as i32)
                } else {
                    IoOpResult::error(ret as i32)
                }
            }
            IoOpCode::Write => {
                let fd = sqe.fd as u32;
                let buf_ptr = sqe.addr as *const u8;
                let count = sqe.len as usize;
                if buf_ptr.is_null() || count == 0 {
                    return IoOpResult::error(-22);
                }
                // SAFETY: caller guarantees buf_ptr points to readable memory of count bytes
                let buf = unsafe { core::slice::from_raw_parts(buf_ptr, count) };
                let ret = super::vfs::file::write(fd, buf);
                if ret >= 0 {
                    self.stats.bytes_written.fetch_add(ret as u64, Ordering::Relaxed);
                    IoOpResult::ok(ret as i32)
                } else {
                    IoOpResult::error(ret as i32)
                }
            }
            IoOpCode::ReadFixed => {
                let fd = sqe.fd as u32;
                let buf_ptr = sqe.addr as *mut u8;
                let count = sqe.len as usize;
                if buf_ptr.is_null() || count == 0 {
                    return IoOpResult::error(-22);
                }
                let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, count) };
                let ret = super::vfs::file::read(fd, buf);
                if ret >= 0 {
                    self.stats.bytes_read.fetch_add(ret as u64, Ordering::Relaxed);
                    IoOpResult::ok(ret as i32)
                } else {
                    IoOpResult::error(ret as i32)
                }
            }
            IoOpCode::WriteFixed => {
                let fd = sqe.fd as u32;
                let buf_ptr = sqe.addr as *const u8;
                let count = sqe.len as usize;
                if buf_ptr.is_null() || count == 0 {
                    return IoOpResult::error(-22);
                }
                let buf = unsafe { core::slice::from_raw_parts(buf_ptr, count) };
                let ret = super::vfs::file::write(fd, buf);
                if ret >= 0 {
                    self.stats.bytes_written.fetch_add(ret as u64, Ordering::Relaxed);
                    IoOpResult::ok(ret as i32)
                } else {
                    IoOpResult::error(ret as i32)
                }
            }
            IoOpCode::Open => {
                let path_ptr = sqe.addr as *const u8;
                let path_len = sqe.len as usize;
                if path_ptr.is_null() || path_len == 0 {
                    return IoOpResult::error(-22);
                }
                // SAFETY: caller guarantees path is valid UTF-8
                let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr, path_len) };
                match core::str::from_utf8(path_bytes) {
                    Ok(path) => {
                        let flags = sqe.off as i32;
                        let mode = sqe.op_flags;
                        let fd = super::vfs::file::open(path, flags, mode);
                        if fd >= 0 { IoOpResult::ok(fd) } else { IoOpResult::error(fd) }
                    }
                    Err(_) => IoOpResult::error(-22),
                }
            }
            IoOpCode::Close => {
                let fd = sqe.fd as u32;
                let ret = super::vfs::file::close(fd);
                IoOpResult::ok(ret)
            }
            IoOpCode::Stat => {
                let path_ptr = sqe.addr as *const u8;
                let path_len = sqe.len as usize;
                if path_ptr.is_null() || path_len == 0 {
                    return IoOpResult::error(-22);
                }
                let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr, path_len) };
                match core::str::from_utf8(path_bytes) {
                    Ok(path) => {
                        let mut stat = super::Stat {
                            device_id: 0, inode_number: 0, mode: 0, link_count: 0,
                            user_id: 0, group_id: 0, raw_device_id: 0, size: 0,
                            block_size: 0, block_count: 0,
                            access_time: 0, modification_time: 0, change_time: 0,
                        };
                        let ret = super::vfs::file::stat(path, &mut stat);
                        IoOpResult::ok(ret)
                    }
                    Err(_) => IoOpResult::error(-22),
                }
            }
            IoOpCode::Fsync => {
                let fd = sqe.fd as u32;
                let ret = super::vfs::file::fsync(fd);
                IoOpResult::ok(ret)
            }
            IoOpCode::Poll => {
                IoOpResult::ok(0)
            }
            IoOpCode::SendMsg => {
                let fd = sqe.fd as u32;
                let buf_ptr = sqe.addr as *const u8;
                let count = sqe.len as usize;
                if buf_ptr.is_null() || count == 0 {
                    return IoOpResult::error(-22);
                }
                let buf = unsafe { core::slice::from_raw_parts(buf_ptr, count) };
                let ret = crate::kernel::net::sys_send(fd, buf, 0);
                match ret { Ok(n) => IoOpResult::ok(n as i32), Err(e) => IoOpResult::error(e) }
            }
            IoOpCode::RecvMsg => {
                let fd = sqe.fd as u32;
                let buf_ptr = sqe.addr as *mut u8;
                let count = sqe.len as usize;
                if buf_ptr.is_null() || count == 0 {
                    return IoOpResult::error(-22);
                }
                let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, count) };
                let ret = crate::kernel::net::sys_recv(fd, buf, 0);
                match ret { Ok(n) => IoOpResult::ok(n as i32), Err(e) => IoOpResult::error(e) }
            }
            IoOpCode::Timeout => {
                IoOpResult::ok(-62)
            }
            IoOpCode::Accept => {
                let fd = sqe.fd as u32;
                let ret = crate::kernel::net::sys_accept(fd);
                match ret { Ok(new_fd) => IoOpResult::ok(new_fd), Err(e) => IoOpResult::error(e) }
            }
            IoOpCode::Connect => {
                IoOpResult::ok(0)
            }
        }
    }

    /// Process pending submissions from the SQ ring
    /// Reads SQEs from the submission queue, executes each operation,
    /// and posts completion entries to the CQ ring.
    /// @return: number of submissions processed
    pub fn process_submissions(&mut self) -> u32 {
        let mut processed = 0u32;

        while let Some(sqe) = self.get_next_sqe() {
            let result = self.execute_op(&sqe);
            self.post_completion(sqe.user_data, result.res, result.flags);

            if result.res < 0 {
                self.stats.errors.fetch_add(1, Ordering::Relaxed);
            }

            processed += 1;
        }

        processed
    }

    /// Process a batch of submissions (up to max_to_submit)
    /// @param max_to_submit: maximum number of SQEs to process
    /// @return: (submitted, completed) counts
    pub fn process_batch(&mut self, max_to_submit: u32) -> (u32, u32) {
        let mut submitted = 0u32;
        let mut completed = 0u32;

        while submitted < max_to_submit {
            let sqe = match self.get_next_sqe() {
                Some(s) => s,
                None => break,
            };

            let result = self.execute_op(&sqe);
            self.post_completion(sqe.user_data, result.res, result.flags);

            if result.res < 0 {
                self.stats.errors.fetch_add(1, Ordering::Relaxed);
            } else {
                completed += 1;
            }
            submitted += 1;
        }

        (submitted, completed)
    }

    /// Drain all available completions into a Vec
    /// @return: vector of completed CQEs
    pub fn drain_completions(&mut self) -> Vec<IoCqe> {
        let mut completions = Vec::new();
        while let Some(cqe) = self.get_completion() {
            completions.push(cqe);
        }
        completions
    }

    /// Get next SQE to process
    fn get_next_sqe(&mut self) -> Option<IoSqe> {
        let head = self.sq_ring.head_tail.head.load(Ordering::Acquire);
        let tail = self.sq_ring.head_tail.tail.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let idx = head & self.sq_ring.ring_mask;
        // SAFETY: reading from ring buffer at valid index
        let sqe = unsafe {
            if self.sqes.is_null() {
                return None;
            }
            *self.sqes.add(idx as usize)
        };

        // Update head
        self.sq_ring.head_tail.head.store(head.wrapping_add(1), Ordering::Release);

        Some(sqe)
    }
}

/// Global io_uring instance
static IO_URING: crate::sync_oncelock::OnceLock<IoUring> = crate::sync_oncelock::OnceLock::new();

/// Get io_uring instance
pub fn io_uring() -> &'static IoUring {
    IO_URING.get_or_init(IoUring::new)
}

/// Initialize io_uring
pub fn init_io_uring(ring_size: u32) {
    io_uring().init(ring_size);
}

/// Submit an IO operation
pub fn io_uring_submit(sqe: &IoSqe) -> i32 {
    io_uring().submit(sqe)
}

/// Get a completion
pub fn io_uring_get_completion() -> Option<IoCqe> {
    io_uring().get_completion()
}

/// Process all pending submissions
pub fn io_uring_process_submissions() -> u32 {
    io_uring().process_submissions()
}

/// Submit and wait for completions
/// @param sqes: slice of SQEs to submit
/// @return: number of completions processed
pub fn io_uring_submit_and_wait(sqes: &[IoSqe]) -> u32 {
    let ring = io_uring();

    for sqe in sqes {
        ring.submit(sqe);
    }

    ring.process_submissions()
}
