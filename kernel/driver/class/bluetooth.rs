/*
 * Nuva OS - Kernel - Bluetooth Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for Bluetooth devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Bluetooth State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtState {
    /// Off
    Off = 0,
    /// On
    On = 1,
    /// Turning on
    TurningOn = 2,
    /// Turning off
    TurningOff = 3,
    /// Discoverable
    Discoverable = 4,
    /// Connectable
    Connectable = 5,
}

/// Bluetooth Address
#[repr(C)]
pub struct BtAddress {
    /// Address bytes (LSB first)
    pub addr: [u8; 6],
    /// Address type
    pub addr_type: BtAddressType,
}

impl BtAddress {
    /// Create from bytes
    pub fn new(addr: [u8; 6], addr_type: BtAddressType) -> Self {
        BtAddress { addr, addr_type }
    }

    /// Check if valid
    pub fn is_valid(&self) -> bool {
        // Check not all zeros
        self.addr.iter().any(|&b| b != 0)
    }
}

/// Bluetooth Address Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtAddressType {
    /// Public
    Public = 0,
    /// Random
    Random = 1,
    /// Resolved public
    ResolvedPublic = 2,
    /// Resolved random
    ResolvedRandom = 3,
}

/// Bluetooth Device Class
#[repr(C)]
pub struct BtDeviceClass {
    /// Service class (bits 0-23)
    pub service_class: u32,
    /// Major device class (bits 8-12)
    pub major: u8,
    /// Minor device class (bits 2-7)
    pub minor: u8,
}

/// Bluetooth Major Device Class
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtMajorClass {
    /// Miscellaneous
    Miscellaneous = 0x00,
    /// Computer
    Computer = 0x01,
    /// Phone
    Phone = 0x02,
    /// LAN/Network
    Lan = 0x03,
    /// Audio/Video
    AudioVideo = 0x04,
    /// Peripheral
    Peripheral = 0x05,
    /// Imaging
    Imaging = 0x06,
    /// Wearable
    Wearable = 0x07,
    /// Toy
    Toy = 0x08,
    /// Health
    Health = 0x09,
    /// Uncategorized
    Uncategorized = 0x1F,
}

/// Bluetooth LE Role
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtLeRole {
    /// Central
    Central = 0,
    /// Peripheral
    Peripheral = 1,
    /// Both
    Both = 2,
}

/// Bluetooth Connection State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtConnState {
    /// Disconnected
    Disconnected = 0,
    /// Connecting
    Connecting = 1,
    /// Connected
    Connected = 2,
    /// Disconnecting
    Disconnecting = 3,
}

/// Bluetooth Connection Info
#[repr(C)]
pub struct BtConnection {
    /// Connection handle
    pub handle: u16,
    /// Remote address
    pub addr: BtAddress,
    /// Connection state
    pub state: BtConnState,
    /// Link type
    pub link_type: BtLinkType,
    /// Encryption
    pub encrypted: bool,
    /// Authenticated
    pub authenticated: bool,
    /// Role
    pub role: BtLeRole,
    /// Interval (for LE, in 1.25ms units)
    pub interval: u16,
    /// Latency
    pub latency: u16,
    /// Timeout (in 10ms units)
    pub timeout: u16,
}

/// Bluetooth Link Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtLinkType {
    /// SCO
    Sco = 0,
    /// ACL
    Acl = 1,
    /// ESCO
    Esco = 2,
    /// LE
    Le = 3,
}

/// Bluetooth Scan Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtScanMode {
    /// None
    None = 0,
    /// Inquiry scan only
    Inquiry = 1,
    /// Page scan only
    Page = 2,
    /// Both
    Both = 3,
}

/// Bluetooth Advertising Parameters
#[repr(C)]
pub struct BtAdvParams {
    /// Minimum interval (0.625ms units)
    pub interval_min: u16,
    /// Maximum interval
    pub interval_max: u16,
    /// Advertising type
    pub adv_type: BtAdvType,
    /// Own address type
    pub own_addr_type: BtAddressType,
    /// Peer address type
    pub peer_addr_type: BtAddressType,
    /// Peer address
    pub peer_addr: [u8; 6],
    /// Channel map
    pub channel_map: u8,
    /// Filter policy
    pub filter_policy: u8,
}

/// Bluetooth Advertising Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtAdvType {
    /// Connectable undirected
    ConnectableUndirected = 0,
    /// Connectable directed
    ConnectableDirected = 1,
    /// Non-connectable undirected
    NonConnectableUndirected = 2,
    /// Scannable undirected
    ScannableUndirected = 3,
}

/// Bluetooth Device Operations
pub struct BtDeviceOps {
    // Power control
    /// Power on
    pub power_on: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Power off
    pub power_off: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Get state
    pub get_state: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> BtState>,

    // Address
    /// Get address
    pub get_address: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut BtAddress) -> i32>,
    /// Set address
    pub set_address: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const BtAddress) -> i32>,

    // Discovery
    /// Start discovery
    pub start_discovery: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Stop discovery
    pub stop_discovery: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    // Connection
    /// Connect
    pub connect: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const BtAddress) -> i32>,
    /// Disconnect
    pub disconnect: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u16) -> i32>,
    /// Get connection
    pub get_connection:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, u16, *mut BtConnection) -> i32>,

    // Advertising (LE)
    /// Start advertising
    pub start_advertising: Option<
        unsafe extern "C" fn(*mut core::ffi::c_void, *const BtAdvParams, *const u8, u8) -> i32,
    >,
    /// Stop advertising
    pub stop_advertising: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    // Scan
    /// Set scan mode
    pub set_scan_mode: Option<unsafe extern "C" fn(*mut core::ffi::c_void, BtScanMode) -> i32>,

    // Data transfer
    /// Send data
    pub send: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u16, *const u8, usize) -> i32>,
    /// Receive data
    pub recv: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u16, *mut u8, usize) -> i32>,
}

/// Bluetooth ioctl commands
pub mod bt_ioctl {
    /// Power on
    pub const POWER_ON: u32 = 0x3001;
    /// Power off
    pub const POWER_OFF: u32 = 0x3002;
    /// Get state
    pub const GET_STATE: u32 = 0x3003;
    /// Get address
    pub const GET_ADDRESS: u32 = 0x3004;
    /// Set address
    pub const SET_ADDRESS: u32 = 0x3005;
    /// Start discovery
    pub const START_DISCOVERY: u32 = 0x3006;
    /// Stop discovery
    pub const STOP_DISCOVERY: u32 = 0x3007;
    /// Connect
    pub const CONNECT: u32 = 0x3008;
    /// Disconnect
    pub const DISCONNECT: u32 = 0x3009;
    /// Start advertising
    pub const START_ADVERTISING: u32 = 0x300A;
    /// Stop advertising
    pub const STOP_ADVERTISING: u32 = 0x300B;
    /// Send
    pub const SEND: u32 = 0x300C;
    /// Recv
    pub const RECV: u32 = 0x300D;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bt_state() {
        assert_eq!(BtState::Off as i32, 0);
        assert_eq!(BtState::On as i32, 1);
    }

    #[test]
    fn test_bt_address() {
        let addr = BtAddress::new([0x11, 0x22, 0x33, 0x44, 0x55, 0x66], BtAddressType::Public);
        assert!(addr.is_valid());
        assert_eq!(addr.addr_type, BtAddressType::Public);

        let zero_addr = BtAddress::new([0; 6], BtAddressType::Public);
        assert!(!zero_addr.is_valid());
    }

    #[test]
    fn test_bt_major_class() {
        assert_eq!(BtMajorClass::Computer as i32, 0x01);
        assert_eq!(BtMajorClass::Phone as i32, 0x02);
    }
}
