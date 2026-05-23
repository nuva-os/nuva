/*
 * Nuva OS - Kernel - WiFi Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for WiFi network devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// WiFi State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiState {
    /// Disconnected
    Disconnected = 0,
    /// Authenticating
    Authenticating = 1,
    /// Associating
    Associating = 2,
    /// Associated
    Associated = 3,
    /// 4-way handshake
    FourWay = 4,
    /// Group handshake
    Group = 5,
    /// Connected
    Connected = 6,
}

/// WiFi Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiMode {
    /// Infrastructure (STA)
    Infra = 0,
    /// Independent (IBSS)
    Adhoc = 1,
    /// Access Point
    Ap = 2,
    /// Monitor
    Monitor = 3,
    /// Mesh
    Mesh = 4,
    /// P2P GO
    P2pGo = 5,
    /// P2P Client
    P2pClient = 6,
    /// OCB
    Ocb = 7,
}

/// WiFi Security
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    /// Open
    Open = 0,
    /// WEP
    Wep = 1,
    /// WPA-PSK
    WpaPsk = 2,
    /// WPA2-PSK
    Wpa2Psk = 3,
    /// WPA3-PSK
    Wpa3Psk = 4,
    /// WPA2-Enterprise
    Wpa2Enterprise = 5,
    /// WPA3-Enterprise
    Wpa3Enterprise = 6,
    /// SAE (WPA3)
    Sae = 7,
    /// OWE (Opportunistic Wireless Encryption)
    Owe = 8,
}

/// WiFi Cipher
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiCipher {
    /// None
    None = 0,
    /// WEP40
    Wep40 = 1,
    /// WEP104
    Wep104 = 2,
    /// TKIP
    Tkip = 3,
    /// CCMP (AES)
    Ccmp = 4,
    /// GCMP
    Gcmp = 5,
    /// GCMP-256
    Gcmp256 = 6,
    /// CCMP-256
    Ccmp256 = 7,
}

/// WiFi Channel
#[repr(C)]
pub struct WifiChannel {
    /// Channel number
    pub channel: u8,
    /// Frequency (MHz)
    pub freq: u16,
    /// Flags
    pub flags: ChannelFlags,
    /// Max power (dBm)
    pub max_power: i8,
    /// Max regulatory power
    pub max_reg_power: i8,
    /// Max antenna gain
    pub max_antenna_gain: i8,
    /// Bandwidth (MHz)
    pub bandwidth: u8,
}

/// Channel Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ChannelFlags: u32 {
        /// 2.4 GHz
        const BAND_2GHZ = 1 << 0;
        /// 5 GHz
        const BAND_5GHZ = 1 << 1;
        /// 6 GHz
        const BAND_6GHZ = 1 << 2;
        /// 60 GHz
        const BAND_60GHZ = 1 << 3;
        /// No IR (no initiating radiation)
        const NO_IR = 1 << 4;
        /// No OFDM
        const NO_OFDM = 1 << 5;
        /// No CCK
        const NO_CCK = 1 << 6;
        /// Indoor only
        const INDOOR_ONLY = 1 << 7;
        /// Passive scan
        const PASSIVE = 1 << 8;
        /// DFS
        const DFS = 1 << 9;
        /// HT40+
        const HT40_PLUS = 1 << 10;
        /// HT40-
        const HT40_MINUS = 1 << 11;
        /// VHT80
        const VHT80 = 1 << 12;
        /// VHT160
        const VHT160 = 1 << 13;
    }
}

/// WiFi SSID
#[repr(C)]
pub struct WifiSsid {
    /// SSID bytes
    pub ssid: [u8; 32],
    /// SSID length
    pub len: u8,
}

impl WifiSsid {
    /// Create from bytes
    pub fn new(ssid: &[u8]) -> Self {
        let mut result = WifiSsid {
            ssid: [0; 32],
            len: ssid.len().min(32) as u8,
        };
        result.ssid[..result.len as usize].copy_from_slice(&ssid[..result.len as usize]);
        result
    }
}

/// WiFi BSSID (MAC address)
#[repr(C)]
pub struct WifiBssid {
    /// MAC address bytes
    pub addr: [u8; 6],
}

impl WifiBssid {
    /// Create from bytes
    pub fn new(addr: [u8; 6]) -> Self {
        WifiBssid { addr }
    }
}

/// WiFi Scan Result
#[repr(C)]
pub struct WifiScanResult {
    /// BSSID
    pub bssid: WifiBssid,
    /// SSID
    pub ssid: WifiSsid,
    /// Frequency (MHz)
    pub freq: u16,
    /// Signal strength (dBm)
    pub signal: i16,
    /// Security
    pub security: WifiSecurity,
    /// Cipher
    pub cipher: WifiCipher,
    /// Channel
    pub channel: u8,
    /// Flags
    pub flags: ScanFlags,
    /// Timestamp
    pub timestamp: u64,
}

/// Scan Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ScanFlags: u32 {
        /// Privacy enabled
        const PRIVACY = 1 << 0;
        /// IBSS
        const IBSS = 1 << 1;
        /// ESS
        const ESS = 1 << 2;
        /// Spectrum management
        const SPECTRUM_MGMT = 1 << 3;
        /// QoS
        const QOS = 1 << 4;
        /// Short slot time
        const SHORT_SLOT = 1 << 5;
        /// APSD
        const APSD = 1 << 6;
        /// Radio measurement
        const RADIO_MEAS = 1 << 7;
        /// HT
        const HT = 1 << 8;
        /// VHT
        const VHT = 1 << 9;
        /// HE (WiFi 6)
        const HE = 1 << 10;
        /// EHT (WiFi 7)
        const EHT = 1 << 11;
        /// WPS
        const WPS = 1 << 12;
        /// WPA
        const WPA = 1 << 13;
        /// WPA2
        const WPA2 = 1 << 14;
        /// WPA3
        const WPA3 = 1 << 15;
    }
}

/// WiFi Connect Parameters
#[repr(C)]
pub struct WifiConnectParams {
    /// SSID
    pub ssid: WifiSsid,
    /// BSSID (optional)
    pub bssid: Option<WifiBssid>,
    /// Security type
    pub security: WifiSecurity,
    /// Password/PSK
    pub password: [u8; 64],
    /// Password length
    pub password_len: u8,
    /// Channel (0 = auto)
    pub channel: u8,
    /// BSSID hint
    pub bssid_hint: WifiBssid,
    /// Key management
    pub key_mgmt: KeyMgmt,
}

/// Key Management
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct KeyMgmt: u32 {
        /// None
        const NONE = 1 << 0;
        /// IEEE 802.1X
        const IEEE8021X = 1 << 1;
        /// PSK
        const PSK = 1 << 2;
        /// FT-PSK
        const FT_PSK = 1 << 3;
        /// SAE
        const SAE = 1 << 4;
        /// FT-SAE
        const FT_SAE = 1 << 5;
        /// OWE
        const OWE = 1 << 6;
        /// EAP-SHA256
        const EAP_SHA256 = 1 << 7;
    }
}

/// WiFi Device Info
#[repr(C)]
pub struct WifiInfo {
    /// MAC address
    pub mac: [u8; 6],
    /// Supported modes
    pub modes: u32,
    /// Supported bands
    pub bands: u32,
    /// Supported ciphers
    pub ciphers: u32,
    /// Max scan SSIDs
    pub max_scan_ssids: u8,
    /// Max connect SSIDs
    pub max_connect_ssids: u8,
    /// Max remain on channel duration (ms)
    pub max_remain_on_channel: u16,
    /// Number of channels
    pub num_channels: u16,
    /// Current state
    pub state: WifiState,
    /// Current mode
    pub mode: WifiMode,
}

/// WiFi Device Operations
pub struct WifiDeviceOps {
    // Device control
    /// Open
    pub open: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Close
    pub close: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Get info
    pub get_info: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut WifiInfo) -> i32>,

    // Scan
    /// Start scan
    pub scan: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const WifiSsid, u8) -> i32>,
    /// Abort scan
    pub abort_scan: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Get scan results
    pub get_scan_results:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut WifiScanResult, usize) -> i32>,

    // Connection
    /// Connect
    pub connect:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const WifiConnectParams) -> i32>,
    /// Disconnect
    pub disconnect: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Get state
    pub get_state: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> WifiState>,

    // AP mode
    /// Start AP
    pub start_ap: Option<
        unsafe extern "C" fn(
            *mut core::ffi::c_void,
            *const WifiSsid,
            *const u8,
            u8,
            WifiSecurity,
            u8,
        ) -> i32,
    >,
    /// Stop AP
    pub stop_ap: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    // Channel
    /// Get channels
    pub get_channels:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut WifiChannel, usize) -> i32>,
    /// Set channel
    pub set_channel: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8) -> i32>,

    // Power save
    /// Set power save
    pub set_power_save: Option<unsafe extern "C" fn(*mut core::ffi::c_void, bool) -> i32>,
    /// Get power save
    pub get_power_save: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> bool>,
}

/// WiFi ioctl commands
pub mod wifi_ioctl {
    /// Get info
    pub const GET_INFO: u32 = 0x4001;
    /// Scan
    pub const SCAN: u32 = 0x4002;
    /// Abort scan
    pub const ABORT_SCAN: u32 = 0x4003;
    /// Get scan results
    pub const GET_SCAN_RESULTS: u32 = 0x4004;
    /// Connect
    pub const CONNECT: u32 = 0x4005;
    /// Disconnect
    pub const DISCONNECT: u32 = 0x4006;
    /// Get state
    pub const GET_STATE: u32 = 0x4007;
    /// Start AP
    pub const START_AP: u32 = 0x4008;
    /// Stop AP
    pub const STOP_AP: u32 = 0x4009;
    /// Get channels
    pub const GET_CHANNELS: u32 = 0x400A;
    /// Set channel
    pub const SET_CHANNEL: u32 = 0x400B;
    /// Set power save
    pub const SET_POWER_SAVE: u32 = 0x400C;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_state() {
        assert_eq!(WifiState::Disconnected as i32, 0);
        assert_eq!(WifiState::Connected as i32, 6);
    }

    #[test]
    fn test_wifi_mode() {
        assert_eq!(WifiMode::Infra as i32, 0);
        assert_eq!(WifiMode::Ap as i32, 2);
    }

    #[test]
    fn test_wifi_ssid() {
        let ssid = WifiSsid::new(b"TestNetwork");
        assert_eq!(ssid.len, 11);
        assert_eq!(&ssid.ssid[..11], b"TestNetwork");
    }

    #[test]
    fn test_channel_flags() {
        let flags = ChannelFlags::BAND_2GHZ | ChannelFlags::PASSIVE;
        assert!(flags.contains(ChannelFlags::BAND_2GHZ));
        assert!(flags.contains(ChannelFlags::PASSIVE));
    }
}
