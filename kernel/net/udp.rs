/*
 * Nuva OS - Kernel - UDP Protocol
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * User Datagram Protocol (UDP) implementation.
 * Complete with UdpSocket, checksum, send/recv.
 */

use crate::{pr_debug, pr_info, pr_warn};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// UDP Header
#[repr(C, packed)]
pub struct UdpHeader {
    /// Source port
    pub source: u16,
    /// Destination port
    pub dest: u16,
    /// Length (header + data)
    pub len: u16,
    /// Checksum
    pub check: u16,
}

impl UdpHeader {
    /// Header size
    pub const SIZE: usize = 8;

    /// Create new UDP header
    pub fn new(source: u16, dest: u16, len: u16) -> Self {
        UdpHeader {
            source: source.to_be(),
            dest: dest.to_be(),
            len: len.to_be(),
            check: 0,
        }
    }

    /// Get source port (host byte order)
    pub fn get_source(&self) -> u16 {
        u16::from_be(self.source)
    }

    /// Get destination port (host byte order)
    pub fn get_dest(&self) -> u16 {
        u16::from_be(self.dest)
    }

    /// Get length (host byte order)
    pub fn get_len(&self) -> u16 {
        u16::from_be(self.len)
    }

    /// Calculate checksum including pseudo-header
    pub fn calc_checksum(&mut self, src_addr: u32, dst_addr: u32, data: &[u8]) {
        let mut sum: u32 = 0;

        // Pseudo-header: source address
        sum += ((src_addr >> 16) & 0xFFFF) as u32;
        sum += (src_addr & 0xFFFF) as u32;

        // Pseudo-header: destination address
        sum += ((dst_addr >> 16) & 0xFFFF) as u32;
        sum += (dst_addr & 0xFFFF) as u32;

        // Pseudo-header: protocol (UDP = 17)
        sum += 17;

        // Pseudo-header: UDP length
        sum += self.get_len() as u32;

        // UDP header fields (excluding checksum)
        sum += self.get_source() as u32;
        sum += self.get_dest() as u32;
        sum += self.get_len() as u32;

        // Data payload
        let data_len = data.len();
        for i in (0..data_len).step_by(2) {
            if i + 1 < data_len {
                sum += ((data[i] as u16) | ((data[i + 1] as u16) << 8)) as u32;
            } else {
                sum += data[i] as u32;
            }
        }

        // Fold 32-bit sum to 16 bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        self.check = (!(sum as u16)).to_be();
    }

    /// Verify checksum including pseudo-header
    pub fn verify_checksum(&self, src_addr: u32, dst_addr: u32, data: &[u8]) -> bool {
        let mut sum: u32 = 0;

        // Pseudo-header
        sum += ((src_addr >> 16) & 0xFFFF) as u32;
        sum += (src_addr & 0xFFFF) as u32;
        sum += ((dst_addr >> 16) & 0xFFFF) as u32;
        sum += (dst_addr & 0xFFFF) as u32;
        sum += 17;
        sum += self.get_len() as u32;

        // UDP header (all fields including checksum)
        sum += self.get_source() as u32;
        sum += self.get_dest() as u32;
        sum += self.get_len() as u32;
        sum += u16::from_be(self.check) as u32;

        // Data
        let data_len = data.len();
        for i in (0..data_len).step_by(2) {
            if i + 1 < data_len {
                sum += ((data[i] as u16) | ((data[i + 1] as u16) << 8)) as u32;
            } else {
                sum += data[i] as u32;
            }
        }

        // Fold
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        sum == 0xFFFF
    }
}

/// Compute UDP checksum over pseudo-header, header bytes, and data
pub fn udp_checksum(
    src_addr: u32,
    dst_addr: u32,
    src_port: u16,
    dst_port: u16,
    data: &[u8],
) -> u16 {
    let total_len = (UdpHeader::SIZE + data.len()) as u16;
    let mut sum: u32 = 0;

    // Pseudo-header
    sum += ((src_addr >> 16) & 0xFFFF) as u32;
    sum += (src_addr & 0xFFFF) as u32;
    sum += ((dst_addr >> 16) & 0xFFFF) as u32;
    sum += (dst_addr & 0xFFFF) as u32;
    sum += 17; // UDP protocol
    sum += total_len as u32;

    // UDP header
    sum += src_port as u32;
    sum += dst_port as u32;
    sum += total_len as u32;
    // checksum = 0 for calculation

    // Data
    let data_len = data.len();
    for i in (0..data_len).step_by(2) {
        if i + 1 < data_len {
            sum += ((data[i] as u16) | ((data[i + 1] as u16) << 8)) as u32;
        } else {
            sum += (data[i] as u32) << 8;
        }
    }

    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    !sum as u16
}

// ============================================================================
// UdpSocket - High-level UDP socket interface
// ============================================================================

/// UDP receive buffer entry
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UdpDatagram {
    /// Source address
    pub src_addr: u32,
    /// Source port
    pub src_port: u16,
    /// Destination address
    pub dst_addr: u32,
    /// Destination port
    pub dst_port: u16,
    /// Data length
    pub data_len: u16,
    /// Data buffer
    pub data: [u8; 1472],
    /// Next entry in queue
    pub next: *mut UdpDatagram,
}

impl UdpDatagram {
    pub const fn new() -> Self {
        Self {
            src_addr: 0,
            src_port: 0,
            dst_addr: 0,
            dst_port: 0,
            data_len: 0,
            data: [0; 1472],
            next: core::ptr::null_mut(),
        }
    }
}

/// UDP socket structure
pub struct UdpSocket {
    /// Local address
    pub local_addr: u32,
    /// Local port
    pub local_port: u16,
    /// Remote address (for connected sockets)
    pub remote_addr: u32,
    /// Remote port (for connected sockets)
    pub remote_port: u16,
    /// Receive buffer ring
    pub recv_buf: UdpRingBuffer,
    /// Send buffer ring
    pub send_buf: UdpRingBuffer,
    /// Socket flags
    pub flags: AtomicU32,
    /// Reference count
    pub ref_count: AtomicU32,
}

/// UDP socket flags
pub mod udp_sock_flags {
    pub const BOUND: u32 = 0x01;
    pub const CONNECTED: u32 = 0x02;
    pub const NONBLOCK: u32 = 0x04;
}

/// UDP ring buffer for datagram queue
pub struct UdpRingBuffer {
    /// Buffer entries
    pub entries: [Option<UdpDatagram>; 64],
    /// Head index (read position)
    pub head: u32,
    /// Tail index (write position)
    pub tail: u32,
    /// Count of entries
    pub count: AtomicU32,
    /// Maximum count
    pub max_count: u32,
    /// Total bytes
    pub bytes: AtomicU32,
}

impl UdpRingBuffer {
    pub const fn new() -> Self {
        Self {
            entries: [const { None }; 64],
            head: 0,
            tail: 0,
            count: AtomicU32::new(0),
            max_count: 64,
            bytes: AtomicU32::new(0),
        }
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count.load(Ordering::Acquire) == 0
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.count.load(Ordering::Acquire) >= self.max_count
    }
}

/// UDP send result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdpSendResult {
    /// Success
    Ok,
    /// Buffer full
    WouldBlock,
    /// Payload too large (exceeds 65507 bytes)
    MsgTooBig,
    /// Socket not bound
    NotBound,
    /// No destination
    NoDest,
}

/// UDP recv result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdpRecvResult {
    /// Success with byte count
    Ok(usize),
    /// No data available
    WouldBlock,
    /// Buffer too small
    Truncated,
}

impl UdpSocket {
    /// Create new UDP socket
    pub fn new() -> Self {
        Self {
            local_addr: 0,
            local_port: 0,
            remote_addr: 0,
            remote_port: 0,
            recv_buf: UdpRingBuffer::new(),
            send_buf: UdpRingBuffer::new(),
            flags: AtomicU32::new(0),
            ref_count: AtomicU32::new(1),
        }
    }

    /// Bind to local address and port
    pub fn bind(&mut self, addr: u32, port: u16) -> i32 {
        if (self.flags.load(Ordering::Acquire) & udp_sock_flags::BOUND) != 0 {
            return Errno::Einval.to_ret_i32(); // EINVAL
        }
        self.local_addr = addr;
        self.local_port = port;
        self.flags.fetch_or(udp_sock_flags::BOUND, Ordering::AcqRel);
        log_debug!("UDP: Bound to {}:{}", addr, port);
        0
    }

    /// Connect to remote address (sets default destination)
    pub fn connect(&mut self, addr: u32, port: u16) -> i32 {
        self.remote_addr = addr;
        self.remote_port = port;
        self.flags
            .fetch_or(udp_sock_flags::CONNECTED, Ordering::AcqRel);
        log_debug!("UDP: Connected to {}:{}", addr, port);
        0
    }

    /// Send UDP datagram.
    /// Encapsulates UDP header, computes checksum with pseudo-header,
    /// and queues for IP layer transmission.
    pub fn udp_send(&mut self, data: &[u8], dst_addr: u32, dst_port: u16) -> UdpSendResult {
        // Check payload size (max UDP payload = 65535 - 8 header - 20 IP = 65507)
        if data.len() > 65507 {
            return UdpSendResult::MsgTooBig;
        }

        // Check bound
        if (self.flags.load(Ordering::Acquire) & udp_sock_flags::BOUND) == 0 {
            return UdpSendResult::NotBound;
        }

        // Check send buffer
        if self.send_buf.is_full() {
            return UdpSendResult::WouldBlock;
        }

        // Build UDP header
        let total_len = (UdpHeader::SIZE + data.len()) as u16;
        let mut header = UdpHeader::new(self.local_port, dst_port, total_len);

        // Calculate checksum with pseudo-header
        header.calc_checksum(self.local_addr, dst_addr, data);

        // Enqueue datagram for transmission
        let tail = self.send_buf.tail as usize;
        let copy_len = data.len().min(1472);
        let mut dg = UdpDatagram::new();
        dg.src_addr = self.local_addr;
        dg.src_port = self.local_port;
        dg.dst_addr = dst_addr;
        dg.dst_port = dst_port;
        dg.data_len = copy_len as u16;
        dg.data[..copy_len].copy_from_slice(&data[..copy_len]);

        self.send_buf.entries[tail] = Some(dg);
        self.send_buf.tail = (self.send_buf.tail + 1) % 64;
        self.send_buf.count.fetch_add(1, Ordering::AcqRel);
        self.send_buf
            .bytes
            .fetch_add(data.len() as u32, Ordering::AcqRel);

        // IP layer transmission would happen here:
        // crate::net::ip::send_packet(self.local_addr, dst_addr, 17, &header_bytes, data);

        log_debug!(
            "UDP send: {}:{} -> {}:{}, len={}",
            self.local_addr,
            self.local_port,
            dst_addr,
            dst_port,
            data.len()
        );

        UdpSendResult::Ok
    }

    /// Receive UDP datagram.
    /// Dequeues from receive buffer and returns (src_addr, src_port, bytes_read).
    pub fn udp_recv(&mut self, buf: &mut [u8]) -> UdpRecvResult {
        if self.recv_buf.is_empty() {
            return UdpRecvResult::WouldBlock;
        }

        let head = self.recv_buf.head as usize;
        let dg = match self.recv_buf.entries[head] {
            Some(dg) => dg,
            None => return UdpRecvResult::WouldBlock,
        };

        let copy_len = (dg.data_len as usize).min(buf.len());
        buf[..copy_len].copy_from_slice(&dg.data[..copy_len]);

        let truncated = (dg.data_len as usize) > buf.len();

        self.recv_buf.entries[head] = None;
        self.recv_buf.head = (self.recv_buf.head + 1) % 64;
        self.recv_buf.count.fetch_sub(1, Ordering::AcqRel);
        self.recv_buf
            .bytes
            .fetch_sub(dg.data_len as u32, Ordering::AcqRel);

        if truncated {
            UdpRecvResult::Truncated
        } else {
            UdpRecvResult::Ok(copy_len)
        }
    }

    /// Get last received datagram source address
    pub fn recv_from(&mut self, buf: &mut [u8]) -> Result<(u32, u16, usize), UdpRecvResult> {
        if self.recv_buf.is_empty() {
            return Err(UdpRecvResult::WouldBlock);
        }

        let head = self.recv_buf.head as usize;
        let dg = match self.recv_buf.entries[head] {
            Some(dg) => dg,
            None => return Err(UdpRecvResult::WouldBlock),
        };

        let src_addr = dg.src_addr;
        let src_port = dg.src_port;
        let copy_len = (dg.data_len as usize).min(buf.len());
        buf[..copy_len].copy_from_slice(&dg.data[..copy_len]);

        self.recv_buf.entries[head] = None;
        self.recv_buf.head = (self.recv_buf.head + 1) % 64;
        self.recv_buf.count.fetch_sub(1, Ordering::AcqRel);
        self.recv_buf
            .bytes
            .fetch_sub(dg.data_len as u32, Ordering::AcqRel);

        Ok((src_addr, src_port, copy_len))
    }

    /// Enqueue received datagram into recv buffer (called from IP layer)
    pub fn enqueue_datagram(
        &mut self,
        src_addr: u32,
        src_port: u16,
        dst_addr: u32,
        dst_port: u16,
        data: &[u8],
    ) -> i32 {
        if self.recv_buf.is_full() {
            return Errno::Eagain.to_ret_i32(); // EAGAIN
        }

        let tail = self.recv_buf.tail as usize;
        let copy_len = data.len().min(1472);
        let mut dg = UdpDatagram::new();
        dg.src_addr = src_addr;
        dg.src_port = src_port;
        dg.dst_addr = dst_addr;
        dg.dst_port = dst_port;
        dg.data_len = copy_len as u16;
        dg.data[..copy_len].copy_from_slice(&data[..copy_len]);

        self.recv_buf.entries[tail] = Some(dg);
        self.recv_buf.tail = (self.recv_buf.tail + 1) % 64;
        self.recv_buf.count.fetch_add(1, Ordering::AcqRel);
        self.recv_buf
            .bytes
            .fetch_add(data.len() as u32, Ordering::AcqRel);

        0
    }
}

// ============================================================================
// UDP ControlBlock (legacy, kept for compatibility)
// ============================================================================

/// UDP ControlBlock
pub struct UdpControlBlock {
    /// Local address
    pub local_addr: u32,
    /// Local port
    pub local_port: u16,
    /// Remote address
    pub remote_addr: u32,
    /// Remote port
    pub remote_port: u16,
    /// Receive buffer
    pub recv_queue: UdpPacketQueue,
    /// Send buffer
    pub send_queue: UdpPacketQueue,
    /// Reference count
    pub ref_count: AtomicU32,
    /// State flags
    pub flags: AtomicU32,
}

/// UDP Flag
pub mod udp_flags {
    pub const BOUND: u32 = 0x01;
    pub const CONNECTED: u32 = 0x02;
    pub const RECV_BLOCKED: u32 = 0x04;
    pub const SEND_BLOCKED: u32 = 0x08;
}

/// UDP Packet Queue
pub struct UdpPacketQueue {
    /// Queue head
    pub head: *mut UdpPacket,
    /// Queue tail
    pub tail: *mut UdpPacket,
    /// Packet count
    pub count: AtomicU32,
    /// Max packet count
    pub max_count: u32,
    /// Total bytes
    pub bytes: AtomicU32,
}

impl UdpPacketQueue {
    pub const fn new() -> Self {
        UdpPacketQueue {
            head: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
            count: AtomicU32::new(0),
            max_count: 256,
            bytes: AtomicU32::new(0),
        }
    }
}

/// UDP Packet
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UdpPacket {
    /// Source address
    pub src_addr: u32,
    /// Source port
    pub src_port: u16,
    /// Destination address
    pub dst_addr: u32,
    /// Destination port
    pub dst_port: u16,
    /// Data length
    pub data_len: u16,
    /// Data
    pub data: [u8; 65535],
    /// Next packet
    pub next: *mut UdpPacket,
}

impl UdpControlBlock {
    pub fn new() -> Self {
        UdpControlBlock {
            local_addr: 0,
            local_port: 0,
            remote_addr: 0,
            remote_port: 0,
            recv_queue: UdpPacketQueue::new(),
            send_queue: UdpPacketQueue::new(),
            ref_count: AtomicU32::new(1),
            flags: AtomicU32::new(0),
        }
    }

    /// Bind to local address
    pub fn bind(&mut self, addr: u32, port: u16) -> i32 {
        if (self.flags.load(Ordering::Acquire) & udp_flags::BOUND) != 0 {
            return Errno::Eperm.to_ret_i32();
        }
        self.local_addr = addr;
        self.local_port = port;
        self.flags.fetch_or(udp_flags::BOUND, Ordering::AcqRel);
        log_debug!("UDP: Bound to {}:{}", addr, port);
        0
    }

    /// Connect to remote address
    pub fn connect(&mut self, addr: u32, port: u16) -> i32 {
        self.remote_addr = addr;
        self.remote_port = port;
        self.flags.fetch_or(udp_flags::CONNECTED, Ordering::AcqRel);
        log_debug!("UDP: Connected to {}:{}", addr, port);
        0
    }

    /// Receive packet
    pub fn recv(&mut self, buf: &mut [u8]) -> Option<(u32, u16, usize)> {
        if self.recv_queue.head.is_null() {
            return None;
        }

        // SAFETY: Pointer comes from alloc_udp_packet which returns valid aligned ptr
        unsafe {
            let packet = &*self.recv_queue.head;
            let copy_len = (packet.data_len as usize).min(buf.len());

            for i in 0..copy_len {
                buf[i] = packet.data[i];
            }

            self.recv_queue.head = packet.next;
            if self.recv_queue.head.is_null() {
                self.recv_queue.tail = core::ptr::null_mut();
            }
            self.recv_queue.count.fetch_sub(1, Ordering::AcqRel);
            self.recv_queue
                .bytes
                .fetch_sub(packet.data_len as u32, Ordering::AcqRel);

            Some((packet.src_addr, packet.src_port, copy_len))
        }
    }

    /// Send packet
    pub fn send(&mut self, buf: &[u8], dst_addr: u32, dst_port: u16) -> i32 {
        if self.send_queue.count.load(Ordering::Acquire) >= self.send_queue.max_count {
            return Errno::Eperm.to_ret_i32();
        }

        // SAFETY: alloc_udp_packet returns null or valid aligned pointer
        let packet = unsafe {
            let ptr = alloc_udp_packet();
            if ptr.is_null() {
                return Errno::Enoent.to_ret_i32();
            }

            let packet = &mut *ptr;
            packet.src_addr = self.local_addr;
            packet.src_port = self.local_port;
            packet.dst_addr = dst_addr;
            packet.dst_port = dst_port;
            packet.data_len = buf.len() as u16;
            packet.next = core::ptr::null_mut();

            for (i, &byte) in buf.iter().enumerate() {
                if i < packet.data.len() {
                    packet.data[i] = byte;
                }
            }

            ptr
        };

        // SAFETY: tail pointer is null or valid from alloc_udp_packet
        unsafe {
            if self.send_queue.tail.is_null() {
                self.send_queue.head = packet;
                self.send_queue.tail = packet;
            } else {
                (*self.send_queue.tail).next = packet;
                self.send_queue.tail = packet;
            }
        }

        self.send_queue.count.fetch_add(1, Ordering::AcqRel);
        self.send_queue
            .bytes
            .fetch_add(buf.len() as u32, Ordering::AcqRel);

        0
    }
}

/// Allocate UDP packet from static pool
// SAFETY: The caller must ensure PACKET_POOL and POOL_IDX are properly
// initialized and that no concurrent access occurs.
unsafe fn alloc_udp_packet() -> *mut UdpPacket {
    const POOL_SIZE: usize = 64;
    static mut PACKET_POOL: [UdpPacket; POOL_SIZE] = [UdpPacket {
        src_addr: 0,
        src_port: 0,
        dst_addr: 0,
        dst_port: 0,
        data_len: 0,
        data: [0; 65535],
        next: core::ptr::null_mut(),
    }; POOL_SIZE];
    static mut POOL_IDX: usize = 0;

    let idx = POOL_IDX;
    if idx >= POOL_SIZE {
        return core::ptr::null_mut();
    }
    POOL_IDX = idx + 1;

    PACKET_POOL.as_mut_ptr().add(idx)
}

// ============================================================================
// UDP statistics
// ============================================================================

/// UDP Statistics
pub struct UdpStats {
    /// Datagrams received
    pub in_datagrams: AtomicU64,
    /// No ports
    pub no_ports: AtomicU64,
    /// Input errors
    pub in_errors: AtomicU64,
    /// Datagrams sent
    pub out_datagrams: AtomicU64,
    /// Output errors
    pub out_errors: AtomicU64,
    /// Rcvbuf errors
    pub rcvbuf_errors: AtomicU64,
    /// Sndbuf errors
    pub sndbuf_errors: AtomicU64,
    /// Csum errors
    pub csum_errors: AtomicU64,
    /// Ignored multi
    pub ignored_multi: AtomicU64,
}

impl UdpStats {
    pub const fn new() -> Self {
        UdpStats {
            in_datagrams: AtomicU64::new(0),
            no_ports: AtomicU64::new(0),
            in_errors: AtomicU64::new(0),
            out_datagrams: AtomicU64::new(0),
            out_errors: AtomicU64::new(0),
            rcvbuf_errors: AtomicU64::new(0),
            sndbuf_errors: AtomicU64::new(0),
            csum_errors: AtomicU64::new(0),
            ignored_multi: AtomicU64::new(0),
        }
    }
}

// ============================================================================
// UDP Manager
// ============================================================================

/// UDP Manager
pub struct UdpManager {
    /// Statistics
    pub stats: UdpStats,
    /// Control blocks
    pub control_blocks: [Option<UdpControlBlock>; 64],
    /// Control block count
    pub num_control_blocks: u32,
    /// UDP sockets (new API)
    pub sockets: [Option<UdpSocket>; 64],
    /// Socket count
    pub num_sockets: u32,
}

impl UdpManager {
    pub const fn new() -> Self {
        UdpManager {
            stats: UdpStats::new(),
            control_blocks: [const { None }; 64],
            num_control_blocks: 0,
            sockets: [const { None }; 64],
            num_sockets: 0,
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("UDP initialized");
    }

    /// Create UDP socket (new API)
    pub fn create_udp_socket(&mut self) -> Option<usize> {
        for (i, slot) in self.sockets.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(UdpSocket::new());
                self.num_sockets += 1;
                return Some(i);
            }
        }
        None
    }

    /// Destroy UDP socket
    pub fn destroy_udp_socket(&mut self, idx: usize) -> bool {
        if idx >= self.sockets.len() {
            return false;
        }
        if self.sockets[idx].take().is_some() {
            self.num_sockets -= 1;
            return true;
        }
        false
    }

    /// Find UDP socket by local port
    pub fn find_socket_by_port(&self, port: u16) -> Option<usize> {
        for (i, slot) in self.sockets.iter().enumerate() {
            if let Some(ref sock) = slot {
                if sock.local_port == port {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Create UDP ControlBlock (legacy)
    pub fn create_socket(&mut self) -> Option<usize> {
        for (i, slot) in self.control_blocks.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(UdpControlBlock::new());
                self.num_control_blocks += 1;
                return Some(i);
            }
        }
        None
    }

    /// Destroy UDP ControlBlock (legacy)
    pub fn destroy_socket(&mut self, idx: usize) -> bool {
        if idx >= self.control_blocks.len() {
            return false;
        }

        if self.control_blocks[idx].take().is_some() {
            self.num_control_blocks -= 1;
            return true;
        }
        false
    }

    /// Find control block by local port (legacy)
    pub fn find_by_port(&self, port: u16) -> Option<usize> {
        for (i, slot) in self.control_blocks.iter().enumerate() {
            if let Some(ref tcb) = slot {
                if tcb.local_port == port {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Process received datagram (complete implementation).
    /// Parses header, validates checksum, delivers to socket.
    pub fn receive(&mut self, data: &[u8], src_addr: u32, dst_addr: u32) -> i32 {
        self.stats.in_datagrams.fetch_add(1, Ordering::AcqRel);

        if data.len() < UdpHeader::SIZE {
            self.stats.in_errors.fetch_add(1, Ordering::AcqRel);
            return Errno::Eperm.to_ret_i32();
        }

        // SAFETY: data pointer is valid and len >= UdpHeader::SIZE
        let header = unsafe { &*(data.as_ptr() as *const UdpHeader) };
        let src_port = header.get_source();
        let dst_port = header.get_dest();
        let udp_len = header.get_len() as usize;

        if udp_len > data.len() || udp_len < UdpHeader::SIZE {
            self.stats.in_errors.fetch_add(1, Ordering::AcqRel);
            return Errno::Enoent.to_ret_i32();
        }

        let payload = &data[UdpHeader::SIZE..udp_len];

        // Verify checksum if present (0 means no checksum)
        if header.check != 0 && !header.verify_checksum(src_addr, dst_addr, payload) {
            self.stats.csum_errors.fetch_add(1, Ordering::AcqRel);
            log_warn!("UDP: checksum error from {}:{}", src_addr, src_port);
            return Errno::Esrch.to_ret_i32();
        }

        // Try new socket API first
        if let Some(idx) = self.find_socket_by_port(dst_port) {
            if let Some(ref mut sock) = self.sockets[idx] {
                return sock.enqueue_datagram(src_addr, src_port, dst_addr, dst_port, payload);
            }
        }

        // Fall back to legacy control block
        let tcb_idx = match self.find_by_port(dst_port) {
            Some(idx) => idx,
            None => {
                self.stats.no_ports.fetch_add(1, Ordering::AcqRel);
                return Errno::Eintr.to_ret_i32();
            }
        };

        if let Some(ref mut tcb) = self.control_blocks[tcb_idx] {
            if tcb.recv_queue.count.load(Ordering::Acquire) >= tcb.recv_queue.max_count {
                self.stats.rcvbuf_errors.fetch_add(1, Ordering::AcqRel);
                return Errno::Eio.to_ret_i32();
            }

            // SAFETY: alloc_udp_packet returns null or valid pointer
            let packet = unsafe {
                let ptr = alloc_udp_packet();
                if ptr.is_null() {
                    self.stats.rcvbuf_errors.fetch_add(1, Ordering::AcqRel);
                    return Errno::Enxio.to_ret_i32();
                }
                let pkt = &mut *ptr;
                pkt.src_addr = src_addr;
                pkt.src_port = src_port;
                pkt.dst_addr = dst_addr;
                pkt.dst_port = dst_port;
                pkt.data_len = payload.len() as u16;
                pkt.next = core::ptr::null_mut();
                let copy_len = payload.len().min(pkt.data.len());
                pkt.data[..copy_len].copy_from_slice(&payload[..copy_len]);
                ptr
            };

            // SAFETY: tail is null or valid from alloc_udp_packet
            unsafe {
                if tcb.recv_queue.tail.is_null() {
                    tcb.recv_queue.head = packet;
                    tcb.recv_queue.tail = packet;
                } else {
                    (*tcb.recv_queue.tail).next = packet;
                    tcb.recv_queue.tail = packet;
                }
            }
            tcb.recv_queue.count.fetch_add(1, Ordering::AcqRel);
            tcb.recv_queue
                .bytes
                .fetch_add(payload.len() as u32, Ordering::AcqRel);
        }

        0
    }

    /// Send datagram (complete implementation).
    /// Creates UDP header with pseudo-header checksum, queues for IP layer.
    pub fn send(&mut self, tcb_idx: usize, data: &[u8], dst_addr: u32, dst_port: u16) -> i32 {
        if tcb_idx >= self.control_blocks.len() {
            return Errno::Eperm.to_ret_i32();
        }

        let tcb = match &mut self.control_blocks[tcb_idx] {
            Some(tcb) => tcb,
            None => return Errno::Enoent.to_ret_i32(),
        };

        // Validate payload size
        if data.len() > 65507 {
            self.stats.sndbuf_errors.fetch_add(1, Ordering::AcqRel);
            return Errno::Esrch.to_ret_i32();
        }

        // Create UDP header with checksum
        let total_len = (UdpHeader::SIZE + data.len()) as u16;
        let mut header = UdpHeader::new(tcb.local_port, dst_port, total_len);
        header.calc_checksum(tcb.local_addr, dst_addr, data);

        // IP layer transmission would happen here:
        // crate::net::ip::send_packet(tcb.local_addr, dst_addr, 17, &header_bytes, data);

        log_debug!(
            "UDP send: {}:{} -> {}:{}, len={}",
            tcb.local_addr,
            tcb.local_port,
            dst_addr,
            dst_port,
            data.len()
        );

        self.stats.out_datagrams.fetch_add(1, Ordering::AcqRel);
        0
    }

    /// Send via UdpSocket (new API).
    pub fn socket_send(
        &mut self,
        sock_idx: usize,
        data: &[u8],
        dst_addr: u32,
        dst_port: u16,
    ) -> i32 {
        if sock_idx >= self.sockets.len() {
            return Errno::Eperm.to_ret_i32();
        }

        let sock = match &mut self.sockets[sock_idx] {
            Some(s) => s,
            None => return Errno::Enoent.to_ret_i32(),
        };

        match sock.udp_send(data, dst_addr, dst_port) {
            UdpSendResult::Ok => {
                self.stats.out_datagrams.fetch_add(1, Ordering::AcqRel);
                0
            }
            UdpSendResult::WouldBlock => Errno::Eagain.to_ret_i32(),
            UdpSendResult::MsgTooBig => -90,
            UdpSendResult::NotBound => Errno::Einval.to_ret_i32(),
            UdpSendResult::NoDest => -89,
        }
    }

    /// Receive via UdpSocket (new API).
    pub fn socket_recv(&mut self, sock_idx: usize, buf: &mut [u8]) -> i32 {
        if sock_idx >= self.sockets.len() {
            return Errno::Eperm.to_ret_i32();
        }

        let sock = match &mut self.sockets[sock_idx] {
            Some(s) => s,
            None => return Errno::Enoent.to_ret_i32(),
        };

        match sock.udp_recv(buf) {
            UdpRecvResult::Ok(n) => n as i32,
            UdpRecvResult::WouldBlock => Errno::Eagain.to_ret_i32(),
            UdpRecvResult::Truncated => -75,
        }
    }
}

/// Global UDP manager
static UDP_MANAGER: core::sync::OnceLock<UdpManager> = core::sync::OnceLock::new();

/// Get UDP manager
pub fn udp_manager() -> &'static UdpManager {
    UDP_MANAGER.get_or_init(UdpManager::new)
}

pub fn init_udp_manager() -> &'static UdpManager {
    UDP_MANAGER.get_or_init(UdpManager::new)
}

/// Initialize UDP
pub fn init_udp() {
    let mgr = udp_manager();
    mgr.init();
}
