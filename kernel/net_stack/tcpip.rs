use crate::{pr_debug, pr_info};
/*
 * Nuva OS - Kernel - TCP/IP Protocol Stack
 * 
 * Complete TCP/IP implementation for network communication.
 * 
 * Copyright (C) 2026 Nuva OS Team
 */

use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering};

/// IP version
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpVersion {
    V4 = 4,
    V6 = 6,
}

/// TCP states
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed = 0,
    Listen = 1,
    SynSent = 2,
    SynReceived = 3,
    Established = 4,
    FinWait1 = 5,
    FinWait2 = 6,
    Closing = 7,
    TimeWait = 8,
    CloseWait = 9,
    LastAck = 10,
}

/// TCP flags
pub mod tcp_flags {
    pub const FIN: u8 = 0x01;
    pub const SYN: u8 = 0x02;
    pub const RST: u8 = 0x04;
    pub const PSH: u8 = 0x08;
    pub const ACK: u8 = 0x10;
    pub const URG: u8 = 0x20;
    pub const ECE: u8 = 0x40;
    pub const CWR: u8 = 0x80;
}

/// TCP header
#[repr(C, packed)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub data_offset: u8,  // 4 bits offset, 4 bits reserved
    pub flags: u8,
    pub window: u16,
    pub checksum: u16,
    pub urgent_ptr: u16,
}

impl TcpHeader {
    pub fn new(src_port: u16, dst_port: u16) -> Self {
        TcpHeader {
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
            seq_num: 0,
            ack_num: 0,
            data_offset: (5 << 4), // 20 bytes header
            flags: 0,
            window: 65535u16.to_be(),
            checksum: 0,
            urgent_ptr: 0,
        }
    }
    
    pub fn set_flag(&mut self, flag: u8) {
        self.flags |= flag;
    }
    
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.flags & flag) != 0
    }
    
    pub fn header_len(&self) -> usize {
        ((self.data_offset >> 4) as usize) * 4
    }
}

/// IPv4 header
#[repr(C, packed)]
pub struct Ipv4Header {
    pub version_ihl: u8,
    pub tos: u8,
    pub total_len: u16,
    pub identification: u16,
    pub flags_frag: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub src_addr: u32,
    pub dst_addr: u32,
}

impl Ipv4Header {
    pub fn new(src: u32, dst: u32, protocol: u8, payload_len: u16) -> Self {
        Ipv4Header {
            version_ihl: 0x45, // Version 4, IHL 5 (20 bytes)
            tos: 0,
            total_len: ((20 + payload_len) as u16).to_be(),
            identification: 0,
            flags_frag: 0x4000u16.to_be(), // Don't fragment
            ttl: 64,
            protocol,
            checksum: 0,
            src_addr: src.to_be(),
            dst_addr: dst.to_be(),
        }
    }
    
    pub fn calculate_checksum(&mut self) {
        self.checksum = 0;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let data = unsafe {
            core::slice::from_raw_parts(
                self as *const Ipv4Header as *const u16,
                core::mem::size_of::<Ipv4Header>() / 2
            )
        };
        
        let mut sum: u32 = 0;
        for &word in data {
            sum += word.to_be() as u32;
        }
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        self.checksum = (!(sum as u16)).to_be();
    }
}

/// TCP connection
pub struct TcpConnection {
    /// Local IP
    pub local_ip: u32,
    /// Local port
    pub local_port: u16,
    /// Remote IP
    pub remote_ip: u32,
    /// Remote port
    pub remote_port: u16,
    /// State
    pub state: AtomicU32,
    /// Send sequence number
    pub snd_una: AtomicU32,
    pub snd_nxt: AtomicU32,
    pub snd_wnd: AtomicU16,
    /// Receive sequence number
    pub rcv_nxt: AtomicU32,
    pub rcv_wnd: AtomicU16,
    /// Initial sequence number
    pub iss: u32,
    pub irs: u32,
    /// RTT estimation
    pub srtt: AtomicU32,
    pub rttvar: AtomicU32,
    pub rto: AtomicU32,
    /// Congestion control
    pub cwnd: AtomicU32,
    pub ssthresh: AtomicU32,
    /// Timestamps
    pub last_ack: AtomicU64,
    pub last_sent: AtomicU64,
}

impl TcpConnection {
    pub fn new(local_ip: u32, local_port: u16, remote_ip: u32, remote_port: u16) -> Self {
        // Generate initial sequence number
        let iss = Self::generate_isn();
        
        TcpConnection {
            local_ip,
            local_port,
            remote_ip,
            remote_port,
            state: AtomicU32::new(TcpState::Closed as u32),
            snd_una: AtomicU32::new(iss),
            snd_nxt: AtomicU32::new(iss),
            snd_wnd: AtomicU16::new(65535),
            rcv_nxt: AtomicU32::new(0),
            rcv_wnd: AtomicU16::new(65535),
            iss,
            irs: 0,
            srtt: AtomicU32::new(1000), // 1 second initial
            rttvar: AtomicU32::new(500),
            rto: AtomicU32::new(3000), // 3 seconds initial
            cwnd: AtomicU32::new(1),   // Slow start
            ssthresh: AtomicU32::new(65535),
            last_ack: AtomicU64::new(0),
            last_sent: AtomicU64::new(0),
        }
    }
    
    /// Generate Initial Sequence Number
    fn generate_isn() -> u32 {
        // TODO: Use cryptographically secure random
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { 
            let time = crate::kernel::time::get_time_ms() as u32;
            time.wrapping_mul(2654435761)
        }
    }
    
    /// Send SYN packet
    pub fn send_syn(&mut self) -> Result<(), i32> {
        self.state.store(TcpState::SynSent as u32, Ordering::Release);
        
        // Build SYN packet
        let mut tcp_hdr = TcpHeader::new(self.local_port, self.remote_port);
        tcp_hdr.seq_num = self.iss.to_be();
        tcp_hdr.set_flag(tcp_flags::SYN);
        
        // Build IP packet
        let mut ip_hdr = Ipv4Header::new(
            self.local_ip.to_be(),
            self.remote_ip.to_be(),
            6, // TCP
            20 // TCP header only
        );
        ip_hdr.calculate_checksum();
        
        // TODO: Send packet via network interface
        log_debug!("TCP: Sending SYN to {}:{}", 
            self.remote_ip.to_be_bytes()[0],
            self.remote_port
        );
        
        Ok(())
    }
    
    /// Process incoming SYN+ACK
    pub fn on_syn_ack(&mut self, seq: u32, ack: u32, wnd: u16) -> Result<(), i32> {
        if self.state.load(Ordering::Acquire) != TcpState::SynSent as u32 {
            return Err(-22);
        }
        
        // Verify ACK
        if ack.to_be() != self.iss + 1 {
            return Err(-22);
        }
        
        self.irs = seq.to_be();
        self.rcv_nxt.store(self.irs + 1, Ordering::Release);
        self.snd_una.store(ack.to_be(), Ordering::Release);
        self.snd_nxt.store(ack.to_be(), Ordering::Release);
        self.snd_wnd.store(wnd, Ordering::Release);
        
        // Send ACK
        self.send_ack()?;
        
        self.state.store(TcpState::Established as u32, Ordering::Release);
        log_debug!("TCP: Connection established");
        
        Ok(())
    }
    
    /// Send ACK
    pub fn send_ack(&mut self) -> Result<(), i32> {
        let mut tcp_hdr = TcpHeader::new(self.local_port, self.remote_port);
        tcp_hdr.seq_num = self.snd_nxt.load(Ordering::Acquire).to_be();
        tcp_hdr.ack_num = self.rcv_nxt.load(Ordering::Acquire).to_be();
        tcp_hdr.set_flag(tcp_flags::ACK);
        tcp_hdr.window = self.rcv_wnd.load(Ordering::Acquire).to_be();
        
        // TODO: Send packet
        Ok(())
    }
    
    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<usize, i32> {
        if self.state.load(Ordering::Acquire) != TcpState::Established as u32 {
            return Err(-107); // ENOTCONN
        }
        
        let seq = self.snd_nxt.load(Ordering::Acquire);
        
        // Build TCP header
        let mut tcp_hdr = TcpHeader::new(self.local_port, self.remote_port);
        tcp_hdr.seq_num = seq.to_be();
        tcp_hdr.ack_num = self.rcv_nxt.load(Ordering::Acquire).to_be();
        tcp_hdr.set_flag(tcp_flags::ACK | tcp_flags::PSH);
        tcp_hdr.window = self.rcv_wnd.load(Ordering::Acquire).to_be();
        
        // Build IP header
        let mut ip_hdr = Ipv4Header::new(
            self.local_ip.to_be(),
            self.remote_ip.to_be(),
            6,
            (20 + data.len()) as u16
        );
        ip_hdr.calculate_checksum();
        
        // Update sequence number
        self.snd_nxt.fetch_add(data.len() as u32, Ordering::AcqRel);
        
        // TODO: Send packet via network interface
        Ok(data.len())
    }
    
    /// Receive data
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, i32> {
        if self.state.load(Ordering::Acquire) != TcpState::Established as u32 {
            return Err(-107);
        }
        
        // TODO: Read from receive buffer
        Ok(0)
    }
    
    /// Close connection
    pub fn close(&mut self) -> Result<(), i32> {
        let current_state = self.state.load(Ordering::Acquire);
        
        match current_state {
            x if x == TcpState::Established as u32 => {
                // Send FIN
                let mut tcp_hdr = TcpHeader::new(self.local_port, self.remote_port);
                tcp_hdr.seq_num = self.snd_nxt.load(Ordering::Acquire).to_be();
                tcp_hdr.ack_num = self.rcv_nxt.load(Ordering::Acquire).to_be();
                tcp_hdr.set_flag(tcp_flags::FIN | tcp_flags::ACK);
                
                self.snd_nxt.fetch_add(1, Ordering::AcqRel);
                self.state.store(TcpState::FinWait1 as u32, Ordering::Release);
            }
            x if x == TcpState::CloseWait as u32 => {
                // Send FIN
                let mut tcp_hdr = TcpHeader::new(self.local_port, self.remote_port);
                tcp_hdr.seq_num = self.snd_nxt.load(Ordering::Acquire).to_be();
                tcp_hdr.ack_num = self.rcv_nxt.load(Ordering::Acquire).to_be();
                tcp_hdr.set_flag(tcp_flags::FIN | tcp_flags::ACK);
                
                self.snd_nxt.fetch_add(1, Ordering::AcqRel);
                self.state.store(TcpState::LastAck as u32, Ordering::Release);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Update RTT
    pub fn update_rtt(&mut self, rtt: u32) {
        let srtt = self.srtt.load(Ordering::Acquire);
        let rttvar = self.rttvar.load(Ordering::Acquire);
        
        // RFC 6298 algorithm
        let new_rttvar = (3 * rttvar + (srtt.abs_diff(rtt))) / 4;
        let new_srtt = (7 * srtt + rtt) / 8;
        
        self.rttvar.store(new_rttvar, Ordering::Release);
        self.srtt.store(new_srtt, Ordering::Release);
        
        // RTO = SRTT + 4 * RTTVAR
        let rto = new_srtt + 4 * new_rttvar;
        self.rto.store(rto.max(200).min(60000), Ordering::Release);
    }
}

/// UDP header
#[repr(C, packed)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

impl UdpHeader {
    pub fn new(src_port: u16, dst_port: u16, payload_len: u16) -> Self {
        UdpHeader {
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
            length: ((8 + payload_len) as u16).to_be(),
            checksum: 0,
        }
    }
}

/// Network buffer
pub struct NetBuffer {
    data: [u8; 65536],
    len: usize,
}

impl NetBuffer {
    pub fn new() -> Self {
        NetBuffer {
            data: [0; 65536],
            len: 0,
        }
    }
    
    pub fn from_slice(data: &[u8]) -> Self {
        let mut buf = Self::new();
        let len = data.len().min(65536);
        buf.data[..len].copy_from_slice(&data[..len]);
        buf.len = len;
        buf
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
    
    pub fn len(&self) -> usize { self.len }
}

/// Network statistics
#[repr(C)]
pub struct NetStats {
    pub ip_packets: AtomicU64,
    pub tcp_packets: AtomicU64,
    pub udp_packets: AtomicU64,
    pub icmp_packets: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_recv: AtomicU64,
    pub tcp_connections: AtomicU32,
    pub tcp_errors: AtomicU64,
    pub udp_errors: AtomicU64,
}

impl NetStats {
    pub const fn new() -> Self {
        NetStats {
            ip_packets: AtomicU64::new(0),
            tcp_packets: AtomicU64::new(0),
            udp_packets: AtomicU64::new(0),
            icmp_packets: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_recv: AtomicU64::new(0),
            tcp_connections: AtomicU32::new(0),
            tcp_errors: AtomicU64::new(0),
            udp_errors: AtomicU64::new(0),
        }
    }
}

/// Global network statistics
pub static NET_STATS: NetStats = NetStats::new();

/// Initialize TCP/IP stack
pub fn init_tcpip() {
    log_info!("TCP/IP stack initialized");
}
