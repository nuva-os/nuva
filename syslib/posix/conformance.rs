/*
 * Nuva OS - Syslib - POSIX Conformance Level and Verification
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use alloc::string::String;
use alloc::format;
use super::deviation::{DeviationSeverity, POSIX_DEVIATIONS};

/// POSIX conformance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConformanceLevel {
    /// No POSIX conformance claimed
    None = 0,
    /// Base Definitions conformance (partial)
    BasePartial = 1,
    /// Base Definitions full conformance
    BaseFull = 2,
    /// System Interfaces conformance (partial)
    SystemPartial = 3,
    /// System Interfaces full conformance
    SystemFull = 4,
    /// Shell and Utilities conformance
    ShellFull = 5,
}

/// Conformance record for a specific POSIX interface
#[derive(Debug, Clone)]
pub struct ConformanceRecord {
    /// Interface name
    pub interface: &'static str,
    /// Current conformance level
    pub level: ConformanceLevel,
    /// Whether the interface is implemented
    pub implemented: bool,
    /// Whether the interface passes POSIX test suite
    pub test_passed: bool,
    /// Number of known deviations
    pub deviation_count: usize,
    /// Notes on conformance status
    pub notes: &'static str,
}

/// Overall POSIX conformance assessment
#[derive(Debug, Clone)]
pub struct ConformanceAssessment {
    /// Claimed conformance level
    pub claimed_level: ConformanceLevel,
    /// Total POSIX interfaces assessed
    pub total_interfaces: usize,
    /// Number of fully conformant interfaces
    pub conformant_count: usize,
    /// Number of partially conformant interfaces
    pub partial_count: usize,
    /// Number of non-conformant interfaces
    pub non_conformant_count: usize,
    /// Number of critical deviations
    pub critical_deviations: usize,
    /// Number of major deviations
    pub major_deviations: usize,
    /// Number of minor deviations
    pub minor_deviations: usize,
}

impl ConformanceAssessment {
    /// Perform a conformance assessment based on the deviation registry
    pub fn assess() -> Self {
        let critical = POSIX_DEVIATIONS.iter()
            .filter(|d| d.severity == DeviationSeverity::Critical)
            .count();
        let major = POSIX_DEVIATIONS.iter()
            .filter(|d| d.severity == DeviationSeverity::Major)
            .count();
        let minor = POSIX_DEVIATIONS.iter()
            .filter(|d| d.severity == DeviationSeverity::Minor)
            .count();

        let total = POSIX_DEVIATIONS.len();
        let conformant = minor;
        let partial = 0;

        let claimed = if critical > 0 {
            ConformanceLevel::BasePartial
        } else if major > 0 {
            ConformanceLevel::SystemPartial
        } else {
            ConformanceLevel::SystemFull
        };

        ConformanceAssessment {
            claimed_level: claimed,
            total_interfaces: total,
            conformant_count: conformant,
            partial_count: partial,
            non_conformant_count: total - conformant - partial,
            critical_deviations: critical,
            major_deviations: major,
            minor_deviations: minor,
        }
    }

    /// Get a summary string of the conformance assessment
    pub fn summary(&self) -> String {
        format!(
            "POSIX Conformance: {:?} | Total: {} | Conformant: {} | Deviations: critical={} major={} minor={}",
            self.claimed_level,
            self.total_interfaces,
            self.conformant_count,
            self.critical_deviations,
            self.major_deviations,
            self.minor_deviations
        )
    }
}

/// Static conformance records for key POSIX interfaces
pub static CONFORMANCE_RECORDS: &[ConformanceRecord] = &[
    ConformanceRecord {
        interface: "fork",
        level: ConformanceLevel::BasePartial,
        implemented: false,
        test_passed: false,
        deviation_count: 1,
        notes: "Architectural limitation: microkernel uses spawn() instead",
    },
    ConformanceRecord {
        interface: "execve",
        level: ConformanceLevel::BasePartial,
        implemented: false,
        test_passed: false,
        deviation_count: 1,
        notes: "Architectural limitation: capability-based process loading",
    },
    ConformanceRecord {
        interface: "pipe",
        level: ConformanceLevel::SystemPartial,
        implemented: true,
        test_passed: true,
        deviation_count: 0,
        notes: "Anonymous pipes fully supported",
    },
    ConformanceRecord {
        interface: "open/close/read/write",
        level: ConformanceLevel::SystemPartial,
        implemented: true,
        test_passed: true,
        deviation_count: 0,
        notes: "Basic file I/O conformant",
    },
    ConformanceRecord {
        interface: "kill/sigaction",
        level: ConformanceLevel::SystemPartial,
        implemented: true,
        test_passed: false,
        deviation_count: 0,
        notes: "Signal infrastructure present, testing pending",
    },
    ConformanceRecord {
        interface: "mkfifo",
        level: ConformanceLevel::BasePartial,
        implemented: false,
        test_passed: false,
        deviation_count: 1,
        notes: "Named FIFOs not yet implemented",
    },
    ConformanceRecord {
        interface: "socket",
        level: ConformanceLevel::BasePartial,
        implemented: true,
        test_passed: false,
        deviation_count: 1,
        notes: "Partial: AF_UNIX/AF_INET only",
    },
    ConformanceRecord {
        interface: "pthread_*",
        level: ConformanceLevel::None,
        implemented: false,
        test_passed: false,
        deviation_count: 1,
        notes: "Architectural: use L4-style lightweight tasks instead",
    },
];
