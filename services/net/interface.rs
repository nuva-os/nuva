/*
 * Nuva OS - SystemService - Net
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


use super::ip::IpAddress;

/// InterfaceType
#[derive(Debug, Clone, Copy)]
pub enum InterfaceType {
    /// Ethernet
    Ethernet = 0,
    /// WiFi
    Wifi = 1,
    /// MoveNetwork
    Mobile = 2,
    /// roundRing
    Loopback = 3,
}

/// InterfaceState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceState {
    /// Close
    Down = 0,
    /// Open
    Up = 1,
}

/// NetworkInterface
pub struct NetworkInterface {
    /// Interface ID
    pub if_id: u32,
    /// InterfaceName
    pub name: &'static str,
    /// InterfaceType
    pub if_type: InterfaceType,
    /// State
    pub state: InterfaceState,
    /// IP Address
    pub ip_addr: Option<IpAddress>,
    /// ChildnetworkMask
    pub netmask: Option<IpAddress>,
    /// networkclose
    pub gateway: Option<IpAddress>,
    /// MAC Address
    pub mac_addr: [u8; 6],
    /// MTU
    pub mtu: u16,
}

/// InterfacemanagementadministrationService
pub struct InterfaceManager {
    /// InterfaceArray
    interfaces: [Option<NetworkInterface>; 8],
    /// Interfacecount
    num_interfaces: u32,
}

impl InterfaceManager {
    pub const fn new() -> Self {
        InterfaceManager {
            interfaces: [None; 8],
            num_interfaces: 0,
        }
    }
    
    /// Initialize
    pub fn init(&mut self) -> i32 {
        log_info!("Interface manager initialized");
        
        // RegisterroundRingInterface
        self.register_interface(NetworkInterface {
            if_id: 0,
            name: "lo",
            if_type: InterfaceType::Loopback,
            state: InterfaceState::Up,
            ip_addr: Some(IpAddress::v4(127, 0, 0, 1)),
            netmask: Some(IpAddress::v4(255, 0, 0, 0)),
            gateway: None,
            mac_addr: [0; 6],
            mtu: 65536,
        });
        
        0
    }
    
    /// RegisterInterface
    pub fn register_interface(&mut self, iface: NetworkInterface) -> i32 {
        for slot in self.interfaces.iter_mut() {
            if slot.is_none() {
                *slot = Some(iface);
                self.num_interfaces += 1;
                return 0;
            }
        }
        -1
    }
    
    /// GetInterface
    pub fn get_interface(&self, if_id: u32) -> Option<&NetworkInterface> {
        for slot in self.interfaces.iter() {
            if let Some(ref iface) = slot {
                if iface.if_id == if_id {
                    return Some(iface);
                }
            }
        }
        None
    }
    
    /// EnableInterface
    pub fn up(&mut self, if_id: u32) -> i32 {
        for slot in self.interfaces.iter_mut() {
            if let Some(ref mut iface) = slot {
                if iface.if_id == if_id {
                    iface.state = InterfaceState::Up;
                    return 0;
                }
            }
        }
        -1
    }
    
    /// DisableInterface
    pub fn down(&mut self, if_id: u32) -> i32 {
        for slot in self.interfaces.iter_mut() {
            if let Some(ref mut iface) = slot {
                if iface.if_id == if_id {
                    iface.state = InterfaceState::Down;
                    return 0;
                }
            }
        }
        -1
    }
}

static INTERFACE_MANAGER: core::sync::OnceLock<InterfaceManager> = core::sync::OnceLock::new();

pub fn get_interface_manager() -> &'static mut InterfaceManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut INTERFACE_MANAGER }
}

pub fn init_interface() {
    let manager = get_interface_manager();
    manager.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_type() {
        assert_eq!(InterfaceType::Ethernet as u32, 0);
        assert_eq!(InterfaceType::Wifi as u32, 1);
        assert_eq!(InterfaceType::Mobile as u32, 2);
        assert_eq!(InterfaceType::Loopback as u32, 3);
    }

    #[test]
    fn test_interface_state() {
        assert_eq!(InterfaceState::Down as u32, 0);
        assert_eq!(InterfaceState::Up as u32, 1);
    }

    #[test]
    fn test_network_interface() {
        let iface = NetworkInterface {
            if_id: 0,
            name: "lo",
            if_type: InterfaceType::Loopback,
            state: InterfaceState::Up,
            ip_addr: Some(IpAddress::v4(127, 0, 0, 1)),
            netmask: Some(IpAddress::v4(255, 0, 0, 0)),
            gateway: None,
            mac_addr: [0; 6],
            mtu: 65536,
        };

        assert_eq!(iface.if_id, 0);
        assert_eq!(iface.name, "lo");
        assert_eq!(iface.if_type, InterfaceType::Loopback);
        assert_eq!(iface.state, InterfaceState::Up);
        assert_eq!(iface.mtu, 65536);
    }

    #[test]
    fn test_interface_manager_new() {
        let manager = InterfaceManager::new();

        assert_eq!(manager.num_interfaces, 0);
    }

    #[test]
    fn test_interface_manager_init() {
        let mut manager = InterfaceManager::new();

        manager.init();

        // shouldtheRegister roundRingInterface
        assert_eq!(manager.num_interfaces, 1);
    }

    #[test]
    fn test_interface_manager_register() {
        let mut manager = InterfaceManager::new();

        let iface = NetworkInterface {
            if_id: 1,
            name: "eth0",
            if_type: InterfaceType::Ethernet,
            state: InterfaceState::Down,
            ip_addr: Some(IpAddress::v4(192, 168, 1, 100)),
            netmask: Some(IpAddress::v4(255, 255, 255, 0)),
            gateway: Some(IpAddress::v4(192, 168, 1, 1)),
            mac_addr: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            mtu: 1500,
        };

        let result = manager.register_interface(iface);
        assert_eq!(result, 0);
        assert_eq!(manager.num_interfaces, 1);
    }

    #[test]
    fn test_interface_manager_get() {
        let mut manager = InterfaceManager::new();

        manager.init();

        let iface = manager.get_interface(0);
        assert!(iface.is_some());

        let iface = iface.unwrap();
        assert_eq!(iface.name, "lo");
    }

    #[test]
    fn test_interface_manager_get_nonexistent() {
        let manager = InterfaceManager::new();

        let iface = manager.get_interface(999);
        assert!(iface.is_none());
    }

    #[test]
    fn test_interface_manager_up_down() {
        let mut manager = InterfaceManager::new();

        let iface = NetworkInterface {
            if_id: 1,
            name: "eth0",
            if_type: InterfaceType::Ethernet,
            state: InterfaceState::Down,
            ip_addr: None,
            netmask: None,
            gateway: None,
            mac_addr: [0; 6],
            mtu: 1500,
        };

        manager.register_interface(iface);

        // EnableInterface
        let result = manager.up(1);
        assert_eq!(result, 0);

        let iface = manager.get_interface(1).unwrap();
        assert_eq!(iface.state, InterfaceState::Up);

        // DisableInterface
        let result = manager.down(1);
        assert_eq!(result, 0);

        let iface = manager.get_interface(1).unwrap();
        assert_eq!(iface.state, InterfaceState::Down);
    }

    #[test]
    fn test_interface_manager_up_nonexistent() {
        let mut manager = InterfaceManager::new();

        let result = manager.up(999);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_interface_with_wifi() {
        let iface = NetworkInterface {
            if_id: 2,
            name: "wlan0",
            if_type: InterfaceType::Wifi,
            state: InterfaceState::Up,
            ip_addr: Some(IpAddress::v4(192, 168, 0, 50)),
            netmask: Some(IpAddress::v4(255, 255, 255, 0)),
            gateway: Some(IpAddress::v4(192, 168, 0, 1)),
            mac_addr: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
            mtu: 1500,
        };

        assert_eq!(iface.if_type, InterfaceType::Wifi);
        assert!(iface.ip_addr.is_some());
    }
}