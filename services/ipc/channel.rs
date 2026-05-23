/*
 * Nuva OS - System Service - IPC Channel
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


use core::sync::atomic::{AtomicU32, Ordering};

/// Channel type
#[derive(Debug, Clone, Copy)]
pub enum ChannelType {
    /// One-way (unidirectional)
    OneWay = 0,
    /// Two-way (bidirectional)
    TwoWay = 1,
}

/// Channel state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelState {
    /// Open
    Open = 0,
    /// Closed
    Closed = 1,
}

/// IPC channel
pub struct IpcChannel {
    /// Channel ID
    pub channel_id: u32,
    /// Channel type
    pub channel_type: ChannelType,
    /// State
    pub state: AtomicU32,
    /// Sender PID
    pub sender_pid: u32,
    /// Receiver PID
    pub receiver_pid: u32,
    /// Buffer size
    pub buffer_size: usize,
}

/// Channel service
pub struct ChannelService {
    /// Channel array
    channels: [Option<IpcChannel>; 32],
    /// Channel count
    num_channels: u32,
}

impl ChannelService {
    pub const fn new() -> Self {
        ChannelService {
            channels: [None; 32],
            num_channels: 0,
        }
    }

    /// Initialize the channel service
    pub fn init(&mut self) -> i32 {
        log_info!("IPC channel service initialized");
        0
    }

    /// Create a channel
    pub fn create(&mut self, channel_type: ChannelType, sender_pid: u32, receiver_pid: u32, buffer_size: usize) -> Option<u32> {
        for (i, slot) in self.channels.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(IpcChannel {
                    channel_id: i as u32,
                    channel_type,
                    state: AtomicU32::new(ChannelState::Open as u32),
                    sender_pid,
                    receiver_pid,
                    buffer_size,
                });
                self.num_channels += 1;

                log_debug!("Created IPC channel: id={}, type={:?}", i, channel_type);
                return Some(i as u32);
            }
        }
        None
    }

    /// Send a message
    pub fn send(&self, channel_id: u32, data: &[u8]) -> i32 {
        if let Some(ref channel) = self.channels.get(channel_id as usize)? {
            if channel.state.load(Ordering::Acquire) != ChannelState::Open as u32 {
                return -1;
            }

            // TODO: Write to channel buffer

            log_debug!("Channel send: {} bytes on channel {}", data.len(), channel_id);
            return data.len() as i32;
        }
        -1
    }

    /// Receive a message
    pub fn recv(&self, channel_id: u32, buf: &mut [u8]) -> i32 {
        if let Some(ref channel) = self.channels.get(channel_id as usize)? {
            if channel.state.load(Ordering::Acquire) != ChannelState::Open as u32 {
                return -1;
            }

            // TODO: Read from channel buffer

            return 0;
        }
        -1
    }

    /// Close a channel
    pub fn close(&mut self, channel_id: u32) -> i32 {
        if let Some(ref channel) = self.channels.get(channel_id as usize)? {
            channel.state.store(ChannelState::Closed as u32, Ordering::Release);
            return 0;
        }
        -1
    }

    /// Destroy a channel
    pub fn destroy(&mut self, channel_id: u32) -> i32 {
        if (channel_id as usize) < self.channels.len() {
            self.channels[channel_id as usize] = None;
            self.num_channels -= 1;
            return 0;
        }
        -1
    }
}

static mut CHANNEL_SERVICE: ChannelService = ChannelService::new();

/// Get the global channel service instance
pub fn get_channel_service() -> &'static mut ChannelService {
    // SAFETY: Single-threaded access; synchronized externally.
    unsafe { &mut CHANNEL_SERVICE }
}

/// Initialize the channel service
pub fn init_channel() {
    let service = get_channel_service();
    service.init();
}
