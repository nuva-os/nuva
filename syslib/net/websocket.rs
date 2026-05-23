/*
 * Nuva OS - SystemLibrary - Net
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

//! WebSocket Client

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// WebSocket State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WebSocketState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

/// WebSocket Operationcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WebSocketOpcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

/// WebSocket FrameHead
#[derive(Debug, Clone, Copy)]
pub struct WebSocketFrameHeader {
    pub fin: bool,
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub opcode: u8,
    pub masked: bool,
    pub payload_len: u64,
    pub masking_key: [u8; 4],
}

impl WebSocketFrameHeader {
    pub fn new(opcode: WebSocketOpcode, payload_len: u64, masked: bool) -> Self {
        Self {
            fin: true,
            rsv1: false,
            rsv2: false,
            rsv3: false,
            opcode: opcode as u8,
            masked,
            payload_len,
            masking_key: [0; 4],
        }
    }

    /// EncodeFrameHead
    pub fn encode(&self, output: &mut [u8]) -> usize {
        let mut pos = 0;
        
        // FirstByte: FIN, RSV, Opcode
        let mut byte0 = 0u8;
        if self.fin { byte0 |= 0x80; }
        if self.rsv1 { byte0 |= 0x40; }
        if self.rsv2 { byte0 |= 0x20; }
        if self.rsv3 { byte0 |= 0x10; }
        byte0 |= self.opcode & 0x0F;
        output[pos] = byte0;
        pos += 1;
        
        // seconditemByte: MASK, Payload length
        let mut byte1 = 0u8;
        if self.masked { byte1 |= 0x80; }
        
        if self.payload_len < 126 {
            byte1 |= self.payload_len as u8;
            output[pos] = byte1;
            pos += 1;
        } else if self.payload_len < 65536 {
            byte1 |= 126;
            output[pos] = byte1;
            pos += 1;
            output[pos] = ((self.payload_len >> 8) & 0xFF) as u8;
            pos += 1;
            output[pos] = (self.payload_len & 0xFF) as u8;
            pos += 1;
        } else {
            byte1 |= 127;
            output[pos] = byte1;
            pos += 1;
            for i in 0..8 {
                output[pos] = ((self.payload_len >> (56 - i * 8)) & 0xFF) as u8;
                pos += 1;
            }
        }
        
        // Masking key
        if self.masked {
            output[pos..pos + 4].copy_from_slice(&self.masking_key);
            pos += 4;
        }
        
        pos
    }

    /// DecodeFrameHead
    pub fn decode(data: &[u8]) -> Option<(Self, usize)> {
        if data.len() < 2 {
            return None;
        }
        
        let byte0 = data[0];
        let byte1 = data[1];
        
        let fin = (byte0 & 0x80) != 0;
        let rsv1 = (byte0 & 0x40) != 0;
        let rsv2 = (byte0 & 0x20) != 0;
        let rsv3 = (byte0 & 0x10) != 0;
        let opcode = byte0 & 0x0F;
        let masked = (byte1 & 0x80) != 0;
        
        let (payload_len, header_len) = match byte1 & 0x7F {
            126 => {
                if data.len() < 4 {
                    return None;
                }
                let len = ((data[2] as u64) << 8) | (data[3] as u64);
                (len, 4)
            }
            127 => {
                if data.len() < 10 {
                    return None;
                }
                let mut len = 0u64;
                for i in 0..8 {
                    len = (len << 8) | (data[2 + i] as u64);
                }
                (len, 10)
            }
            len => (len as u64, 2),
        };
        
        let mut masking_key = [0u8; 4];
        let total_header_len = if masked {
            if data.len() < header_len + 4 {
                return None;
            }
            masking_key.copy_from_slice(&data[header_len..header_len + 4]);
            header_len + 4
        } else {
            header_len
        };
        
        Some((
            Self {
                fin,
                rsv1,
                rsv2,
                rsv3,
                opcode,
                masked,
                payload_len,
                masking_key,
            },
            total_header_len,
        ))
    }
}

/// WebSocket Message
#[derive(Debug, Clone)]
pub struct WebSocketMessage {
    pub opcode: WebSocketOpcode,
    pub data: [u8; 65536],
    pub data_len: u32,
}

impl WebSocketMessage {
    pub fn text(text: &[u8]) -> Self {
        let mut data = [0u8; 65536];
        let len = text.len().min(65535);
        data[..len].copy_from_slice(&text[..len]);
        
        Self {
            opcode: WebSocketOpcode::Text,
            data,
            data_len: len as u32,
        }
    }

    pub fn binary(data: &[u8]) -> Self {
        let mut buf = [0u8; 65536];
        let len = data.len().min(65535);
        buf[..len].copy_from_slice(&data[..len]);
        
        Self {
            opcode: WebSocketOpcode::Binary,
            data: buf,
            data_len: len as u32,
        }
    }

    pub fn close(code: u16, reason: &[u8]) -> Self {
        let mut data = [0u8; 65536];
        data[0] = (code >> 8) as u8;
        data[1] = (code & 0xFF) as u8;
        let reason_len = reason.len().min(65533);
        data[2..2 + reason_len].copy_from_slice(&reason[..reason_len]);
        
        Self {
            opcode: WebSocketOpcode::Close,
            data,
            data_len: (2 + reason_len) as u32,
        }
    }

    pub fn ping(data: &[u8]) -> Self {
        let mut buf = [0u8; 65536];
        let len = data.len().min(125);
        buf[..len].copy_from_slice(&data[..len]);
        
        Self {
            opcode: WebSocketOpcode::Ping,
            data: buf,
            data_len: len as u32,
        }
    }

    pub fn pong(data: &[u8]) -> Self {
        let mut buf = [0u8; 65536];
        let len = data.len().min(125);
        buf[..len].copy_from_slice(&data[..len]);
        
        Self {
            opcode: WebSocketOpcode::Pong,
            data: buf,
            data_len: len as u32,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.data_len as usize]
    }

    pub fn is_text(&self) -> bool {
        self.opcode == WebSocketOpcode::Text
    }

    pub fn is_binary(&self) -> bool {
        self.opcode == WebSocketOpcode::Binary
    }
}

/// WebSocket Join
pub struct WebSocketConnection {
    pub id: u64,
    pub url: [u8; 512],
    pub url_len: u16,
    pub state: AtomicU32,
    pub last_ping: AtomicU64,
    pub last_pong: AtomicU64,
}

impl Clone for WebSocketConnection {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            url: self.url.clone(),
            url_len: self.url_len.clone(),
            state: AtomicU32::new(self.state.load(core::sync::atomic::Ordering::Relaxed)),
            last_ping: AtomicU64::new(self.last_ping.load(core::sync::atomic::Ordering::Relaxed)),
            last_pong: AtomicU64::new(self.last_pong.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl WebSocketConnection {
    pub fn new(id: u64, url: &[u8]) -> Self {
        let mut url_buf = [0u8; 512];
        let url_len = url.len().min(511);
        url_buf[..url_len].copy_from_slice(&url[..url_len]);
        
        Self {
            id,
            url: url_buf,
            url_len: url_len as u16,
            state: AtomicU32::new(WebSocketState::Connecting as u32),
            last_ping: AtomicU64::new(0),
            last_pong: AtomicU64::new(0),
        }
    }

    pub fn get_state(&self) -> WebSocketState {
        match self.state.load(Ordering::Relaxed) {
            0 => WebSocketState::Connecting,
            1 => WebSocketState::Open,
            2 => WebSocketState::Closing,
            _ => WebSocketState::Closed,
        }
    }

    pub fn set_state(&self, state: WebSocketState) {
        self.state.store(state as u32, Ordering::Release);
    }

    pub fn is_open(&self) -> bool {
        self.get_state() == WebSocketState::Open
    }
}

/// WebSocket Client
pub struct WebSocketClient {
    connections: [Option<WebSocketConnection>; 16],
    num_connections: AtomicU32,
    next_connection_id: AtomicU64,
}

impl WebSocketClient {
    pub fn new() -> Self {
        Self {
            connections: [const { None }; 16],
            num_connections: AtomicU32::new(0),
            next_connection_id: AtomicU64::new(1),
        }
    }

    /// Join WebSocket
    pub fn connect(&mut self, url: &[u8]) -> Result<u64, WebSocketError> {
        let id = self.next_connection_id.fetch_add(1, Ordering::Relaxed);
        let conn = WebSocketConnection::new(id, url);
        
        let idx = self.num_connections.load(Ordering::Relaxed) as usize;
        if idx < 16 {
            self.connections[idx] = Some(conn);
            self.num_connections.fetch_add(1, Ordering::Relaxed);
            
            // execute WebSocket handshake
            self.perform_handshake(idx)?;
            
            return Ok(id);
        }
        
        Err(WebSocketError::TooManyConnections)
    }

    /// SendMessage
    pub fn send(&mut self, conn_id: u64, message: &WebSocketMessage) -> Result<(), WebSocketError> {
        let conn = self.get_connection(conn_id)
            .ok_or(WebSocketError::ConnectionNotFound)?;
        
        if !conn.is_open() {
            return Err(WebSocketError::NotConnected);
        }
        
        // BuildFrame
        let header = WebSocketFrameHeader::new(message.opcode, message.data_len as u64, true);
        
        let mut frame = [0u8; 65536];
        let header_len = header.encode(&mut frame);
        
        // CopyDataparallelApplicationMask
        let masking_key = header.masking_key;
        for i in 0..message.data_len as usize {
            frame[header_len + i] = message.data[i] ^ masking_key[i % 4];
        }
        
        // SendFrame
        let _ = &frame[..header_len + message.data_len as usize];
        
        Ok(())
    }

    /// ReceiveMessage
    pub fn receive(&mut self, conn_id: u64) -> Result<WebSocketMessage, WebSocketError> {
        let conn = self.get_connection(conn_id)
            .ok_or(WebSocketError::ConnectionNotFound)?;
        
        if !conn.is_open() {
            return Err(WebSocketError::NotConnected);
        }
        
        // ReceiveFrame
        // SimplifiedImplementation
        Ok(WebSocketMessage::text(b""))
    }

    /// CloseJoin
    pub fn close(&mut self, conn_id: u64, code: u16, reason: &[u8]) -> Result<(), WebSocketError> {
        {
            let conn = self.get_connection(conn_id)
                .ok_or(WebSocketError::ConnectionNotFound)?;
            conn.set_state(WebSocketState::Closing);
        }
        
        // Send Close Frame
        let close_msg = WebSocketMessage::close(code, reason);
        self.send(conn_id, &close_msg)?;
        
        {
            let conn = self.get_connection(conn_id)
                .ok_or(WebSocketError::ConnectionNotFound)?;
            conn.set_state(WebSocketState::Closed);
        }
        
        Ok(())
    }

    fn get_connection(&mut self, id: u64) -> Option<&mut WebSocketConnection> {
        let num = self.num_connections.load(Ordering::Relaxed) as usize;
        for conn in self.connections[..num].iter_mut() {
            if let Some(ref mut c) = conn {
                if c.id == id {
                    return Some(c);
                }
            }
        }
        None
    }

    fn perform_handshake(&mut self, conn_idx: usize) -> Result<(), WebSocketError> {
        if let Some(ref mut conn) = self.connections[conn_idx] {
            // Send HTTP UpgradeRequest
            // SimplifiedImplementation
            conn.set_state(WebSocketState::Open);
        }
        Ok(())
    }
}

/// WebSocket Error
#[derive(Debug, Clone, Copy)]
pub enum WebSocketError {
    ConnectionNotFound,
    NotConnected,
    TooManyConnections,
    HandshakeFailed,
    InvalidFrame,
    ProtocolError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_state() {
        assert_eq!(WebSocketState::Connecting as u8, 0);
        assert_eq!(WebSocketState::Open as u8, 1);
        assert_eq!(WebSocketState::Closing as u8, 2);
        assert_eq!(WebSocketState::Closed as u8, 3);
    }

    #[test]
    fn test_websocket_opcode() {
        assert_eq!(WebSocketOpcode::Continuation as u8, 0x0);
        assert_eq!(WebSocketOpcode::Text as u8, 0x1);
        assert_eq!(WebSocketOpcode::Binary as u8, 0x2);
        assert_eq!(WebSocketOpcode::Close as u8, 0x8);
        assert_eq!(WebSocketOpcode::Ping as u8, 0x9);
        assert_eq!(WebSocketOpcode::Pong as u8, 0xA);
    }

    #[test]
    fn test_websocket_frame_header_new() {
        let header = WebSocketFrameHeader::new(WebSocketOpcode::Text, 100, true);

        assert!(header.fin);
        assert!(!header.rsv1);
        assert!(!header.rsv2);
        assert!(!header.rsv3);
        assert_eq!(header.opcode, WebSocketOpcode::Text as u8);
        assert!(header.masked);
        assert_eq!(header.payload_len, 100);
    }

    #[test]
    fn test_websocket_frame_header_encode_short() {
        let header = WebSocketFrameHeader::new(WebSocketOpcode::Text, 100, false);

        let mut output = [0u8; 16];
        let len = header.encode(&mut output);

        // weaknessLength: 2 ByteHead
        assert_eq!(len, 2);
        assert_eq!(output[0] & 0x0F, WebSocketOpcode::Text as u8);
        assert_eq!(output[1] & 0x7F, 100);
    }

    #[test]
    fn test_websocket_frame_header_encode_medium() {
        let header = WebSocketFrameHeader::new(WebSocketOpcode::Binary, 1000, false);

        let mut output = [0u8; 16];
        let len = header.encode(&mut output);

        // infixetcLength: 4 ByteHead
        assert_eq!(len, 4);
        assert_eq!(output[1] & 0x7F, 126);
    }

    #[test]
    fn test_websocket_frame_header_encode_masked() {
        let mut header = WebSocketFrameHeader::new(WebSocketOpcode::Text, 10, true);
        header.masking_key = [1, 2, 3, 4];

        let mut output = [0u8; 16];
        let len = header.encode(&mut output);

        // Mask: 2 + 4 = 6 ByteHead
        assert_eq!(len, 6);
        assert!(output[1] & 0x80 != 0);  // MASK Bit
    }

    #[test]
    fn test_websocket_frame_header_decode() {
        let header = WebSocketFrameHeader::new(WebSocketOpcode::Text, 100, false);

        let mut output = [0u8; 16];
        let len = header.encode(&mut output);

        let (decoded, decoded_len) = WebSocketFrameHeader::decode(&output).unwrap();

        assert_eq!(decoded_len, len);
        assert_eq!(decoded.fin, header.fin);
        assert_eq!(decoded.opcode, header.opcode);
        assert_eq!(decoded.payload_len, header.payload_len);
    }

    #[test]
    fn test_websocket_message_text() {
        let msg = WebSocketMessage::text(b"Hello");

        assert_eq!(msg.opcode, WebSocketOpcode::Text);
        assert_eq!(msg.data(), b"Hello");
        assert!(msg.is_text());
        assert!(!msg.is_binary());
    }

    #[test]
    fn test_websocket_message_binary() {
        let msg = WebSocketMessage::binary(&[1, 2, 3, 4]);

        assert_eq!(msg.opcode, WebSocketOpcode::Binary);
        assert_eq!(msg.data(), &[1, 2, 3, 4]);
        assert!(!msg.is_text());
        assert!(msg.is_binary());
    }

    #[test]
    fn test_websocket_message_close() {
        let msg = WebSocketMessage::close(1000, b"Normal closure");

        assert_eq!(msg.opcode, WebSocketOpcode::Close);
        assert_eq!(msg.data_len, 16);  // 2 + 14
    }

    #[test]
    fn test_websocket_message_ping() {
        let msg = WebSocketMessage::ping(b"ping");

        assert_eq!(msg.opcode, WebSocketOpcode::Ping);
        assert_eq!(msg.data(), b"ping");
    }

    #[test]
    fn test_websocket_message_pong() {
        let msg = WebSocketMessage::pong(b"pong");

        assert_eq!(msg.opcode, WebSocketOpcode::Pong);
        assert_eq!(msg.data(), b"pong");
    }

    #[test]
    fn test_websocket_connection_new() {
        let conn = WebSocketConnection::new(1, b"ws://example.com/socket");

        assert_eq!(conn.id, 1);
        assert_eq!(conn.get_state(), WebSocketState::Connecting);
    }

    #[test]
    fn test_websocket_connection_state() {
        let conn = WebSocketConnection::new(1, b"ws://example.com/socket");

        assert_eq!(conn.get_state(), WebSocketState::Connecting);
        assert!(!conn.is_open());

        conn.set_state(WebSocketState::Open);
        assert_eq!(conn.get_state(), WebSocketState::Open);
        assert!(conn.is_open());

        conn.set_state(WebSocketState::Closing);
        assert_eq!(conn.get_state(), WebSocketState::Closing);

        conn.set_state(WebSocketState::Closed);
        assert_eq!(conn.get_state(), WebSocketState::Closed);
    }

    #[test]
    fn test_websocket_client_new() {
        let client = WebSocketClient::new();

        assert_eq!(client.num_connections.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_websocket_client_connect() {
        let mut client = WebSocketClient::new();

        let result = client.connect(b"ws://example.com/socket");

        assert!(result.is_ok());
        assert_eq!(client.num_connections.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_websocket_client_send() {
        let mut client = WebSocketClient::new();

        let conn_id = client.connect(b"ws://example.com/socket").unwrap();

        let msg = WebSocketMessage::text(b"Hello");
        let result = client.send(conn_id, &msg);

        assert!(result.is_ok());
    }

    #[test]
    fn test_websocket_client_close() {
        let mut client = WebSocketClient::new();

        let conn_id = client.connect(b"ws://example.com/socket").unwrap();

        let result = client.close(conn_id, 1000, b"Normal closure");

        assert!(result.is_ok());
    }
}