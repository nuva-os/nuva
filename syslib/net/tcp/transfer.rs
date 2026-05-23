/*
 * Nuva OS - TCP Data Transfer
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

//! TCP Data Transfer
/*!*/
//! Implements send, receive, sequencing, and flow control.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// TCP segment
#[derive(Debug, Clone)]
pub struct TcpSegment {
    /// Sequence number
    pub seq: u32,

    /// Acknowledgment number
    pub ack: u32,

    /// Flags
    pub flags: u16,

    /// Window size
    pub window: u16,

    /// Data
    pub data: &'static [u8],
}

/// Send buffer
pub struct SendBuffer {
    /// Buffer data
    data: [u8; 65536],

    /// Write position
    write_pos: AtomicU32,

    /// Read position
    read_pos: AtomicU32,

    /// First unacknowledged sequence number
    snd_una: AtomicU32,

    /// Next sequence number to send
    snd_nxt: AtomicU32,

    /// Send window
    snd_wnd: AtomicU32,

    /// Window scale
    wscale: u8,
}

impl SendBuffer {
    pub const fn new() -> Self {
        Self {
            data: [0; 65536],
            write_pos: AtomicU32::new(0),
            read_pos: AtomicU32::new(0),
            snd_una: AtomicU32::new(0),
            snd_nxt: AtomicU32::new(0),
            snd_wnd: AtomicU32::new(65535),
            wscale: 0,
        }
    }

    /// Initialize sequence numbers
    pub fn init(&self, iss: u32) {
        self.snd_una.store(iss, Ordering::Relaxed);
        self.snd_nxt.store(iss, Ordering::Relaxed);
    }

    /// Get available space
    pub fn available(&self) -> u32 {
        let write = self.write_pos.load(Ordering::Acquire);
        let read = self.read_pos.load(Ordering::Acquire);
        65536 - (write.wrapping_sub(read) as usize % 65536) as u32
    }

    /// Get bytes in buffer
    pub fn len(&self) -> u32 {
        let write = self.write_pos.load(Ordering::Acquire);
        let read = self.read_pos.load(Ordering::Acquire);
        (write.wrapping_sub(read) as usize % 65536) as u32
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Write data to buffer
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = self.available() as usize;
        let to_write = data.len().min(available);

        if to_write == 0 {
            return 0;
        }

        let write_pos = self.write_pos.load(Ordering::Acquire) as usize;

        // Handle wrap-around
        let first_chunk = to_write.min(65536 - (write_pos % 65536));
        self.data[write_pos % 65536..write_pos % 65536 + first_chunk]
            .copy_from_slice(&data[..first_chunk]);

        if first_chunk < to_write {
            self.data[..to_write - first_chunk]
                .copy_from_slice(&data[first_chunk..to_write]);
        }

        self.write_pos.fetch_add(to_write as u32, Ordering::Release);
        to_write
    }

    /// Get next segment to send
    pub fn get_segment(&self, mss: u32) -> Option<(u32, u32)> {
        let len = self.len();
        if len == 0 {
            return None;
        }

        // Check flow control
        let snd_una = self.snd_una.load(Ordering::Acquire);
        let snd_nxt = self.snd_nxt.load(Ordering::Acquire);
        let snd_wnd = self.snd_wnd.load(Ordering::Acquire);

        // Flight size
        let flight = snd_nxt.wrapping_sub(snd_una);

        // Can send more?
        if flight >= snd_wnd {
            return None;
        }

        let can_send = (snd_wnd - flight).min(len).min(mss);
        if can_send == 0 {
            return None;
        }

        let seq = snd_nxt;
        let read_pos = self.read_pos.load(Ordering::Acquire);

        Some((seq, can_send))
    }

    /// Advance send pointer
    pub fn advance(&self, bytes: u32) {
        self.snd_nxt.fetch_add(bytes, Ordering::Release);
        self.read_pos.fetch_add(bytes, Ordering::Release);
    }

    /// Process ACK
    pub fn process_ack(&self, ack: u32) -> u32 {
        let snd_una = self.snd_una.load(Ordering::Acquire);
        let snd_nxt = self.snd_nxt.load(Ordering::Acquire);

        // Check if ACK is valid
        if ack < snd_una || ack > snd_nxt {
            return 0; // Old or invalid ACK
        }

        // Calculate acknowledged bytes
        let acked = ack.wrapping_sub(snd_una);

        // Update SND.UNA
        self.snd_una.store(ack, Ordering::Release);

        acked
    }

    /// Update send window
    pub fn update_window(&self, window: u16) {
        let scaled_window = (window as u32) << self.wscale;
        self.snd_wnd.store(scaled_window, Ordering::Release);
    }

    /// Get SND.NXT
    pub fn get_snd_nxt(&self) -> u32 {
        self.snd_nxt.load(Ordering::Acquire)
    }

    /// Get SND.UNA
    pub fn get_snd_una(&self) -> u32 {
        self.snd_una.load(Ordering::Acquire)
    }
}

/// Receive buffer
pub struct ReceiveBuffer {
    /// Buffer data
    data: [u8; 65536],

    /// Write position
    write_pos: AtomicU32,

    /// Read position
    read_pos: AtomicU32,

    /// Next expected sequence number
    rcv_nxt: AtomicU32,

    /// Receive window
    rcv_wnd: AtomicU32,

    /// Window scale
    wscale: u8,

    /// Out-of-sequence segments
    oos_queue: [Option<OutOfSeqSegment>; 16],
}

/// Out-of-sequence segment
#[derive(Debug, Clone, Copy)]
struct OutOfSeqSegment {
    seq: u32,
    len: u32,
}

impl ReceiveBuffer {
    pub const fn new() -> Self {
        Self {
            data: [0; 65536],
            write_pos: AtomicU32::new(0),
            read_pos: AtomicU32::new(0),
            rcv_nxt: AtomicU32::new(0),
            rcv_wnd: AtomicU32::new(65535),
            wscale: 0,
            oos_queue: [None; 16],
        }
    }

    /// Initialize sequence number
    pub fn init(&self, irs: u32) {
        self.rcv_nxt.store(irs, Ordering::Relaxed);
    }

    /// Get available space
    pub fn available(&self) -> u32 {
        let write = self.write_pos.load(Ordering::Acquire);
        let read = self.read_pos.load(Ordering::Acquire);
        65536 - (write.wrapping_sub(read) as usize % 65536) as u32
    }

    /// Get bytes in buffer
    pub fn len(&self) -> u32 {
        let write = self.write_pos.load(Ordering::Acquire);
        let read = self.read_pos.load(Ordering::Acquire);
        (write.wrapping_sub(read) as usize % 65536) as u32
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Receive segment
    pub fn receive(&mut self, seq: u32, data: &[u8]) -> ReceiveResult {
        let rcv_nxt = self.rcv_nxt.load(Ordering::Acquire);
        let len = data.len() as u32;

        // Check if segment is in sequence
        if seq == rcv_nxt {
            // In-sequence: accept immediately
            self.write_data(data);
            self.rcv_nxt.fetch_add(len, Ordering::Release);

            // Check for queued out-of-sequence segments
            let mut total_accepted = len;
            loop {
                let rcv_nxt = self.rcv_nxt.load(Ordering::Acquire);
                let mut found = false;

                for i in 0..16 {
                    if let Some(oos) = self.oos_queue[i] {
                        if oos.seq == rcv_nxt {
                            // Accept queued segment
                            self.rcv_nxt.fetch_add(oos.len, Ordering::Release);
                            total_accepted += oos.len;
                            self.oos_queue[i] = None;
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    break;
                }
            }

            ReceiveResult::Accepted {
                ack: self.rcv_nxt.load(Ordering::Acquire),
                bytes: total_accepted,
            }
        } else if seq > rcv_nxt {
            // Out-of-sequence: queue if within window
            let rcv_wnd = self.rcv_wnd.load(Ordering::Acquire);
            if seq < rcv_nxt + rcv_wnd {
                // Find free slot in OOS queue
                for i in 0..16 {
                    if self.oos_queue[i].is_none() {
                        self.oos_queue[i] = Some(OutOfSeqSegment { seq, len });
                        return ReceiveResult::OutOfSequence;
                    }
                }
            }
            ReceiveResult::Dropped
        } else {
            // Old segment: already acknowledged
            ReceiveResult::Duplicate
        }
    }

    /// Write data to buffer
    fn write_data(&mut self, data: &[u8]) {
        let write_pos = self.write_pos.load(Ordering::Acquire) as usize;
        let len = data.len();

        // Handle wrap-around
        let first_chunk = len.min(65536 - (write_pos % 65536));
        self.data[write_pos % 65536..write_pos % 65536 + first_chunk]
            .copy_from_slice(&data[..first_chunk]);

        if first_chunk < len {
            self.data[..len - first_chunk]
                .copy_from_slice(&data[first_chunk..len]);
        }

        self.write_pos.fetch_add(len as u32, Ordering::Release);
    }

    /// Read data from buffer
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let len = self.len() as usize;
        if len == 0 {
            return 0;
        }

        let to_read = buf.len().min(len);
        let read_pos = self.read_pos.load(Ordering::Acquire) as usize;

        // Handle wrap-around
        let first_chunk = to_read.min(65536 - (read_pos % 65536));
        buf[..first_chunk]
            .copy_from_slice(&self.data[read_pos % 65536..read_pos % 65536 + first_chunk]);

        if first_chunk < to_read {
            buf[first_chunk..to_read]
                .copy_from_slice(&self.data[..to_read - first_chunk]);
        }

        self.read_pos.fetch_add(to_read as u32, Ordering::Release);
        to_read
    }

    /// Get RCV.NXT
    pub fn get_rcv_nxt(&self) -> u32 {
        self.rcv_nxt.load(Ordering::Acquire)
    }

    /// Get receive window
    pub fn get_window(&self) -> u16 {
        let available = self.available();
        let scaled = available >> self.wscale;
        scaled.min(65535) as u16
    }
}

/// Receive result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveResult {
    /// Segment accepted
    Accepted { ack: u32, bytes: u32 },

    /// Out-of-sequence, queued
    OutOfSequence,

    /// Duplicate, dropped
    Duplicate,

    /// Dropped (buffer full or other reason)
    Dropped,
}

/// TCP retransmission queue
pub struct RetransmitQueue {
    segments: [Option<RtxSegment>; 32],
    count: AtomicU32,
}

/// Retransmission segment
#[derive(Debug, Clone, Copy)]
struct RtxSegment {
    seq: u32,
    len: u32,
    transmit_time: u64,
    retransmit_count: u8,
}

impl RetransmitQueue {
    pub const fn new() -> Self {
        Self {
            segments: [None; 32],
            count: AtomicU32::new(0),
        }
    }

    /// Add segment to queue
    pub fn add(&mut self, seq: u32, len: u32, transmit_time: u64) -> bool {
        if self.count.load(Ordering::Relaxed) >= 32 {
            return false;
        }

        for i in 0..32 {
            if self.segments[i].is_none() {
                self.segments[i] = Some(RtxSegment {
                    seq,
                    len,
                    transmit_time,
                    retransmit_count: 0,
                });
                self.count.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    /// Remove acknowledged segments
    pub fn remove_acked(&mut self, ack: u32) -> u32 {
        let mut removed = 0;

        for i in 0..32 {
            if let Some(seg) = self.segments[i] {
                if seg.seq + seg.len <= ack {
                    self.segments[i] = None;
                    self.count.fetch_sub(1, Ordering::Relaxed);
                    removed += seg.len;
                }
            }
        }

        removed
    }

    /// Get segments to retransmit
    pub fn get_timeout_segments(&self, current_time: u64, rto: u64) -> Vec<(u32, u32)> {
        let mut result = Vec::new();

        for i in 0..32 {
            if let Some(seg) = self.segments[i] {
                if current_time >= seg.transmit_time + rto {
                    result.push((seg.seq, seg.len));
                }
            }
        }

        result
    }

    /// Mark segment as retransmitted
    pub fn mark_retransmitted(&mut self, seq: u32, current_time: u64) {
        for i in 0..32 {
            if let Some(ref mut seg) = self.segments[i] {
                if seg.seq == seq {
                    seg.transmit_time = current_time;
                    seg.retransmit_count += 1;
                    break;
                }
            }
        }
    }

    /// Check if segment has exceeded max retransmits
    pub fn exceeded_max_retransmits(&self, seq: u32, max: u8) -> bool {
        for i in 0..32 {
            if let Some(seg) = self.segments[i] {
                if seg.seq == seq {
                    return seg.retransmit_count >= max;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_buffer_new() {
        let buf = SendBuffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.available(), 65536);
    }

    #[test]
    fn test_send_buffer_write() {
        let mut buf = SendBuffer::new();
        buf.init(1000);

        let data = [1u8; 100];
        let written = buf.write(&data);
        assert_eq!(written, 100);
        assert_eq!(buf.len(), 100);
    }

    #[test]
    fn test_send_buffer_process_ack() {
        let buf = SendBuffer::new();
        buf.init(1000);

        buf.snd_nxt.store(1100, Ordering::Release);

        let acked = buf.process_ack(1050);
        assert_eq!(acked, 50);
        assert_eq!(buf.get_snd_una(), 1050);
    }

    #[test]
    fn test_send_buffer_flow_control() {
        let buf = SendBuffer::new();
        buf.init(1000);

        // Set small window
        buf.snd_wnd.store(100, Ordering::Release);
        buf.snd_nxt.store(1000, Ordering::Release);
        buf.snd_una.store(950, Ordering::Release);

        // Flight size is 50, window is 100, can send 50 more
        let seg = buf.get_segment(1460);
        assert!(seg.is_some());
    }

    #[test]
    fn test_receive_buffer_new() {
        let buf = ReceiveBuffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.available(), 65536);
    }

    #[test]
    fn test_receive_buffer_in_sequence() {
        let mut buf = ReceiveBuffer::new();
        buf.init(1000);

        let data = [1u8; 100];
        let result = buf.receive(1000, &data);

        match result {
            ReceiveResult::Accepted { ack, bytes } => {
                assert_eq!(ack, 1100);
                assert_eq!(bytes, 100);
            }
            _ => panic!("Expected Accepted"),
        }

        assert_eq!(buf.get_rcv_nxt(), 1100);
    }

    #[test]
    fn test_receive_buffer_out_of_sequence() {
        let mut buf = ReceiveBuffer::new();
        buf.init(1000);

        // Receive out-of-sequence segment
        let data = [1u8; 100];
        let result = buf.receive(1100, &data);
        assert_eq!(result, ReceiveResult::OutOfSequence);

        // RCV.NXT should not change
        assert_eq!(buf.get_rcv_nxt(), 1000);

        // Now receive in-sequence segment
        let result = buf.receive(1000, &data);
        match result {
            ReceiveResult::Accepted { ack, bytes } => {
                // Should accept both segments
                assert_eq!(ack, 1200);
                assert_eq!(bytes, 200);
            }
            _ => panic!("Expected Accepted"),
        }
    }

    #[test]
    fn test_receive_buffer_duplicate() {
        let mut buf = ReceiveBuffer::new();
        buf.init(1000);

        let data = [1u8; 100];
        buf.receive(1000, &data);

        // Receive duplicate
        let result = buf.receive(1000, &data);
        assert_eq!(result, ReceiveResult::Duplicate);
    }

    #[test]
    fn test_receive_buffer_read() {
        let mut buf = ReceiveBuffer::new();
        buf.init(1000);

        let data = [1u8; 100];
        buf.receive(1000, &data);

        let mut out = [0u8; 100];
        let read = buf.read(&mut out);
        assert_eq!(read, 100);
        assert_eq!(out, data);
    }

    #[test]
    fn test_retransmit_queue() {
        let mut queue = RetransmitQueue::new();

        // Add segment
        assert!(queue.add(1000, 100, 0));

        // Remove acknowledged
        let removed = queue.remove_acked(1100);
        assert_eq!(removed, 100);
    }

    #[test]
    fn test_retransmit_queue_timeout() {
        let mut queue = RetransmitQueue::new();

        // Add segment at time 0
        queue.add(1000, 100, 0);

        // Check for timeout at time 1000 (RTO = 500)
        let timeout = queue.get_timeout_segments(1000, 500);
        assert_eq!(timeout.len(), 1);
        assert_eq!(timeout[0], (1000, 100));
    }
}
