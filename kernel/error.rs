/*
 * Nuva OS - Kernel - Unified Error Type
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

use core::fmt;

/**
 * Unified kernel error type covering all kernel subsystem error categories.
 *
 * All kernel modules should use `Result<T, KernelError>` for fallible
 * operations instead of panic/unwrap/expect.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum KernelError {
    // Memory management errors (0x01xx)
    /** Out of physical or virtual memory */
    OutOfMemory,
    /** Memory allocation failed for unspecified reason */
    AllocationFailed,
    /** Invalid virtual or physical address */
    InvalidAddress,
    /** Page fault error */
    PageFault,
    /** Memory region already mapped */
    AlreadyMapped,
    /** Memory region not mapped */
    NotMapped,
    /** VMA operation failed */
    VmaError,
    /** COW fault resolution failed */
    CowError,
    /** NUMA migration failed */
    NumaMigrateError,
    /** Huge page allocation failed */
    HugePageError,
    /** Memory compaction failed */
    CompactionError,
    /** Memory reclaim failed */
    ReclaimError,
    /** OOM killer invoked */
    OomKill,
    /** Memory pool exhausted */
    PoolExhausted,
    /** Page table operation failed */
    PageTableError,

    // Scheduler errors (0x02xx)
    /** No runnable task found */
    NoRunnableTask,
    /** Invalid task ID */
    InvalidTaskId,
    /** Task already terminated */
    TaskTerminated,
    /** Scheduling policy not supported */
    InvalidSchedPolicy,
    /** Deadline scheduling error */
    DeadlineError,
    /** CPU affinity error */
    AffinityError,
    /** Load balance failed */
    LoadBalanceError,
    /** Context switch failed */
    ContextSwitchError,
    /** Invalid scheduling priority */
    InvalidPriority,
    /** Would block on operation */
    WouldBlock,

    // IPC errors (0x03xx)
    /** IPC channel closed */
    ChannelClosed,
    /** IPC channel full */
    ChannelFull,
    /** IPC permission denied */
    PermissionDenied,
    /** Invalid IPC message */
    InvalidMessage,
    /** IPC timeout */
    Timeout,
    /** Shared memory mapping failed */
    ShmMapError,
    /** Binder transaction failed */
    BinderError,
    /** Zero-copy transfer failed */
    ZeroCopyError,
    /** No data available */
    NoData,
    /** Port not found */
    PortNotFound,
    /** No send permission on IPC port */
    NoSendPermission,
    /** No receive permission on IPC port */
    NoReceivePermission,
    /** IPC port is dead (destroyed) */
    PortDead,
    /** IPC namespace is full */
    NamespaceFull,
    /** IPC object already exists */
    AlreadyExists,
    /** IPC object not found */
    NotFound,
    /** Message too large for channel */
    MessageTooLarge,
    /** Internal IPC error */
    InternalError,

    // Driver/device errors (0x04xx)
    /** Device not found */
    DeviceNotFound,
    /** Device busy */
    DeviceBusy,
    /** Device probe failed */
    ProbeFailed,
    /** Device bind failed */
    BindFailed,
    /** Resource conflict (e.g., IRQ, MMIO overlap) */
    ResourceConflict,
    /** Driver not found */
    DriverNotFound,
    /** Invalid device configuration */
    InvalidConfig,
    /** DMA mapping failed */
    DmaError,
    /** Interrupt registration failed */
    IrqError,
    /** Power management operation failed */
    PmError,

    // File system errors (0x05xx)
    /** File not found */
    FileNotFound,
    /** I/O error */
    IoError,
    /** Read-only filesystem */
    ReadOnlyFs,
    /** File already exists */
    FileExists,
    /** Not a directory */
    NotDirectory,
    /** Directory not empty */
    DirectoryNotEmpty,
    /** Filesystem corruption detected */
    FsCorruption,
    /** Journal error */
    JournalError,

    // Synchronization errors (0x06xx)
    /** Deadlock detected */
    DeadlockDetected,
    /** Lock not held by current owner */
    LockNotHeld,
    /** Mutex would deadlock (recursive) */
    WouldDeadlock,
    /** Semaphore overflow */
    SemaphoreOverflow,

    // Security errors (0x07xx)
    /** Access denied by capability check */
    AccessDenied,
    /** Invalid capability */
    InvalidCapability,
    /** Authentication failed */
    AuthenticationFailed,
    /** Sandbox violation */
    SandboxViolation,
    /** Signature verification failed */
    SignatureError,
    /** Encryption/decryption failed */
    CryptoError,
    /** Capability token denied (nuva native) */
    CapabilityDenied,
    /** Capability token expired or revoked */
    CapabilityExpired,
    /** Capability derivation failed (child rights not subset of parent) */
    CapabilityDerivationFailed,
    /** Capability transfer failed */
    CapabilityTransferFailed,
    /** Cross-level direct memory access denied (three-level architecture) */
    CrossLevelAccessDenied,
    /** NvSupervisorCall capability gate check failed */
    SupervisorCallDenied,
    /** Equipment mode service exceeded restart threshold, unrecoverable */
    ServiceUnrecoverable,
    /** Equipment mode service heartbeat timeout */
    HeartbeatTimeout,
    /** Port is in fault recovery transitioning state */
    PortTransitioning,

    // Generic errors (0x08xx)
    /** Invalid argument */
    InvalidArgument,
    /** Operation not supported */
    NotSupported,
    /** Operation interrupted */
    Interrupted,
    /** Object already initialized */
    AlreadyInitialized,
    /** Object not initialized */
    NotInitialized,
    /** Buffer too small */
    BufferTooSmall,
    /** State mismatch */
    InvalidState,
    /** Feature not enabled */
    FeatureDisabled,
    /** Unknown/unclassified error */
    Unknown,
}

impl KernelError {
    /**
     * Map kernel error to POSIX errno value.
     *
     * This enables consistent syscall return codes across all platforms.
     */
    pub fn to_errno(self) -> i32 {
        match self {
            // Memory
            KernelError::OutOfMemory => 12,      // ENOMEM
            KernelError::AllocationFailed => 12, // ENOMEM
            KernelError::InvalidAddress => 14,   // EFAULT
            KernelError::PageFault => 14,        // EFAULT
            KernelError::AlreadyMapped => 17,    // EEXIST
            KernelError::NotMapped => 22,        // EINVAL
            KernelError::VmaError => 22,         // EINVAL
            KernelError::CowError => 12,         // ENOMEM
            KernelError::NumaMigrateError => 12, // ENOMEM
            KernelError::HugePageError => 12,    // ENOMEM
            KernelError::CompactionError => 12,  // ENOMEM
            KernelError::ReclaimError => 12,     // ENOMEM
            KernelError::OomKill => 12,          // ENOMEM
            KernelError::PoolExhausted => 12,    // ENOMEM
            KernelError::PageTableError => 14,   // EFAULT

            // Scheduler
            KernelError::NoRunnableTask => 3,      // ESRCH
            KernelError::InvalidTaskId => 3,       // ESRCH
            KernelError::TaskTerminated => 3,      // ESRCH
            KernelError::InvalidSchedPolicy => 22, // EINVAL
            KernelError::DeadlineError => 22,      // EINVAL
            KernelError::AffinityError => 22,      // EINVAL
            KernelError::LoadBalanceError => 22,   // EINVAL
            KernelError::ContextSwitchError => 5,  // EIO
            KernelError::InvalidPriority => 22,    // EINVAL
            KernelError::WouldBlock => 11,         // EAGAIN

            // IPC
            KernelError::ChannelClosed => 32,       // EPIPE
            KernelError::ChannelFull => 11,         // EAGAIN
            KernelError::PermissionDenied => 13,    // EACCES
            KernelError::InvalidMessage => 22,      // EINVAL
            KernelError::Timeout => 110,            // ETIMEDOUT
            KernelError::ShmMapError => 12,         // ENOMEM
            KernelError::BinderError => 5,          // EIO
            KernelError::ZeroCopyError => 5,        // EIO
            KernelError::NoData => 11,              // EAGAIN
            KernelError::PortNotFound => 2,         // ENOENT
            KernelError::NoSendPermission => 13,    // EACCES
            KernelError::NoReceivePermission => 13, // EACCES
            KernelError::PortDead => 32,            // EPIPE
            KernelError::NamespaceFull => 28,       // ENOSPC
            KernelError::AlreadyExists => 17,       // EEXIST
            KernelError::NotFound => 2,             // ENOENT
            KernelError::MessageTooLarge => 90,     // EMSGSIZE
            KernelError::InternalError => 5,        // EIO

            // Driver
            KernelError::DeviceNotFound => 2,    // ENOENT
            KernelError::DeviceBusy => 16,       // EBUSY
            KernelError::ProbeFailed => 5,       // EIO
            KernelError::BindFailed => 5,        // EIO
            KernelError::ResourceConflict => 16, // EBUSY
            KernelError::DriverNotFound => 2,    // ENOENT
            KernelError::InvalidConfig => 22,    // EINVAL
            KernelError::DmaError => 5,          // EIO
            KernelError::IrqError => 5,          // EIO
            KernelError::PmError => 5,           // EIO

            // Filesystem
            KernelError::FileNotFound => 2,       // ENOENT
            KernelError::IoError => 5,            // EIO
            KernelError::ReadOnlyFs => 30,        // EROFS
            KernelError::FileExists => 17,        // EEXIST
            KernelError::NotDirectory => 20,      // ENOTDIR
            KernelError::DirectoryNotEmpty => 39, // ENOTEMPTY
            KernelError::FsCorruption => 5,       // EIO
            KernelError::JournalError => 5,       // EIO

            // Synchronization
            KernelError::DeadlockDetected => 11,  // EAGAIN
            KernelError::LockNotHeld => 22,       // EINVAL
            KernelError::WouldDeadlock => 11,     // EAGAIN
            KernelError::SemaphoreOverflow => 22, // EINVAL

            // Security
            KernelError::AccessDenied => 13,         // EACCES
            KernelError::InvalidCapability => 22,    // EINVAL
            KernelError::AuthenticationFailed => 13, // EACCES
            KernelError::SandboxViolation => 13,     // EACCES
            KernelError::SignatureError => 13,       // EACCES
            KernelError::CryptoError => 5,           // EIO
            KernelError::CapabilityDenied => 13,     // EACCES
            KernelError::CapabilityExpired => 13,    // EACCES
            KernelError::CapabilityDerivationFailed => 22, // EINVAL
            KernelError::CapabilityTransferFailed => 13,   // EACCES
            KernelError::CrossLevelAccessDenied => 13,     // EACCES
            KernelError::SupervisorCallDenied => 13,       // EACCES
            KernelError::ServiceUnrecoverable => 5,        // EIO
            KernelError::HeartbeatTimeout => 110,          // ETIMEDOUT
            KernelError::PortTransitioning => 11,          // EAGAIN

            // Generic
            KernelError::InvalidArgument => 22,    // EINVAL
            KernelError::NotSupported => 38,       // ENOSYS
            KernelError::Interrupted => 4,         // EINTR
            KernelError::AlreadyInitialized => 17, // EEXIST
            KernelError::NotInitialized => 22,     // EINVAL
            KernelError::BufferTooSmall => 7,      // E2BIG
            KernelError::InvalidState => 22,       // EINVAL
            KernelError::FeatureDisabled => 38,    // ENOSYS
            KernelError::Unknown => 5,             // EIO
        }
    }

    /**
     * Check if this error is recoverable (operation can be retried).
     */
    pub fn is_recoverable(self) -> bool {
        matches!(
            self,
            KernelError::OutOfMemory
                | KernelError::AllocationFailed
                | KernelError::ChannelFull
                | KernelError::Timeout
                | KernelError::NoData
                | KernelError::DeviceBusy
                | KernelError::Interrupted
                | KernelError::DeadlockDetected
                | KernelError::WouldDeadlock
                | KernelError::AlreadyExists
                | KernelError::NotFound
                | KernelError::NotInitialized
                | KernelError::WouldBlock
        )
    }

    /**
     * Check if this error is caused by invalid input.
     */
    pub fn is_user_error(self) -> bool {
        matches!(
            self,
            KernelError::InvalidArgument
                | KernelError::InvalidAddress
                | KernelError::InvalidTaskId
                | KernelError::InvalidMessage
                | KernelError::InvalidConfig
                | KernelError::InvalidSchedPolicy
                | KernelError::InvalidState
                | KernelError::BufferTooSmall
                | KernelError::PermissionDenied
                | KernelError::MessageTooLarge
                | KernelError::InvalidPriority
        )
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Memory
            KernelError::OutOfMemory => write!(f, "out of memory"),
            KernelError::AllocationFailed => write!(f, "memory allocation failed"),
            KernelError::InvalidAddress => write!(f, "invalid address"),
            KernelError::PageFault => write!(f, "page fault"),
            KernelError::AlreadyMapped => write!(f, "address already mapped"),
            KernelError::NotMapped => write!(f, "address not mapped"),
            KernelError::VmaError => write!(f, "VMA operation failed"),
            KernelError::CowError => write!(f, "COW fault resolution failed"),
            KernelError::NumaMigrateError => write!(f, "NUMA migration failed"),
            KernelError::HugePageError => write!(f, "huge page allocation failed"),
            KernelError::CompactionError => write!(f, "memory compaction failed"),
            KernelError::ReclaimError => write!(f, "memory reclaim failed"),
            KernelError::OomKill => write!(f, "OOM killer invoked"),
            KernelError::PoolExhausted => write!(f, "memory pool exhausted"),
            KernelError::PageTableError => write!(f, "page table operation failed"),

            // Scheduler
            KernelError::NoRunnableTask => write!(f, "no runnable task"),
            KernelError::InvalidTaskId => write!(f, "invalid task ID"),
            KernelError::TaskTerminated => write!(f, "task already terminated"),
            KernelError::InvalidSchedPolicy => write!(f, "invalid scheduling policy"),
            KernelError::DeadlineError => write!(f, "deadline scheduling error"),
            KernelError::AffinityError => write!(f, "CPU affinity error"),
            KernelError::LoadBalanceError => write!(f, "load balance failed"),
            KernelError::ContextSwitchError => write!(f, "context switch failed"),
            KernelError::InvalidPriority => write!(f, "invalid scheduling priority"),
            KernelError::WouldBlock => write!(f, "operation would block"),

            // IPC
            KernelError::ChannelClosed => write!(f, "IPC channel closed"),
            KernelError::ChannelFull => write!(f, "IPC channel full"),
            KernelError::PermissionDenied => write!(f, "permission denied"),
            KernelError::InvalidMessage => write!(f, "invalid IPC message"),
            KernelError::Timeout => write!(f, "operation timed out"),
            KernelError::ShmMapError => write!(f, "shared memory mapping failed"),
            KernelError::BinderError => write!(f, "binder transaction failed"),
            KernelError::ZeroCopyError => write!(f, "zero-copy transfer failed"),
            KernelError::NoData => write!(f, "no data available"),
            KernelError::PortNotFound => write!(f, "IPC port not found"),
            KernelError::NoSendPermission => write!(f, "no send permission on IPC port"),
            KernelError::NoReceivePermission => write!(f, "no receive permission on IPC port"),
            KernelError::PortDead => write!(f, "IPC port is dead"),
            KernelError::NamespaceFull => write!(f, "IPC namespace is full"),
            KernelError::AlreadyExists => write!(f, "IPC object already exists"),
            KernelError::NotFound => write!(f, "IPC object not found"),
            KernelError::MessageTooLarge => write!(f, "message too large for channel"),
            KernelError::InternalError => write!(f, "internal IPC error"),

            // Driver
            KernelError::DeviceNotFound => write!(f, "device not found"),
            KernelError::DeviceBusy => write!(f, "device busy"),
            KernelError::ProbeFailed => write!(f, "device probe failed"),
            KernelError::BindFailed => write!(f, "device bind failed"),
            KernelError::ResourceConflict => write!(f, "resource conflict"),
            KernelError::DriverNotFound => write!(f, "driver not found"),
            KernelError::InvalidConfig => write!(f, "invalid device configuration"),
            KernelError::DmaError => write!(f, "DMA mapping failed"),
            KernelError::IrqError => write!(f, "IRQ registration failed"),
            KernelError::PmError => write!(f, "power management error"),

            // Filesystem
            KernelError::FileNotFound => write!(f, "file not found"),
            KernelError::IoError => write!(f, "I/O error"),
            KernelError::ReadOnlyFs => write!(f, "read-only filesystem"),
            KernelError::FileExists => write!(f, "file already exists"),
            KernelError::NotDirectory => write!(f, "not a directory"),
            KernelError::DirectoryNotEmpty => write!(f, "directory not empty"),
            KernelError::FsCorruption => write!(f, "filesystem corruption"),
            KernelError::JournalError => write!(f, "journal error"),

            // Synchronization
            KernelError::DeadlockDetected => write!(f, "deadlock detected"),
            KernelError::LockNotHeld => write!(f, "lock not held by current owner"),
            KernelError::WouldDeadlock => write!(f, "operation would deadlock"),
            KernelError::SemaphoreOverflow => write!(f, "semaphore overflow"),

            // Security
            KernelError::AccessDenied => write!(f, "access denied"),
            KernelError::InvalidCapability => write!(f, "invalid capability"),
            KernelError::AuthenticationFailed => write!(f, "authentication failed"),
            KernelError::SandboxViolation => write!(f, "sandbox violation"),
            KernelError::SignatureError => write!(f, "signature verification failed"),
            KernelError::CryptoError => write!(f, "cryptographic operation failed"),
            KernelError::CapabilityDenied => write!(f, "capability token denied"),
            KernelError::CapabilityExpired => write!(f, "capability token expired or revoked"),
            KernelError::CapabilityDerivationFailed => write!(f, "capability derivation failed"),
            KernelError::CapabilityTransferFailed => write!(f, "capability transfer failed"),
            KernelError::CrossLevelAccessDenied => write!(f, "cross-level direct memory access denied"),
            KernelError::SupervisorCallDenied => write!(f, "supervisor call capability gate denied"),
            KernelError::ServiceUnrecoverable => write!(f, "equipment service unrecoverable"),
            KernelError::HeartbeatTimeout => write!(f, "equipment service heartbeat timeout"),
            KernelError::PortTransitioning => write!(f, "port in fault recovery transitioning state"),

            // Generic
            KernelError::InvalidArgument => write!(f, "invalid argument"),
            KernelError::NotSupported => write!(f, "operation not supported"),
            KernelError::Interrupted => write!(f, "operation interrupted"),
            KernelError::AlreadyInitialized => write!(f, "already initialized"),
            KernelError::NotInitialized => write!(f, "not initialized"),
            KernelError::BufferTooSmall => write!(f, "buffer too small"),
            KernelError::InvalidState => write!(f, "invalid state"),
            KernelError::FeatureDisabled => write!(f, "feature not enabled"),
            KernelError::Unknown => write!(f, "unknown error"),
        }
    }
}

/** Kernel result type alias for convenience */
pub type KernelResult<T> = Result<T, KernelError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_errno_mapping() {
        assert_eq!(KernelError::OutOfMemory.to_errno(), 12);
        assert_eq!(KernelError::InvalidArgument.to_errno(), 22);
        assert_eq!(KernelError::PermissionDenied.to_errno(), 13);
        assert_eq!(KernelError::Timeout.to_errno(), 110);
        assert_eq!(KernelError::FileNotFound.to_errno(), 2);
        assert_eq!(KernelError::IoError.to_errno(), 5);
        assert_eq!(KernelError::DeadlockDetected.to_errno(), 11);
    }

    #[test]
    fn test_recoverable() {
        assert!(KernelError::OutOfMemory.is_recoverable());
        assert!(KernelError::Timeout.is_recoverable());
        assert!(!KernelError::InvalidArgument.is_recoverable());
        assert!(!KernelError::AccessDenied.is_recoverable());
    }

    #[test]
    fn test_user_error() {
        assert!(KernelError::InvalidArgument.is_user_error());
        assert!(KernelError::InvalidAddress.is_user_error());
        assert!(!KernelError::OutOfMemory.is_user_error());
        assert!(!KernelError::IoError.is_user_error());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            alloc::format!("{}", KernelError::OutOfMemory),
            "out of memory"
        );
        assert_eq!(
            alloc::format!("{}", KernelError::DeadlockDetected),
            "deadlock detected"
        );
    }
}
