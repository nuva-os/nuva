/*
 * Nuva OS - Kernel - USB/Type-C Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for USB and Type-C devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// USB Speed
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    /// Unknown
    Unknown = 0,
    /// Low speed (1.5 Mbps)
    Low = 1,
    /// Full speed (12 Mbps)
    Full = 2,
    /// High speed (480 Mbps)
    High = 3,
    /// Super speed (5 Gbps)
    Super = 4,
    /// Super speed+ (10 Gbps)
    SuperPlus = 5,
}

/// USB Direction
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDirection {
    /// Host to device
    Out = 0,
    /// Device to host
    In = 1,
}

/// USB Transfer Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransferType {
    /// Control
    Control = 0,
    /// Isochronous
    Isochronous = 1,
    /// Bulk
    Bulk = 2,
    /// Interrupt
    Interrupt = 3,
}

/// USB Device Descriptor
#[repr(C)]
pub struct UsbDeviceDescriptor {
    /// USB version (BCD)
    pub usb_version: u16,
    /// Device class
    pub device_class: u8,
    /// Device subclass
    pub device_subclass: u8,
    /// Protocol
    pub protocol: u8,
    /// Max packet size (EP0)
    pub max_packet_size: u8,
    /// Vendor ID
    pub vendor_id: u16,
    /// Product ID
    pub product_id: u16,
    /// Device version (BCD)
    pub device_version: u16,
    /// Number of configurations
    pub num_configurations: u8,
}

/// USB Endpoint Descriptor
#[repr(C)]
pub struct UsbEndpointDescriptor {
    /// Endpoint address
    pub address: u8,
    /// Direction
    pub direction: UsbDirection,
    /// Transfer type
    pub transfer_type: UsbTransferType,
    /// Max packet size
    pub max_packet_size: u16,
    /// Interval (for interrupt/isoc)
    pub interval: u8,
}

/// USB Device
#[repr(C)]
pub struct UsbDevice {
    /// Device address (1-127)
    pub address: u8,
    /// Bus number
    pub bus: u8,
    /// Port number
    pub port: u8,
    /// Speed
    pub speed: UsbSpeed,
    /// Device descriptor
    pub descriptor: UsbDeviceDescriptor,
    /// Number of interfaces
    pub num_interfaces: u8,
    /// Configuration value
    pub config_value: u8,
    /// Parent hub (0 if root)
    pub parent: u8,
    /// Device state
    pub state: UsbDeviceState,
}

/// USB Device State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDeviceState {
    /// Not attached
    NotAttached = 0,
    /// Attached
    Attached = 1,
    /// Powered
    Powered = 2,
    /// Default
    Default = 3,
    /// Address assigned
    Address = 4,
    /// Configured
    Configured = 5,
    /// Suspended
    Suspended = 6,
}

/// USB Transfer Request
#[repr(C)]
pub struct UsbTransfer {
    /// Device address
    pub device_addr: u8,
    /// Endpoint address
    pub endpoint: u8,
    /// Direction
    pub direction: UsbDirection,
    /// Transfer type
    pub transfer_type: UsbTransferType,
    /// Buffer
    pub buffer: *mut u8,
    /// Buffer size
    pub size: usize,
    /// Actual length transferred
    pub actual_length: usize,
    /// Timeout (ms)
    pub timeout: u32,
    /// Status
    pub status: UsbTransferStatus,
}

/// USB Transfer Status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransferStatus {
    /// Completed successfully
    Completed = 0,
    /// Pending
    Pending = 1,
    /// Cancelled
    Cancelled = 2,
    /// Error
    Error = 3,
    /// Timeout
    Timeout = 4,
    /// Stall
    Stall = 5,
    /// Overflow
    Overflow = 6,
    /// No device
    NoDevice = 7,
}

/// Type-C Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCMode {
    /// Not connected
    Disconnected = 0,
    /// Source (DFP)
    Source = 1,
    /// Sink (UFP)
    Sink = 2,
    /// DRP (Dual Role)
    Drp = 3,
    /// Accessory
    Accessory = 4,
}

/// Type-C Power Role
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCPowerRole {
    /// Source
    Source = 0,
    /// Sink
    Sink = 1,
}

/// Type-C Data Role
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCDataRole {
    /// DFP (Downstream Facing Port)
    Dfp = 0,
    /// UFP (Upstream Facing Port)
    Ufp = 1,
}

/// Type-C Port Status
#[repr(C)]
pub struct TypeCPortStatus {
    /// Port number
    pub port: u8,
    /// Current mode
    pub mode: TypeCMode,
    /// Power role
    pub power_role: TypeCPowerRole,
    /// Data role
    pub data_role: TypeCDataRole,
    /// Connected
    pub connected: bool,
    /// PD contract valid
    pub pd_contract: bool,
    /// Requested voltage (mV)
    pub voltage_mv: u16,
    /// Requested current (mA)
    pub current_ma: u16,
    /// Maximum power (mW)
    pub max_power_mw: u32,
}

/// PD (Power Delivery) Request
#[repr(C)]
pub struct PdRequest {
    /// Voltage in mV
    pub voltage_mv: u16,
    /// Current in mA
    pub current_ma: u16,
    /// Position in source capabilities
    pub position: u8,
}

/// USB/Type-C Device Operations
pub struct UsbDeviceOps {
    // USB operations
    /// Enumerate devices
    pub enumerate:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut UsbDevice, usize) -> i32>,
    /// Get device
    pub get_device: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, u8) -> *mut UsbDevice>,
    /// Submit transfer
    pub submit_transfer:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut UsbTransfer) -> i32>,
    /// Cancel transfer
    pub cancel_transfer:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut UsbTransfer) -> i32>,
    /// Reset device
    pub reset_device: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, u8) -> i32>,
    /// Set configuration
    pub set_config: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, u8, u8) -> i32>,
    /// Set interface
    pub set_interface: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, u8, u8, u8) -> i32>,

    // Type-C operations
    /// Get port status
    pub get_port_status:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, *mut TypeCPortStatus) -> i32>,
    /// Set mode
    pub set_mode: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, TypeCMode) -> i32>,
    /// Set power role
    pub set_power_role:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, TypeCPowerRole) -> i32>,
    /// Set data role
    pub set_data_role:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, TypeCDataRole) -> i32>,
    /// Send PD request
    pub send_pd_request:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, *const PdRequest) -> i32>,
    /// Get PD capabilities
    pub get_pd_capabilities:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, *mut PdRequest, usize) -> i32>,
}

/// USB ioctl commands
pub mod usb_ioctl {
    /// Enumerate devices
    pub const ENUMERATE: u32 = 0xD001;
    /// Get device
    pub const GET_DEVICE: u32 = 0xD002;
    /// Submit transfer
    pub const SUBMIT_TRANSFER: u32 = 0xD003;
    /// Cancel transfer
    pub const CANCEL_TRANSFER: u32 = 0xD004;
    /// Reset device
    pub const RESET_DEVICE: u32 = 0xD005;
    /// Set configuration
    pub const SET_CONFIG: u32 = 0xD006;
    /// Set interface
    pub const SET_INTERFACE: u32 = 0xD007;
    /// Get port status (Type-C)
    pub const GET_PORT_STATUS: u32 = 0xD010;
    /// Set mode (Type-C)
    pub const SET_MODE: u32 = 0xD011;
    /// Set power role (Type-C)
    pub const SET_POWER_ROLE: u32 = 0xD012;
    /// Set data role (Type-C)
    pub const SET_DATA_ROLE: u32 = 0xD013;
    /// Send PD request
    pub const SEND_PD_REQUEST: u32 = 0xD014;
    /// Get PD capabilities
    pub const GET_PD_CAPS: u32 = 0xD015;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_speed_values() {
        assert_eq!(UsbSpeed::Low as i32, 1);
        assert_eq!(UsbSpeed::High as i32, 3);
        assert_eq!(UsbSpeed::Super as i32, 4);
    }

    #[test]
    fn test_typec_mode_values() {
        assert_eq!(TypeCMode::Disconnected as i32, 0);
        assert_eq!(TypeCMode::Source as i32, 1);
        assert_eq!(TypeCMode::Sink as i32, 2);
    }

    #[test]
    fn test_usb_device_state_values() {
        assert_eq!(UsbDeviceState::Configured as i32, 5);
        assert_eq!(UsbDeviceState::Suspended as i32, 6);
    }
}
