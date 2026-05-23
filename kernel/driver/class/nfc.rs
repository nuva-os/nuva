/*
 * Nuva OS - Kernel - NFC Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for NFC (Near Field Communication) devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// NFC State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcState {
    /// Disabled
    Disabled = 0,
    /// Enabled
    Enabled = 1,
    /// Polling
    Polling = 2,
    /// Listening
    Listening = 3,
    /// Connected
    Connected = 4,
    /// Busy
    Busy = 5,
}

/// NFC Protocol
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcProtocol {
    /// Unknown
    Unknown = 0,
    /// NFC-A (ISO 14443-3A)
    NfcA = 1,
    /// NFC-B (ISO 14443-3B)
    NfcB = 2,
    /// NFC-F (FeliCa)
    NfcF = 3,
    /// NFC-V (ISO 15693)
    NfcV = 4,
    /// ISO 7816
    Iso7816 = 5,
    /// ISO 14443
    Iso14443 = 6,
    /// ISO 15693
    Iso15693 = 7,
    /// MIFARE
    Mifare = 8,
    /// NFC-DEP (P2P)
    NfcDep = 9,
}

/// NFC Target Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcTargetType {
    /// Tag
    Tag = 0,
    /// Card (secure element)
    Card = 1,
    /// Device (P2P)
    Device = 2,
}

/// NFC Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcMode {
    /// Initiator (reader/writer)
    Initiator = 0,
    /// Target (card emulation)
    Target = 1,
    /// Both
    Both = 2,
}

/// NFC Poll Mode
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct NfcPollMode: u32 {
        /// NFC-A
        const NFC_A = 1 << 0;
        /// NFC-B
        const NFC_B = 1 << 1;
        /// NFC-F
        const NFC_F = 1 << 2;
        /// NFC-V
        const NFC_V = 1 << 3;
        /// NFC-A active
        const NFC_A_ACTIVE = 1 << 4;
        /// NFC-F active
        const NFC_F_ACTIVE = 1 << 5;
        /// NFC-DEP
        const NFC_DEP = 1 << 6;
    }
}

/// NFC Target Info
#[repr(C)]
pub struct NfcTarget {
    /// Target ID
    pub target_id: u32,
    /// Protocol
    pub protocol: NfcProtocol,
    /// Target type
    pub target_type: NfcTargetType,
    /// Supported protocols
    pub supported_protocols: u32,
    /// UID
    pub uid: [u8; 16],
    /// UID length
    pub uid_len: u8,
    /// Sens Res (ATQA for NFC-A)
    pub sens_res: [u8; 2],
    /// Sel Res (SAK for NFC-A)
    pub sel_res: u8,
    /// NFCID1
    pub nfcid1: [u8; 10],
    /// NFCID1 length
    pub nfcid1_len: u8,
    /// Historical bytes
    pub hist: [u8; 16],
    /// Historical bytes length
    pub hist_len: u8,
}

/// NFC Data
#[repr(C)]
pub struct NfcData {
    /// Data buffer
    pub data: [u8; 256],
    /// Data length
    pub len: u16,
    /// Status
    pub status: NfcStatus,
}

/// NFC Status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcStatus {
    /// Success
    Success = 0,
    /// Timeout
    Timeout = 1,
    /// RF error
    RfError = 2,
    /// Buffer overflow
    Overflow = 3,
    /// CRC error
    CrcError = 4,
    /// Parity error
    ParityError = 5,
    /// Framing error
    FramingError = 6,
    /// Protocol error
    ProtocolError = 7,
    /// Not supported
    NotSupported = 8,
    /// RF field off
    RfFieldOff = 9,
}

/// NFC SE (Secure Element) Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcSeType {
    /// UICC (SIM)
    Uicc = 0,
    /// eSE (embedded)
    Ese = 1,
    /// SD card
    Sd = 2,
    /// HCE (Host Card Emulation)
    Hce = 3,
}

/// NFC SE Info
#[repr(C)]
pub struct NfcSeInfo {
    /// SE ID
    pub se_id: u32,
    /// SE type
    pub se_type: NfcSeType,
    /// Connected
    pub connected: bool,
    /// AID list length
    pub num_aids: u8,
}

/// NFC AID (Application Identifier)
#[repr(C)]
pub struct NfcAid {
    /// AID bytes
    pub aid: [u8; 16],
    /// AID length
    pub len: u8,
}

/// NFC Device Operations
pub struct NfcDeviceOps {
    // Device control
    /// Open
    pub open: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Close
    pub close: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Enable
    pub enable: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Disable
    pub disable: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Get state
    pub get_state: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> NfcState>,

    // Polling
    /// Start polling
    pub start_poll: Option<unsafe extern "C" fn(*mut core::ffi::c_void, NfcPollMode, u32) -> i32>,
    /// Stop polling
    pub stop_poll: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    // Target
    /// Get target
    pub get_target: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut NfcTarget) -> i32>,
    /// Connect to target
    pub connect: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Disconnect
    pub disconnect: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,

    // Data transfer
    /// Send data
    pub send: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, *const u8, usize) -> i32>,
    /// Receive data
    pub recv: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, *mut u8, usize) -> i32>,
    /// Transceive (send + receive)
    pub transceive: Option<
        unsafe extern "C" fn(*mut core::ffi::c_void, u32, *const u8, usize, *mut u8, usize) -> i32,
    >,

    // Secure element
    /// Get SE count
    pub get_se_count: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Get SE info
    pub get_se_info:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, u32, *mut NfcSeInfo) -> i32>,
    /// Enable SE
    pub enable_se: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Disable SE
    pub disable_se: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,

    // Card emulation
    /// Start emulation
    pub start_emulation: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Stop emulation
    pub stop_emulation: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// NFC ioctl commands
pub mod nfc_ioctl {
    /// Enable
    pub const ENABLE: u32 = 0x5001;
    /// Disable
    pub const DISABLE: u32 = 0x5002;
    /// Get state
    pub const GET_STATE: u32 = 0x5003;
    /// Start polling
    pub const START_POLL: u32 = 0x5004;
    /// Stop polling
    pub const STOP_POLL: u32 = 0x5005;
    /// Get target
    pub const GET_TARGET: u32 = 0x5006;
    /// Connect
    pub const CONNECT: u32 = 0x5007;
    /// Disconnect
    pub const DISCONNECT: u32 = 0x5008;
    /// Send
    pub const SEND: u32 = 0x5009;
    /// Recv
    pub const RECV: u32 = 0x500A;
    /// Transceive
    pub const TRANSCEIVE: u32 = 0x500B;
    /// Get SE count
    pub const GET_SE_COUNT: u32 = 0x500C;
    /// Get SE info
    pub const GET_SE_INFO: u32 = 0x500D;
    /// Enable SE
    pub const ENABLE_SE: u32 = 0x500E;
    /// Disable SE
    pub const DISABLE_SE: u32 = 0x500F;
    /// Start emulation
    pub const START_EMULATION: u32 = 0x5010;
    /// Stop emulation
    pub const STOP_EMULATION: u32 = 0x5011;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfc_state() {
        assert_eq!(NfcState::Disabled as i32, 0);
        assert_eq!(NfcState::Connected as i32, 4);
    }

    #[test]
    fn test_nfc_protocol() {
        assert_eq!(NfcProtocol::NfcA as i32, 1);
        assert_eq!(NfcProtocol::Mifare as i32, 8);
    }

    #[test]
    fn test_nfc_poll_mode() {
        let mode = NfcPollMode::NFC_A | NfcPollMode::NFC_B;
        assert!(mode.contains(NfcPollMode::NFC_A));
        assert!(mode.contains(NfcPollMode::NFC_B));
    }

    #[test]
    fn test_nfc_se_type() {
        assert_eq!(NfcSeType::Uicc as i32, 0);
        assert_eq!(NfcSeType::Ese as i32, 1);
        assert_eq!(NfcSeType::Hce as i32, 3);
    }
}
