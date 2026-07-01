/*
 * Nuva OS - Kernel - Net - Netdev
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
 * Nuva OS - Kernel - Network Device
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Network device abstraction.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Network Device ID
pub type NetDevId = u32;

/// Network Device Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetDeviceType {
    /// Loopback
    Loopback = 0,
    /// Ethernet
    Ethernet = 1,
    /// Wireless (802.11)
    Wireless = 801,
    /// PPP
    Ppp = 512,
    /// Tunnel
    Tunnel = 768,
    /// Virtual
    Virtual = 1024,
}

/// Network Device Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct NetDeviceFlags: u32 {
        /// Interface is up
        const IFF_UP = 0x0001;
        /// Broadcast address valid
        const IFF_BROADCAST = 0x0002;
        /// Debugging on
        const IFF_DEBUG = 0x0004;
        /// Loopback
        const IFF_LOOPBACK = 0x0008;
        /// Is a point-to-point link
        const IFF_POINTOPOINT = 0x0010;
        /// Avoid trailers
        const IFF_NOTRAILERS = 0x0020;
        /// Interface is running
        const IFF_RUNNING = 0x0040;
        /// No ARP protocol
        const IFF_NOARP = 0x0080;
        /// Promiscuous mode
        const IFF_PROMISC = 0x0100;
        /// Receive all multicast
        const IFF_ALLMULTI = 0x0200;
        /// Master of a slave
        const IFF_MASTER = 0x0400;
        /// Slave of a master
        const IFF_SLAVE = 0x0800;
        /// Supports multicast
        const IFF_MULTICAST = 0x1000;
        /// Can set port type
        const IFF_PORTSEL = 0x2000;
        /// Media type auto-selected
        const IFF_AUTOMEDIA = 0x4000;
        /// Dynamic address
        const IFF_DYNAMIC = 0x8000;
        /// Lower layer up
        const IFF_LOWER_UP = 0x10000;
        /// Carrier detected
        const IFF_CARRIER = 0x20000;
        /// Dormant
        const IFF_DORMANT = 0x40000;
        /// Echo sent packets
        const IFF_ECHO = 0x80000;
    }
}

/// Network Device Operations
pub struct NetDeviceOps {
    /// Open device
    pub open: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Close device
    pub stop: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Start transmission
    pub start_xmit: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut SkBuff) -> i32>,
    /// Hard header
    pub hard_header: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut SkBuff, u16, *const u8, *const u8, u32) -> i32>,
    /// Rebuild header
    pub rebuild_header: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut SkBuff) -> i32>,
    /// Set MAC address
    pub set_mac_address: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const u8) -> i32>,
    /// Do ioctl
    pub do_ioctl: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, *mut core::ffi::c_void) -> i32>,
    /// Get stats
    pub get_stats: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> NetDevStats>,
    /// Change MTU
    pub change_mtu: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Set RX mode
    pub set_rx_mode: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Validate address
    pub validate_addr: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> i32>,
}

/// Network Device Statistics
#[repr(C)]
pub struct NetDevStats {
    /// Received packets
    pub rx_packets: u64,
    /// Transmitted packets
    pub tx_packets: u64,
    /// Received bytes
    pub rx_bytes: u64,
    /// Transmitted bytes
    pub tx_bytes: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Receive dropped
    pub rx_dropped: u64,
    /// Transmit dropped
    pub tx_dropped: u64,
    /// Multicast received
    pub multicast: u64,
    /// Collisions
    pub collisions: u64,
    /// Receive length errors
    pub rx_length_errors: u64,
    /// Receive over errors
    pub rx_over_errors: u64,
    /// Receive CRC errors
    pub rx_crc_errors: u64,
    /// Receive frame errors
    pub rx_frame_errors: u64,
    /// Receive FIFO errors
    pub rx_fifo_errors: u64,
    /// Receive missed errors
    pub rx_missed_errors: u64,
    /// Transmit aborted errors
    pub tx_aborted_errors: u64,
    /// Transmit carrier errors
    pub tx_carrier_errors: u64,
    /// Transmit FIFO errors
    pub tx_fifo_errors: u64,
    /// Transmit heartbeat errors
    pub tx_heartbeat_errors: u64,
    /// Transmit window errors
    pub tx_window_errors: u64,
}

/// Socket Buffer (sk_buff)
#[repr(C)]
pub struct SkBuff {
    /// Next buffer in list
    pub next: *mut SkBuff,
    /// Previous buffer in list
    pub prev: *mut SkBuff,
    /// Data pointer
    pub data: *mut u8,
    /// Tail pointer
    pub tail: *mut u8,
    /// End pointer
    pub end: *mut u8,
    /// Head pointer
    pub head: *mut u8,
    /// Data length
    pub len: u32,
    /// Data length (linear)
    pub data_len: u32,
    /// MAC header length
    pub mac_len: u16,
    /// Protocol
    pub protocol: u16,
    /// Transport header offset
    pub transport_header: u16,
    /// Network header offset
    pub network_header: u16,
    /// MAC header offset
    pub mac_header: u16,
    /// Device
    pub dev: *mut NetDevice,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Flags
    pub flags: u32,
    /// Checksum
    pub csum: u32,
    /// Packet type
    pub pkt_type: u8,
    /// IP summed
    pub ip_summed: u8,
}

impl SkBuff {
    pub fn new() -> Self {
        SkBuff {
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
            data: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
            end: core::ptr::null_mut(),
            head: core::ptr::null_mut(),
            len: 0,
            data_len: 0,
            mac_len: 0,
            protocol: 0,
            transport_header: 0,
            network_header: 0,
            mac_header: 0,
            dev: core::ptr::null_mut(),
            ref_count: AtomicU32::new(1),
            flags: 0,
            csum: 0,
            pkt_type: 0,
            ip_summed: 0,
        }
    }
    
    /// Get data length
    pub fn len(&self) -> u32 {
        self.len
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Network Device
pub struct NetDevice {
    /// Device name
    pub name: [u8; 16],
    /// Device ID
    pub id: NetDevId,
    /// Device type
    pub dev_type: NetDeviceType,
    /// Flags
    pub flags: AtomicU32,
    /// MTU
    pub mtu: u32,
    /// Hardware address length
    pub addr_len: u8,
    /// Hardware address
    pub dev_addr: [u8; 32],
    /// Broadcast address
    pub broadcast: [u8; 32],
    /// Operations
    pub ops: NetDeviceOps,
    /// Private data
    pub priv_data: *mut core::ffi::c_void,
    /// Statistics
    pub stats: NetDevStats,
    /// TX queue length
    pub tx_queue_len: u32,
    /// Interface index
    pub ifindex: u32,
    /// Carrier
    pub carrier: AtomicU32,
    /// State
    pub state: AtomicU32,
}

impl NetDevice {
    pub fn new(name: &[u8], dev_type: NetDeviceType) -> Self {
        let mut name_arr = [0u8; 16];
        let len = name.len().min(15);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        NetDevice {
            name: name_arr,
            id: 0,
            dev_type,
            flags: AtomicU32::new(0),
            mtu: 1500,
            addr_len: 6,
            dev_addr: [0; 32],
            broadcast: [0; 32],
            ops: NetDeviceOps {
                open: None,
                stop: None,
                start_xmit: None,
                hard_header: None,
                rebuild_header: None,
                set_mac_address: None,
                do_ioctl: None,
                get_stats: None,
                change_mtu: None,
                set_rx_mode: None,
                validate_addr: None,
            },
            priv_data: core::ptr::null_mut(),
            stats: NetDevStats {
                rx_packets: 0,
                tx_packets: 0,
                rx_bytes: 0,
                tx_bytes: 0,
                rx_errors: 0,
                tx_errors: 0,
                rx_dropped: 0,
                tx_dropped: 0,
                multicast: 0,
                collisions: 0,
                rx_length_errors: 0,
                rx_over_errors: 0,
                rx_crc_errors: 0,
                rx_frame_errors: 0,
                rx_fifo_errors: 0,
                rx_missed_errors: 0,
                tx_aborted_errors: 0,
                tx_carrier_errors: 0,
                tx_fifo_errors: 0,
                tx_heartbeat_errors: 0,
                tx_window_errors: 0,
            },
            tx_queue_len: 1000,
            ifindex: 0,
            carrier: AtomicU32::new(1),
            state: AtomicU32::new(0),
        }
    }
    
    /// Open device
    pub fn open(&mut self) -> i32 {
        if let Some(open) = self.ops.open {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { open(self.priv_data) }
        } else {
            0
        }
    }
    
    /// Close device
    pub fn stop(&mut self) -> i32 {
        if let Some(stop) = self.ops.stop {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { stop(self.priv_data) }
        } else {
            0
        }
    }
    
    /// Transmit packet
    pub fn transmit(&mut self, skb: *mut SkBuff) -> i32 {
        if let Some(xmit) = self.ops.start_xmit {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { xmit(self.priv_data, skb) }
        } else {
            -1
        }
    }
    
    /// Check if up
    pub fn is_up(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & NetDeviceFlags::IFF_UP.bits()) != 0
    }
    
    /// Check if running
    pub fn is_running(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & NetDeviceFlags::IFF_RUNNING.bits()) != 0
    }
    
    /// Check if carrier ok
    pub fn carrier_ok(&self) -> bool {
        self.carrier.load(Ordering::Acquire) != 0
    }
}

/// Network Device Manager
pub struct NetDevManager {
    /// Device count
    dev_count: AtomicU32,
    /// Next ifindex
    next_ifindex: AtomicU32,
}

impl NetDevManager {
    pub const fn new() -> Self {
        NetDevManager {
            dev_count: AtomicU32::new(0),
            next_ifindex: AtomicU32::new(1),
        }
    }
    
    /// Register device
    pub fn register_device(&mut self, dev: &mut NetDevice) -> u32 {
        dev.id = self.dev_count.fetch_add(1, Ordering::AcqRel);
        dev.ifindex = self.next_ifindex.fetch_add(1, Ordering::AcqRel);
        dev.id
    }
}

/// Global device manager
static NETDEV_MANAGER: crate::sync_oncelock::OnceLock<NetDevManager> = crate::sync_oncelock::OnceLock::new();

/// Get device manager
pub fn netdev_manager() -> &'static NetDevManager {
    NETDEV_MANAGER.get_or_init(NetDevManager::new)
}

pub fn init_netdev_manager() -> &'static NetDevManager {
    NETDEV_MANAGER.get_or_init(NetDevManager::new)
}
