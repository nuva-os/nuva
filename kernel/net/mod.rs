/*
 * Nuva OS - Kernel - Network Protocol Stack
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Network protocol stack main module.
 */

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod socket;
pub mod net_device;
pub mod ethernet;
pub mod arp;
pub mod ip;
pub mod ipv6;
pub mod icmp;
pub mod icmpv6;
pub mod tcp;
pub mod udp;
pub mod route;
pub mod netlink;
pub mod tcp_fastpath;
pub mod firewall;
pub mod security;
pub mod nfs;
pub mod smb;

// Re-export key types
pub use tcp_fastpath::{TcpFastPathProcessor, init_tcp_fast_path};
pub use tcp_fastpath::TcpConnection as FastPathTcpConnection;

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

// Re-export types
pub use socket::{Socket, SocketManager, SocketType, AddressFamily, Protocol, SockAddrInet};
pub use net_device::{NetDevice, NetDeviceOps, NetDeviceType, NetDeviceFlags};
pub use ethernet::{EthernetHeader, EthernetType};
pub use arp::{ArpHeader, ArpEntry, ArpTable};
pub use ip::{IpHeader, IpAddr};
pub use ipv6::{Ipv6Header, Ipv6Addr, Ipv6Manager};
pub use icmp::{IcmpHeader, IcmpType};
pub use icmpv6::{Icmpv6Header, Icmpv6Manager};
pub use tcp::{TcpHeader, TcpState, TcpConnection, TcpTimers, TcpTimerConfig, SegmentResult, TcpTimerType};

// Re-export socket syscall functions
pub use socket::{
    sys_socket, sys_bind, sys_listen, sys_accept, sys_connect,
    sys_send, sys_recv, sys_sendto, sys_recvfrom, sys_shutdown,
    sys_setsockopt, sys_getsockopt,
};
pub use route::{RouteEntry, RouteTable};
pub use firewall::{FirewallManager, FirewallRule, FirewallAction, FirewallProtocol};
pub use security::{SecurityManager, SecurityPolicy, SecurityLevel};

/// Network Manager
pub struct NetManager {
    /// Device count
    dev_count: AtomicU32,
    /// Statistics
    stats: NetStats,
}

/// Network Statistics
pub struct NetStats {
    /// Packets received
    pub rx_packets: AtomicU64,
    /// Packets transmitted
    pub tx_packets: AtomicU64,
    /// Bytes received
    pub rx_bytes: AtomicU64,
    /// Bytes transmitted
    pub tx_bytes: AtomicU64,
    /// Receive errors
    pub rx_errors: AtomicU64,
    /// Transmit errors
    pub tx_errors: AtomicU64,
    /// Dropped packets
    pub dropped: AtomicU64,
}

impl NetStats {
    pub const fn new() -> Self {
        NetStats {
            rx_packets: AtomicU64::new(0),
            tx_packets: AtomicU64::new(0),
            rx_bytes: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            rx_errors: AtomicU64::new(0),
            tx_errors: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        }
    }
}

impl NetManager {
    pub const fn new() -> Self {
        NetManager {
            dev_count: AtomicU32::new(0),
            stats: NetStats::new(),
        }
    }
    
    /// Initialize network manager
    pub fn init(&self) {
        log_info!("Network manager initialized");
    }
    
    /// Register network device
    pub fn register_device(&mut self) -> u32 {
        self.dev_count.fetch_add(1, Ordering::AcqRel)
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &NetStats {
        &self.stats
    }
}

/// Global network manager
static NET_MANAGER: core::sync::OnceLock<NetManager> = core::sync::OnceLock::new();

/// Get network manager
pub fn net_manager() -> &'static NetManager {
    NET_MANAGER.get_or_init(NetManager::new)
}

pub fn init_net_manager() -> &'static NetManager {
    NET_MANAGER.get_or_init(NetManager::new)
}

/// Initialize network subsystem
pub fn init_net() {
    let mgr = net_manager();
    mgr.init();
}
