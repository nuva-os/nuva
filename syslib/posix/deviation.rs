/*
 * Nuva OS - Syslib - POSIX Deviation Registry
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use alloc::vec::Vec;

/// Severity of a POSIX deviation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviationSeverity {
    /// No functional impact; behavior differs but outcome is equivalent
    Minor,
    /// Functional difference but workaround exists
    Major,
    /// Significant incompatibility; no workaround
    Critical,
}

/// Category of a POSIX deviation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviationCategory {
    /// Feature not implemented (returns ENOSYS)
    Unimplemented,
    /// Feature partially implemented
    Partial,
    /// Behavior differs from POSIX specification
    Behavioral,
    /// Microkernel architecture prevents exact POSIX semantics
    Architectural,
    /// Extended beyond POSIX specification
    Extension,
}

/// A single POSIX deviation entry
#[derive(Debug, Clone)]
pub struct PosixDeviation {
    /// The POSIX interface name (e.g., "fork", "execve")
    pub interface: &'static str,
    /// The POSIX section reference (e.g., "2.2.2")
    pub section: &'static str,
    /// Deviation severity
    pub severity: DeviationSeverity,
    /// Deviation category
    pub category: DeviationCategory,
    /// Human-readable description of the deviation
    pub description: &'static str,
    /// POSIX errno returned when unimplemented
    pub errno_on_call: i32,
    /// Whether the deviation is planned to be resolved
    pub planned_fix: bool,
}

/// Global POSIX deviations registry
pub static POSIX_DEVIATIONS: &[PosixDeviation] = &[
    PosixDeviation {
        interface: "fork",
        section: "2.2.2",
        severity: DeviationSeverity::Major,
        category: DeviationCategory::Architectural,
        description: "Microkernel does not support full address space duplication. \
            fork() returns ENOSYS. Use spawn() for process creation.",
        errno_on_call: 38,
        planned_fix: false,
    },
    PosixDeviation {
        interface: "execve",
        section: "2.2.3",
        severity: DeviationSeverity::Major,
        category: DeviationCategory::Architectural,
        description: "Microkernel uses capability-based process loading instead of \
            execve(). execve() returns ENOSYS. Use process_spawn_with_capabilities().",
        errno_on_call: 38,
        planned_fix: false,
    },
    PosixDeviation {
        interface: "mkfifo",
        section: "2.3.4",
        severity: DeviationSeverity::Minor,
        category: DeviationCategory::Unimplemented,
        description: "Named pipes (FIFOs) not yet implemented. \
            mkfifo() returns ENOSYS. Anonymous pipes are supported via pipe().",
        errno_on_call: 38,
        planned_fix: true,
    },
    PosixDeviation {
        interface: "socket",
        section: "2.10.2",
        severity: DeviationSeverity::Major,
        category: DeviationCategory::Partial,
        description: "Socket API partially implemented. AF_UNIX and AF_INET supported. \
            AF_INET6, AF_NETLINK return EAFNOSUPPORT. Raw sockets return EPROTONOSUPPORT.",
        errno_on_call: 0,
        planned_fix: true,
    },
    PosixDeviation {
        interface: "pthread_*",
        section: "2.7",
        severity: DeviationSeverity::Major,
        category: DeviationCategory::Architectural,
        description: "POSIX threads not supported. Microkernel uses lightweight tasks \
            (L4-style threads) with different semantics. pthread_create() returns ENOSYS. \
            Use task_spawn() for concurrent execution.",
        errno_on_call: 38,
        planned_fix: false,
    },
];

/// Look up a deviation by interface name
pub fn find_deviation(interface: &str) -> Option<&'static PosixDeviation> {
    POSIX_DEVIATIONS.iter().find(|d| d.interface == interface)
}

/// Get all deviations of a given severity
pub fn deviations_by_severity(severity: DeviationSeverity) -> Vec<&'static PosixDeviation> {
    POSIX_DEVIATIONS.iter().filter(|d| d.severity == severity).collect()
}

/// Get all deviations of a given category
pub fn deviations_by_category(category: DeviationCategory) -> Vec<&'static PosixDeviation> {
    POSIX_DEVIATIONS.iter().filter(|d| d.category == category).collect()
}

/// Count total registered deviations
pub fn deviation_count() -> usize {
    POSIX_DEVIATIONS.len()
}

/// Check if an interface has known deviations
pub fn has_deviation(interface: &str) -> bool {
    POSIX_DEVIATIONS.iter().any(|d| d.interface == interface)
}
