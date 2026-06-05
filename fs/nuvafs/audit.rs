/*
 * Nuva OS - NuvaFS Audit Log
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

//! NuvaFS Audit Log
//! Lock-free ring buffer audit logging for filesystem operations.
//! Uses a fixed-size 256-entry ring buffer with AtomicU32 indices
//! for concurrent write access without locks.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use alloc::vec::Vec;

/// Ring buffer capacity (must be power of 2)
pub const AUDIT_RING_SIZE: u32 = 256;
/// Mask for wrapping indices
const AUDIT_RING_MASK: u32 = AUDIT_RING_SIZE - 1;

/// Audit event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AuditEventType {
    /// File read
    Read = 0,
    /// File write
    Write = 1,
    /// File create
    Create = 2,
    /// File delete
    Delete = 3,
    /// Snapshot create
    SnapshotCreate = 4,
    /// Snapshot delete
    SnapshotDelete = 5,
    /// Snapshot rollback
    SnapshotRollback = 6,
    /// Capability grant
    CapabilityGrant = 7,
    /// Capability revoke
    CapabilityRevoke = 8,
    /// Mount operation
    Mount = 9,
    /// Unmount operation
    Unmount = 10,
    /// Checkpoint
    Checkpoint = 11,
    /// Permission denied
    PermissionDenied = 12,
    /// Custom event
    Custom = 255,
}

/// Audit event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum AuditSeverity {
    /// Informational
    Info = 0,
    /// Warning
    Warning = 1,
    /// Error
    Error = 2,
    /// Critical
    Critical = 3,
}

/// A single audit event entry in the ring buffer.
#[derive(Debug, Clone, Copy)]
pub struct AuditEntry {
    /// Monotonic sequence number
    pub sequence: u64,
    /// Event type
    pub event_type: AuditEventType,
    /// Severity level
    pub severity: AuditSeverity,
    /// Subject ID (who performed the operation)
    pub subject: u64,
    /// Object ID (what was operated on)
    pub object: u64,
    /// Additional data (e.g., block number, size)
    pub data: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Result code (0 = success, non-zero = error)
    pub result: u32,
}

impl AuditEntry {
    /// Create a new audit entry
    pub const fn new(
        sequence: u64,
        event_type: AuditEventType,
        severity: AuditSeverity,
        subject: u64,
        object: u64,
        data: u64,
        timestamp: u64,
        result: u32,
    ) -> Self {
        Self {
            sequence,
            event_type,
            severity,
            subject,
            object,
            data,
            timestamp,
            result,
        }
    }

    /// Check if the entry represents a successful operation
    pub fn is_success(&self) -> bool {
        self.result == 0
    }

    /// Check if the entry represents a failure
    pub fn is_failure(&self) -> bool {
        self.result != 0
    }
}

/// Lock-free ring buffer audit log.
///
/// Uses a fixed-size array of 256 entries with atomic head/tail
/// indices. Writers advance head atomically; readers scan from
/// tail. Overflow wraps and overwrites the oldest entries.
pub struct AuditLog {
    /// Ring buffer entries
    entries: [AuditEntry; AUDIT_RING_SIZE as usize],
    /// Write position (monotonically increasing, masked for index)
    head: AtomicU32,
    /// Total events written (for sequence numbering)
    total_written: AtomicU64,
    /// Whether audit logging is enabled
    enabled: AtomicBool,
    /// Filter: minimum severity to log
    min_severity: AtomicU32,
}

/// Sentinel entry for uninitialized slots
const AUDIT_ENTRY_ZERO: AuditEntry = AuditEntry::new(
    0, AuditEventType::Custom, AuditSeverity::Info, 0, 0, 0, 0, 0,
);

impl AuditLog {
    /// Create a new audit log with all entries zeroed
    pub const fn new() -> Self {
        Self {
            entries: [AUDIT_ENTRY_ZERO; AUDIT_RING_SIZE as usize],
            head: AtomicU32::new(0),
            total_written: AtomicU64::new(0),
            enabled: AtomicBool::new(true),
            min_severity: AtomicU32::new(AuditSeverity::Info as u32),
        }
    }

    /// Check if audit logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable audit logging
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    /// Disable audit logging
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    /// Set minimum severity filter
    pub fn set_min_severity(&self, severity: AuditSeverity) {
        self.min_severity.store(severity as u32, Ordering::Relaxed);
    }

    /// Get minimum severity filter
    pub fn min_severity(&self) -> AuditSeverity {
        match self.min_severity.load(Ordering::Relaxed) {
            0 => AuditSeverity::Info,
            1 => AuditSeverity::Warning,
            2 => AuditSeverity::Error,
            3 => AuditSeverity::Critical,
            _ => AuditSeverity::Info,
        }
    }

    /// Write an audit event to the ring buffer.
    /// Returns the sequence number assigned, or None if logging is disabled
    /// or the event is below the severity filter.
    pub fn write(
        &self,
        event_type: AuditEventType,
        severity: AuditSeverity,
        subject: u64,
        object: u64,
        data: u64,
        timestamp: u64,
        result: u32,
    ) -> Option<u64> {
        // Check if logging is enabled
        if !self.enabled.load(Ordering::Relaxed) {
            return None;
        }

        // Check severity filter
        let min_sev = self.min_severity.load(Ordering::Relaxed);
        if (severity as u32) < min_sev {
            return None;
        }

        // Allocate sequence number and slot
        let seq = self.total_written.fetch_add(1, Ordering::Relaxed);
        let slot = self.head.fetch_add(1, Ordering::Relaxed);
        let index = (slot & AUDIT_RING_MASK) as usize;

        // Write entry
        let entry = AuditEntry::new(seq, event_type, severity, subject, object, data, timestamp, result);
        // SAFETY: We use a raw pointer to write to a shared array slot.
        // This is safe because each slot is only written by one writer at a time
        // (the slot index is uniquely allocated via fetch_add), and readers
        // only read complete entries.
        unsafe {
            let ptr = self.entries.as_ptr().add(index) as *mut AuditEntry;
            ptr.write(entry);
        }

        Some(seq)
    }

    /// Read the most recent `count` entries from the ring buffer.
    /// Returns entries in chronological order (oldest first).
    pub fn read_recent(&self, count: u32) -> Vec<AuditEntry> {
        let total = self.total_written.load(Ordering::Relaxed);
        if total == 0 {
            return Vec::new();
        }

        let count = count.min(total as u32).min(AUDIT_RING_SIZE);
        let head = self.head.load(Ordering::Relaxed);

        let mut result = Vec::with_capacity(count as usize);
        for i in 0..count {
            let slot = head.wrapping_sub(count).wrapping_add(i);
            let index = (slot & AUDIT_RING_MASK) as usize;
            let entry = self.entries[index];
            if entry.sequence > 0 || total <= AUDIT_RING_SIZE as u64 {
                result.push(entry);
            }
        }
        result
    }

    /// Get the total number of events written
    pub fn total_written(&self) -> u64 {
        self.total_written.load(Ordering::Relaxed)
    }

    /// Get the current ring buffer utilization (0..=AUDIT_RING_SIZE)
    pub fn utilization(&self) -> u32 {
        let total = self.total_written.load(Ordering::Relaxed);
        (total as u32).min(AUDIT_RING_SIZE)
    }

    /// Search for entries matching a predicate
    pub fn find<F>(&self, predicate: F) -> Vec<AuditEntry>
    where
        F: Fn(&AuditEntry) -> bool,
    {
        let mut result = Vec::new();
        let head = self.head.load(Ordering::Relaxed);
        let total = self.total_written.load(Ordering::Relaxed);
        let scan_count = total.min(AUDIT_RING_SIZE as u64) as u32;

        for i in 0..scan_count {
            let slot = head.wrapping_sub(scan_count).wrapping_add(i);
            let index = (slot & AUDIT_RING_MASK) as usize;
            let entry = self.entries[index];
            if predicate(&entry) {
                result.push(entry);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry() {
        let entry = AuditEntry::new(1, AuditEventType::Write, AuditSeverity::Info, 100, 200, 0, 1000, 0);
        assert!(entry.is_success());
        assert!(!entry.is_failure());
        assert_eq!(entry.sequence, 1);
    }

    #[test]
    fn test_audit_log_write_read() {
        let log = AuditLog::new();
        let seq = log.write(AuditEventType::Write, AuditSeverity::Info, 100, 200, 0, 1000, 0);
        assert!(seq.is_some());
        assert_eq!(log.total_written(), 1);

        let entries = log.read_recent(1);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_audit_log_disabled() {
        let log = AuditLog::new();
        log.disable();
        let seq = log.write(AuditEventType::Write, AuditSeverity::Info, 100, 200, 0, 1000, 0);
        assert!(seq.is_none());
    }

    #[test]
    fn test_audit_log_severity_filter() {
        let log = AuditLog::new();
        log.set_min_severity(AuditSeverity::Error);

        // Info event should be filtered out
        let seq = log.write(AuditEventType::Write, AuditSeverity::Info, 100, 200, 0, 1000, 0);
        assert!(seq.is_none());

        // Error event should pass
        let seq = log.write(AuditEventType::PermissionDenied, AuditSeverity::Error, 100, 200, 0, 1000, 1);
        assert!(seq.is_some());
    }

    #[test]
    fn test_audit_log_ring_wrap() {
        let log = AuditLog::new();
        // Write more than ring size to test wrapping
        for i in 0..300u64 {
            log.write(AuditEventType::Write, AuditSeverity::Info, i, 0, 0, i, 0);
        }
        assert_eq!(log.total_written(), 300);
        // Utilization should be capped at ring size
        assert_eq!(log.utilization(), AUDIT_RING_SIZE);
    }

    #[test]
    fn test_audit_log_find() {
        let log = AuditLog::new();
        log.write(AuditEventType::Read, AuditSeverity::Info, 100, 200, 0, 1000, 0);
        log.write(AuditEventType::Write, AuditSeverity::Info, 100, 201, 0, 1001, 0);
        log.write(AuditEventType::Delete, AuditSeverity::Warning, 100, 202, 0, 1002, 0);

        let writes = log.find(|e| e.event_type == AuditEventType::Write);
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].object, 201);
    }
}
