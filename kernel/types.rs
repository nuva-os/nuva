/*
 * Nuva OS - Kernel - Types
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
/*
 * Nuva OS - Kernel - Nuva Native Type System
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva OS native type definitions for the kernel.
 * These types form the foundation of the Nuva native system call
 * interface and security model, replacing POSIX/Unix type semantics.
 */

use core::fmt;

/// Nuva native process identifier (capability-based, 64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NuvaProcessId(pub u64);

impl NuvaProcessId {
    pub const fn new(id: u64) -> Self {
        NuvaProcessId(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn is_kernel(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for NuvaProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NuvaProcessId({})", self.0)
    }
}

/// Nuva native thread identifier (64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NuvaThreadId(pub u64);

impl NuvaThreadId {
    pub const fn new(id: u64) -> Self {
        NuvaThreadId(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Nuva native capability identifier (64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NuvaCapabilityId(pub u64);

impl NuvaCapabilityId {
    pub const fn new(id: u64) -> Self {
        NuvaCapabilityId(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Nuva native file handle (64-bit, replaces fd_t)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NuvaFileHandle(pub u64);

impl NuvaFileHandle {
    pub const fn new(handle: u64) -> Self {
        NuvaFileHandle(handle)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// Nuva native file offset (64-bit, replaces off_t)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NuvaFileOffset(pub u64);

impl NuvaFileOffset {
    pub const fn new(offset: u64) -> Self {
        NuvaFileOffset(offset)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Nuva native inode identifier (64-bit, replaces ino_t)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NuvaInodeId(pub u64);

impl NuvaInodeId {
    pub const fn new(id: u64) -> Self {
        NuvaInodeId(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Nuva native memory region handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NuvaMemoryRegion(pub u64);

impl NuvaMemoryRegion {
    pub const fn new(handle: u64) -> Self {
        NuvaMemoryRegion(handle)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Nuva native event notification port
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NuvaNotificationPort(pub u64);

impl NuvaNotificationPort {
    pub const fn new(port: u64) -> Self {
        NuvaNotificationPort(port)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

bitflags::bitflags! {
    /// Nuva native access rights (replaces Unix mode_t permission bits)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NuvaAccessRight: u32 {
        const READ      = 0b0000_0001;
        const WRITE     = 0b0000_0010;
        const EXECUTE   = 0b0000_0100;
        const CREATE    = 0b0000_1000;
        const DESTROY   = 0b0001_0000;
        const GRANT     = 0b0010_0000;
        const REVOKE    = 0b0100_0000;
        const TRANSFER  = 0b1000_0000;
        const ADMIN     = 0b0001_0000_0000;
        const ALL       = 0b1111_1111_1111;
    }
}

/// Nuva native error type (replaces i32 errno pattern)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NuvaError {
    Success             = 0,
    CapabilityDenied    = 1,
    CapabilityExpired   = 2,
    InvalidCall         = 3,
    InvalidParameter    = 4,
    ResourceNotFound    = 5,
    ResourceBusy        = 6,
    Timeout             = 7,
    NoMemory            = 8,
    MessageTooLarge     = 9,
    PortNotFound        = 10,
    PortDead            = 11,
    WouldBlock          = 12,
    PermissionDenied    = 13,
    IoError             = 14,
    InternalError       = 15,
}

impl NuvaError {
    pub fn is_success(&self) -> bool {
        matches!(self, NuvaError::Success)
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

impl fmt::Display for NuvaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NuvaError::Success          => write!(f, "Success"),
            NuvaError::CapabilityDenied => write!(f, "CapabilityDenied"),
            NuvaError::CapabilityExpired=> write!(f, "CapabilityExpired"),
            NuvaError::InvalidCall      => write!(f, "InvalidCall"),
            NuvaError::InvalidParameter => write!(f, "InvalidParameter"),
            NuvaError::ResourceNotFound => write!(f, "ResourceNotFound"),
            NuvaError::ResourceBusy     => write!(f, "ResourceBusy"),
            NuvaError::Timeout          => write!(f, "Timeout"),
            NuvaError::NoMemory         => write!(f, "NoMemory"),
            NuvaError::MessageTooLarge  => write!(f, "MessageTooLarge"),
            NuvaError::PortNotFound     => write!(f, "PortNotFound"),
            NuvaError::PortDead         => write!(f, "PortDead"),
            NuvaError::WouldBlock       => write!(f, "WouldBlock"),
            NuvaError::PermissionDenied => write!(f, "PermissionDenied"),
            NuvaError::IoError          => write!(f, "IoError"),
            NuvaError::InternalError    => write!(f, "InternalError"),
        }
    }
}

/// Nuva resource handle type (identifies the kind of resource being accessed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum NuvaResourceHandle {
    Process = 0,
    File    = 1,
    Memory  = 2,
    Port    = 3,
    Device  = 4,
    Network = 5,
    System  = 6,
}

/// Nuva event type (replaces POSIX signal model)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NuvaEventType {
    Interrupt        = 0,
    TimerExpired     = 1,
    IoComplete       = 2,
    ProcessExit      = 3,
    MemoryPressure   = 4,
    Custom           = 5,
}

/// Nuva native event (replaces POSIX signal)
#[derive(Debug, Clone, Copy)]
pub struct NuvaEvent {
    pub event_type: NuvaEventType,
    pub source: NuvaProcessId,
    pub payload: u64,
}

impl NuvaEvent {
    pub fn new(event_type: NuvaEventType, source: NuvaProcessId, payload: u64) -> Self {
        NuvaEvent { event_type, source, payload }
    }
}

/// Nuva terminate reason (replaces signal-based termination)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NuvaTerminateReason {
    NormalExit       = 0,
    Oom              = 1,
    SecurityViolation= 2,
    ParentRequest    = 3,
    ResourceExhausted= 4,
}

/// NvIPC port identifier (global kernel port ID)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NvPortId(pub u64);

impl NvPortId {
    pub const fn new(id: u64) -> Self {
        NvPortId(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// NvIPC port name (task-local port namespace name)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NvPortName(pub u64);

impl NvPortName {
    pub const fn new(name: u64) -> Self {
        NvPortName(name)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Nuva memory region identifier (capability-controlled)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NvMemRegionId(pub u64);

impl NvMemRegionId {
    pub const fn new(id: u64) -> Self {
        NvMemRegionId(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// Nuva virtual address (replaces raw usize vaddr)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NvVAddr(pub u64);

impl NvVAddr {
    pub const fn new(addr: u64) -> Self {
        NvVAddr(addr)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn is_aligned(&self, align: u64) -> bool {
        self.0 % align == 0
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
}

/// Nuva physical page number (replaces raw usize pfn)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NvPhysPage(pub u64);

impl NvPhysPage {
    pub const fn new(pfn: u64) -> Self {
        NvPhysPage(pfn)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Nuva timestamp (monotonic nanosecond timestamp)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NvTimestamp(pub u64);

impl NvTimestamp {
    pub const fn new(ts: u64) -> Self {
        NvTimestamp(ts)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const ZERO: NvTimestamp = NvTimestamp(0);
}

/// Nuva duration (nanosecond time span)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NvDuration(pub u64);

impl NvDuration {
    pub const fn new(d: u64) -> Self {
        NvDuration(d)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const ZERO: NvDuration = NvDuration(0);

    pub const INFINITE: NvDuration = NvDuration(u64::MAX);
}

/// Nuva fault domain identifier (for microkernel service isolation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NvFaultDomainId(pub u64);

impl NvFaultDomainId {
    pub const fn new(id: u64) -> Self {
        NvFaultDomainId(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const KERNEL: NvFaultDomainId = NvFaultDomainId(0);
}

/// Nuva three-level privilege level (EL2/EL1/EL0)
///
/// EL2: Minimal kernel mode (scheduler, IPC, MM, cap mgr, IRQ, timer)
/// EL1: Equipment mode (filesystem, network, drivers, display services)
/// EL0: User mode (applications)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum NvPrivilegeLevel {
    UserMode        = 0,
    EquipmentMode   = 1,
    KernelMode      = 2,
}

impl NvPrivilegeLevel {
    pub const fn is_kernel(&self) -> bool {
        matches!(self, NvPrivilegeLevel::KernelMode)
    }

    pub const fn is_equipment(&self) -> bool {
        matches!(self, NvPrivilegeLevel::EquipmentMode)
    }

    pub const fn is_user(&self) -> bool {
        matches!(self, NvPrivilegeLevel::UserMode)
    }
}

/// NvSupervisorCall operation types (EL1→EL2 controlled interface)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum NvSupervisorOp {
    MapDeviceMemory     = 0,
    UnmapDeviceMemory   = 1,
    DmaMap              = 2,
    DmaUnmap            = 3,
    IrqRequest          = 4,
    IrqRelease          = 5,
    IrqEnable           = 6,
    IrqDisable          = 7,
    TimerSet            = 8,
    TimerCancel         = 9,
    CapDeriveForService = 10,
    CapRevokeFromService= 11,
    PortCreateForService= 12,
    PortDestroyForService=13,
}

/// Nuva address space identifier (for independent fault domain isolation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NvAddressSpaceId(pub u64);

impl NvAddressSpaceId {
    pub const fn new(id: u64) -> Self {
        NvAddressSpaceId(id)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const KERNEL: NvAddressSpaceId = NvAddressSpaceId(0);
}

/// Nuva service name identifier (for equipment mode service registration)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NvServiceName(pub u64);

impl NvServiceName {
    pub const fn new(name: u64) -> Self {
        NvServiceName(name)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Nuva diagnostic topic (replaces /proc filesystem queries)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NuvaDiagTopic {
    ProcessList     = 0,
    ProcessInfo     = 1,
    MemoryInfo      = 2,
    CpuInfo         = 3,
    DeviceList      = 4,
    NetworkStats    = 5,
    FileSystemInfo  = 6,
    CapabilityInfo  = 7,
    Custom          = 8,
}
