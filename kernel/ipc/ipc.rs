/*
 * Nuva OS - IPC Rust FFI Bindings
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

/// Zero-Copy Inter-Process Communication
/// This module provides safe Rust bindings to the IPC C implementation,
/// offering efficient message passing between processes.

use core::ptr;

/// IPC error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    /// Invalid parameter
    InvalidParameter,
    /// Channel is busy
    Busy,
    /// Channel is empty
    Empty,
    /// Channel is full
    Full,
    /// Operation timed out
    Timeout,
    /// Permission denied
    PermissionDenied,
    /// Unknown error
    Unknown,
    RecvFailed,
}

impl IpcError {
    fn from_code(code: i32) -> Self {
        match code {
            -1 => IpcError::InvalidParameter,
            -2 => IpcError::Busy,
            -3 => IpcError::Empty,
            -4 => IpcError::Full,
            -5 => IpcError::Timeout,
            -6 => IpcError::PermissionDenied,
            _ => IpcError::Unknown,
        }
    }
}

/// IPC channel state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcState {
    /// Channel is empty
    Empty,
    /// Message is being sent
    Sending,
    /// Message is ready to receive
    Ready,
    /// Message is being received
    Receiving,
}

/// IPC channel
pub struct IpcChannel {
    /// Channel ID
    id: u64,
    /// Sender process ID
    sender: u64,
    /// Receiver process ID
    receiver: u64,
    /// Buffer size
    buffer_size: usize,
}

impl IpcChannel {
    /// Create new IPC channel
    pub fn new(sender: u64, receiver: u64, buffer_size: usize) -> Result<Self, IpcError> {
        // TODO: Allocate channel from kernel
        Ok(IpcChannel {
            id: 0,
            sender,
            receiver,
            buffer_size,
        })
    }

    /// Send message (zero-copy)
    pub fn send(&self, message: &[u8]) -> Result<(), IpcError> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe {
            ipc_send(
                self.id,
                message.as_ptr() as *mut u8,
                message.len(),
                0, // Non-blocking
            )
        };

        if result == 0 {
            Ok(())
        } else {
            Err(IpcError::from_code(result))
        }
    }

    /// Receive message (zero-copy)
    pub fn receive(&self) -> Result<IpcMessage, IpcError> {
        let mut message_ptr: *mut u8 = ptr::null_mut();
        let mut size: usize = 0;

        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe {
            ipc_receive(
                self.id,
                &mut message_ptr,
                &mut size,
                0, // Non-blocking
            )
        };

        if result == 0 {
            Ok(IpcMessage {
                channel: self.id,
                data: message_ptr,
                size,
            })
        } else {
            Err(IpcError::from_code(result))
        }
    }

    /// Get channel state
    pub fn state(&self) -> IpcState {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let state = unsafe { ipc_get_state(self.id) };
        match state {
            0 => IpcState::Empty,
            1 => IpcState::Sending,
            2 => IpcState::Ready,
            3 => IpcState::Receiving,
            _ => IpcState::Empty,
        }
    }

    /// Check if channel is empty
    pub fn is_empty(&self) -> bool {
        self.state() == IpcState::Empty
    }

    /// Check if channel is ready
    pub fn is_ready(&self) -> bool {
        self.state() == IpcState::Ready
    }

    /// Get sender process ID
    pub fn sender(&self) -> u64 {
        self.sender
    }

    /// Get receiver process ID
    pub fn receiver(&self) -> u64 {
        self.receiver
    }

    /// Get buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

/// IPC message (zero-copy view)
pub struct IpcMessage {
    channel: u64,
    data: *mut u8,
    size: usize,
}

impl IpcMessage {
    /// Get message data as bytes
    pub fn as_bytes(&self) -> &[u8] {
        if self.data.is_null() || self.size == 0 {
            &[]
        } else {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { core::slice::from_raw_parts(self.data, self.size) }
        }
    }

    /// Get message size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Release message (mark channel as empty)
    pub fn release(self) -> Result<(), IpcError> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let result = unsafe { ipc_release(self.channel) };
        if result == 0 {
            Ok(())
        } else {
            Err(IpcError::from_code(result))
        }
    }
}

/// IPC statistics
#[derive(Debug, Clone, Copy)]
pub struct IpcStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub send_errors: u64,
    pub receive_errors: u64,
}

// FFI declarations
extern "C" {
    fn ipc_send(channel: u64, message: *mut u8, size: usize, timeout_ms: u32) -> i32;
    fn ipc_receive(channel: u64, message: *mut *mut u8, size: *mut usize, timeout_ms: u32) -> i32;
    fn ipc_release(channel: u64) -> i32;
    fn ipc_get_state(channel: u64) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_channel_creation() {
        let result = IpcChannel::new(1, 2, 1024);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ipc_send_receive() {
        let channel = IpcChannel::new(1, 2, 1024).unwrap();
        
        // Send message
        let message = b"Hello, IPC!";
        let result = channel.send(message);
        assert!(result.is_ok());
        
        // Receive message
        let result = channel.receive();
        assert!(result.is_ok());
        
        let received = result.unwrap();
        assert_eq!(received.as_bytes(), message);
        
        // Release message
        received.release().unwrap();
    }
}
