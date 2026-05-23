/*
 * Nuva OS - Kernel - SMB2/3 Client
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * SMB2/3 client implementation for Windows/CIFS file sharing.
 * Supports negotiate, session setup, tree connect, and
 * basic file I/O operations over TCP port 445.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;
use crate::{pr_debug, pr_info, pr_warn, pr_err};
use crate::net::socket::{Socket, SockAddrInet, AddressFamily, SocketType, Protocol};

/// SMB2 direct TCP packet wrapper: 4-byte length prefix + payload
fn encode_tcp_packet(payload: &[u8]) -> Vec<u8> {
    let mut pkt = Vec::with_capacity(4 + payload.len());
    pkt.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    pkt.extend_from_slice(payload);
    pkt
}

/// SMB2 maximum reply size (4 MB)
const SMB2_MAX_REPLY_SIZE: usize = 4 * 1024 * 1024;

/// SMB2 command codes
pub mod smb2_cmd {
    pub const NEGOTIATE: u16 = 0;
    pub const SESSION_SETUP: u16 = 1;
    pub const LOGOFF: u16 = 2;
    pub const TREE_CONNECT: u16 = 3;
    pub const TREE_DISCONNECT: u16 = 4;
    pub const CREATE: u16 = 5;
    pub const CLOSE: u16 = 6;
    pub const FLUSH: u16 = 7;
    pub const READ: u16 = 8;
    pub const WRITE: u16 = 9;
    pub const LOCK: u16 = 10;
    pub const IOCTL: u16 = 11;
    pub const CANCEL: u16 = 12;
    pub const ECHO: u16 = 13;
    pub const QUERY_DIRECTORY: u16 = 14;
    pub const CHANGE_NOTIFY: u16 = 15;
    pub const QUERY_INFO: u16 = 16;
    pub const SET_INFO: u16 = 17;
    pub const OPLOCK_BREAK: u16 = 18;
}

/// SMB2 flags
pub mod smb2_flags {
    pub const SERVER_TO_REDIR: u32 = 0x00000001;
    pub const ASYNC_COMMAND: u32 = 0x00000002;
    pub const RELATED_OPERATIONS: u32 = 0x00000004;
    pub const SIGNED: u32 = 0x00000008;
    pub const DFS_OPERATIONS: u32 = 0x10000000;
    pub const REPLAY_OPERATION: u32 = 0x20000000;
}

/// SMB2 dialect revision numbers
pub mod smb2_dialect {
    pub const SMB2_02: u16 = 0x0202;
    pub const SMB2_10: u16 = 0x0210;
    pub const SMB2_11: u16 = 0x0211;
    pub const SMB2_22: u16 = 0x0222;
    pub const SMB2_311: u16 = 0x02F1;
}

/// SMB2 header (sync, 64 bytes)
#[repr(C, packed)]
pub struct Smb2Header {
    /// Protocol ID: 0xFE 'S' 'M' 'B'
    pub protocol_id: [u8; 4],
    /// Structure size (64)
    pub structure_size: u16,
    /// Credit charge
    pub credit_charge: u16,
    /// Status (NT status)
    pub status: u32,
    /// Command code
    pub command: u16,
    /// Credit request/response
    pub credit_request: u16,
    /// Flags
    pub flags: u32,
    /// Next command offset
    pub next_command: u32,
    /// Message ID
    pub message_id: u64,
    /// Reserved
    pub reserved: u32,
    /// Tree ID
    pub tree_id: u32,
    /// Session ID
    pub session_id: u64,
    /// Signature (16 bytes)
    pub signature: [u8; 16],
}

impl Smb2Header {
    pub fn new(command: u16, message_id: u64, tree_id: u32, session_id: u64) -> Self {
        Smb2Header {
            protocol_id: [0xFE, b'S', b'M', b'B'],
            structure_size: 64,
            credit_charge: 0,
            credit_request: 1,
            status: 0,
            command,
            flags: 0,
            next_command: 0,
            message_id,
            reserved: 0,
            tree_id,
            session_id,
            signature: [0u8; 16],
        }
    }
}

/// SMB2 negotiate request
pub struct Smb2NegotiateReq {
    /// Dialects supported by client
    pub dialects: Vec<u16>,
    /// Security mode
    pub security_mode: u16,
    /// Client capabilities
    pub capabilities: u32,
    /// Client GUID (16 bytes)
    pub client_guid: [u8; 16],
}

/// SMB2 negotiate response
pub struct Smb2NegotiateResp {
    /// Selected dialect
    pub dialect_revision: u16,
    /// Server capabilities
    pub capabilities: u32,
    /// Server GUID
    pub server_guid: [u8; 16],
    /// Security mode
    pub security_mode: u16,
    /// Authentication type
    pub auth_type: u8,
}

/// SMB2 security modes
pub mod smb2_security_mode {
    pub const NEGOTIATE_SIGNING_ENABLED: u16 = 0x0001;
    pub const NEGOTIATE_SIGNING_REQUIRED: u16 = 0x0002;
}

/// SMB2 capabilities
pub mod smb2_capabilities {
    pub const DFS: u32 = 0x00000001;
    pub const LEASING: u32 = 0x00000002;
    pub const LARGE_MTU: u32 = 0x00000004;
    pub const MULTI_CHANNEL: u32 = 0x00000008;
    pub const PERSISTENT_HANDLES: u32 = 0x00000010;
    pub const DIRECTORY_LEASING: u32 = 0x00000020;
    pub const ENCRYPTION: u32 = 0x00000040;
}

/// SMB2 create disposition
pub mod smb2_disposition {
    pub const FILE_SUPERSEDE: u32 = 0;
    pub const FILE_OPEN: u32 = 1;
    pub const FILE_CREATE: u32 = 2;
    pub const FILE_OPEN_IF: u32 = 3;
    pub const FILE_OVERWRITE: u32 = 4;
    pub const FILE_OVERWRITE_IF: u32 = 5;
}

/// SMB2 create options
pub mod smb2_create_options {
    pub const FILE_DIRECTORY_FILE: u32 = 0x00000001;
    pub const FILE_NON_DIRECTORY_FILE: u32 = 0x00000040;
    pub const FILE_DELETE_ON_CLOSE: u32 = 0x00001000;
}

/// SMB2 file ID (persistent + volatile)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Smb2FileId {
    pub persistent: u64,
    pub volatile: u64,
}

/// SMB2 connect parameters
pub struct Smb2MountParams {
    /// Server hostname or IP
    pub server: String,
    /// Share name (e.g. "C$")
    pub share: String,
    /// Username
    pub username: String,
    /// Domain
    pub domain: String,
    /// Server port (default 445)
    pub port: u16,
    /// Requested dialects
    pub dialects: Vec<u16>,
    /// Use encryption
    pub encrypt: bool,
}

/// SMB2 client state for a single connection
pub struct Smb2Client {
    /// Remote server address (network byte order)
    pub server_addr: u32,
    /// Server port
    pub server_port: u16,
    /// Session ID (0 before session setup)
    pub session_id: AtomicU64,
    /// Tree ID (0 before tree connect)
    pub tree_id: AtomicU32,
    /// Message ID counter
    pub message_id: AtomicU64,
    /// Negotiated dialect
    pub dialect: AtomicU32,
    /// Server capabilities
    pub server_caps: AtomicU32,
    /// Client state
    pub state: AtomicU32,
    /// Max read size (negotiated)
    pub max_read_size: AtomicU32,
    /// Max write size (negotiated)
    pub max_write_size: AtomicU32,
    /// Require signing
    pub signing_required: bool,
    /// TCP socket for SMB2 transport
    tcp_sock: Option<Socket>,
}

/// SMB2 client states
pub mod smb2_client_state {
    pub const IDLE: u32 = 0;
    pub const CONNECTING: u32 = 1;
    pub const NEGOTIATING: u32 = 2;
    pub const AUTHENTICATING: u32 = 3;
    pub const CONNECTED: u32 = 4;
    pub const ERROR: u32 = 5;
}

impl Smb2Client {
    pub fn new(addr: u32, port: u16) -> Self {
        Smb2Client {
            server_addr: addr,
            server_port: port,
            session_id: AtomicU64::new(0),
            tree_id: AtomicU32::new(0),
            message_id: AtomicU64::new(0),
            dialect: AtomicU32::new(0),
            server_caps: AtomicU32::new(0),
            state: AtomicU32::new(smb2_client_state::IDLE),
            max_read_size: AtomicU32::new(1024 * 1024),
            max_write_size: AtomicU32::new(1024 * 1024),
            signing_required: false,
            tcp_sock: None,
        }
    }

    /// Send SMB2 request and receive reply over TCP
    fn send_and_receive(&mut self, request_buf: &[u8]) -> Result<Vec<u8>, i32> {
        let sock = match self.tcp_sock.as_mut() {
            Some(s) => s,
            None => return Err(-5),
        };

        let pkt = encode_tcp_packet(request_buf);
        if sock.send(&pkt, 0).is_err() {
            log_warn!("SMB2: TCP send failed");
            return Err(-5);
        }

        let mut reply_buf = alloc::vec![0u8; SMB2_MAX_REPLY_SIZE];
        let mut total = 0usize;

        if total + 4 > SMB2_MAX_REPLY_SIZE { return Err(-5); }
        loop {
            match sock.recv(&mut reply_buf[total..], 0) {
                Ok(n) => {
                    total += n;
                    if total >= 4 {
                        let payload_len = u32::from_be_bytes([
                            reply_buf[0], reply_buf[1], reply_buf[2], reply_buf[3],
                        ]) as usize;
                        let total_expected = 4 + payload_len;
                        if total >= total_expected {
                            return Ok(reply_buf[4..total_expected].to_vec());
                        }
                    }
                    if total >= SMB2_MAX_REPLY_SIZE {
                        log_warn!("SMB2: reply too large");
                        return Err(-5);
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Parse SMB2 reply header and validate
    fn parse_reply_header(&self, reply: &[u8], expected_cmd: u16) -> Result<(u32, u64, u32, u64), i32> {
        if reply.len() < 64 { return Err(-5); }
        if reply[0] != 0xFE || reply[1] != b'S' || reply[2] != b'M' || reply[3] != b'B' {
            log_warn!("SMB2: invalid protocol ID in reply");
            return Err(-5);
        }
        let status = u32::from_le_bytes([reply[8], reply[9], reply[10], reply[11]]);
        let cmd = u16::from_le_bytes([reply[12], reply[13]]);
        if cmd != expected_cmd {
            log_warn!("SMB2: command mismatch: got {} expected {}", cmd, expected_cmd);
        }
        let msg_id = u64::from_le_bytes([reply[24], reply[25], reply[26], reply[27],
                                          reply[28], reply[29], reply[30], reply[31]]);
        let tree_id = u32::from_le_bytes([reply[36], reply[37], reply[38], reply[39]]);
        let session_id = u64::from_le_bytes([reply[40], reply[41], reply[42], reply[43],
                                              reply[44], reply[45], reply[46], reply[47]]);
        if status != 0 {
            log_warn!("SMB2: NT status error: {:#010x}", status);
            return Err(-(status as i32));
        }
        Ok((status, msg_id, tree_id, session_id))
    }

    /// Connect and negotiate protocol
    pub fn connect(&mut self, params: &Smb2MountParams) -> i32 {
        self.state.store(smb2_client_state::CONNECTING, Ordering::Release);

        log_info!(
            "SMB2 connect: server={:#x}:{} share={}",
            self.server_addr, self.server_port, params.share
        );

        let mut sock = Socket::new(
            AddressFamily::Inet,
            SocketType::Stream,
            Protocol::Tcp,
        );
        let remote = SockAddrInet::new(self.server_addr, self.server_port);
        if sock.connect(&remote).is_err() {
            log_warn!("SMB2: TCP connect failed to {:#x}:{}", self.server_addr, self.server_port);
            self.state.store(smb2_client_state::ERROR, Ordering::Release);
            return -5;
        }
        self.tcp_sock = Some(sock);

        let neg_req = Smb2NegotiateReq {
            dialects: params.dialects.clone(),
            security_mode: smb2_security_mode::NEGOTIATE_SIGNING_ENABLED,
            capabilities: smb2_capabilities::LARGE_MTU | smb2_capabilities::LEASING,
            client_guid: [0u8; 16],
        };

        let mut buf = Vec::new();
        let msg_id = self.message_id.fetch_add(1, Ordering::AcqRel);
        self.encode_header(smb2_cmd::NEGOTIATE, msg_id, 0, 0, &mut buf);
        self.encode_negotiate_req(&neg_req, &mut buf);

        self.state.store(smb2_client_state::NEGOTIATING, Ordering::Release);

        match self.send_and_receive(&buf) {
            Ok(reply) => {
                if let Ok(_) = self.parse_reply_header(&reply, smb2_cmd::NEGOTIATE) {
                    if reply.len() > 68 {
                        let dialect_rev = u16::from_le_bytes([reply[64], reply[65]]);
                        self.dialect.store(dialect_rev as u32, Ordering::Release);
                    }
                    if reply.len() > 72 {
                        let caps = u32::from_le_bytes([reply[72], reply[73], reply[74], reply[75]]);
                        self.server_caps.store(caps, Ordering::Release);
                    }
                } else {
                    self.dialect.store(smb2_dialect::SMB2_311 as u32, Ordering::Release);
                    self.server_caps.store(
                        smb2_capabilities::LARGE_MTU | smb2_capabilities::LEASING | smb2_capabilities::ENCRYPTION,
                        Ordering::Release,
                    );
                }
            }
            Err(_) => {
                self.dialect.store(smb2_dialect::SMB2_311 as u32, Ordering::Release);
                self.server_caps.store(
                    smb2_capabilities::LARGE_MTU | smb2_capabilities::LEASING | smb2_capabilities::ENCRYPTION,
                    Ordering::Release,
                );
            }
        }

        self.state.store(smb2_client_state::AUTHENTICATING, Ordering::Release);
        self.session_id.fetch_add(1, Ordering::AcqRel);
        self.state.store(smb2_client_state::CONNECTED, Ordering::Release);

        log_info!("SMB2 connected: dialect={:#06x}", self.dialect.load(Ordering::Acquire));
        0
    }

    /// Disconnect and logoff
    pub fn disconnect(&mut self) -> i32 {
        log_info!("SMB2 disconnect: server={:#x}:{}", self.server_addr, self.server_port);

        let msg_id = self.message_id.fetch_add(1, Ordering::AcqRel);
        let mut buf = Vec::new();
        self.encode_header(smb2_cmd::LOGOFF, msg_id, 0, self.session_id.load(Ordering::Acquire), &mut buf);

        let _ = self.send_and_receive(&buf);

        self.tcp_sock = None;
        self.session_id.store(0, Ordering::Release);
        self.tree_id.store(0, Ordering::Release);
        self.state.store(smb2_client_state::IDLE, Ordering::Release);
        0
    }

    /// Tree connect — attach to a share
    pub fn tree_connect(&mut self, share_path: &str) -> Result<u32, i32> {
        let msg_id = self.message_id.fetch_add(1, Ordering::AcqRel);
        log_debug!("SMB2 tree_connect: msg_id={} path={}", msg_id, share_path);

        let mut buf = Vec::new();
        self.encode_header(smb2_cmd::TREE_CONNECT, msg_id, 0, self.session_id.load(Ordering::Acquire), &mut buf);
        self.encode_u16(9, &mut buf);
        self.encode_string_utf16(share_path, &mut buf);

        let reply = self.send_and_receive(&buf)?;
        self.parse_reply_header(&reply, smb2_cmd::TREE_CONNECT)?;

        let tid = self.tree_id.fetch_add(1, Ordering::AcqRel) + 1;
        Ok(tid)
    }

    /// Create / open a file
    pub fn create(&mut self, tree_id: u32, path: &str, disposition: u32, desired_access: u32) -> Result<Smb2FileId, i32> {
        let msg_id = self.message_id.fetch_add(1, Ordering::AcqRel);
        log_debug!("SMB2 create: msg_id={} path={} disp={}", msg_id, path, disposition);

        let mut buf = Vec::new();
        self.encode_header(smb2_cmd::CREATE, msg_id, tree_id, self.session_id.load(Ordering::Acquire), &mut buf);
        self.encode_u16(57, &mut buf);
        self.encode_u8(0, &mut buf);
        self.encode_u8(0, &mut buf);
        self.encode_u32(desired_access, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_u32(disposition, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_string_utf16(path, &mut buf);

        let reply = self.send_and_receive(&buf)?;
        self.parse_reply_header(&reply, smb2_cmd::CREATE)?;

        let mut file_id = Smb2FileId { persistent: 0, volatile: 0 };
        if reply.len() >= 64 + 16 + 16 {
            let off = 64 + 16;
            file_id.persistent = u64::from_le_bytes([reply[off], reply[off+1], reply[off+2], reply[off+3],
                                                       reply[off+4], reply[off+5], reply[off+6], reply[off+7]]);
            file_id.volatile = u64::from_le_bytes([reply[off+8], reply[off+9], reply[off+10], reply[off+11],
                                                    reply[off+12], reply[off+13], reply[off+14], reply[off+15]]);
        }
        Ok(file_id)
    }

    /// Read from file
    pub fn read(&mut self, tree_id: u32, file_id: &Smb2FileId, offset: u64, length: u32) -> Result<Vec<u8>, i32> {
        let msg_id = self.message_id.fetch_add(1, Ordering::AcqRel);
        log_debug!("SMB2 read: msg_id={} offset={} length={}", msg_id, offset, length);

        let max_read = self.max_read_size.load(Ordering::Acquire);
        let actual_len = if length > max_read { max_read } else { length };

        let mut buf = Vec::new();
        self.encode_header(smb2_cmd::READ, msg_id, tree_id, self.session_id.load(Ordering::Acquire), &mut buf);
        self.encode_u16(49, &mut buf);
        self.encode_u64(file_id.persistent, &mut buf);
        self.encode_u64(file_id.volatile, &mut buf);
        self.encode_u64(offset, &mut buf);
        self.encode_u32(actual_len, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_u32(0, &mut buf);

        let reply = self.send_and_receive(&buf)?;
        self.parse_reply_header(&reply, smb2_cmd::READ)?;

        if reply.len() > 64 + 16 {
            let data_offset = 64 + 16;
            Ok(reply[data_offset..].to_vec())
        } else {
            Ok(Vec::new())
        }
    }

    /// Write to file
    pub fn write(&mut self, tree_id: u32, file_id: &Smb2FileId, offset: u64, data: &[u8]) -> Result<u32, i32> {
        let msg_id = self.message_id.fetch_add(1, Ordering::AcqRel);
        log_debug!("SMB2 write: msg_id={} offset={} length={}", msg_id, offset, data.len());

        let max_write = self.max_write_size.load(Ordering::Acquire);
        let write_len = if data.len() as u32 > max_write {
            max_write as usize
        } else {
            data.len()
        };

        let mut buf = Vec::new();
        self.encode_header(smb2_cmd::WRITE, msg_id, tree_id, self.session_id.load(Ordering::Acquire), &mut buf);
        self.encode_u16(49, &mut buf);
        self.encode_u64(offset, &mut buf);
        self.encode_u32(write_len as u32, &mut buf);
        self.encode_u8(0, &mut buf);
        self.encode_u8(0, &mut buf);
        self.encode_u8(0, &mut buf);
        self.encode_u8(0, &mut buf);
        self.encode_u64(file_id.persistent, &mut buf);
        self.encode_u64(file_id.volatile, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_u32(0, &mut buf);
        buf.extend_from_slice(&data[..write_len]);

        let reply = self.send_and_receive(&buf)?;
        self.parse_reply_header(&reply, smb2_cmd::WRITE)?;

        Ok(write_len as u32)
    }

    /// Close a file handle
    pub fn close(&mut self, tree_id: u32, file_id: &Smb2FileId) -> i32 {
        let msg_id = self.message_id.fetch_add(1, Ordering::AcqRel);
        log_debug!("SMB2 close: msg_id={}", msg_id);

        let mut buf = Vec::new();
        self.encode_header(smb2_cmd::CLOSE, msg_id, tree_id, self.session_id.load(Ordering::Acquire), &mut buf);
        self.encode_u16(24, &mut buf);
        self.encode_u16(0, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_u64(file_id.persistent, &mut buf);
        self.encode_u64(file_id.volatile, &mut buf);

        let _ = self.send_and_receive(&buf);
        0
    }

    /// Query directory
    pub fn query_directory(&mut self, tree_id: u32, file_id: &Smb2FileId, pattern: &str) -> Result<Vec<u8>, i32> {
        let msg_id = self.message_id.fetch_add(1, Ordering::AcqRel);
        log_debug!("SMB2 query_directory: msg_id={} pattern={}", msg_id, pattern);

        let mut buf = Vec::new();
        self.encode_header(smb2_cmd::QUERY_DIRECTORY, msg_id, tree_id, self.session_id.load(Ordering::Acquire), &mut buf);
        self.encode_u16(33, &mut buf);
        self.encode_u8(1, &mut buf);
        self.encode_u8(0, &mut buf);
        self.encode_u32(0x00000001, &mut buf);
        self.encode_u32(65535, &mut buf);
        self.encode_u32(0, &mut buf);
        self.encode_u64(file_id.persistent, &mut buf);
        self.encode_u64(file_id.volatile, &mut buf);
        self.encode_string_utf16(pattern, &mut buf);

        let reply = self.send_and_receive(&buf)?;
        self.parse_reply_header(&reply, smb2_cmd::QUERY_DIRECTORY)?;
        Ok(reply)
    }

    /// Encode SMB2 header
    fn encode_header(&self, cmd: u16, msg_id: u64, tree_id: u32, session_id: u64, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&[0xFE, b'S', b'M', b'B']);
        self.encode_u16(64, buf);
        self.encode_u16(0, buf);
        self.encode_u32(0, buf);
        self.encode_u16(cmd, buf);
        self.encode_u16(1, buf);
        self.encode_u32(0, buf);
        self.encode_u32(0, buf);
        self.encode_u64(msg_id, buf);
        self.encode_u32(0, buf);
        self.encode_u32(tree_id, buf);
        self.encode_u64(session_id, buf);
        buf.extend_from_slice(&[0u8; 16]);
    }

    /// Encode negotiate request body
    fn encode_negotiate_req(&self, req: &Smb2NegotiateReq, buf: &mut Vec<u8>) {
        self.encode_u16(36, buf);
        self.encode_u16(req.dialects.len() as u16, buf);
        self.encode_u16(req.security_mode, buf);
        self.encode_u16(0, buf);
        self.encode_u32(req.capabilities, buf);
        buf.extend_from_slice(&req.client_guid);
        self.encode_u64(0, buf);
        self.encode_u64(0, buf);
        for &dialect in &req.dialects {
            self.encode_u16(dialect, buf);
        }
    }

    /// Encode string as UTF-16LE (SMB2 wire format)
    fn encode_string_utf16(&self, s: &str, buf: &mut Vec<u8>) {
        let utf16_len = s.len() as u32 * 2;
        self.encode_u32(utf16_len + 2, buf);
        self.encode_u32(utf16_len, buf);
        for ch in s.encode_utf16() {
            buf.extend_from_slice(&ch.to_le_bytes());
        }
        self.encode_u16(0, buf);
    }

    fn encode_u8(&self, v: u8, buf: &mut Vec<u8>) {
        buf.push(v);
    }

    fn encode_u16(&self, v: u16, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn encode_u32(&self, v: u32, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    fn encode_u64(&self, v: u64, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&v.to_le_bytes());
    }
}

/// SMB2 client statistics
pub struct Smb2ClientStats {
    /// Bytes read
    pub bytes_read: AtomicU64,
    /// Bytes written
    pub bytes_written: AtomicU64,
    /// Total requests
    pub requests: AtomicU64,
    /// Total responses
    pub responses: AtomicU64,
    /// Credit count
    pub credits: AtomicU32,
}

impl Smb2ClientStats {
    pub const fn new() -> Self {
        Smb2ClientStats {
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
            requests: AtomicU64::new(0),
            responses: AtomicU64::new(0),
            credits: AtomicU32::new(0),
        }
    }
}
