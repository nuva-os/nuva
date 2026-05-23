/*
 * Nuva OS - Kernel - POSIX Capability Set
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * POSIX capability set and constants.
 */

/// POSIX capability constants
pub mod cap {
    /// Change file ownership
    pub const CHOWN: u32 = 0;
    /// Override DAC read search
    pub const DAC_READ_SEARCH: u32 = 1;
    /// Override DAC write
    pub const DAC_OVERRIDE: u32 = 2;
    /// Override file owner
    pub const FOWNER: u32 = 3;
    /// Override file set-ID
    pub const FSETID: u32 = 4;
    /// Send kill signal
    pub const KILL: u32 = 5;
    /// Set group ID
    pub const SETGID: u32 = 6;
    /// Set user ID
    pub const SETUID: u32 = 7;
    /// Set process capabilities
    pub const SETPCAP: u32 = 8;
    /// Immutable capability (renamed from LINUX_IMMUTABLE)
    pub const IMMUTABLE: u32 = 9;
    /// Deprecated alias: Use IMMUTABLE instead
    #[deprecated(since = "0.2.0", note = "Use IMMUTABLE instead")]
    pub const LINUX_IMMUTABLE: u32 = IMMUTABLE;
    /// Bind to privileged network service
    pub const NET_BIND_SERVICE: u32 = 10;
    /// Network broadcast
    pub const NET_BROADCAST: u32 = 11;
    /// Network administration
    pub const NET_ADMIN: u32 = 12;
    /// Use raw sockets
    pub const NET_RAW: u32 = 13;
    /// Lock memory pages
    pub const IPC_LOCK: u32 = 14;
    /// Override IPC ownership
    pub const IPC_OWNER: u32 = 15;
    /// Load/unload kernel modules
    pub const SYS_MODULE: u32 = 16;
    /// Perform raw I/O
    pub const SYS_RAWIO: u32 = 17;
    /// Change root directory
    pub const SYS_CHROOT: u32 = 18;
    /// Trace processes
    pub const SYS_PTRACE: u32 = 19;
    /// Process accounting
    pub const SYS_PACCT: u32 = 20;
    /// System administration
    pub const SYS_ADMIN: u32 = 21;
    /// Boot the system
    pub const SYS_BOOT: u32 = 22;
    /// Set process priority
    pub const SYS_NICE: u32 = 23;
    /// Override resource limits
    pub const SYS_RESOURCE: u32 = 24;
    /// Set system clock
    pub const SYS_TIME: u32 = 25;
    /// Configure TTY devices
    pub const SYS_TTY_CONFIG: u32 = 26;
    /// Create special files
    pub const MKNOD: u32 = 27;
    /// File leases
    pub const LEASE: u32 = 28;
    /// Write to audit log
    pub const AUDIT_WRITE: u32 = 29;
    /// Configure audit subsystem
    pub const AUDIT_CONTROL: u32 = 30;
    /// Set file capabilities
    pub const SETFCAP: u32 = 31;
    /// Override MAC policy
    pub const MAC_OVERRIDE: u32 = 32;
    /// Configure MAC policy
    pub const MAC_ADMIN: u32 = 33;
    /// Syslog access
    pub const SYSLOG: u32 = 34;
    /// Wake alarm
    pub const WAKE_ALARM: u32 = 35;
    /// Block suspend
    pub const BLOCK_SUSPEND: u32 = 36;
    /// Read from audit log
    pub const AUDIT_READ: u32 = 37;

    /// Last capability index
    pub const LAST_CAP: u32 = 37;
}

/// POSIX capability set (64-bit bitmap)
#[repr(C)]
pub struct CapSet {
    /// Capability bits (two u32 for 64-bit coverage)
    pub caps: [u32; 2],
}

impl CapSet {
    /// Create an empty capability set
    pub const fn new() -> Self {
        CapSet { caps: [0; 2] }
    }

    /// Add a capability
    pub fn set(&mut self, cap: u32) {
        if cap <= 31 {
            self.caps[0] |= 1u32 << cap;
        } else if cap <= 63 {
            self.caps[1] |= 1u32 << (cap - 32);
        }
    }

    /// Remove a capability
    pub fn clear(&mut self, cap: u32) {
        if cap <= 31 {
            self.caps[0] &= !(1u32 << cap);
        } else if cap <= 63 {
            self.caps[1] &= !(1u32 << (cap - 32));
        }
    }

    /// Check if a capability is set
    pub fn has(&self, cap: u32) -> bool {
        if cap <= 31 {
            (self.caps[0] & (1u32 << cap)) != 0
        } else if cap <= 63 {
            (self.caps[1] & (1u32 << (cap - 32))) != 0
        } else {
            false
        }
    }

    /// Check if no capabilities are set
    pub fn is_empty(&self) -> bool {
        self.caps[0] == 0 && self.caps[1] == 0
    }

    /// Check if all 64 bits are set
    pub fn is_full(&self) -> bool {
        self.caps[0] == u32::MAX && self.caps[1] == u32::MAX
    }
}
