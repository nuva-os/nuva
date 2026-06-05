/*
 * Nuva OS - Kernel - Driver - Adapter - CAbi
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
 * Nuva OS - Kernel - C Driver Adapter
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Provides C ABI compatible interface for vendor drivers.
 * Enables seamless integration of C library drivers.
 */

use crate::{pr_debug, pr_warn};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// DDF ABI Version - must match between kernel and driver
pub const DDF_ABI_VERSION: u32 = 1;

/// Maximum driver name length
pub const MAX_DRIVER_NAME: usize = 64;

/// Maximum device name length
pub const MAX_DEVICE_NAME: usize = 32;

/// C Driver Information
/// Provided by the driver to describe itself.
#[repr(C)]
pub struct CDriverInfo {
    /// Driver name
    pub name: [u8; MAX_DRIVER_NAME],
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID
    pub device_id: u16,
    /// Driver version (major << 16 | minor << 8 | patch)
    pub version: u32,
    /// ABI version (must match DDF_ABI_VERSION)
    pub abi_version: u32,
    /// Supported device class
    pub device_class: CDeviceClass,
    /// Driver operations
    pub ops: CDriverOps,
}

/// C Device Class Enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CDeviceClass {
    /// Touch screen
    Touch = 0,
    /// Audio output
    Audio = 1,
    /// Microphone
    Mic = 2,
    /// Sensor
    Sensor = 3,
    /// Power/Battery
    Power = 4,
    /// Keyboard
    Keyboard = 5,
    /// Mouse
    Mouse = 6,
    /// USB
    Usb = 7,
    /// Type-C
    TypeC = 8,
    /// Display
    Display = 9,
    /// Camera
    Camera = 10,
    /// Bluetooth
    Bluetooth = 11,
    /// WiFi
    Wifi = 12,
    /// GPS
    Gps = 13,
    /// Custom device
    Custom = 255,
}

/// C Driver Operations
/// Function pointers for driver operations.
/// All functions use C calling convention.
#[repr(C)]
pub struct CDriverOps {
    /// Initialize driver (called once at load)
    pub init: Option<unsafe extern "C" fn() -> i32>,
    /// Cleanup driver (called at unload)
    pub cleanup: Option<unsafe extern "C" fn()>,

    /// Probe device (return 0 if device is supported)
    pub probe: Option<unsafe extern "C" fn(*const CDeviceId) -> i32>,
    /// Remove device
    pub remove: Option<unsafe extern "C" fn(*mut CDeviceContext) -> i32>,

    /// Open device
    pub open: Option<unsafe extern "C" fn(*mut CDeviceContext) -> i32>,
    /// Close device
    pub close: Option<unsafe extern "C" fn(*mut CDeviceContext) -> i32>,
    /// Read from device
    pub read: Option<unsafe extern "C" fn(*mut CDeviceContext, *mut u8, usize, *mut usize) -> i32>,
    /// Write to device
    pub write:
        Option<unsafe extern "C" fn(*mut CDeviceContext, *const u8, usize, *mut usize) -> i32>,
    /// I/O control
    pub ioctl: Option<unsafe extern "C" fn(*mut CDeviceContext, u32, u64) -> i32>,
    /// Memory map
    pub mmap: Option<unsafe extern "C" fn(*mut CDeviceContext, *mut CDmaBuffer) -> i32>,
    /// Poll for events
    pub poll: Option<unsafe extern "C" fn(*mut CDeviceContext, u32) -> u32>,

    /// Suspend device
    pub suspend: Option<unsafe extern "C" fn(*mut CDeviceContext) -> i32>,
    /// Resume device
    pub resume: Option<unsafe extern "C" fn(*mut CDeviceContext) -> i32>,
}

/// C Device ID
/// Used for device matching.
#[repr(C)]
pub struct CDeviceId {
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID
    pub device_id: u16,
    /// Subsystem vendor ID
    pub subvendor_id: u16,
    /// Subsystem device ID
    pub subdevice_id: u16,
    /// Class code
    pub class: u32,
    /// Compatible string (device tree)
    pub compatible: [u8; 64],
}

/// C Device Context
/// Passed to driver operations, contains device state.
#[repr(C)]
pub struct CDeviceContext {
    /// Driver private data
    pub priv_data: *mut core::ffi::c_void,
    /// Device register base address (MMIO)
    pub reg_base: u64,
    /// Register size
    pub reg_size: usize,
    /// Interrupt number
    pub irq: u32,
    /// Device name
    pub name: [u8; MAX_DEVICE_NAME],
    /// Device class
    pub device_class: CDeviceClass,
    /// Kernel callback table
    pub callbacks: CCallbackTable,
    /// Device flags
    pub flags: u32,
    /// Reserved
    pub reserved: [u64; 4],
}

/// C Callback Table
/// Functions that drivers can call to request kernel services.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CCallbackTable {
    // Memory management
    /// Allocate memory
    pub mem_alloc: unsafe extern "C" fn(usize) -> *mut core::ffi::c_void,
    /// Free memory
    pub mem_free: unsafe extern "C" fn(*mut core::ffi::c_void),
    /// Allocate DMA buffer
    pub dma_alloc: unsafe extern "C" fn(usize, usize, *mut u64) -> *mut core::ffi::c_void,
    /// Free DMA buffer
    pub dma_free: unsafe extern "C" fn(*mut core::ffi::c_void, usize, u64),

    // Interrupt management
    /// Request interrupt handler
    pub request_irq: unsafe extern "C" fn(
        u32,
        unsafe extern "C" fn(u32, *mut core::ffi::c_void),
        *mut core::ffi::c_void,
    ) -> i32,
    /// Free interrupt handler
    pub free_irq: unsafe extern "C" fn(u32, *mut core::ffi::c_void),
    /// Enable interrupt
    pub enable_irq: unsafe extern "C" fn(u32),
    /// Disable interrupt
    pub disable_irq: unsafe extern "C" fn(u32),

    // I/O operations
    /// Read MMIO register (8-bit)
    pub mmio_read8: unsafe extern "C" fn(u64) -> u8,
    /// Read MMIO register (16-bit)
    pub mmio_read16: unsafe extern "C" fn(u64) -> u16,
    /// Read MMIO register (32-bit)
    pub mmio_read32: unsafe extern "C" fn(u64) -> u32,
    /// Read MMIO register (64-bit)
    pub mmio_read64: unsafe extern "C" fn(u64) -> u64,
    /// Write MMIO register (8-bit)
    pub mmio_write8: unsafe extern "C" fn(u64, u8),
    /// Write MMIO register (16-bit)
    pub mmio_write16: unsafe extern "C" fn(u64, u16),
    /// Write MMIO register (32-bit)
    pub mmio_write32: unsafe extern "C" fn(u64, u32),
    /// Write MMIO register (64-bit)
    pub mmio_write64: unsafe extern "C" fn(u64, u64),

    // Logging
    /// Log message
    pub log_print: unsafe extern "C" fn(i32, *const i8),

    // Event notification
    /// Notify event to subscribers
    pub notify_event:
        unsafe extern "C" fn(*mut CDeviceContext, u32, *const core::ffi::c_void, usize),

    // Time
    /// Get current time in nanoseconds
    pub get_time_ns: unsafe extern "C" fn() -> u64,
    /// Sleep for microseconds
    pub usleep: unsafe extern "C" fn(u32),
    /// Sleep for milliseconds
    pub msleep: unsafe extern "C" fn(u32),
}

/// C DMA Buffer
#[repr(C)]
pub struct CDmaBuffer {
    /// Virtual address
    pub vaddr: *mut u8,
    /// Physical address
    pub paddr: u64,
    /// DMA address (device view)
    pub dma_addr: u64,
    /// Size
    pub size: usize,
    /// Attributes
    pub attr: u32,
}

/// C Driver Adapter
/// Wraps a C library driver and provides Rust interface.
pub struct CDriverAdapter {
    /// Driver information
    pub info: CDriverInfo,
    /// Dynamic library handle (if loaded from .so)
    pub dl_handle: u64,
    /// Number of active devices
    pub device_count: AtomicU32,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Callback table
    pub callbacks: CCallbackTable,
}

impl CDriverAdapter {
    /// Create adapter from driver info
    pub fn from_info(info: CDriverInfo) -> Result<Self, i32> {
        // Check ABI version
        if info.abi_version != DDF_ABI_VERSION {
            log_warn!(
                "Driver ABI version mismatch: expected {}, got {}",
                DDF_ABI_VERSION,
                info.abi_version
            );
            return Err(-22); // EINVAL
        }

        Ok(CDriverAdapter {
            info,
            dl_handle: 0,
            device_count: AtomicU32::new(0),
            ref_count: AtomicU32::new(1),
            callbacks: create_callback_table(),
        })
    }

    /// Initialize the driver
    pub fn init(&self) -> i32 {
        if let Some(init) = self.info.ops.init {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { init() }
        } else {
            0
        }
    }

    /// Cleanup the driver
    pub fn cleanup(&self) {
        if let Some(cleanup) = self.info.ops.cleanup {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { cleanup() }
        }
    }

    /// Probe a device
    pub fn probe(&self, device_id: &CDeviceId) -> i32 {
        if let Some(probe) = self.info.ops.probe {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { probe(device_id as *const CDeviceId) }
        } else {
            0
        }
    }

    /// Create device context
    pub fn create_context(
        &self,
        name: &[u8],
        reg_base: u64,
        reg_size: usize,
        irq: u32,
    ) -> CDeviceContext {
        let mut ctx = CDeviceContext {
            priv_data: core::ptr::null_mut(),
            reg_base,
            reg_size,
            irq,
            name: [0; MAX_DEVICE_NAME],
            device_class: self.info.device_class,
            callbacks: self.callbacks,
            flags: 0,
            reserved: [0; 4],
        };

        let len = name.len().min(MAX_DEVICE_NAME - 1);
        ctx.name[..len].copy_from_slice(&name[..len]);

        ctx
    }

    /// Open device
    pub fn open(&self, ctx: &mut CDeviceContext) -> i32 {
        if let Some(open) = self.info.ops.open {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { open(ctx as *mut CDeviceContext) }
        } else {
            0
        }
    }

    /// Close device
    pub fn close(&self, ctx: &mut CDeviceContext) -> i32 {
        if let Some(close) = self.info.ops.close {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { close(ctx as *mut CDeviceContext) }
        } else {
            0
        }
    }

    /// Read from device
    pub fn read(&self, ctx: &mut CDeviceContext, buf: &mut [u8]) -> Result<usize, i32> {
        if let Some(read) = self.info.ops.read {
            let mut bytes_read = 0usize;
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe {
                read(
                    ctx as *mut CDeviceContext,
                    buf.as_mut_ptr(),
                    buf.len(),
                    &mut bytes_read,
                )
            };
            if ret < 0 {
                Err(ret)
            } else {
                Ok(bytes_read)
            }
        } else {
            Err(-95) // EOPNOTSUPP
        }
    }

    /// Write to device
    pub fn write(&self, ctx: &mut CDeviceContext, buf: &[u8]) -> Result<usize, i32> {
        if let Some(write) = self.info.ops.write {
            let mut bytes_written = 0usize;
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe {
                write(
                    ctx as *mut CDeviceContext,
                    buf.as_ptr(),
                    buf.len(),
                    &mut bytes_written,
                )
            };
            if ret < 0 {
                Err(ret)
            } else {
                Ok(bytes_written)
            }
        } else {
            Err(-95) // EOPNOTSUPP
        }
    }

    /// I/O control
    pub fn ioctl(&self, ctx: &mut CDeviceContext, cmd: u32, arg: u64) -> i32 {
        if let Some(ioctl) = self.info.ops.ioctl {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { ioctl(ctx as *mut CDeviceContext, cmd, arg) }
        } else {
            -95 // EOPNOTSUPP
        }
    }

    /// Suspend device
    pub fn suspend(&self, ctx: &mut CDeviceContext) -> i32 {
        if let Some(suspend) = self.info.ops.suspend {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { suspend(ctx as *mut CDeviceContext) }
        } else {
            0
        }
    }

    /// Resume device
    pub fn resume(&self, ctx: &mut CDeviceContext) -> i32 {
        if let Some(resume) = self.info.ops.resume {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { resume(ctx as *mut CDeviceContext) }
        } else {
            0
        }
    }

    /// Get driver name
    pub fn name(&self) -> &str {
        let nul_pos = self
            .info
            .name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(MAX_DRIVER_NAME);
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::str::from_utf8_unchecked(&self.info.name[..nul_pos]) }
    }
}

/// Create the callback table for kernel services
fn create_callback_table() -> CCallbackTable {
    CCallbackTable {
        // Memory management
        mem_alloc: c_mem_alloc,
        mem_free: c_mem_free,
        dma_alloc: c_dma_alloc,
        dma_free: c_dma_free,

        // Interrupt management
        request_irq: c_request_irq,
        free_irq: c_free_irq,
        enable_irq: c_enable_irq,
        disable_irq: c_disable_irq,

        // I/O operations
        mmio_read8: c_mmio_read8,
        mmio_read16: c_mmio_read16,
        mmio_read32: c_mmio_read32,
        mmio_read64: c_mmio_read64,
        mmio_write8: c_mmio_write8,
        mmio_write16: c_mmio_write16,
        mmio_write32: c_mmio_write32,
        mmio_write64: c_mmio_write64,

        // Logging
        log_print: c_log_print,

        // Event notification
        notify_event: c_notify_event,

        // Time
        get_time_ns: c_get_time_ns,
        usleep: c_usleep,
        msleep: c_msleep,
    }
}

// Callback implementations

unsafe extern "C" fn c_mem_alloc(size: usize) -> *mut core::ffi::c_void {
    // TODO: Use actual kernel allocator
    log_debug!("c_mem_alloc: size={}", size);
    core::ptr::null_mut()
}

unsafe extern "C" fn c_mem_free(ptr: *mut core::ffi::c_void) {
    // TODO: Use actual kernel deallocator
    log_debug!("c_mem_free: ptr={:?}", ptr);
}

unsafe extern "C" fn c_dma_alloc(
    size: usize,
    _align: usize,
    dma_addr: *mut u64,
) -> *mut core::ffi::c_void {
    // TODO: Use actual DMA allocator
    log_debug!("c_dma_alloc: size={}", size);
    if !dma_addr.is_null() {
        *dma_addr = 0;
    }
    core::ptr::null_mut()
}

unsafe extern "C" fn c_dma_free(ptr: *mut core::ffi::c_void, _size: usize, _dma_addr: u64) {
    // TODO: Use actual DMA deallocator
    log_debug!("c_dma_free: ptr={:?}", ptr);
}

unsafe extern "C" fn c_request_irq(
    _irq: u32,
    _handler: unsafe extern "C" fn(u32, *mut core::ffi::c_void),
    _dev_id: *mut core::ffi::c_void,
) -> i32 {
    // TODO: Use actual IRQ manager
    log_debug!("c_request_irq: irq={}", _irq);
    0
}

unsafe extern "C" fn c_free_irq(_irq: u32, _dev_id: *mut core::ffi::c_void) {
    // TODO: Use actual IRQ manager
    log_debug!("c_free_irq: irq={}", _irq);
}

unsafe extern "C" fn c_enable_irq(_irq: u32) {
    // TODO: Use actual IRQ manager
}

unsafe extern "C" fn c_disable_irq(_irq: u32) {
    // TODO: Use actual IRQ manager
}

unsafe extern "C" fn c_mmio_read8(addr: u64) -> u8 {
    core::ptr::read_volatile(addr as *const u8)
}

unsafe extern "C" fn c_mmio_read16(addr: u64) -> u16 {
    core::ptr::read_volatile(addr as *const u16)
}

unsafe extern "C" fn c_mmio_read32(addr: u64) -> u32 {
    core::ptr::read_volatile(addr as *const u32)
}

unsafe extern "C" fn c_mmio_read64(addr: u64) -> u64 {
    core::ptr::read_volatile(addr as *const u64)
}

unsafe extern "C" fn c_mmio_write8(addr: u64, val: u8) {
    core::ptr::write_volatile(addr as *mut u8, val);
}

unsafe extern "C" fn c_mmio_write16(addr: u64, val: u16) {
    core::ptr::write_volatile(addr as *mut u16, val);
}

unsafe extern "C" fn c_mmio_write32(addr: u64, val: u32) {
    core::ptr::write_volatile(addr as *mut u32, val);
}

unsafe extern "C" fn c_mmio_write64(addr: u64, val: u64) {
    core::ptr::write_volatile(addr as *mut u64, val);
}

unsafe extern "C" fn c_log_print(_level: i32, _msg: *const i8) {
    // TODO: Route to actual logger
}

unsafe extern "C" fn c_notify_event(
    _ctx: *mut CDeviceContext,
    _event_type: u32,
    _data: *const core::ffi::c_void,
    _size: usize,
) {
    // TODO: Route to event system
}

unsafe extern "C" fn c_get_time_ns() -> u64 {
    // TODO: Use actual timer
    0
}

unsafe extern "C" fn c_usleep(_us: u32) {
    // TODO: Use actual delay
}

unsafe extern "C" fn c_msleep(_ms: u32) {
    // TODO: Use actual delay
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_version() {
        assert_eq!(DDF_ABI_VERSION, 1);
    }

    #[test]
    fn test_device_class_values() {
        assert_eq!(CDeviceClass::Touch as i32, 0);
        assert_eq!(CDeviceClass::Audio as i32, 1);
        assert_eq!(CDeviceClass::Sensor as i32, 3);
        assert_eq!(CDeviceClass::Usb as i32, 7);
    }

    #[test]
    fn test_driver_info_size() {
        // Ensure structure is C-compatible
        assert!(core::mem::size_of::<CDriverInfo>() > 0);
    }

    #[test]
    fn test_device_context_size() {
        assert!(core::mem::size_of::<CDeviceContext>() > 0);
    }
}
