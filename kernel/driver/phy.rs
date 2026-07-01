/*
 * Nuva OS - Kernel - Driver - Phy
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
 * Nuva OS - Kernel - PHY Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * PHY (Physical Layer) framework for network and other PHYs.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// PHY ID
pub type PhyId = u32;

/// PHY State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhyState {
    /// Down
    Down = 0,
    /// Starting
    Starting = 1,
    /// Ready
    Ready = 2,
    /// Pending
    Pending = 3,
    /// Up
    Up = 4,
    /// Running
    Running = 5,
    /// NOLINK
    NoLink = 6,
    /// Error
    Error = 7,
    /// Halted
    Halted = 8,
}

/// PHY Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhyMode {
    /// Unknown
    Unknown = 0,
    /// MII
    Mii = 1,
    /// GMII
    Gmii = 2,
    /// SGMII
    Sgmii = 3,
    /// TBI
    Tbi = 4,
    /// RTBI
    Rtbi = 5,
    /// RMII
    Rmii = 6,
    /// RGMII
    Rgmii = 7,
    /// RGMII-ID
    RgmiiId = 8,
    /// RGMII-RXID
    RgmiiRxid = 9,
    /// RGMII-TXID
    RgmiiTxid = 10,
    /// RTKGMII
    RtKgmii = 11,
    /// XGMII
    Xgmii = 12,
    /// USXGMII
    Usxgmii = 13,
    /// QSGMII
    Qsgmii = 14,
    /// 1000BASE-X
    Base1000x = 15,
    /// 2500BASE-X
    Base2500x = 16,
    /// 5GBASE-R
    Base5gr = 17,
    /// 10GBASE-R
    Base10gr = 18,
    /// 10GBASE-KR
    Base10kr = 19,
    /// 10GBASE-CR
    Base10cr = 20,
    /// 25GBASE-CR
    Base25cr = 21,
    /// 40GBASE-CR4
    Base40cr4 = 22,
    /// 100GBASE-CR4
    Base100cr4 = 23,
    /// NA
    Na = 0xFF,
}

/// PHY Speed
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhySpeed {
    /// 10 Mbps
    Speed10 = 10,
    /// 100 Mbps
    Speed100 = 100,
    /// 1000 Mbps
    Speed1000 = 1000,
    /// 2.5 Gbps
    Speed2500 = 2500,
    /// 5 Gbps
    Speed5000 = 5000,
    /// 10 Gbps
    Speed10000 = 10000,
    /// 25 Gbps
    Speed25000 = 25000,
    /// 40 Gbps
    Speed40000 = 40000,
    /// 100 Gbps
    Speed100000 = 100000,
    /// Unknown
    Unknown = 0,
}

/// PHY Duplex
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhyDuplex {
    /// Half duplex
    Half = 0,
    /// Full duplex
    Full = 1,
    /// Unknown
    Unknown = 2,
}

/// PHY Link
#[repr(C)]
pub struct PhyLink {
    /// Link up
    pub up: bool,
    /// Speed
    pub speed: PhySpeed,
    /// Duplex
    pub duplex: PhyDuplex,
    /// Pause
    pub pause: bool,
    /// Asymmetric pause
    pub asym_pause: bool,
    /// Auto-negotiation
    pub autoneg: bool,
}

/// PHY Info
#[repr(C)]
pub struct PhyInfo {
    /// PHY ID (IEEE OUI + model + revision)
    pub phy_id: u32,
    /// PHY ID mask
    pub phy_id_mask: u32,
    /// Name
    pub name: [u8; 32],
    /// Features
    pub features: PhyFeatures,
    /// Flags
    pub flags: PhyFlags,
    /// Max speed
    pub max_speed: u32,
    /// Number of ports
    pub ports: u8,
    /// Driver data
    pub driver_data: *mut core::ffi::c_void,
}

/// PHY Features
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PhyFeatures: u32 {
        /// 10 Mbps half
        const SPEED_10_HALF = 1 << 0;
        /// 10 Mbps full
        const SPEED_10_FULL = 1 << 1;
        /// 100 Mbps half
        const SPEED_100_HALF = 1 << 2;
        /// 100 Mbps full
        const SPEED_100_FULL = 1 << 3;
        /// 1000 Mbps half
        const SPEED_1000_HALF = 1 << 4;
        /// 1000 Mbps full
        const SPEED_1000_FULL = 1 << 5;
        /// 2.5 Gbps
        const SPEED_2500 = 1 << 6;
        /// 5 Gbps
        const SPEED_5000 = 1 << 7;
        /// 10 Gbps
        const SPEED_10000 = 1 << 8;
        /// 25 Gbps
        const SPEED_25000 = 1 << 9;
        /// 40 Gbps
        const SPEED_40000 = 1 << 10;
        /// 100 Gbps
        const SPEED_100000 = 1 << 11;
        /// Auto-negotiation
        const AUTONEG = 1 << 12;
        /// Pause
        const PAUSE = 1 << 13;
        /// Asymmetric pause
        const ASYM_PAUSE = 1 << 14;
        /// Fiber
        const FIBER = 1 << 15;
        /// Copper
        const COPPER = 1 << 16;
        /// Power down
        const POWER_DOWN = 1 << 17;
        /// Interrupt
        const INTERRUPT = 1 << 18;
    }
}

/// PHY Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PhyFlags: u32 {
        /// Is internal
        const IS_INTERNAL = 1 << 0;
        /// Has interrupts
        const HAS_INTERRUPT = 1 << 1;
        /// MDIO bus
        const MDIO = 1 << 2;
        /// Reset GPIO
        const RESET_GPIO = 1 << 3;
        /// Magic packet
        const MAGIC_PACKET = 1 << 4;
        /// Wake on LAN
        const WOL = 1 << 5;
    }
}

/// PHY Operations
pub struct PhyOps {
    /// Config init
    pub config_init: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Config aneg
    pub config_aneg: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Aneg done
    pub aneg_done: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Read status
    pub read_status: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Soft reset
    pub soft_reset: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Suspend
    pub suspend: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Resume
    pub resume: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Config interrupt
    pub config_intr: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Handle interrupt
    pub handle_interrupt: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Link change
    pub link_change: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Get rate
    pub get_rate: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Set rate
    pub set_rate: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
}

/// PHY Device
pub struct PhyDevice {
    /// PHY ID
    pub phy_id: PhyId,
    /// MDIO address
    pub addr: u8,
    /// Bus ID
    pub bus_id: u32,
    /// State
    pub state: AtomicU32,
    /// Link
    pub link: PhyLink,
    /// Info
    pub info: PhyInfo,
    /// Operations
    pub ops: PhyOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// IRQ
    pub irq: i32,
    /// Attached
    pub attached: bool,
}

/// PHY Manager
pub struct PhyManager {
    /// PHY count
    phy_count: AtomicU32,
    /// Statistics
    stats: PhyStats,
}

/// PHY Statistics
pub struct PhyStats {
    /// Connect count
    pub connect_count: AtomicU64,
    /// Disconnect count
    pub disconnect_count: AtomicU64,
    /// Link change count
    pub link_change_count: AtomicU64,
}

impl PhyStats {
    pub const fn new() -> Self {
        PhyStats {
            connect_count: AtomicU64::new(0),
            disconnect_count: AtomicU64::new(0),
            link_change_count: AtomicU64::new(0),
        }
    }
}

impl PhyManager {
    pub const fn new() -> Self {
        PhyManager {
            phy_count: AtomicU32::new(0),
            stats: PhyStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("PHY manager initialized");
    }

    /// Register PHY
    pub fn register(&mut self, _phy: &PhyDevice) -> PhyId {
        self.phy_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Connect PHY
    pub fn connect(&mut self, phy_id: PhyId) -> i32 {
        self.stats.connect_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("phy_connect: id={}", phy_id);
        0
    }

    /// Disconnect PHY
    pub fn disconnect(&mut self, phy_id: PhyId) {
        self.stats.disconnect_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("phy_disconnect: id={}", phy_id);
    }

    /// Get link state
    pub fn get_link(&self, phy_id: PhyId) -> PhyLink {
        log_debug!("phy_get_link: id={}", phy_id);
        PhyLink {
            up: false,
            speed: PhySpeed::Unknown,
            duplex: PhyDuplex::Unknown,
            pause: false,
            asym_pause: false,
            autoneg: false,
        }
    }

    /// Start auto-negotiation
    pub fn start_aneg(&mut self, phy_id: PhyId) -> i32 {
        log_debug!("phy_start_aneg: id={}", phy_id);
        0
    }

    /// Read status
    pub fn read_status(&mut self, phy_id: PhyId) -> i32 {
        log_debug!("phy_read_status: id={}", phy_id);
        0
    }
}

/// Global PHY manager
static PHY_MANAGER: crate::sync_oncelock::OnceLock<PhyManager> = crate::sync_oncelock::OnceLock::new();

/// Get PHY manager
pub fn phy_manager() -> &'static PhyManager {
    PHY_MANAGER.get_or_init(PhyManager::new)
}

/// Initialize PHY manager
pub fn init_phy_manager() {
    let mgr = phy_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phy_state() {
        assert_eq!(PhyState::Down as i32, 0);
        assert_eq!(PhyState::Running as i32, 5);
    }

    #[test]
    fn test_phy_mode() {
        assert_eq!(PhyMode::Rgmii as i32, 7);
        assert_eq!(PhyMode::Sgmii as i32, 3);
    }

    #[test]
    fn test_phy_speed() {
        assert_eq!(PhySpeed::Speed1000 as i32, 1000);
        assert_eq!(PhySpeed::Speed10000 as i32, 10000);
    }

    #[test]
    fn test_phy_features() {
        let features = PhyFeatures::SPEED_1000_FULL | PhyFeatures::AUTONEG;
        assert!(features.contains(PhyFeatures::SPEED_1000_FULL));
        assert!(features.contains(PhyFeatures::AUTONEG));
    }
}
