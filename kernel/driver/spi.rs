/*
 * Nuva OS - Kernel - Driver - Spi
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
 * Nuva OS - Kernel - SPI Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * SPI bus management for device drivers.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// SPI Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    /// Mode 0: CPOL=0, CPHA=0
    Mode0 = 0,
    /// Mode 1: CPOL=0, CPHA=1
    Mode1 = 1,
    /// Mode 2: CPOL=1, CPHA=0
    Mode2 = 2,
    /// Mode 3: CPOL=1, CPHA=1
    Mode3 = 3,
}

/// SPI Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct SpiFlags: u32 {
        /// LSB first
        const LSB_FIRST = 1 << 0;
        /// 3-wire (half duplex)
        const THREE_WIRE = 1 << 1;
        /// Loopback
        const LOOP = 1 << 2;
        /// No chip select
        const NO_CS = 1 << 3;
        /// Chip select high
        const CS_HIGH = 1 << 4;
        /// TX single
        const TX_SINGLE = 1 << 5;
        /// TX dual
        const TX_DUAL = 1 << 6;
        /// TX quad
        const TX_QUAD = 1 << 7;
        /// RX single
        const RX_SINGLE = 1 << 8;
        /// RX dual
        const RX_DUAL = 1 << 9;
        /// RX quad
        const RX_QUAD = 1 << 10;
    }
}

/// SPI Transfer
#[repr(C)]
pub struct SpiTransfer {
    /// TX buffer
    pub tx_buf: *const u8,
    /// RX buffer
    pub rx_buf: *mut u8,
    /// Length
    pub len: usize,
    /// Speed (Hz), 0 = use default
    pub speed_hz: u32,
    /// Bits per word, 0 = use default
    pub bits_per_word: u8,
    /// Delay after transfer (us)
    pub delay_usecs: u16,
    /// CS change
    pub cs_change: bool,
    /// TX nbits (1, 2, 4)
    pub tx_nbits: u8,
    /// RX nbits (1, 2, 4)
    pub rx_nbits: u8,
}

impl SpiTransfer {
    /// Create write transfer
    pub const fn write(tx_buf: *const u8, len: usize) -> Self {
        SpiTransfer {
            tx_buf,
            rx_buf: core::ptr::null_mut(),
            len,
            speed_hz: 0,
            bits_per_word: 0,
            delay_usecs: 0,
            cs_change: false,
            tx_nbits: 1,
            rx_nbits: 1,
        }
    }

    /// Create read transfer
    pub fn read(rx_buf: *mut u8, len: usize) -> Self {
        SpiTransfer {
            tx_buf: core::ptr::null(),
            rx_buf,
            len,
            speed_hz: 0,
            bits_per_word: 0,
            delay_usecs: 0,
            cs_change: false,
            tx_nbits: 1,
            rx_nbits: 1,
        }
    }

    /// Create read/write transfer
    pub fn read_write(tx_buf: *const u8, rx_buf: *mut u8, len: usize) -> Self {
        SpiTransfer {
            tx_buf,
            rx_buf,
            len,
            speed_hz: 0,
            bits_per_word: 0,
            delay_usecs: 0,
            cs_change: false,
            tx_nbits: 1,
            rx_nbits: 1,
        }
    }
}

/// SPI Device Configuration
#[repr(C)]
pub struct SpiDeviceConfig {
    /// SPI mode
    pub mode: SpiMode,
    /// Maximum speed (Hz)
    pub max_speed_hz: u32,
    /// Bits per word
    pub bits_per_word: u8,
    /// Flags
    pub flags: SpiFlags,
    /// Chip select
    pub chip_select: u8,
}

impl Default for SpiDeviceConfig {
    fn default() -> Self {
        SpiDeviceConfig {
            mode: SpiMode::Mode0,
            max_speed_hz: 1_000_000,
            bits_per_word: 8,
            flags: SpiFlags::empty(),
            chip_select: 0,
        }
    }
}

/// SPI Controller Operations
pub struct SpiControllerOps {
    /// Setup device
    pub setup: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut SpiDeviceConfig) -> i32>,
    /// Cleanup device
    pub cleanup: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const SpiDeviceConfig)>,
    /// Transfer one
    pub transfer_one: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut SpiTransfer) -> i32>,
    /// Transfer one message
    pub transfer_one_message:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut SpiMessage) -> i32>,
    /// Prepare transfer hardware
    pub prepare_transfer_hardware: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Unprepare transfer hardware
    pub unprepare_transfer_hardware: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Set CS
    pub set_cs: Option<unsafe extern "C" fn(*mut core::ffi::c_void, bool, u8)>,
}

/// SPI Message (collection of transfers)
pub struct SpiMessage {
    /// Transfers
    pub transfers: [SpiTransfer; 16],
    /// Number of transfers
    pub num_transfers: u8,
    /// Status
    pub status: i32,
    /// Actual length
    pub actual_length: usize,
    /// Is DMA mapped
    pub is_dma_mapped: bool,
}

impl SpiMessage {
    pub fn new() -> Self {
        SpiMessage {
            transfers: [const { SpiTransfer::write(core::ptr::null(), 0) }; 16],
            num_transfers: 0,
            status: 0,
            actual_length: 0,
            is_dma_mapped: false,
        }
    }

    /// Add transfer
    pub fn add_transfer(&mut self, xfer: SpiTransfer) -> bool {
        if self.num_transfers as usize >= self.transfers.len() {
            return false;
        }
        self.transfers[self.num_transfers as usize] = xfer;
        self.num_transfers += 1;
        true
    }
}

/// SPI Controller (Master)
pub struct SpiController {
    /// Controller name
    pub name: [u8; 32],
    /// Controller ID
    pub id: u32,
    /// Bus number
    pub bus_num: i32,
    /// Number of chip selects
    pub num_chipselect: u16,
    /// Operations
    pub ops: SpiControllerOps,
    /// Controller data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// Maximum speed
    pub max_speed_hz: u32,
    /// Minimum speed
    pub min_speed_hz: u32,
    /// Bits per word mask
    pub bits_per_word_mask: u32,
    /// Flags
    pub flags: SpiFlags,
    /// Mode bits supported
    pub mode_bits: u8,
    /// Use count
    pub use_count: AtomicU32,
}

/// SPI Device
#[repr(C)]
pub struct SpiDevice {
    /// Device name
    pub name: [u8; 32],
    /// Controller ID
    pub controller_id: u32,
    /// Chip select
    pub chip_select: u8,
    /// Configuration
    pub config: SpiDeviceConfig,
    /// Driver data
    pub driver_data: *mut core::ffi::c_void,
    /// IRQ
    pub irq: i32,
    /// Max speed
    pub max_speed_hz: u32,
}

/// SPI Manager
pub struct SpiManager {
    /// Controller count
    controller_count: AtomicU32,
    /// Statistics
    stats: SpiStats,
}

/// SPI Statistics
pub struct SpiStats {
    /// Transfer count
    pub xfer_count: AtomicU64,
    /// Byte count
    pub byte_count: AtomicU64,
    /// Error count
    pub error_count: AtomicU64,
}

impl SpiStats {
    pub fn new() -> Self {
        SpiStats {
            xfer_count: AtomicU64::new(0),
            byte_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }
}

impl SpiManager {
    pub fn new() -> Self {
        SpiManager {
            controller_count: AtomicU32::new(0),
            stats: SpiStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("SPI manager initialized");
    }

    /// Register controller
    pub fn register_controller(&mut self, _ctrl: &SpiController) -> u32 {
        let id = self.controller_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Transfer
    pub fn transfer(&mut self, dev: &SpiDevice, xfers: &[SpiTransfer]) -> i32 {
        self.stats.xfer_count.fetch_add(1, Ordering::AcqRel);

        let mut total_bytes = 0u64;
        for xfer in xfers.iter() {
            total_bytes += xfer.len as u64;
        }
        self.stats
            .byte_count
            .fetch_add(total_bytes, Ordering::AcqRel);

        log_debug!(
            "spi_transfer: dev={}, xfers={}",
            dev.controller_id,
            xfers.len()
        );

        // TODO: Call controller's transfer
        xfers.len() as i32
    }

    /// Write then read
    pub fn write_then_read(&mut self, dev: &SpiDevice, tx_buf: &[u8], rx_buf: &mut [u8]) -> i32 {
        log_debug!(
            "spi_write_then_read: dev={}, tx_len={}, rx_len={}",
            dev.controller_id,
            tx_buf.len(),
            rx_buf.len()
        );

        let xfers = [
            SpiTransfer::write(tx_buf.as_ptr(), tx_buf.len()),
            SpiTransfer::read(rx_buf.as_mut_ptr(), rx_buf.len()),
        ];

        self.transfer(dev, &xfers)
    }

    /// Write
    pub fn write(&mut self, dev: &SpiDevice, buf: &[u8]) -> i32 {
        log_debug!("spi_write: dev={}, len={}", dev.controller_id, buf.len());

        let xfer = SpiTransfer::write(buf.as_ptr(), buf.len());
        self.transfer(dev, &[xfer])
    }

    /// Read
    pub fn read(&mut self, dev: &SpiDevice, buf: &mut [u8]) -> i32 {
        log_debug!("spi_read: dev={}, len={}", dev.controller_id, buf.len());

        let xfer = SpiTransfer::read(buf.as_mut_ptr(), buf.len());
        self.transfer(dev, &[xfer])
    }
}

/// Global SPI manager
static SPI_MANAGER: crate::sync_oncelock::OnceLock<SpiManager> = crate::sync_oncelock::OnceLock::new();

/// Get SPI manager
pub fn get_spi_manager() -> &'static mut SpiManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { SPI_MANAGER.assume_init_mut() }
}

/// Initialize SPI manager
pub fn init_spi_manager() {
    // SAFETY: SPI_MANAGER is only written here during init
    unsafe {
        SPI_MANAGER.write(SpiManager::new());
    }
    let mgr = get_spi_manager();
    mgr.init();
}

// Convenience functions

/// SPI transfer
pub fn spi_transfer(dev: &SpiDevice, xfers: &[SpiTransfer]) -> i32 {
    get_spi_manager().transfer(dev, xfers)
}

/// SPI write
pub fn spi_write(dev: &SpiDevice, buf: &[u8]) -> i32 {
    get_spi_manager().write(dev, buf)
}

/// SPI read
pub fn spi_read(dev: &SpiDevice, buf: &mut [u8]) -> i32 {
    get_spi_manager().read(dev, buf)
}

/// SPI write then read
pub fn spi_write_then_read(dev: &SpiDevice, tx_buf: &[u8], rx_buf: &mut [u8]) -> i32 {
    get_spi_manager().write_then_read(dev, tx_buf, rx_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spi_mode() {
        assert_eq!(SpiMode::Mode0 as i32, 0);
        assert_eq!(SpiMode::Mode1 as i32, 1);
        assert_eq!(SpiMode::Mode2 as i32, 2);
        assert_eq!(SpiMode::Mode3 as i32, 3);
    }

    #[test]
    fn test_spi_flags() {
        let flags = SpiFlags::LSB_FIRST | SpiFlags::CS_HIGH;
        assert!(flags.contains(SpiFlags::LSB_FIRST));
        assert!(flags.contains(SpiFlags::CS_HIGH));
    }

    #[test]
    fn test_spi_transfer() {
        let xfer = SpiTransfer::write(core::ptr::null(), 100);
        assert_eq!(xfer.len, 100);
        assert!(!xfer.cs_change);

        let xfer = SpiTransfer::read(core::ptr::null_mut(), 100);
        assert_eq!(xfer.len, 100);
    }

    #[test]
    fn test_spi_message() {
        let mut msg = SpiMessage::new();
        assert_eq!(msg.num_transfers, 0);

        let xfer = SpiTransfer::write(core::ptr::null(), 10);
        assert!(msg.add_transfer(xfer));
        assert_eq!(msg.num_transfers, 1);
    }

    #[test]
    fn test_spi_device_config() {
        let config = SpiDeviceConfig::default();
        assert_eq!(config.mode, SpiMode::Mode0);
        assert_eq!(config.bits_per_word, 8);
    }
}
