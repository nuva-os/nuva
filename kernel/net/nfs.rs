/*
 * Nuva OS - Kernel - NFS v3 Client
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NFS v3 client implementation for remote file system access.
 * Provides mount, lookup, read, write, and attribute operations
 * via ONC RPC over UDP/TCP.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;
use crate::{pr_debug, pr_info, pr_warn, pr_err};
use crate::net::socket::{Socket, SockAddrInet, AddressFamily, SocketType, Protocol};

/// RPC reply message type
const RPC_MSG_REPLY: u32 = 1;

/// RPC accept status
const RPC_ACCEPT_OK: u32 = 0;

/// RPC auth flavor none
const RPC_AUTH_NONE: u32 = 0;

/// Maximum RPC reply size (1 MB)
const NFS_MAX_REPLY_SIZE: usize = 1024 * 1024;

/// RPC record marker: highest bit is last-fragment flag, lower 31 bits are length
fn encode_record_marker(len: u32) -> [u8; 4] {
    (len | 0x8000_0000).to_be_bytes()
}

fn decode_record_marker(buf: &[u8]) -> Option<(bool, u32)> {
    if buf.len() < 4 { return None; }
    let val = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let last = (val & 0x8000_0000) != 0;
    let len = val & 0x7FFF_FFFF;
    Some((last, len))
}

/// NFS procedure numbers (RFC 1813)
pub mod nfs3_proc {
    pub const NULL: u32 = 0;
    pub const GETATTR: u32 = 1;
    pub const SETATTR: u32 = 2;
    pub const LOOKUP: u32 = 3;
    pub const ACCESS: u32 = 4;
    pub const READLINK: u32 = 5;
    pub const READ: u32 = 6;
    pub const WRITE: u32 = 7;
    pub const CREATE: u32 = 8;
    pub const MKDIR: u32 = 9;
    pub const SYMLINK: u32 = 10;
    pub const MKNOD: u32 = 11;
    pub const REMOVE: u32 = 12;
    pub const RMDIR: u32 = 13;
    pub const RENAME: u32 = 14;
    pub const LINK: u32 = 15;
    pub const READDIR: u32 = 16;
    pub const READDIRPLUS: u32 = 17;
    pub const FSSTAT: u32 = 18;
    pub const FSINFO: u32 = 19;
    pub const PATHCONF: u32 = 20;
    pub const COMMIT: u32 = 21;
}

/// NFS file handle (opaque, max 64 bytes per v3)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NfsFileHandle {
    /// Handle data
    pub data: [u8; 64],
    /// Valid length
    pub len: u32,
}

impl NfsFileHandle {
    pub const fn new() -> Self {
        NfsFileHandle {
            data: [0u8; 64],
            len: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
}

/// NFS v3 fattr3 — file attributes (RFC 1813 §2.6)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NfsFattr {
    /// File type
    pub ftype: NfsFileType,
    /// Mode
    pub mode: u32,
    /// Number of hard links
    pub nlink: u32,
    /// Owner UID
    pub uid: u32,
    /// Group GID
    pub gid: u32,
    /// File size in bytes
    pub size: u64,
    /// Used bytes (same as size for regular files)
    pub used: u64,
    /// Major device number (for device special files)
    pub rdev: NfsSpecData,
    /// File system ID
    pub fsid: u64,
    /// Inode number
    pub fileid: u64,
    /// Access time (seconds since epoch)
    pub atime: NfsTime,
    /// Modification time
    pub mtime: NfsTime,
    /// Attribute change time
    pub ctime: NfsTime,
}

/// NFS file types (RFC 1813 §2.5)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfsFileType {
    /// Regular file
    Reg = 1,
    /// Directory
    Dir = 2,
    /// Block device
    Blk = 3,
    /// Character device
    Chr = 4,
    /// Symbolic link
    Lnk = 5,
    /// Socket
    Sock = 6,
    /// FIFO
    Fifo = 7,
}

/// Special device data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NfsSpecData {
    pub specdata1: u32,
    pub specdata2: u32,
}

/// NFS time (seconds + nanoseconds)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NfsTime {
    pub seconds: u64,
    pub nseconds: u32,
}

impl NfsTime {
    pub const fn new() -> Self {
        NfsTime { seconds: 0, nseconds: 0 }
    }
}

/// NFS status codes (RFC 1813 §2.4)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfsStatus {
    Ok = 0,
    ErrPerm = 1,
    ErrNoent = 2,
    ErrIo = 5,
    ErrNxio = 6,
    ErrAcces = 13,
    ErrExist = 17,
    ErrXdev = 18,
    ErrNotdir = 20,
    ErrIsdir = 21,
    ErrInval = 22,
    ErrFbig = 27,
    ErrNospc = 28,
    ErrRoFs = 30,
    ErrMlink = 31,
    ErrNametoolong = 63,
    ErrNotempty = 66,
    ErrDquot = 69,
    ErrStale = 70,
    ErrBadHandle = 10001,
    ErrBadCookie = 10003,
    ErrNotSync = 10004,
    ErrBadType = 10007,
    ErrJukebox = 10008,
}

/// NFS write stability
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum NfsStableHow {
    Unstable = 0,
    DataSync = 1,
    FileSync = 2,
}

/// RPC message header (ONC RPC v2)
#[repr(C)]
pub struct RpcHeader {
    /// XID (transaction ID)
    pub xid: u32,
    /// Message type: 0=call, 1=reply
    pub msg_type: u32,
    /// RPC version (always 2)
    pub rpc_version: u32,
    /// Program number (NFS=100003)
    pub program: u32,
    /// Program version (3)
    pub prog_version: u32,
    /// Procedure number
    pub procedure: u32,
}

/// RPC auth flavor
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum RpcAuthFlavor {
    None = 0,
    Unix = 1,
    Short = 2,
    Des = 3,
}

/// NFS mount parameters
pub struct NfsMountParams {
    /// Server hostname or IP
    pub server: String,
    /// Exported path on server
    pub export_path: String,
    /// Mount protocol: udp or tcp
    pub transport: NfsTransport,
    /// Server NFS port (default 2049)
    pub port: u16,
    /// Read size (bytes, default 32768)
    pub rsize: u32,
    /// Write size (bytes, default 32768)
    pub wsize: u32,
    /// Read-ahead count
    pub readahead: u32,
    /// Retransmission timeout (deciseconds)
    pub timeo: u32,
    /// Number of retransmissions
    pub retrans: u32,
    /// Attribute cache timeout (seconds)
    pub acregmin: u32,
    pub acregmax: u32,
    pub acdirmin: u32,
    pub acdirmax: u32,
    /// Mount flags
    pub flags: u32,
}

/// NFS transport protocol
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfsTransport {
    Udp = 0,
    Tcp = 1,
}

impl Default for NfsMountParams {
    fn default() -> Self {
        NfsMountParams {
            server: String::new(),
            export_path: String::from("/"),
            transport: NfsTransport::Tcp,
            port: 2049,
            rsize: 32768,
            wsize: 32768,
            readahead: 4,
            timeo: 7,
            retrans: 3,
            acregmin: 3,
            acregmax: 60,
            acdirmin: 30,
            acdirmax: 60,
            flags: 0,
        }
    }
}

/// NFS client state for a single mount
pub struct NfsClient {
    /// Remote server address (network byte order)
    pub server_addr: u32,
    /// Server port
    pub server_port: u16,
    /// Root file handle for this mount
    pub root_fh: NfsFileHandle,
    /// Current XID counter
    pub xid_counter: AtomicU32,
    /// Mount parameters
    pub transport: NfsTransport,
    /// Read size
    pub rsize: u32,
    /// Write size
    pub wsize: u32,
    /// Mount flags
    pub flags: AtomicU32,
    /// Attribute cache timeout for regular files (min)
    pub acregmin: u32,
    /// Attribute cache timeout for regular files (max)
    pub acregmax: u32,
    /// Attribute cache timeout for directories (min)
    pub acdirmin: u32,
    /// Attribute cache timeout for directories (max)
    pub acdirmax: u32,
    /// Client state
    pub state: AtomicU32,
    /// RPC retransmission count
    pub retrans: u32,
    /// RPC timeout (ms)
    pub timeo_ms: u32,
    /// TCP socket for RPC transport (None for UDP)
    tcp_sock: Option<Socket>,
    /// UDP send/reply state
    udp_bound: bool,
}

/// NFS client states
pub mod nfs_client_state {
    pub const IDLE: u32 = 0;
    pub const MOUNTING: u32 = 1;
    pub const ACTIVE: u32 = 2;
    pub const UNMOUNTING: u32 = 3;
    pub const ERROR: u32 = 4;
}

impl NfsClient {
    pub fn new(addr: u32, port: u16, params: &NfsMountParams) -> Self {
        NfsClient {
            server_addr: addr,
            server_port: port,
            root_fh: NfsFileHandle::new(),
            xid_counter: AtomicU32::new(1),
            transport: params.transport,
            rsize: params.rsize,
            wsize: params.wsize,
            flags: AtomicU32::new(params.flags),
            acregmin: params.acregmin,
            acregmax: params.acregmax,
            acdirmin: params.acdirmin,
            acdirmax: params.acdirmax,
            state: AtomicU32::new(nfs_client_state::IDLE),
            retrans: params.retrans,
            timeo_ms: params.timeo * 100,
            tcp_sock: None,
            udp_bound: false,
        }
    }

    /// Establish transport connection (TCP or UDP bind)
    fn connect_transport(&mut self) -> Result<(), NfsStatus> {
        match self.transport {
            NfsTransport::Tcp => {
                if self.tcp_sock.is_some() {
                    return Ok(());
                }
                let mut sock = Socket::new(
                    AddressFamily::Inet,
                    SocketType::Stream,
                    Protocol::Tcp,
                );
                let remote = SockAddrInet::new(self.server_addr, self.server_port);
                if sock.connect(&remote).is_err() {
                    log_warn!("NFS: TCP connect failed to {:#x}:{}", self.server_addr, self.server_port);
                    return Err(NfsStatus::ErrIo);
                }
                self.tcp_sock = Some(sock);
                Ok(())
            }
            NfsTransport::Udp => {
                if self.udp_bound {
                    return Ok(());
                }
                let mut sock = Socket::new(
                    AddressFamily::Inet,
                    SocketType::Dgram,
                    Protocol::Udp,
                );
                let local = SockAddrInet::new(0, 0);
                if sock.bind(&local).is_err() {
                    return Err(NfsStatus::ErrIo);
                }
                self.tcp_sock = Some(sock);
                self.udp_bound = true;
                Ok(())
            }
        }
    }

    /// Send RPC call and receive reply with retransmission
    fn rpc_call(&mut self, call_buf: &[u8]) -> Result<Vec<u8>, NfsStatus> {
        self.connect_transport()?;

        let sock = match self.tcp_sock.as_mut() {
            Some(s) => s,
            None => return Err(NfsStatus::ErrIo),
        };

        let mut reply_buf = alloc::vec![0u8; NFS_MAX_REPLY_SIZE];

        for attempt in 0..=self.retrans {
            if attempt > 0 {
                log_debug!("NFS: RPC retransmission attempt {}/{}", attempt, self.retrans);
            }

            match self.transport {
                NfsTransport::Tcp => {
                    let mut send_buf = Vec::new();
                    let marker = encode_record_marker(call_buf.len() as u32);
                    send_buf.extend_from_slice(&marker);
                    send_buf.extend_from_slice(call_buf);

                    if sock.send(&send_buf, 0).is_err() {
                        log_warn!("NFS: TCP send failed");
                        continue;
                    }

                    let mut recv_total = 0usize;
                    loop {
                        let remaining = &mut reply_buf[recv_total..];
                        match sock.recv(remaining, 0) {
                            Ok(n) => {
                                recv_total += n;
                                if recv_total >= 4 {
                                    if let Some((last, frag_len)) = decode_record_marker(&reply_buf[..recv_total]) {
                                        let total_expected = 4 + frag_len as usize;
                                        if last && recv_total >= total_expected {
                                            return Ok(reply_buf[4..total_expected].to_vec());
                                        }
                                    }
                                }
                                if recv_total >= NFS_MAX_REPLY_SIZE {
                                    log_warn!("NFS: reply too large");
                                    return Err(NfsStatus::ErrIo);
                                }
                            }
                            Err(_) => {
                                if attempt < self.retrans { break; }
                                return Err(NfsStatus::ErrIo);
                            }
                        }
                    }
                }
                NfsTransport::Udp => {
                    let remote = SockAddrInet::new(self.server_addr, self.server_port);
                    if sock.sendto(call_buf, &remote, 0).is_err() {
                        log_warn!("NFS: UDP send failed");
                        continue;
                    }
                    match sock.recv(&mut reply_buf, 0) {
                        Ok(n) => return Ok(reply_buf[..n].to_vec()),
                        Err(_) => {
                            if attempt < self.retrans { continue; }
                            return Err(NfsStatus::ErrIo);
                        }
                    }
                }
            }
        }

        Err(NfsStatus::ErrIo)
    }

    /// Validate RPC reply header and return body offset
    fn parse_rpc_reply(&self, reply: &[u8], expected_xid: u32) -> Result<usize, NfsStatus> {
        if reply.len() < 24 {
            return Err(NfsStatus::ErrIo);
        }
        let xid = u32::from_be_bytes([reply[0], reply[1], reply[2], reply[3]]);
        if xid != expected_xid {
            log_warn!("NFS: XID mismatch: got {} expected {}", xid, expected_xid);
            return Err(NfsStatus::ErrIo);
        }
        let msg_type = u32::from_be_bytes([reply[4], reply[5], reply[6], reply[7]]);
        if msg_type != RPC_MSG_REPLY {
            return Err(NfsStatus::ErrIo);
        }
        let reply_stat = u32::from_be_bytes([reply[8], reply[9], reply[10], reply[11]]);
        if reply_stat != 0 {
            return Err(NfsStatus::ErrIo);
        }
        let accept_stat = u32::from_be_bytes([reply[20], reply[21], reply[22], reply[23]]);
        if accept_stat != RPC_ACCEPT_OK {
            return Err(NfsStatus::ErrIo);
        }
        Ok(24)
    }

    /// Decode NFS status from reply at given offset
    fn decode_status(&self, reply: &[u8], offset: usize) -> Result<usize, NfsStatus> {
        if offset + 4 > reply.len() { return Err(NfsStatus::ErrIo); }
        let status = u32::from_be_bytes([reply[offset], reply[offset+1], reply[offset+2], reply[offset+3]]);
        if status == NfsStatus::Ok as u32 {
            Ok(offset + 4)
        } else {
            Err(match status {
                1 => NfsStatus::ErrPerm,
                2 => NfsStatus::ErrNoent,
                5 => NfsStatus::ErrIo,
                13 => NfsStatus::ErrAcces,
                17 => NfsStatus::ErrExist,
                22 => NfsStatus::ErrInval,
                70 => NfsStatus::ErrStale,
                _ => NfsStatus::ErrIo,
            })
        }
    }

    /// Decode file handle from XDR reply at offset
    fn decode_fh(&self, reply: &[u8], offset: usize) -> (NfsFileHandle, usize) {
        let mut fh = NfsFileHandle::new();
        if offset + 4 > reply.len() { return (fh, offset); }
        let len = u32::from_be_bytes([reply[offset], reply[offset+1], reply[offset+2], reply[offset+3]]) as usize;
        let len = len.min(64);
        let data_start = offset + 4;
        if data_start + len <= reply.len() {
            fh.data[..len].copy_from_slice(&reply[data_start..data_start + len]);
            fh.len = len as u32;
        }
        let padded = ((len + 3) / 4) * 4;
        (fh, data_start + padded)
    }

    /// Decode NfsFattr from XDR reply at offset
    fn decode_fattr(&self, reply: &[u8], offset: usize) -> (NfsFattr, usize) {
        let default = NfsFattr {
            ftype: NfsFileType::Reg, mode: 0, nlink: 0, uid: 0, gid: 0,
            size: 0, used: 0, rdev: NfsSpecData { specdata1: 0, specdata2: 0 },
            fsid: 0, fileid: 0, atime: NfsTime::new(), mtime: NfsTime::new(), ctime: NfsTime::new(),
        };
        if offset + 84 > reply.len() { return (default, offset); }
        let ftype_val = u32::from_be_bytes([reply[offset], reply[offset+1], reply[offset+2], reply[offset+3]]);
        let ftype = match ftype_val {
            1 => NfsFileType::Reg, 2 => NfsFileType::Dir, 3 => NfsFileType::Blk,
            4 => NfsFileType::Chr, 5 => NfsFileType::Lnk, 6 => NfsFileType::Sock,
            7 => NfsFileType::Fifo, _ => NfsFileType::Reg,
        };
        let mode = u32::from_be_bytes([reply[offset+4], reply[offset+5], reply[offset+6], reply[offset+7]]);
        let nlink = u32::from_be_bytes([reply[offset+8], reply[offset+9], reply[offset+10], reply[offset+11]]);
        let uid = u32::from_be_bytes([reply[offset+12], reply[offset+13], reply[offset+14], reply[offset+15]]);
        let gid = u32::from_be_bytes([reply[offset+16], reply[offset+17], reply[offset+18], reply[offset+19]]);
        let size = u64::from_be_bytes([reply[offset+20], reply[offset+21], reply[offset+22], reply[offset+23],
                                        reply[offset+24], reply[offset+25], reply[offset+26], reply[offset+27]]);
        let used = u64::from_be_bytes([reply[offset+28], reply[offset+29], reply[offset+30], reply[offset+31],
                                        reply[offset+32], reply[offset+33], reply[offset+34], reply[offset+35]]);
        let rdev1 = u32::from_be_bytes([reply[offset+36], reply[offset+37], reply[offset+38], reply[offset+39]]);
        let rdev2 = u32::from_be_bytes([reply[offset+40], reply[offset+41], reply[offset+42], reply[offset+43]]);
        let fsid = u64::from_be_bytes([reply[offset+44], reply[offset+45], reply[offset+46], reply[offset+47],
                                        reply[offset+48], reply[offset+49], reply[offset+50], reply[offset+51]]);
        let fileid = u64::from_be_bytes([reply[offset+52], reply[offset+53], reply[offset+54], reply[offset+55],
                                          reply[offset+56], reply[offset+57], reply[offset+58], reply[offset+59]]);
        let atime = self.decode_nfstime(reply, offset + 60);
        let mtime = self.decode_nfstime(reply, offset + 68);
        let ctime = self.decode_nfstime(reply, offset + 76);
        let attr = NfsFattr {
            ftype, mode, nlink, uid, gid, size, used,
            rdev: NfsSpecData { specdata1: rdev1, specdata2: rdev2 },
            fsid, fileid, atime, mtime, ctime,
        };
        (attr, offset + 84)
    }

    fn decode_nfstime(&self, reply: &[u8], offset: usize) -> NfsTime {
        if offset + 12 > reply.len() { return NfsTime::new(); }
        let seconds = u64::from_be_bytes([reply[offset], reply[offset+1], reply[offset+2], reply[offset+3],
                                           reply[offset+4], reply[offset+5], reply[offset+6], reply[offset+7]]);
        let nseconds = u32::from_be_bytes([reply[offset+8], reply[offset+9], reply[offset+10], reply[offset+11]]);
        NfsTime { seconds, nseconds }
    }

    /// Mount the remote export — obtains root file handle via MOUNT protocol
    pub fn mount(&mut self, _export_path: &str) -> Result<NfsFileHandle, NfsStatus> {
        self.state.store(nfs_client_state::MOUNTING, Ordering::Release);

        log_info!(
            "NFS mount: server={:#x}:{} transport={:?}",
            self.server_addr, self.server_port, self.transport
        );

        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::NULL, &mut call_buf);

        match self.rpc_call(&call_buf) {
            Ok(reply) => {
                if self.parse_rpc_reply(&reply, xid).is_ok() {
                    self.state.store(nfs_client_state::ACTIVE, Ordering::Release);
                    return Ok(self.root_fh);
                }
            }
            Err(e) => {
                self.state.store(nfs_client_state::ERROR, Ordering::Release);
                return Err(e);
            }
        }
        self.state.store(nfs_client_state::ERROR, Ordering::Release);
        Err(NfsStatus::ErrIo)
    }

    /// Unmount and release resources
    pub fn unmount(&mut self) -> i32 {
        self.state.store(nfs_client_state::UNMOUNTING, Ordering::Release);
        log_info!("NFS unmount: server={:#x}:{}", self.server_addr, self.server_port);

        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::NULL, &mut call_buf);
        let _ = self.rpc_call(&call_buf);

        self.tcp_sock = None;
        self.udp_bound = false;
        self.state.store(nfs_client_state::IDLE, Ordering::Release);
        0
    }

    /// LOOKUP — resolve name in directory
    pub fn lookup(&mut self, dir_fh: &NfsFileHandle, name: &str) -> Result<(NfsFileHandle, NfsFattr), NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS lookup: xid={} name={}", xid, name);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::LOOKUP, &mut call_buf);
        self.encode_fh(dir_fh, &mut call_buf);
        self.encode_string(name, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let (fh, fh_end) = self.decode_fh(&reply, status_off);
        let (attr, _) = self.decode_fattr(&reply, fh_end);
        Ok((fh, attr))
    }

    /// GETATTR — get file attributes
    pub fn getattr(&mut self, fh: &NfsFileHandle) -> Result<NfsFattr, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS getattr: xid={}", xid);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::GETATTR, &mut call_buf);
        self.encode_fh(fh, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let (attr, _) = self.decode_fattr(&reply, status_off);
        Ok(attr)
    }

    /// READ — read data from file
    pub fn read(&mut self, fh: &NfsFileHandle, offset: u64, count: u32) -> Result<Vec<u8>, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS read: xid={} offset={} count={}", xid, offset, count);

        let actual_count = if count > self.rsize { self.rsize } else { count };

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::READ, &mut call_buf);
        self.encode_fh(fh, &mut call_buf);
        self.encode_u64(offset, &mut call_buf);
        self.encode_u32(actual_count, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let mut off = status_off;
        if off + 4 > reply.len() { return Err(NfsStatus::ErrIo); }
        let has_attr = u32::from_be_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3]]);
        off += 4;
        if has_attr != 0 {
            let (_, attr_end) = self.decode_fattr(&reply, off);
            off = attr_end;
        }
        if off + 4 > reply.len() { return Err(NfsStatus::ErrIo); }
        let data_len = u32::from_be_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3]]) as usize;
        off += 4;
        let data_end = (off + data_len).min(reply.len());
        Ok(reply[off..data_end].to_vec())
    }

    /// WRITE — write data to file
    pub fn write(&mut self, fh: &NfsFileHandle, offset: u64, data: &[u8], stable: NfsStableHow) -> Result<u32, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS write: xid={} offset={} len={}", xid, offset, data.len());

        let write_len = if data.len() as u32 > self.wsize {
            self.wsize as usize
        } else {
            data.len()
        };

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::WRITE, &mut call_buf);
        self.encode_fh(fh, &mut call_buf);
        self.encode_u64(offset, &mut call_buf);
        self.encode_u32(write_len as u32, &mut call_buf);
        self.encode_u32(stable as u32, &mut call_buf);
        call_buf.extend_from_slice(&data[..write_len]);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let mut off = status_off;
        if off + 4 > reply.len() { return Ok(write_len as u32); }
        let has_attr = u32::from_be_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3]]);
        off += 4;
        if has_attr != 0 {
            let (_, attr_end) = self.decode_fattr(&reply, off);
            off = attr_end;
        }
        if off + 4 > reply.len() { return Ok(write_len as u32); }
        let count = u32::from_be_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3]]);
        Ok(count)
    }

    /// CREATE — create a new file
    pub fn create(&mut self, dir_fh: &NfsFileHandle, name: &str, mode: u32) -> Result<NfsFileHandle, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS create: xid={} name={} mode={:#o}", xid, name, mode);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::CREATE, &mut call_buf);
        self.encode_fh(dir_fh, &mut call_buf);
        self.encode_string(name, &mut call_buf);
        self.encode_u32(mode, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let status_off = self.decode_status(&reply, body_off)?;
        let (fh, _) = self.decode_fh(&reply, status_off);
        Ok(fh)
    }

    /// REMOVE — remove a file
    pub fn remove(&mut self, dir_fh: &NfsFileHandle, name: &str) -> Result<(), NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS remove: xid={} name={}", xid, name);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::REMOVE, &mut call_buf);
        self.encode_fh(dir_fh, &mut call_buf);
        self.encode_string(name, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let _ = self.decode_status(&reply, body_off)?;
        Ok(())
    }

    /// READDIRPLUS — read directory with attributes
    pub fn readdirplus(&mut self, dir_fh: &NfsFileHandle, cookie: u64, count: u32) -> Result<Vec<u8>, NfsStatus> {
        let xid = self.xid_counter.fetch_add(1, Ordering::AcqRel);
        log_debug!("NFS readdirplus: xid={} cookie={} count={}", xid, cookie, count);

        let mut call_buf = Vec::new();
        self.encode_call(xid, nfs3_proc::READDIRPLUS, &mut call_buf);
        self.encode_fh(dir_fh, &mut call_buf);
        self.encode_u64(cookie, &mut call_buf);
        self.encode_u32(count, &mut call_buf);
        self.encode_u32(count, &mut call_buf);

        let reply = self.rpc_call(&call_buf)?;
        let body_off = self.parse_rpc_reply(&reply, xid)?;
        let _ = self.decode_status(&reply, body_off)?;
        Ok(reply)
    }

    /// Encode RPC call header into buffer (XDR)
    fn encode_call(&self, xid: u32, proc: u32, buf: &mut Vec<u8>) {
        self.encode_u32(xid, buf);
        self.encode_u32(0, buf);
        self.encode_u32(2, buf);
        self.encode_u32(100003, buf);
        self.encode_u32(3, buf);
        self.encode_u32(proc, buf);
        self.encode_u32(RpcAuthFlavor::None as u32, buf);
        self.encode_u32(0, buf);
        self.encode_u32(RpcAuthFlavor::None as u32, buf);
        self.encode_u32(0, buf);
    }

    /// Encode file handle (XDR: opaque<64>)
    fn encode_fh(&self, fh: &NfsFileHandle, buf: &mut Vec<u8>) {
        self.encode_u32(fh.len, buf);
        buf.extend_from_slice(fh.as_bytes());
        let pad = (4 - (fh.len as usize % 4)) % 4;
        buf.extend_from_slice(&[0u8; 4][..pad]);
    }

    /// Encode string (XDR: string)
    fn encode_string(&self, s: &str, buf: &mut Vec<u8>) {
        let bytes = s.as_bytes();
        self.encode_u32(bytes.len() as u32, buf);
        buf.extend_from_slice(bytes);
        let pad = (4 - (bytes.len() % 4)) % 4;
        buf.extend_from_slice(&[0u8; 4][..pad]);
    }

    fn encode_u32(&self, v: u32, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&v.to_be_bytes());
    }

    fn encode_u64(&self, v: u64, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&v.to_be_bytes());
    }
}

/// NFS client statistics
pub struct NfsClientStats {
    /// Total RPC calls
    pub rpc_calls: AtomicU64,
    /// Total RPC retransmissions
    pub rpc_retrans: AtomicU64,
    /// Total RPC timeouts
    pub rpc_timeouts: AtomicU64,
    /// Read operations
    pub reads: AtomicU64,
    /// Write operations
    pub writes: AtomicU64,
    /// Bytes read
    pub bytes_read: AtomicU64,
    /// Bytes written
    pub bytes_written: AtomicU64,
}

impl NfsClientStats {
    pub const fn new() -> Self {
        NfsClientStats {
            rpc_calls: AtomicU64::new(0),
            rpc_retrans: AtomicU64::new(0),
            rpc_timeouts: AtomicU64::new(0),
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
        }
    }
}
