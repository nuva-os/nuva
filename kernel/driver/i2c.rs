/*
 * Nuva OS - Kernel - Driver - I2c
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
 * Nuva OS - Kernel - I2C Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * I2C bus management for device drivers.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// I2C Address
pub type I2cAddr = u16;

/// I2C Speed Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cSpeed {
    /// Standard mode (100 kHz)
    Standard = 100_000,
    /// Fast mode (400 kHz)
    Fast = 400_000,
    /// Fast Plus mode (1 MHz)
    FastPlus = 1_000_000,
    /// High Speed mode (3.4 MHz)
    HighSpeed = 3_400_000,
}

/// I2C Transfer Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct I2cFlags: u32 {
        /// Write operation
        const WRITE = 0;
        /// Read operation
        const READ = 1 << 0;
        /// 10-bit address
        const ADDR_10BIT = 1 << 1;
        /// Stop condition
        const STOP = 1 << 2;
        /// No start condition
        const NOSTART = 1 << 3;
        /// Repeated start
        const RESTART = 1 << 4;
        /// Ignore NACK
        const IGNORE_NACK = 1 << 5;
        /// Generate PEC
        const PEC = 1 << 6;
        /// DMA safe
        const DMA_SAFE = 1 << 7;
    }
}

/// I2C Message
#[repr(C)]
pub struct I2cMsg {
    /// Slave address
    pub addr: I2cAddr,
    /// Flags
    pub flags: I2cFlags,
    /// Buffer
    pub buf: *mut u8,
    /// Buffer length
    pub len: u16,
}

impl I2cMsg {
    /// Create write message
    pub fn write(addr: I2cAddr, buf: *mut u8, len: u16) -> Self {
        I2cMsg {
            addr,
            flags: I2cFlags::WRITE,
            buf,
            len,
        }
    }

    /// Create read message
    pub fn read(addr: I2cAddr, buf: *mut u8, len: u16) -> Self {
        I2cMsg {
            addr,
            flags: I2cFlags::READ,
            buf,
            len,
        }
    }
}

/// I2C Transfer Result
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2cResult {
    /// Success
    Ok = 0,
    /// NACK received
    Nack = 1,
    /// Timeout
    Timeout = 2,
    /// Bus error
    BusError = 3,
    /// Arbitration lost
    ArbitrationLost = 4,
    /// Unsupported
    Unsupported = 5,
}

/// I2C Algorithm Operations
pub struct I2cAlgo {
    /// Master transfer
    pub master_xfer: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut I2cMsg, i32) -> i32>,
    /// SMBus transfer
    pub smbus_xfer:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, I2cAddr, u8, u8, i32, *mut u8) -> i32>,
    /// Functionality
    pub functionality: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
}

/// I2C Functionality Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct I2cFunc: u32 {
        /// I2C
        const I2C = 1 << 0;
        /// 10-bit address
        const ADDR_10BIT = 1 << 1;
        /// Protocol mangling
        const PROTOCOL_MANGLING = 1 << 2;
        /// SMBus PEC
        const SMBUS_PEC = 1 << 3;
        /// NOSTART
        const NOSTART = 1 << 4;
        /// Slave
        const SLAVE = 1 << 5;
        /// SMBus Block
        const SMBUS_BLOCK = 1 << 6;
    }
}

/// I2C Adapter
pub struct I2cAdapter {
    /// Adapter name
    pub name: [u8; 32],
    /// Adapter ID
    pub id: u32,
    /// Algorithm
    pub algo: I2cAlgo,
    /// Algorithm data
    pub algo_data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// Bus number
    pub nr: i32,
    /// Timeout (ms)
    pub timeout: u32,
    /// Retries
    pub retries: u8,
    /// Bus frequency
    pub frequency: u32,
    /// Use count
    pub use_count: AtomicU32,
}

/// I2C Client (Device)
#[repr(C)]
pub struct I2cClient {
    /// Client name
    pub name: [u8; 32],
    /// Adapter ID
    pub adapter_id: u32,
    /// Address
    pub addr: I2cAddr,
    /// Flags
    pub flags: I2cFlags,
    /// Driver data
    pub driver_data: *mut core::ffi::c_void,
    /// IRQ
    pub irq: i32,
}

/// I2C Manager
pub struct I2cManager {
    /// Adapter count
    adapter_count: AtomicU32,
    /// Statistics
    stats: I2cStats,
}

/// I2C Statistics
pub struct I2cStats {
    /// Transfer count
    pub xfer_count: AtomicU64,
    /// Byte count
    pub byte_count: AtomicU64,
    /// Error count
    pub error_count: AtomicU64,
}

impl I2cStats {
    pub const fn new() -> Self {
        I2cStats {
            xfer_count: AtomicU64::new(0),
            byte_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }
}

impl I2cManager {
    pub const fn new() -> Self {
        I2cManager {
            adapter_count: AtomicU32::new(0),
            stats: I2cStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("I2C manager initialized");
    }

    /// Register adapter
    pub fn register_adapter(&mut self, _adap: &I2cAdapter) -> u32 {
        let id = self.adapter_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Transfer
    pub fn transfer(&mut self, adap_id: u32, msgs: &mut [I2cMsg]) -> i32 {
        self.stats.xfer_count.fetch_add(1, Ordering::AcqRel);

        let mut total_bytes = 0u64;
        for msg in msgs.iter() {
            total_bytes += msg.len as u64;
        }
        self.stats
            .byte_count
            .fetch_add(total_bytes, Ordering::AcqRel);

        log_debug!("i2c_transfer: adap={}, msgs={}", adap_id, msgs.len());

        // TODO: Call adapter's master_xfer
        msgs.len() as i32
    }

    /// SMBus read byte
    pub fn smbus_read_byte(&mut self, adap_id: u32, addr: I2cAddr) -> i32 {
        log_debug!("i2c_smbus_read_byte: adap={}, addr={:#x}", adap_id, addr);
        0
    }

    /// SMBus write byte
    pub fn smbus_write_byte(&mut self, adap_id: u32, addr: I2cAddr, value: u8) -> i32 {
        log_debug!(
            "i2c_smbus_write_byte: adap={}, addr={:#x}, value={:#x}",
            adap_id,
            addr,
            value
        );
        0
    }

    /// SMBus read byte data
    pub fn smbus_read_byte_data(&mut self, adap_id: u32, addr: I2cAddr, reg: u8) -> i32 {
        log_debug!(
            "i2c_smbus_read_byte_data: adap={}, addr={:#x}, reg={:#x}",
            adap_id,
            addr,
            reg
        );
        0
    }

    /// SMBus write byte data
    pub fn smbus_write_byte_data(
        &mut self,
        adap_id: u32,
        addr: I2cAddr,
        reg: u8,
        value: u8,
    ) -> i32 {
        log_debug!(
            "i2c_smbus_write_byte_data: adap={}, addr={:#x}, reg={:#x}, value={:#x}",
            adap_id,
            addr,
            reg,
            value
        );
        0
    }

    /// SMBus read word data
    pub fn smbus_read_word_data(&mut self, adap_id: u32, addr: I2cAddr, reg: u8) -> i32 {
        log_debug!(
            "i2c_smbus_read_word_data: adap={}, addr={:#x}, reg={:#x}",
            adap_id,
            addr,
            reg
        );
        0
    }

    /// SMBus write word data
    pub fn smbus_write_word_data(
        &mut self,
        adap_id: u32,
        addr: I2cAddr,
        reg: u8,
        value: u16,
    ) -> i32 {
        log_debug!(
            "i2c_smbus_write_word_data: adap={}, addr={:#x}, reg={:#x}, value={:#x}",
            adap_id,
            addr,
            reg,
            value
        );
        0
    }
}

/// Global I2C manager
static I2C_MANAGER: crate::sync_oncelock::OnceLock<I2cManager> = crate::sync_oncelock::OnceLock::new();

/// Get I2C manager
pub fn i2c_manager() -> &'static I2cManager {
    I2C_MANAGER.get_or_init(I2cManager::new)
}

/// Initialize I2C manager
pub fn init_i2c_manager() {
    let mgr = i2c_manager();
    mgr.init();
}

// Convenience functions

/// I2C transfer
pub fn i2c_transfer(adap_id: u32, msgs: &mut [I2cMsg]) -> i32 {
    i2c_manager().transfer(adap_id, msgs)
}

/// SMBus read byte
pub fn i2c_smbus_read_byte(adap_id: u32, addr: I2cAddr) -> i32 {
    i2c_manager().smbus_read_byte(adap_id, addr)
}

/// SMBus write byte
pub fn i2c_smbus_write_byte(adap_id: u32, addr: I2cAddr, value: u8) -> i32 {
    i2c_manager().smbus_write_byte(adap_id, addr, value)
}

/// SMBus read byte data
pub fn i2c_smbus_read_byte_data(adap_id: u32, addr: I2cAddr, reg: u8) -> i32 {
    i2c_manager().smbus_read_byte_data(adap_id, addr, reg)
}

/// SMBus write byte data
pub fn i2c_smbus_write_byte_data(adap_id: u32, addr: I2cAddr, reg: u8, value: u8) -> i32 {
    i2c_manager().smbus_write_byte_data(adap_id, addr, reg, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i2c_speed() {
        assert_eq!(I2cSpeed::Standard as i32, 100_000);
        assert_eq!(I2cSpeed::Fast as i32, 400_000);
        assert_eq!(I2cSpeed::HighSpeed as i32, 3_400_000);
    }

    #[test]
    fn test_i2c_flags() {
        let flags = I2cFlags::READ | I2cFlags::STOP;
        assert!(flags.contains(I2cFlags::READ));
        assert!(flags.contains(I2cFlags::STOP));
    }

    #[test]
    fn test_i2c_msg() {
        let msg = I2cMsg::write(0x50, core::ptr::null_mut(), 10);
        assert_eq!(msg.addr, 0x50);
        assert_eq!(msg.len, 10);
        assert!(msg.flags.contains(I2cFlags::WRITE));

        let msg = I2cMsg::read(0x50, core::ptr::null_mut(), 10);
        assert!(msg.flags.contains(I2cFlags::READ));
    }
}
