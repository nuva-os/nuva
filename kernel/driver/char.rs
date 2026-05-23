/*
 * Nuva OS - Kernel - Kernel
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

use crate::pr_info;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Character device ID
pub type CharDeviceId = u32;

/// Character device flags
pub mod char_flags {
    pub const READABLE: u32 = 1 << 0; // Readable
    pub const WRITABLE: u32 = 1 << 1; // Writable
    pub const NONBLOCK: u32 = 1 << 2; // Non-blocking
    pub const EXCLUSIVE: u32 = 1 << 3; // Exclusive access
    pub const TTY: u32 = 1 << 4; // Terminal
    pub const SERIAL: u32 = 1 << 5; // Serial
    pub const CONSOLE: u32 = 1 << 6; // Console
    pub const RANDOM: u32 = 1 << 7; // Random device
}

/// Character device operations
pub struct CharDeviceOps {
    /// Open
    pub open: fn(dev: &CharDevice) -> i32,
    /// Close
    pub close: fn(dev: &CharDevice) -> i32,
    /// Read
    pub read: fn(dev: &CharDevice, buf: &mut [u8]) -> i64,
    /// Write
    pub write: fn(dev: &CharDevice, buf: &[u8]) -> i64,
    /// Poll
    pub poll: fn(dev: &CharDevice, events: u32) -> u32,
    /// IO control
    pub ioctl: fn(dev: &CharDevice, cmd: u32, arg: u64) -> i32,
}

/// Character device
pub struct CharDevice {
    /// Device ID
    pub dev_id: CharDeviceId,
    /// Major device number
    pub major: u32,
    /// Minor device number
    pub minor: u32,
    /// Device name
    pub name: [u8; 32],
    /// Flags
    pub flags: AtomicU32,
    /// Operations
    pub ops: Option<CharDeviceOps>,
    /// Private data
    pub private: u64,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Open count
    pub open_count: AtomicU32,
    /// Read count
    pub read_count: AtomicU64,
    /// Write count
    pub write_count: AtomicU64,
    /// Read bytes
    pub read_bytes: AtomicU64,
    /// Write bytes
    pub write_bytes: AtomicU64,
}

impl CharDevice {
    /// Create character device
    pub fn new(dev_id: CharDeviceId, major: u32, minor: u32, name: &[u8]) -> Self {
        let mut dev = CharDevice {
            dev_id,
            major,
            minor,
            name: [0; 32],
            flags: AtomicU32::new(char_flags::READABLE | char_flags::WRITABLE),
            ops: None,
            private: 0,
            ref_count: AtomicU32::new(0),
            open_count: AtomicU32::new(0),
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            read_bytes: AtomicU64::new(0),
            write_bytes: AtomicU64::new(0),
        };

        let len = name.len().min(31);
        dev.name[..len].copy_from_slice(&name[..len]);

        dev
    }

    /// Get device name
    pub fn get_name(&self) -> &[u8] {
        let mut len = 0;
        for i in 0..32 {
            if self.name[i] == 0 {
                break;
            }
            len = i + 1;
        }
        &self.name[..len]
    }

    /// OpenDevice
    pub fn open(&self) -> i32 {
        if let Some(ref ops) = self.ops {
            let result = (ops.open)(self);

            if result == 0 {
                self.open_count.fetch_add(1, Ordering::AcqRel);
            }

            result
        } else {
            -1
        }
    }

    /// CloseDevice
    pub fn close(&self) -> i32 {
        if let Some(ref ops) = self.ops {
            let result = (ops.close)(self);

            if result == 0 {
                self.open_count.fetch_sub(1, Ordering::AcqRel);
            }

            result
        } else {
            -1
        }
    }

    /// Read
    pub fn read(&self, buf: &mut [u8]) -> i64 {
        if let Some(ref ops) = self.ops {
            let result = (ops.read)(self, buf);

            if result > 0 {
                self.read_count.fetch_add(1, Ordering::AcqRel);
                self.read_bytes.fetch_add(result as u64, Ordering::AcqRel);
            }

            result
        } else {
            -1
        }
    }

    /// Write
    pub fn write(&self, buf: &[u8]) -> i64 {
        if let Some(ref ops) = self.ops {
            let result = (ops.write)(self, buf);

            if result > 0 {
                self.write_count.fetch_add(1, Ordering::AcqRel);
                self.write_bytes.fetch_add(result as u64, Ordering::AcqRel);
            }

            result
        } else {
            -1
        }
    }

    /// Poll
    pub fn poll(&self, events: u32) -> u32 {
        if let Some(ref ops) = self.ops {
            (ops.poll)(self, events)
        } else {
            0
        }
    }

    /// IO Control
    pub fn ioctl(&self, cmd: u32, arg: u64) -> i32 {
        if let Some(ref ops) = self.ops {
            (ops.ioctl)(self, cmd, arg)
        } else {
            -1
        }
    }

    /// Check if readable
    pub fn is_readable(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & char_flags::READABLE) != 0
    }

    /// Check if writable
    pub fn is_writable(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & char_flags::WRITABLE) != 0
    }

    /// Check if non-blocking
    pub fn is_nonblock(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & char_flags::NONBLOCK) != 0
    }

    /// Check if is terminal
    pub fn is_tty(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & char_flags::TTY) != 0
    }

    /// Check if is console
    pub fn is_console(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & char_flags::CONSOLE) != 0
    }

    /// Increase reference
    pub fn get(&self) {
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }

    /// Decrease reference
    pub fn put(&self) {
        self.ref_count.fetch_sub(1, Ordering::AcqRel);
    }
}

/// TTY device
pub struct TtyDevice {
    /// Base character device
    pub dev: CharDevice,
    /// Input buffer
    pub in_buffer: [u8; 4096],
    /// Input buffer head
    pub in_head: AtomicU32,
    /// Input buffer tail
    pub in_tail: AtomicU32,
    /// Output buffer
    pub out_buffer: [u8; 4096],
    /// Output buffer head
    pub out_head: AtomicU32,
    /// Output buffer tail
    pub out_tail: AtomicU32,
    /// Echo
    pub echo: AtomicU32,
    /// Canonical mode
    pub canonical: AtomicU32,
}

impl TtyDevice {
    /// Create TTY Device
    pub fn new(dev_id: CharDeviceId, minor: u32, name: &[u8]) -> Self {
        TtyDevice {
            dev: CharDevice::new(dev_id, 5, minor, name), // TTY major device number 5
            in_buffer: [0; 4096],
            in_head: AtomicU32::new(0),
            in_tail: AtomicU32::new(0),
            out_buffer: [0; 4096],
            out_head: AtomicU32::new(0),
            out_tail: AtomicU32::new(0),
            echo: AtomicU32::new(1),
            canonical: AtomicU32::new(1),
        }
    }

    /// WriteInputBuffer
    pub fn input(&mut self, data: &[u8]) -> u32 {
        let mut count = 0u32;

        for &byte in data {
            let head = self.in_head.load(Ordering::Acquire);
            let next = (head + 1) % 4096;

            if next == self.in_tail.load(Ordering::Acquire) {
                break; // Buffer full
            }

            self.in_buffer[head as usize] = byte;
            self.in_head.store(next, Ordering::Release);
            count += 1;
        }

        count
    }

    /// Read input buffer
    pub fn output(&mut self, buf: &mut [u8]) -> u32 {
        let mut count = 0u32;

        for byte in buf.iter_mut() {
            let tail = self.in_tail.load(Ordering::Acquire);

            if tail == self.in_head.load(Ordering::Acquire) {
                break; // Buffer empty
            }

            *byte = self.in_buffer[tail as usize];
            self.in_tail.store((tail + 1) % 4096, Ordering::Release);
            count += 1;
        }

        count
    }
}

/// Character device manager
pub struct CharDeviceManager {
    /// Device count
    pub device_count: AtomicU32,
    /// Next device ID
    pub next_dev_id: AtomicU32,
    /// Total read count
    pub total_reads: AtomicU64,
    /// Total write count
    pub total_writes: AtomicU64,
}

impl CharDeviceManager {
    pub const fn new() -> Self {
        CharDeviceManager {
            device_count: AtomicU32::new(0),
            next_dev_id: AtomicU32::new(1),
            total_reads: AtomicU64::new(0),
            total_writes: AtomicU64::new(0),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Char device manager initialized");
    }

    /// RegisterDevice
    pub fn register(&mut self, _dev: &mut CharDevice) -> CharDeviceId {
        let dev_id = self.next_dev_id.fetch_add(1, Ordering::AcqRel);
        self.device_count.fetch_add(1, Ordering::AcqRel);

        log_info!("Registered char device: dev_id={}", dev_id);

        dev_id
    }

    /// Get device count
    pub fn get_device_count(&self) -> u32 {
        self.device_count.load(Ordering::Acquire)
    }
}

/// Global character device manager
static CHAR_DEVICE_MANAGER: core::sync::OnceLock<CharDeviceManager> = core::sync::OnceLock::new();

pub fn char_device_manager() -> &'static CharDeviceManager {
    CHAR_DEVICE_MANAGER.get_or_init(CharDeviceManager::new)
}

pub fn init_char_device_manager() -> &'static CharDeviceManager {
    CHAR_DEVICE_MANAGER.get_or_init(CharDeviceManager::new)
}

pub fn init_char_device() {
    let mgr = char_device_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_flags() {
        assert_eq!(char_flags::READABLE, 1 << 0);
        assert_eq!(char_flags::WRITABLE, 1 << 1);
        assert_eq!(char_flags::NONBLOCK, 1 << 2);
        assert_eq!(char_flags::EXCLUSIVE, 1 << 3);
        assert_eq!(char_flags::TTY, 1 << 4);
        assert_eq!(char_flags::SERIAL, 1 << 5);
        assert_eq!(char_flags::CONSOLE, 1 << 6);
        assert_eq!(char_flags::RANDOM, 1 << 7);
    }

    #[test]
    fn test_char_device_new() {
        let dev = CharDevice::new(1, 1, 0, b"console");

        assert_eq!(dev.dev_id, 1);
        assert_eq!(dev.major, 1);
        assert_eq!(dev.minor, 0);
        assert_eq!(dev.get_name(), b"console");
        assert!(dev.is_readable());
        assert!(dev.is_writable());
    }

    #[test]
    fn test_char_device_name() {
        let dev = CharDevice::new(1, 1, 0, b"ttyS0");

        assert_eq!(dev.get_name(), b"ttyS0");
    }

    #[test]
    fn test_char_device_flags() {
        let dev = CharDevice::new(1, 1, 0, b"test");

        assert!(dev.is_readable());
        assert!(dev.is_writable());
        assert!(!dev.is_nonblock());
        assert!(!dev.is_tty());
        assert!(!dev.is_console());

        // Set non-blocking
        dev.flags.fetch_or(char_flags::NONBLOCK, Ordering::Relaxed);
        assert!(dev.is_nonblock());

        // Set terminal
        dev.flags.fetch_or(char_flags::TTY, Ordering::Relaxed);
        assert!(dev.is_tty());

        // Set console
        dev.flags.fetch_or(char_flags::CONSOLE, Ordering::Relaxed);
        assert!(dev.is_console());
    }

    #[test]
    fn test_char_device_ref_count() {
        let dev = CharDevice::new(1, 1, 0, b"test");

        assert_eq!(dev.ref_count.load(Ordering::Relaxed), 0);

        dev.get();
        assert_eq!(dev.ref_count.load(Ordering::Relaxed), 1);

        dev.get();
        assert_eq!(dev.ref_count.load(Ordering::Relaxed), 2);

        dev.put();
        assert_eq!(dev.ref_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_char_device_open_without_ops() {
        let dev = CharDevice::new(1, 1, 0, b"test");

        let result = dev.open();
        assert_eq!(result, -1);
    }

    #[test]
    fn test_char_device_close_without_ops() {
        let dev = CharDevice::new(1, 1, 0, b"test");

        let result = dev.close();
        assert_eq!(result, -1);
    }

    #[test]
    fn test_char_device_read_without_ops() {
        let dev = CharDevice::new(1, 1, 0, b"test");
        let mut buf = [0u8; 100];

        let result = dev.read(&mut buf);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_char_device_write_without_ops() {
        let dev = CharDevice::new(1, 1, 0, b"test");
        let buf = [0u8; 100];

        let result = dev.write(&buf);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_char_device_poll_without_ops() {
        let dev = CharDevice::new(1, 1, 0, b"test");

        let result = dev.poll(0);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_char_device_ioctl_without_ops() {
        let dev = CharDevice::new(1, 1, 0, b"test");

        let result = dev.ioctl(0, 0);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_char_device_stats() {
        let dev = CharDevice::new(1, 1, 0, b"test");

        assert_eq!(dev.read_count.load(Ordering::Relaxed), 0);
        assert_eq!(dev.write_count.load(Ordering::Relaxed), 0);
        assert_eq!(dev.read_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(dev.write_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(dev.open_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_tty_device_new() {
        let tty = TtyDevice::new(1, 0, b"tty0");

        assert_eq!(tty.dev.dev_id, 1);
        assert_eq!(tty.dev.major, 5); // TTY major device number
        assert_eq!(tty.dev.minor, 0);
        assert_eq!(tty.dev.get_name(), b"tty0");
    }

    #[test]
    fn test_tty_device_buffers() {
        let tty = TtyDevice::new(1, 0, b"tty0");

        // Buffer initialized as empty
        assert_eq!(tty.in_head.load(Ordering::Relaxed), 0);
        assert_eq!(tty.in_tail.load(Ordering::Relaxed), 0);
        assert_eq!(tty.out_head.load(Ordering::Relaxed), 0);
        assert_eq!(tty.out_tail.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_tty_device_echo() {
        let tty = TtyDevice::new(1, 0, b"tty0");

        // Default echo enabled
        assert_eq!(tty.echo.load(Ordering::Relaxed), 1);
        // Default canonical mode
        assert_eq!(tty.canonical.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_tty_device_input() {
        let mut tty = TtyDevice::new(1, 0, b"tty0");

        let data = b"hello";
        let count = tty.input(data);

        assert_eq!(count, 5);
        assert_eq!(tty.in_head.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_tty_device_output() {
        let mut tty = TtyDevice::new(1, 0, b"tty0");

        // First input some data
        tty.input(b"hello");

        // Then read
        let mut buf = [0u8; 10];
        let count = tty.output(&mut buf);

        assert_eq!(count, 5);
        assert_eq!(&buf[..5], b"hello");
        assert_eq!(tty.in_tail.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_tty_device_input_output_fifo() {
        let mut tty = TtyDevice::new(1, 0, b"tty0");

        // InputData
        tty.input(b"abc");
        tty.input(b"def");

        // ReadData
        let mut buf = [0u8; 10];
        let count = tty.output(&mut buf);

        assert_eq!(count, 6);
        assert_eq!(&buf[..6], b"abcdef");
    }

    #[test]
    fn test_tty_device_empty_output() {
        let mut tty = TtyDevice::new(1, 0, b"tty0");

        let mut buf = [0u8; 10];
        let count = tty.output(&mut buf);

        assert_eq!(count, 0);
    }

    #[test]
    fn test_char_device_manager_new() {
        let mgr = CharDeviceManager::new();

        assert_eq!(mgr.get_device_count(), 0);
        assert_eq!(mgr.total_reads.load(Ordering::Relaxed), 0);
        assert_eq!(mgr.total_writes.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_char_device_manager_register() {
        let mut mgr = CharDeviceManager::new();
        let mut dev = CharDevice::new(0, 1, 0, b"console");

        let id1 = mgr.register(&mut dev);
        assert_eq!(id1, 1);
        assert_eq!(mgr.get_device_count(), 1);

        let id2 = mgr.register(&mut dev);
        assert_eq!(id2, 2);
        assert_eq!(mgr.get_device_count(), 2);
    }

    #[test]
    fn test_tty_device_buffer_wrap() {
        let mut tty = TtyDevice::new(1, 0, b"tty0");

        // Fill buffer
        for _ in 0..4095 {
            tty.input(b"x");
        }

        // Buffer should be full
        let count = tty.input(b"y");
        assert_eq!(count, 0);
    }
}
